mod components;
mod systems;
pub mod consts;

pub use consts::*;

use vermarine_lib::{
    shipyard::{
        *,
    },
    tetra::{
        self,
        graphics::{
            self, 
            Color,
            Camera,
        },
        input::{
            self,
            Key,
            MouseButton,
            InputContext,
        },
        math::{
            Vec2,
        },
        Context,
        ContextBuilder,
        Result,
    },
    physics::{
        PhysicsWorkloadCreator,
        PhysicsWorkloadSystems,
        PhysicsBody,
        CollisionBody,
        Collider,
        world::{
            PhysicsWorld,
        }
    },
    components::{
        Transform,
    },
    rendering::{
        Drawables,
        Sprite,
        RenderingWorkloadCreator,
        RenderingWorkloadSystems,
        draw_buffer::{
            DrawCommand,
            DrawBuffer,
        },
    },
    pushdown_automaton_state::{
        PushdownAutomaton,
        PDAState,
        Trans,
    },
};

use rand::rngs::StdRng;
use rand::SeedableRng;

use components::*;
use systems::*;


pub struct AsteroidGame {
    asteroid_timer: i32,
    spinner_timer: i32,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    lmb_down: bool,
    shoot_angle: f64,
}

impl AsteroidGame {
    pub fn new(asteroid_timer: i32, spinner_timer: i32) -> Self {
        AsteroidGame {
            asteroid_timer,
            spinner_timer,
            move_left: false,
            move_right: false,
            move_down: false,
            move_up: false,
            lmb_down: false,
            shoot_angle: 0f64,
        }
    }
}

type Res = Drawables;
fn main() -> tetra::Result {
    ContextBuilder::new("Asteroids", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(|ctx| PushdownAutomaton::new(ctx, GameState::new, Drawables::new))
}

struct GameState {
    world: World,
}

impl PDAState<Res> for GameState {
    fn update(&mut self, ctx: &mut Context, _: &mut Res) -> Result<Trans<Res>> {
        let input_ctx = ctx.input_context();
        self.world.run(|mut ctx: UniqueViewMut<InputContext>| {
            *ctx = (*input_ctx).clone();
        });

        self.handle_input();
        self.world.run_workload("Main");
        self.world.run_workload("Physics");

        if self.player_is_dead() {
            return Ok(Trans::Switch(Box::new(DeadState)));
        }
        Ok(Trans::None)
    }

    fn draw(&mut self, ctx: &mut Context, _: &mut Res) -> Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.world.run_workload("Rendering");
        self.world.run(|mut draw_buff: UniqueViewMut<DrawBuffer>, mut camera: UniqueViewMut<Camera>| {
            camera.update();
            draw_buff.transform_mat = camera.as_matrix();
        });
        self.world.run_with_data(DrawBuffer::flush, ctx);

        Ok(())
    }
}

impl GameState {
    fn new(ctx: &mut Context, res: &mut Res) -> Result<GameState> {
        let mut world = World::new();
        world.add_unique(AsteroidGame::new(50i32, 50i32));
        world.add_unique(StdRng::from_entropy());
        world.add_unique(Camera::with_window_size(ctx));
        world.add_unique((*ctx.input_context()).clone());
        world.add_unique_non_send_sync((*res).clone());

        world.run(|mut camera: UniqueViewMut<Camera>| {
            camera.zoom = 1.0;
        });

        world
            .add_workload("Main")
            .with_system(system!(player_input))
            .with_system(system!(iframe_counter))
            .with_system(system!(spawn_asteroids))
            .with_system(system!(spawn_spinners))
            .with_system(system!(shoot_spinners))
            .with_system(system!(apply_physics))
            .with_system(system!(move_player_bullets))
            .with_system(system!(wrap_asteroids))
            .with_system(system!(destroy_offscreen))
            .with_system(system!(player_damage))
            .with_system(system!(asteroid_damage))
            .with_system(system!(destroy_bullets))
            .with_system(system!(move_camera))
            .build();
        
        world
            .add_physics_workload(50.0, 50.0)
            .with_physics_systems()
            .build();

        world
            .add_rendering_workload(ctx)
            .with_rendering_systems()
            .build();

        world.run(
            |drawables: NonSendSync<UniqueView<Drawables>>,
            mut entities: EntitiesViewMut,
            mut sprites: ViewMut<Sprite>,
            mut healths: ViewMut<Health>,
            mut physicses: ViewMut<Physics>,
            mut players: ViewMut<Player>,
            mut physics_bodies: ViewMut<PhysicsBody>,
            mut physics_world: UniqueViewMut<PhysicsWorld>,
            mut transforms: ViewMut<Transform>, | {
                // Player
                let player = entities.add_entity(
                    (
                        &mut sprites,
                        &mut healths,
                        &mut physicses,
                        &mut players,
                    ),
                    (
                        create_sprite(drawables.alias[textures::SQUARE], 10.0, Color::rgb(0.0, 1.0, 0.0), draw_layers::PLAYER),
                        Health::new(3, 20, Some(Color::RED)),
                        Physics::default(),
                        Player {},
                    ),
                );

                physics_world.create_body(
                    &mut entities,
                    &mut physics_bodies,
                    player,
                    &mut transforms,
                    Transform::new(0.0, 0.0),
                    CollisionBody::from_parts(
                        // Collider
                        vec![Collider::half_extents(
                            10f64, 
                            10f64, 
                            layers::PLAYER, 
                            layers::WALL,
                        )], 
                        // Sensor
                        vec![Collider::half_extents(
                            10f64,
                            10f64,
                            layers::PLAYER,
                            layers::ENEMY | layers::BULLET_ENEMY | layers::ASTEROID,
                        )]),
                );
            },
        );

        world.run_with_data(
            create_wall,
            (-1020.0, 0.0, 20.0, 540.0),
        );
        world.run_with_data(
            create_wall,
            (1020.0, 0.0, 20.0, 540.0),
        );
        world.run_with_data(
            create_wall,
            (0.0, -520.0, 1040.0, 20.0),
        );
        world.run_with_data(
            create_wall,
            (0.0, 520.0, 1040.0, 20.0),
        );

        Ok(GameState { world })
    }

