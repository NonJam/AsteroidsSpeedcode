use std::error::Error;

type WithResult<T> = Result<T, Box<dyn Error>>;

fn main() -> WithResult<()> {
    println!("Hello, world!");
    Ok(())
}
