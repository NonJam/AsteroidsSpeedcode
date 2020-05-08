//use std::error::Error;
use tetra::graphics::{self, Color, Texture};
use tetra::{Context, ContextBuilder, State};

//type WithResult<T> = Result<T, Box<dyn Error>>;

struct GameState {
    asteroid_tex: Texture,
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        Ok(())
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
           asteroid_tex: Texture::new(ctx, "asteroid.png")?,
        })
    }

    /// # This draws an asteroid at position 500,500 that has a radius of 100
    /// 
    /// self.draw_asteroid(ctx, 500f64, 500f64, 100f64);
    fn draw_asteroid(&self, ctx: &mut Context, x: f64, y: f64, r: f64) {
        use tetra::math::Vec2;
        use tetra::graphics::DrawParams;
        let scale = r / 1024f64;
        let params = DrawParams::new()
            .position(Vec2::new(x as f32, y as f32))
            .scale(Vec2::new(scale as f32, scale as f32))
            .origin(Vec2::new(512f32, 512f32));
    
        graphics::draw(ctx, &self.asteroid_tex, params);
    }
}

fn main() -> tetra::Result {
  ContextBuilder::new("Hello, world!", 1280, 720)
      .build()?
      .run(GameState::new)
}

fn overlaps(x1: f64, y1: f64, r1: f64, x2: f64, y2: f64, r2: f64) -> bool {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    (dx * dx + dy * dy).sqrt() > r1 + r2
}