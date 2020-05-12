use legion::prelude::*;
use rand::Rng;
use rand::rngs::StdRng;

use crate::AsteroidGame;
use crate::components::*;

type System = Box<dyn Schedulable>;

pub fn spawn_asteroids() -> System {
    SystemBuilder::<()>::new("Spawn Asteroids System")
    .write_resource::<StdRng>()
    .write_resource::<AsteroidGame>()
    .build(move |commands, world, (rand, game), _|{

        game.asteroid_timer += 1;
        while game.asteroid_timer > 50 {
            game.asteroid_timer -= 50;

            // Timer proc
            let radius = rand.gen_range(10f64, 100f64);
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

                    y = rand.gen_range(0f64 - radius, 720f64 + radius);
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

                    x = rand.gen_range(0f64 - radius, 1280f64 + radius);
                }

                (x, y)
            };

            let transform = Transform::new(x as f64, y as f64, radius);
            let mut angle = transform.get_angle_to(640f64, 360f64);
            angle += rand.gen_range(-22f64, 22f64);
            let speed = rand.gen_range(5f64, 10f64);

            create_asteroid(
                commands,
                transform,
                Physics {
                    speed,
                    angle,
                    ..Physics::default() 
                },
            );
        }
    })
}


//
// Helpers
fn create_asteroid(commands: &mut CommandBuffer, transform: Transform, physics: Physics) {
    commands.insert((Asteroid,), vec![
        (transform,physics)
    ]);
}