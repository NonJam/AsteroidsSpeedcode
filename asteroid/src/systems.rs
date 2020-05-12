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
    .build(move |commands, _world, (rand, game), _queryies|{

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

pub fn apply_physics() -> System {
    SystemBuilder::<()>::new("Physics System")
        .with_query(<(Write<Transform>, Write<Physics>)>::query())
        .build(move |_commands, world, _resource, query|{

            for (mut transform, mut physics) in query.iter_mut(world) {
                
                physics.speed += physics.accel;
                physics.angle += physics.curve;
    
                transform.x += physics.dx + physics.angle.to_radians().sin() * physics.speed;
                transform.y += physics.dy - physics.angle.to_radians().cos() * physics.speed;
            }
        })
}

pub fn wrap_asteroids() -> System {
    SystemBuilder::<()>::new("Wrap Asteroids System")
        .with_query(<(Write<Transform>, Tagged<Asteroid>)>::query())
        .build(move |_commands, world, _resource, query|{

            for (mut asteroid, _) in query.iter_mut(world) {
        
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
        })
}

pub fn destroy_offscreen() -> System {
    SystemBuilder::<()>::new("Destroy Offscreen System")
        .with_query(<(Read<Transform>,)>::query())
        .build(move |commands, world, _resource, query|{

            for (entity,(transform,)) in query.iter_entities(world) {
                if transform.x < -1000f64 || transform.x > 2280f64 || transform.y < -1000f64 || transform.y > 1720f64 {
                    commands.delete(entity)
                }
            }
        })
}

pub fn bullet_collision() -> System {
    SystemBuilder::<()>::new("Bullet Collision System")
        .with_query(<(Read<Transform>, Read<Physics>, Tagged<Bullet>)>::query())
        .with_query(<(Read<Transform>, Tagged<Asteroid>)>::query())
        .build(move |commands, world, _resource, (bullets, asteroids)| {

            for (bullet, (bullet_t, bullet_p, _)) in bullets.iter_entities(world) {
                for (asteroid, (asteroid_t, _)) in asteroids.iter_entities(world) {

                    if bullet_t.collides_with(*asteroid_t) {
                        commands.add_component(asteroid, Collision{ angle: bullet_p.angle });
                        commands.delete(bullet);
                    }
                }
            }
        })
}

pub fn split_asteroids() -> System {
    SystemBuilder::<()>::new("Split Asteroids System")
        .with_query(<(Write<Transform>, Write<Physics>, Read<Collision>, Tagged<Asteroid>)>::query())
        .write_resource::<StdRng>()
        .build(move |commands, world, rand, asteroids| {

            for (asteroid, (mut asteroid_t, mut asteroid_p, collision, _)) in asteroids.iter_entities_mut(world) {
                commands.remove_component::<Collision>(asteroid);

                asteroid_t.r = asteroid_t.r / 1.5f64;

                if asteroid_t.r < 15f64 {
                    commands.delete(asteroid);
                } 
                else {
                    let mut new_physics = Physics { ..*asteroid_p };
                    asteroid_p.angle = collision.angle - rand.gen_range(0f64, 140f64);
                    new_physics.angle = collision.angle + rand.gen_range(0f64, 140f64);

                    create_asteroid(commands, *asteroid_t, new_physics);
                }
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