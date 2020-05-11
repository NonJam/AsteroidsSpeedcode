use tetra::graphics::{self, Color, Texture};
use tetra::{Context, ContextBuilder, State, Trans, Result, input::{self, Key, MouseButton, get_mouse_position }};
use tetra::math::Vec2;
use tetra::graphics::DrawParams;
use legion::prelude::*;

use rand::prelude::*;

mod components;
use components::*;

type Res = (Resources, Textures);

fn main() -> tetra::Result {
    ContextBuilder::new("Asteroids", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(GameState::new, |ctx| { 
            let res = Resources::default();
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
    rand: ThreadRng,
    asteroid_timer: i32
}

impl State<Res> for GameState {
    fn update(&mut self, ctx: &mut Context, _resources: &mut Res) -> Result<Trans<Res>> {

        self.handle_input(ctx);
        self.spawn_asteroids();
        self.apply_physics();
        self.wrap_asteroids();
        self.destroy_offscreen();
        self.destroy_asteroids();
        self.clean_up();

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

        Ok(GameState {
            rand: rand::thread_rng(),
            asteroid_timer: 0,
            world
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

    fn spawn_asteroids(&mut self) {
        self.asteroid_timer += 1;
        while self.asteroid_timer > 50 {
            self.asteroid_timer -= 50;

            // Timer proc
            let radius = self.rand.gen_range(10f64, 100f64);
            let (x, y) = {
                let x: f64;
                let y: f64;

                // Align vertically
                if rand::random() {
                    // Left
                    if rand::random() {
                        x = 0f64 - radius;
                    }
                    // Right 
                    else {
                        x = 1279f64 + radius;
                    }

                    y = self.rand.gen_range(0f64 - radius, 720f64 + radius);
                } 
                // Align horizontally
                else {
                    // Top
                    if rand::random() {
                        y = 0f64 - radius;
                    }
                    // Bottom
                    else {
                        y = 719f64 + radius;
                    }

                    x = self.rand.gen_range(0f64 - radius, 1280f64 + radius);
                }

                (x, y)
            };

            let transform = Transform::new(x as f64, y as f64, radius);
            let mut angle = transform.get_angle_to(640f64, 360f64);
            angle += self.rand.gen_range(-22f64, 22f64);
            let speed = self.rand.gen_range(5f64, 10f64);

            self.create_asteroid(
                transform,
                Physics {
                    speed,
                    angle,
                    ..Physics::default() 
                },
            );
        }
    }

    fn apply_physics(&mut self) {
        let  query = <(Write<Transform>, Write<Physics>)>::query();

        for (mut transform, mut physics) in query.iter_mut(&mut self.world) {
            physics.speed += physics.accel;
            physics.angle += physics.curve;

            transform.x += physics.dx;
            transform.y += physics.dy;

            transform.x += physics.angle.to_radians().sin() * physics.speed;
            transform.y -= physics.angle.to_radians().cos() * physics.speed;
        }
    }

    fn wrap_asteroids(&mut self) {
        let asteroids = <(Write<Transform>, Tagged<Asteroid>)>::query();

        for (mut asteroid, _) in asteroids.iter_mut(&mut self.world) {

            // Wrap X
            if asteroid.x > 1280f64 + asteroid.r {
                asteroid.x = -(asteroid.x - 1280f64);
            }
            else if asteroid.x < -asteroid.r {
                asteroid.x = 1280f64 + (-asteroid.x);
            }

            // Wrap Y
            if asteroid.y > 720f64 + asteroid.r {
                asteroid.y = -(asteroid.y - 720f64);
            }
            else if asteroid.y < 0f64 - asteroid.r {
                asteroid.y = 720f64 + (-asteroid.y);
            }
        }
    }

    fn destroy_offscreen(&mut self) {
        let query = <(Read<Transform>,)>::query();

        let mut add_tag = vec![];

        for (entity,(transform,)) in query.iter_entities(&self.world) {
            if transform.x < -1000f64 || transform.x > 2280f64 || transform.y < -1000f64 || transform.y > 1720f64 {
                add_tag.push(entity)
            }
        }

        for data in add_tag.into_iter() {
            self.world.add_tag(data, Delete).ok();
        }
    }

    fn destroy_asteroids(&mut self) {
        let bullets = <(Read<Transform>, Read<Physics>, Tagged<Bullet>)>::query();
        let asteroids = <(Read<Transform>, Read<Physics>, Tagged<Asteroid>)>::query();
        
        let mut collisions = vec![];

        for (bullet, (bullet_t, _, _)) in bullets.iter_entities(&self.world) {
            for (asteroid, (asteroid_t, _, _)) in asteroids.iter_entities(&self.world) {
                if bullet_t.collides_with(*asteroid_t) {
                    collisions.push((bullet, asteroid));
                }
            }
        }

        let mut set_angle = vec![];
        let mut create_asteroid = vec![];
        for (bullet, ast) in collisions.into_iter() {
            self.world.add_tag(bullet, Delete).ok();

            {
                let mut asteroid_t = self.world.get_component_mut::<Transform>(ast).unwrap();
                asteroid_t.r = asteroid_t.r / 1.5f64;
            }

            if self.world.get_component::<Transform>(ast).unwrap().r < 15f64 {
                self.world.add_tag(ast, Delete).ok();
                continue;
            } else {
                    let asteroid_t = self.world.get_component::<Transform>(ast).unwrap();
                    let asteroid_p = self.world.get_component::<Physics>(ast).unwrap();
                    let bullet_p = self.world.get_component::<Physics>(bullet).unwrap();

                    let mut new_physics = Physics { ..*asteroid_p };
                    set_angle.push((ast, bullet_p.angle - self.rand.gen_range(0f64, 140f64)));
                    new_physics.angle = bullet_p.angle + self.rand.gen_range(0f64, 140f64);

                    create_asteroid.push(((*asteroid_t).clone(), new_physics));
            }
        }

        for (ast, physics) in create_asteroid.into_iter() {
            self.create_asteroid(ast, physics);
        }

        for (ast, desired) in set_angle.into_iter() {
            self.world.get_component_mut::<Physics>(ast).unwrap().angle = desired;
        }
    }

    fn clean_up(&mut self) {
        let query = <(Tagged<Delete>,)>::query();

        let mut delete = vec![];

        for (entity, _) in query.iter_entities(&self.world) {
            delete.push(entity);
        }

        for entity in delete.into_iter() {
            self.world.delete(entity);

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

    fn create_asteroid(&mut self, transform: Transform, physics: Physics) {
        self.world.insert((Asteroid,), vec![
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