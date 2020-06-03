mod components;
mod systems;
pub mod consts;

pub use consts::*;

use vermarine_lib::{
    shipyard::{
        self,
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
        },
        Context,
        ContextBuilder,
        Result,
        State,
        Trans,
    },
    physics::{
        PhysicsWorkloadCreator,
        PhysicsWorkloadSystems,
        PhysicsBody,
        CollisionBody,
        Collider,
        Transform,
        world::{
            PhysicsWorld,
        }
    },
    rendering::{
        Drawables,
        Sprite,
        RenderingWorkloadCreator,
        RenderingWorkloadSystems,
        draw_buffer::{
            DrawBuffer,
        }
    },
};

use rand::rngs::StdRng;
use rand::SeedableRng;

use components::*;
use systems::*;

type Res = Drawables;

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

fn main() -> tetra::Result {
    ContextBuilder::new("Asteroids", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(GameState::new, |ctx| Ok(Drawables::new(ctx)?))
}

struct GameState {
    world: World,
}

impl State<Res> for GameState {
    fn update(&mut self, ctx: &mut Context, _: &mut Res) -> Result<Trans<Res>> {
        self.handle_input(ctx);
        self.world.run_workload("Main");
        self.world.run_workload("Physics");

        if self.player_is_dead() {
            return Ok(Trans::Switch(Box::new(DeadState)));
        }
        Ok(Trans::None)
    }

    fn draw(&mut self, ctx: &mut Context, resources: &mut Res) -> Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.world.run_workload("Rendering");
        let (mut buffer, mut camera) = self.world.borrow::<(UniqueViewMut<DrawBuffer>, UniqueViewMut<Camera>)>();
        buffer.flush(ctx, resources, &mut *camera);

        if input::is_key_down(ctx, Key::Z) {
            buffer.debug_command_buffer();
        }

        Ok(())
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> Result<GameState> {
        let mut world = World::new();
        world.add_unique(AsteroidGame::new(50i32, 50i32));
        world.add_unique(StdRng::from_entropy());
        world.add_unique(Camera::with_window_size(ctx));

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
            .with_system(system!(wrap_player))
            .with_system(system!(destroy_offscreen))
            .with_system(system!(player_damage))
            .with_system(system!(asteroid_damage))
            .with_system(system!(destroy_bullets))
            //.with_system(system!(move_camera))
            .build();
        
        world
            .add_physics_workload(20.0, 20.0)
            .with_physics_systems()
            .build();

        world
            .add_rendering_workload()
            .with_rendering_systems()
            .build();

        world.run(
            |mut entities: EntitiesViewMut,
             mut sprites: ViewMut<Sprite>,
             mut healths: ViewMut<Health>,
             mut physicses: ViewMut<Physics>,
             mut players: ViewMut<Player>,
             mut physics_bodies: ViewMut<PhysicsBody>,
             mut physics_world: UniqueViewMut<PhysicsWorld>| {
                // Player
                let player = entities.add_entity(
                    (
                        &mut sprites,
                        &mut healths,
                        &mut physicses,
                        &mut players,
                    ),
                    (
                        create_sprite(textures::SQUARE, 10.0, Color::rgb(0.0, 1.0, 0.0), draw_layers::PLAYER),
                        Health::new(3, 20, Some(Color::RED)),
                        Physics::default(),
                        Player {},
                    ),
                );

                physics_world.create_body(
                    &mut entities,
                    &mut physics_bodies,
                    player, 
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

                // Stationary circle to take dmg from
                let circle = entities.add_entity((&mut sprites, &mut physicses), (
                    create_sprite(textures::ASTEROID, 40.0, Color::BLACK, draw_layers::WALL),
                    Physics::default(),
                ));

                physics_world.create_body(
                    &mut entities,
                    &mut physics_bodies,
                    circle, 
                    Transform::new(-440.0, -160.0),
                    CollisionBody::from_collider(Collider::circle(40.0, layers::WALL, 0))
                );

                // Stationary square to take dmg from
                let square = entities.add_entity((&mut sprites, &mut physicses), (
                    create_sprite(textures::SQUARE, 40.0, Color::BLACK, draw_layers::WALL),
                    Physics::default(),
                ));

                physics_world.create_body(
                    &mut entities,
                    &mut physics_bodies,
                    square, 
                    Transform::new(160.0, -160.0),
                    CollisionBody::from_collider(Collider::half_extents(40.0, 40.0, layers::WALL, 0))
                );
            },
        );

        Ok(GameState { world })
    }

    fn handle_input(&mut self, ctx: &Context) {
        self.world.run_with_data(|
            ctx: &Context,
            mut game: UniqueViewMut<AsteroidGame>, 
            physics_bodies: View<PhysicsBody>,
            players: View<Player>, 
            physics_world: UniqueView<PhysicsWorld>,
            camera: UniqueView<Camera>, | {
                game.move_right = input::is_key_down(ctx, Key::D);
                game.move_left = input::is_key_down(ctx, Key::A);
                game.move_up = input::is_key_down(ctx, Key::W);
                game.move_down = input::is_key_down(ctx, Key::S);
                game.lmb_down = input::is_mouse_button_down(ctx, MouseButton::Left);
                if game.lmb_down {
                    let body = match (&physics_bodies, &players).iter().with_id().next() {
                        Some((id, _)) => id,
                        _ => return,
                    };
        
                    let transform = physics_world.transform(body);
        
                    let pos = camera.mouse_position(ctx);

                    let angle = transform.get_angle_to(pos.x as f64, pos.y as f64);
                    game.shoot_angle = angle;
                }
            }, ctx);
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

impl State<Res> for DeadState {
    fn update(&mut self, ctx: &mut Context, _resources: &mut Res) -> Result<Trans<Res>> {
        if input::is_key_down(ctx, Key::Space) {
            return Ok(Trans::Switch(Box::new(GameState::new(ctx)?)));
        }

        Ok(Trans::None)
    }

    fn draw(&mut self, ctx: &mut Context, _resources: &mut Res) -> Result {
        graphics::clear(ctx, Color::rgb(0.45, 0.65, 1.0));

        Ok(())
    }
}
