use tetra::graphics::{self, Color, Texture};
use tetra::{Context, ContextBuilder, State, input::{self, Key, MouseButton, get_mouse_position }};
use tetra::math::Vec2;
use tetra::graphics::DrawParams;

use rand::prelude::*;

struct Physics2D {
    x: f64,
    y: f64,
    r: f64,
    dx: f64,
    dy: f64,
    speed: Option<f64>,
    angle: Option<f64>,
    accel: Option<f64>,
    delete: bool
}

impl Physics2D {
    fn new(x: f64, y: f64, radius: f64) -> Self {
        let mut blah = Physics2D::default();
        blah.x = x;
        blah.y = y;
        blah.r = radius;
        blah
    }

    fn apply_physics(&mut self) {
        if let (Some(speed), Some(angle)) = (self.speed, self.angle) {
            if let Some(accel) = self.accel {
                self.speed = Some(speed + accel)
            }

            self.dx = angle.to_radians().sin() * speed;
            self.dy = -angle.to_radians().cos() * speed;
        }

        self.x += self.dx;
        self.y += self.dy;

        if self.x < -1000f64 || self.x > 2280f64 || self.y < -1000f64 || self.y > 1720f64 {
            self.delete = true
        }
    }

    fn get_angle_to(&self, x: f64, y: f64) -> f64 {
        let result = (self.y - y).to_radians().atan2((self.x - x).to_radians()).to_degrees();
        if result < 0f64 {
            (result + 630f64) % 360f64
        }
        else {
            (result + 270f64) % 360f64
        }
    }

    fn collides_with(&self, other: &Physics2D) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        (dx * dx + dy * dy).sqrt() < self.r + other.r
    }
}

impl Default for Physics2D {
    fn default() -> Self {
        Physics2D {
            x: 0f64,
            y: 0f64,
            r: 0f64,
            dx: 0f64,
            dy: 0f64,
            speed: None,
            angle: None,
            accel: None,
            delete: false
        }
    }
}

struct GameState {
    game_over: bool,
    rand: ThreadRng,
    asteroid_timer: i32,
    asteroid_tex: Texture,
    asteroids: Vec<Physics2D>,
    bullets: Vec<Physics2D>,
    player: Physics2D,
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.game_over {
            if input::is_key_down(ctx, Key::Space) {
                self.reset_state();
            }

            return Ok(());
        }

        player_input(ctx, &mut self.player, &mut self.bullets);
        self.player.apply_physics();

        self.asteroid_spawning();

        for asteroid in self.asteroids.iter_mut() {
            asteroid.apply_physics();
        }
        for bullet in self.bullets.iter_mut() {
            bullet.apply_physics();

            for asteroid in self.asteroids.iter_mut() {
                if bullet.collides_with(asteroid) {
                    asteroid.r = 0f64.max(asteroid.r - 2f64);
                    if asteroid.r <= 0f64 {
                        asteroid.delete = true
                    }
                    bullet.delete = true
                }
            }
        }

        for asteroid in self.asteroids.iter() {
            if self.player.collides_with(asteroid) {
                self.game_over = true;
            }
        }
        if self.game_over {
            self.asteroids.clear();
            self.bullets.clear();
        }
        
        wrap_bodies(&mut self.asteroids);
        wrap_body(&mut self.player);

        self.asteroids.retain(|a| !a.delete);
        self.bullets.retain(|b| !b.delete);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        for ast in self.asteroids.iter() {
            self.draw_asteroid(ctx, ast.x, ast.y, ast.r);
        }

        for bullet in self.bullets.iter() {
            self.draw_asteroid(ctx, bullet.x, bullet.y, bullet.r);
        }

        self.draw_asteroid(ctx, self.player.x, self.player.y, self.player.r);

