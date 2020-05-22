use vermarine_lib::*;

use shipyard::*;
use tetra::graphics::DrawParams;
use tetra::graphics::{self, Color, Texture};
use tetra::math::Vec2;
use tetra::{
    input::{self, get_mouse_position, Key, MouseButton},
    Context, ContextBuilder, Result, State, Trans,
};

use rand::rngs::StdRng;
use rand::SeedableRng;

mod components;
use components::*;
mod systems;
use systems::*;

type Res = Textures;

pub mod layers {
    pub const PLAYER: u64 = 1;
    pub const ENEMY: u64 = 1 << 1;
    pub const ASTEROID: u64 = 1 << 2;
    pub const BULLET_PLAYER: u64 = 1 << 3;
    pub const BULLET_ENEMY: u64 = 1 << 4;
    pub const WALL: u64 = 1 << 5;
}

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
        .run(GameState::new, |ctx| Ok(Textures::new(ctx)?))
}

struct Textures {
    asteroid: Texture,
    square: Texture,
}

impl Textures {
    fn new(ctx: &mut Context) -> Result<Self> {
        Ok(Textures {
            asteroid: Texture::new(ctx, "asteroid.png")?,
            square: Texture::new(ctx, "square.png")?,
        })
    }
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
        self.render(ctx, resources);

        Ok(())
    }
}

impl GameState {
    fn new(_ctx: &mut Context) -> Result<GameState> {
        let mut world = World::new();
        world.add_unique(AsteroidGame::new(50i32, 50i32));
        world.add_unique(StdRng::from_entropy());

        physics_workload(&mut world);

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
            .build();

        world.run(
            |mut entities: EntitiesViewMut,
             mut renderables: ViewMut<Renderable>,
             mut healths: ViewMut<Health>,
             mut physicses: ViewMut<Physics>,
             mut players: ViewMut<Player>,
             mut physics_bodies: ViewMut<PhysicsBody>,
             mut physics_world: UniqueViewMut<PhysicsWorld>| {
                // Player
                let player = entities.add_entity(
                    (
                        &mut renderables,
                        &mut healths,
                        &mut physicses,
                        &mut players,
                    ),
                    (
                        Renderable::new_sprite(
                            "square",
                            tetra::graphics::Color::rgb(0.0, 1.0, 0.0),
                            10f64,
                        ),
                        Health::new(3, 20, Some(Color::RED)),
                        Physics::default(),
                        Player {},
                    ),
                );

                physics_world.create_body(
                    &mut entities,
                    &mut physics_bodies,
                    player, 
                    Transform::new(640f64, 360f64),
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
                let circle = entities.add_entity((&mut renderables, &mut physicses), (
                    Renderable::new_sprite("asteroid", Color::BLACK, 40.0),
                    Physics::default(),
                ));

                physics_world.create_body(
                    &mut entities,
                    &mut physics_bodies,
                    circle, 
                    Transform::new(200.0, 200.0),
                    CollisionBody::from_collider(Collider::circle(40.0, layers::WALL, 0))
                );

                // Stationary square to take dmg from
                let square = entities.add_entity((&mut renderables, &mut physicses), (
                    Renderable::new_sprite("square", Color::BLACK, 40.0),
                    Physics::default(),
                ));

                physics_world.create_body(
                    &mut entities,
                    &mut physics_bodies,
                    square, 
                    Transform::new(800.0, 200.0),
                    CollisionBody::from_collider(Collider::half_extents(40.0, 40.0, layers::WALL, 0))
                );
            },
        );

        Ok(GameState { world })
    }

    fn handle_input(&mut self, ctx: &Context) {
        let (mut game, physics_bodies, players, mut physics_world) =
            self.world
                .borrow::<(UniqueViewMut<AsteroidGame>, View<PhysicsBody>, View<Player>, UniqueViewMut<PhysicsWorld>)>();
        game.move_right = input::is_key_down(ctx, Key::D);
        game.move_left = input::is_key_down(ctx, Key::A);
        game.move_up = input::is_key_down(ctx, Key::W);
        game.move_down = input::is_key_down(ctx, Key::S);
        game.lmb_down = input::is_mouse_button_down(ctx, MouseButton::Left);
        if game.lmb_down {
            let body = match (&physics_bodies, &players).iter().next() {
                Some(p) => p.0,
                _ => return,
            };

            let transform = body.transform(&mut physics_world);

            let pos = get_mouse_position(ctx);
            let angle = transform.get_angle_to(pos.x as f64, pos.y as f64);
            game.shoot_angle = angle;
        }
    }

    fn player_is_dead(&self) -> bool {
        let players = self.world.borrow::<View<Player>>();
        match players.iter().next() {
            Some(_) => return false,
            _ => return true,
        };
    }

    fn render(&self, ctx: &mut Context, resources: &mut Res) {
        let renderables = self.world.run(get_renderables);

        for (transform, renderable, health) in renderables.into_iter() {
            let mut color = renderable.color;

            if let Some(health) = health {
                if health.iframe_count > 0 && health.iframe_col.is_some() {
                    color = health.iframe_col.unwrap();
                }
            }

            let scale = renderable.r / 1024f64 * 2f64;

            let params = DrawParams::new()
                .position(Vec2::new(transform.x as f32, transform.y as f32))
                .scale(Vec2::new(scale as f32, scale as f32))
                .origin(Vec2::new(512f32, 512f32))
                .color(color);

            if renderable.sprite == "asteroid" {
                graphics::draw(ctx, &resources.asteroid, params);
            } else if renderable.sprite == "square" {
                graphics::draw(ctx, &resources.square, params);
            }
        }
    }
}

fn get_renderables(
    physics_bodies: View<PhysicsBody>,
    renderables: View<Renderable>,
    health: View<Health>,
    physics_world: UniqueViewMut<PhysicsWorld>,
) -> Vec<(Transform, Renderable, Option<Health>)> {
    let mut output = vec![];
    for (e, (body, renderable)) in (&physics_bodies, &renderables).iter().with_id() {
        let health = if health.contains(e) {
            Some(health.get(e).ok().unwrap().clone())
        } else {
            None
        };
        let body = physics_world.transform(body);
        output.push((*body, *renderable, health));
    }
    output
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