    fn handle_input(&mut self) {
        self.world.run(|
            ctx: UniqueView<InputContext>,
            mut game: UniqueViewMut<AsteroidGame>, 
            physics_bodies: View<PhysicsBody>,
            players: View<Player>, 
            physics_world: UniqueView<PhysicsWorld>,
            camera: UniqueView<Camera>, | {
                game.move_right = input::is_key_down(&ctx, Key::D);
                game.move_left = input::is_key_down(&ctx, Key::A);
                game.move_up = input::is_key_down(&ctx, Key::W);
                game.move_down = input::is_key_down(&ctx, Key::S);
                game.lmb_down = input::is_mouse_button_down(&ctx, MouseButton::Left);
                if game.lmb_down {
                    let body = match (&physics_bodies, &players).iter().with_id().next() {
                        Some((id, _)) => id,
                        _ => return,
                    };
        
                    let transform = physics_world.transform(body);
        
                    let pos = camera.mouse_position(&ctx);

                    let angle = transform.get_angle_to(pos.x as f64, pos.y as f64);
                    game.shoot_angle = angle;
                }
            });
    }

    fn player_is_dead(&self) -> bool {
        let players = self.world.borrow::<View<Player>>();
        match players.iter().next() {
            Some(_) => return false,
            _ => return true,
        };
    }
}

struct DeadState;

impl PDAState<Res> for DeadState {
    fn update(&mut self, ctx: &mut Context, res: &mut Res) -> Result<Trans<Res>> {
        if input::is_key_down(ctx.input_context(), Key::Space) {
            return Ok(Trans::Switch(Box::new(GameState::new(ctx, res)?)));
        }

        Ok(Trans::None)
    }

    fn draw(&mut self, ctx: &mut Context, _resources: &mut Res) -> Result {
        graphics::clear(ctx, Color::rgb(0.45, 0.65, 1.0));

        Ok(())
    }
}

fn create_wall(
    data: (f64, f64, f64, f64),
    drawables: NonSendSync<UniqueView<Drawables>>,
    mut entities: EntitiesViewMut, 
    mut sprites: ViewMut<Sprite>, 
    mut physicses: ViewMut<Physics>, 
    mut physics_world: UniqueViewMut<PhysicsWorld>, 
    mut physics_bodies: ViewMut<PhysicsBody>,
    mut transforms: ViewMut<Transform>, ) 
    {
        let (pos_x, pos_y, scale_x, scale_y) = data;

        // Stationary square
        let scale_calc = |s: f64| { (s / 1024.0 * 2.0) as f32 };
        let square = entities.add_entity((&mut sprites, &mut physicses), (
            Sprite::from_command(
                DrawCommand::new(drawables.alias[textures::SQUARE])
                .scale(Vec2::new(scale_calc(scale_x), scale_calc(scale_y)))
                .origin(Vec2::new(512.0, 512.0))
                .color(Color::BLACK)
                .draw_layer(draw_layers::WALL)
            ),
            Physics::default(),
        ));

        physics_world.create_body(
            &mut entities,
            &mut physics_bodies,
            square,
            &mut transforms,
            Transform::new(pos_x, pos_y),
            CollisionBody::from_collider(Collider::half_extents(scale_x, scale_y, layers::WALL, 0))
        );
}