        Ok(())
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            game_over: false,
            rand: rand::thread_rng(),
            asteroid_timer: 0,
            asteroids: vec![],
            bullets: vec![],
            asteroid_tex: Texture::new(ctx, "asteroid.png")?,
            player: Physics2D::new(640f64, 360f64, 10f64),
        })
    }

    /// # This draws an asteroid at position 500,500 that has a radius of 100
    /// 
    /// self.draw_asteroid(ctx, 500f64, 500f64, 100f64);
    fn draw_asteroid(&self, ctx: &mut Context, x: f64, y: f64, r: f64) {
        let scale = (r / 1024f64) * 2f64;
        let params = DrawParams::new()
            .position(Vec2::new(x as f32, y as f32))
            .scale(Vec2::new(scale as f32, scale as f32))
            .origin(Vec2::new(512f32, 512f32));
    
        graphics::draw(ctx, &self.asteroid_tex, params);
    }

    fn asteroid_spawning(&mut self) {
        self.asteroid_timer += 1;
        while self.asteroid_timer > 50 {
            self.asteroid_timer -= 50;

            // Timer proc
            let (x, y) = {
                let x: i32;
                let y: i32;

                // Align vertically
                if rand::random() {
                    // Left
                    if rand::random() {
                        x = 0;
                    }
                    // Right 
                    else {
                        x = 1279;
                    }

                    y = self.rand.gen_range(0, 720);
                } 
                // Align horizontally
                else {
                    // Top
                    if rand::random() {
                        y = 0;
                    }
                    // Bottom
                    else {
                        y = 719;
                    }

                    x = self.rand.gen_range(0, 1280);
                }

                (x, y)
            };

            let mut ast = Physics2D::new(x as f64, y as f64, self.rand.gen_range(10f64, 100f64));
            let mut angle = ast.get_angle_to(640f64, 360f64);
            angle += self.rand.gen_range(-22f64, 22f64);
            ast.angle = Some(angle);
            ast.speed = Some(self.rand.gen_range(5f64, 10f64));

            self.asteroids.push(ast);
        }
    }

    fn reset_state(&mut self) {
        self.game_over = false;
        self.asteroid_timer = 0;
        self.asteroids = vec![];
        self.bullets = vec![];
        self.player = Physics2D::new(640f64, 360f64, 10f64);
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Hello, world!", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(GameState::new)
}

fn player_input(ctx: &mut Context, player: &mut Physics2D, bullets: &mut Vec<Physics2D>) {
    let mut input = Vec2::<i32>::default();
    if input::is_key_down(ctx, Key::A) {
        input.x -= 1;
    }
    if input::is_key_down(ctx, Key::D) {
        input.x += 1;
    }
    if input::is_key_down(ctx, Key::W) {
        input.y -= 1;
    }
    if input::is_key_down(ctx, Key::S) {
        input.y += 1;
    }

    let mut input = Vec2::new(input.x as f64, input.y as f64);
    if input != Vec2::new(0f64, 0f64) {
        input.normalize();
        input = input * 5f64;
    }

    player.dx = input.x;
    player.dy = input.y;


    if input::is_mouse_button_down(ctx, MouseButton::Left) {
        let pos = get_mouse_position(ctx);
        let angle = player.get_angle_to(pos.x as f64, pos.y as f64);
        bullets.push(Physics2D {
            x: player.x, 
            y: player.y,
            r: 6f64,
            speed: Some(10f64),
            accel: Some(1f64),
            angle: Some(angle),
            ..Physics2D::default()
        });
    }
}

fn wrap_bodies(bodies: &mut Vec<Physics2D>) {
    for body in bodies.iter_mut() {
        wrap_body(body);
    }
}

fn wrap_body(body: &mut Physics2D) {
    if body.x > 1280f64 + body.r {
        body.x = -(body.x - 1280f64);
    }
    else if body.x < -body.r {
        body.x = 1280f64 + (-body.x);
    }

    if body.y > 720f64 + body.r {
        body.y = -(body.y - 720f64);
    }
    else if body.y < 0f64 - body.r {
        body.y = 720f64 + (-body.y);
    }
}