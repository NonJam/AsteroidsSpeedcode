use tetra::graphics::{self, Color, Texture};
use tetra::{Context, ContextBuilder, State, Trans, Result, input::{self, Key, MouseButton, get_mouse_position }};
use tetra::math::Vec2;
use tetra::graphics::DrawParams;
use legion::prelude::*;

use rand::{SeedableRng};
use rand::rngs::StdRng;

mod components;
use components::*;
mod systems;
use systems::*;


type Res = (Resources, Textures);

struct AsteroidGame {
    asteroid_timer: i32
}

fn main() -> tetra::Result {
    ContextBuilder::new("Asteroids", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(GameState::new, |ctx| { 
            let mut res = Resources::default();
            res.insert(AsteroidGame {
                asteroid_timer: 50i32
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
    systems: Executor
}

impl State<Res> for GameState {
    fn update(&mut self, ctx: &mut Context, resources: &mut Res) -> Result<Trans<Res>> {

        self.handle_input(ctx);
        self.systems.execute(&mut self.world, &mut resources.0);

        if self.player_is_dead() {
            return Ok(Trans::Switch(Box::new(DeadState)))
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

        world.insert((Player,), vec![
            (Transform::new(640f64, 360f64, 10f64),)
        ]);

        let systems = Executor::new(vec![
            spawn_asteroids(),
            apply_physics(),
            wrap_asteroids(),
            destroy_offscreen(),
            bullet_collision(),
            split_asteroids()
        ]);

        Ok(GameState {
            world,
            systems
        })
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
                ));
            }
        }

        for data in create_bullets.into_iter() {
            self.create_bullet(data.0, data.1);
        }
    }

    fn player_is_dead(&self) -> bool {
        let players = <(Read<Transform>, Tagged<Player>)>::query();
        let asteroids = <(Read<Transform>, Tagged<Asteroid>)>::query();

        for (player, _) in players.iter(&self.world) {
            for (asteroid, _) in asteroids.iter(&self.world) {

                if player.collides_with(*asteroid) {
                    return true
                }
            }
        }

        return false
    }

    fn render(&self, ctx: &mut Context, resources: &mut Res) {
        let query = <(Read<Transform>,)>::query();

        for (renderable,) in query.iter(&self.world) {

            let scale = renderable.r / 1024f64 * 2f64;

            let params = DrawParams::new()
                .position(Vec2::new(renderable.x as f32, renderable.y as f32))
                .scale(Vec2::new(scale as f32, scale as f32))
                .origin(Vec2::new(512f32, 512f32));
    
            graphics::draw(ctx, &resources.1.asteroid, params);
        }
    }

    fn create_bullet(&mut self, transform: Transform, physics: Physics) {
        self.world.insert((Bullet,), vec![
            (transform,physics)
        ]);
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