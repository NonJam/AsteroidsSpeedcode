//use std::error::Error;
use tetra::graphics::{self, Color};
use tetra::{Context, ContextBuilder, State};

//type WithResult<T> = Result<T, Box<dyn Error>>;


struct GameState;

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

fn main() -> tetra::Result {
  ContextBuilder::new("Hello, world!", 1280, 720)
      .build()?
      .run(|_| Ok(GameState))
}

fn overlaps(x1: f64, y1: f64, r1: f64, x2: f64, y2: f64, r2: f64) -> bool {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    (dx * dx + dy * dy).sqrt() > r1 + r2
}
