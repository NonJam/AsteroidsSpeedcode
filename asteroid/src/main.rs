use legion::prelude::*;
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

type Res = (Resources, Textures);

struct AsteroidGame {
    asteroid_timer: i32,
}

fn main() -> tetra::Result {
    ContextBuilder::new("Asteroids", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(GameState::new, |ctx| {
            let mut res = Resources::default();
            res.insert(AsteroidGame {
                asteroid_timer: 50i32,
            });
            res.insert(StdRng::from_entropy());
            Ok((res, Textures::new(ctx)?))
        })
}

struct Textures {
    asteroid: Texture,
}

impl Textures {
    fn new(ctx: &mut Context) -> Result<Self> {
        Ok(Textures {
            asteroid: Texture::new(ctx, "asteroid.png")?,
        })
    }
}

struct GameState {
    world: World,
    systems: Executor,
}

impl State<Res> for GameState {
    fn update(&mut self, ctx: &mut Context, resources: &mut Res) -> Result<Trans<Res>> {
        self.handle_input(ctx);
        self.systems.execute(&mut self.world, &mut resources.0);

        if self.player_is_dead() {
            return Ok(Trans::Switch(Box::new(DeadState)));
        }
        Ok(Trans::None)
    }

    fn draw(&mut self, ctx: &mut Context, resources: &mut Res) -> tetra::Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        self.render(ctx, resources);

        Ok(())
    }
}

impl GameState {
    fn new(_ctx: &mut Context) -> tetra::Result<GameState> {
        let mut world = World::new();

        world.insert(
            (Player,),
            vec![(
                Transform::new(640f64, 360f64, 10f64),
                Renderable::new(tetra::graphics::Color::rgb(0.0, 1.0, 0.0)),
                Health::new(3, 20, Some(Color::RED)),
                Physics::default(),
            )],
        );

        let systems = Executor::new(vec![
            iframe_counter(),
            spawn_asteroids(),
            apply_physics(),
            wrap_asteroids(),
            destroy_offscreen(),
            bullet_collision(),
            player_collision(),
            asteroid_collision(),
            split_asteroids(),
        ]);

        Ok(GameState { world, systems })
    }

    fn handle_input(&mut self, ctx: &Context) {
        let query = <(Write<Transform>, Tagged<Player>)>::query();

        let mut create_bullets = vec![];

        for (mut player, _) in query.iter_mut(&mut self.world) {
            // Movement
            if input::is_key_down(ctx, Key::W) {
                player.y -= 5f64;
            }
            if input::is_key_down(ctx, Key::A) {
                player.x -= 5f64;
            }
            if input::is_key_down(ctx, Key::S) {
                player.y += 5f64;
            }
            if input::is_key_down(ctx, Key::D) {
                player.x += 5f64;
            }

            // Shooting
            if input::is_mouse_button_down(ctx, MouseButton::Left) {
                let pos = get_mouse_position(ctx);
                let angle = player.get_angle_to(pos.x as f64, pos.y as f64);

                create_bullets.push((
                    Transform {
                        x: player.x,
                        y: player.y,
                        r: 6f64,
                        ..Transform::default()
                    },
                    Physics {
                        speed: 10f64,
                        accel: 1f64,
                        angle,
                        ..Physics::default()
                    },
                    Renderable {
                        color: Color::rgba(0.02, 0.23, 0.81, 0.5),
                    },
                    Bullet::new(Team::Player),
                ));
            }
        }

        for data in create_bullets.into_iter() {
            self.world.insert((), vec![data]);
        }
    }

    fn player_is_dead(&self) -> bool {
        let players = <(Read<Transform>, Tagged<Player>)>::query();

        for _ in players.iter(&self.world) {
            return false;
        }

        true
    }

    fn render(&self, ctx: &mut Context, resources: &mut Res) {
        let query = <(Read<Transform>,)>::query();

        for (entity, (transform,)) in query.iter_entities(&self.world) {
            let mut color = if let Some(renderable) = self.world.get_component::<Renderable>(entity)
            {
                renderable.color
            } else {
                Color::BLACK
            };

            if let Some(health) = self.world.get_component::<Health>(entity) {
                if health.iframe_count > 0 && health.iframe_col.is_some() {
                    color = health.iframe_col.unwrap();
                }
            }

            let scale = transform.r / 1024f64 * 2f64;

            let params = DrawParams::new()
                .position(Vec2::new(transform.x as f32, transform.y as f32))
                .scale(Vec2::new(scale as f32, scale as f32))
                .origin(Vec2::new(512f32, 512f32))
                .color(color);

            graphics::draw(ctx, &resources.1.asteroid, params);
        }
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
