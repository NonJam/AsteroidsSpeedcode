use rand::rngs::StdRng;
use rand::Rng;
use tetra::graphics::Color;
use shipyard::*;

use crate::components::*;
use crate::AsteroidGame;

pub fn spawn_asteroids(mut entities: EntitiesViewMut, mut rand: UniqueViewMut<StdRng>, mut game: UniqueViewMut<AsteroidGame>, mut transforms: ViewMut<Transform>, mut physicses: ViewMut<Physics>, mut renderables: ViewMut<Renderable>, mut asteroids: ViewMut<Asteroid>) {
    game.asteroid_timer += 1;
    while game.asteroid_timer > 100 {
        game.asteroid_timer -= 100;

        let radius = rand.gen_range(10f64, 100f64);
        let (x, y) =  
            // Align vertically
            if rand::random() {(
                // Left
                if rand::random() {
                    0f64 - radius
                }
                // Right
                else {
                    1279f64 + radius
                },
                rand.gen_range(0f64 - radius, 720f64 + radius),
            )}
            // Align horizontally
            else {(
                rand.gen_range(0f64 - radius, 1280f64 + radius),
                // Top
                if rand::random() {
                    0f64 - radius
                }
                // Bottom
                else {
                    719f64 + radius
                },
            )};

        let transform = Transform::new(x as f64, y as f64, radius);
        let mut angle = transform.get_angle_to(640f64, 360f64);
        angle += rand.gen_range(-22f64, 22f64);
        let speed = rand.gen_range(5f64, 10f64);

        entities.add_entity((&mut transforms, &mut physicses, &mut renderables, &mut asteroids), (transform, Physics { speed, angle, ..Physics::default() }, Renderable { color: Color::BLACK }, Asteroid {}));
    }
}

pub fn apply_physics(mut transforms: ViewMut<Transform>, mut physicses: ViewMut<Physics>) {
    for (transform, physics) in (&mut transforms, &mut physicses).iter() {
        physics.speed += physics.accel;
        physics.angle += physics.curve;

        transform.x += physics.dx + physics.angle.to_radians().sin() * physics.speed;
        transform.y += physics.dy - physics.angle.to_radians().cos() * physics.speed;
    }
}


pub fn wrap_asteroids(mut transforms: ViewMut<Transform>, asteroids: View<Asteroid>) {
    for (asteroid, _) in (&mut transforms, &asteroids).iter() {
        // Wrap X
        if asteroid.x > 1280f64 + asteroid.r {
            asteroid.x = -(asteroid.x - 1280f64);
        } else if asteroid.x < -asteroid.r {
            asteroid.x = 1280f64 + (-asteroid.x);
        }

        // Wrap Y
        if asteroid.y > 720f64 + asteroid.r {
            asteroid.y = -(asteroid.y - 720f64);
        } else if asteroid.y < 0f64 - asteroid.r {
            asteroid.y = 720f64 + (-asteroid.y);
        }
    }
}

pub fn wrap_player(mut transforms: ViewMut<Transform>, players: View<Player>) {
    for (player, _) in (&mut transforms, &players).iter() {
        // Wrap X
        if player.x > 1280f64 + player.r {
            player.x = -(player.x - 1280f64);
        } else if player.x < -player.r {
            player.x = 1280f64 + (-player.x);
        }

        // Wrap Y
        if player.y > 720f64 + player.r {
            player.y = -(player.y - 720f64);
        } else if player.y < 0f64 - player.r {
            player.y = 720f64 + (-player.y);
        }
    }
}

pub fn destroy_offscreen(mut all_storages: AllStoragesViewMut) {
    let mut deferred = vec![];

    {let transforms = all_storages.borrow::<ViewMut<Transform>>();
    for (e, _) in (&transforms).iter().with_id()
        .filter(|(_, transform)| {
            transform.x < -1000f64 ||
            transform.x > 2280f64 ||
            transform.y < -1000f64 ||
            transform.y > 1720f64
        }) {
        deferred.push(e);
    }}

    for e in deferred.into_iter() {
        all_storages.delete(e);
    }
}

pub fn spawn_spinners(mut entities: EntitiesViewMut, mut rand: UniqueViewMut<StdRng>, mut game: UniqueViewMut<AsteroidGame>, players: View<Player>, mut transforms: ViewMut<Transform>, mut physicses: ViewMut<Physics>, mut spinners: ViewMut<Spinner>, mut renderables: ViewMut<Renderable>) {
    game.spinner_timer += 1;
    while game.spinner_timer > 400 {
        game.spinner_timer -= 400;

        // Timer proc
        let radius = 20f64;
        let (x, y) =  
            // Align vertically
            if rand::random() {(
                // Left
                if rand::random() {
                    0f64 - radius
                }
                // Right
                else {
                    1279f64 + radius
                },
                rand.gen_range(0f64 - radius, 720f64 + radius),
            )}
            // Align horizontally
            else {(
                rand.gen_range(0f64 - radius, 1280f64 + radius),
                // Top
                if rand::random() {
                    0f64 - radius
                }
                // Bottom
                else {
                    719f64 + radius
                },
            )};

        let player = match (&transforms, &players).iter().next() {
            Some(p) => p.0.clone(),
            _ => return,
        };

        let transform = Transform::new(x as f64, y as f64, radius);
        let angle = transform.get_angle_to(player.x, player.y);
        entities.add_entity((&mut spinners, &mut transforms, &mut physicses, &mut renderables), (Spinner { angle, cooldown: 0 }, transform, Physics { accel: 0.18f64, angle, ..Physics::default() }, Renderable { color: Color::BLACK }));
    }
}

pub fn shoot_spinners(mut entities: EntitiesViewMut, mut transforms: ViewMut<Transform>, mut spinners: ViewMut<Spinner>, mut bullets: ViewMut<Bullet>, mut physicses: ViewMut<Physics>, mut renderables: ViewMut<Renderable>) {
    let mut deferred = vec![];
    
    for (transform, spinner) in (&transforms, &mut spinners).iter() {
        if spinner.cooldown > 0 {
            spinner.cooldown -= 1;
        } else {
            for i in 0..4 {
                spinner.cooldown = 4;

                spinner.angle += 4f64;

                deferred.push((
                    Bullet {
                        team: Team::Ast
                    },
                    Transform {
                        r: 5f64,
                        ..*transform
                    },
                    Physics {
                        speed: 3f64,
                        angle: spinner.angle + i as f64 * 90f64,
                        ..Physics::default()
                    },
                    Renderable {
                        color: Color::BLACK,
                    },
                ));
            }
        }
    }

    for data in deferred.into_iter() {
        entities.add_entity((&mut bullets, &mut transforms, &mut physicses, &mut renderables), data);
    }
}

pub fn bullet_collision(mut all_storages: AllStoragesViewMut) {    
    let mut collisions = vec![];
    
    {let (transforms, physicses, bullets, asteroids, players) = all_storages.borrow::<(View<Transform>, View<Physics>, View<Bullet>, View<Asteroid>, View<Player>)>();
    for (entity, (transform, physics, bullet)) in (&transforms, &physicses, &bullets).iter().with_id() {
        for (entity2, (transform2, _, _)) in (&transforms, &physicses, &players).iter().with_id() {
            if entity == entity2 {
                continue;
            }
            if bullet.team == Team::Ast {
                if transform.collides_with(*transform2) {
                    collisions.push((entity, entity2, physics.angle));
                }
            }
        }
        for (entity2, (transform2, _, _)) in (&transforms, &physicses, &asteroids).iter().with_id() {
            if entity == entity2 {
                continue;
            }
            if bullet.team == Team::Player {
                if transform.collides_with(*transform2) {
                    collisions.push((entity, entity2, physics.angle));
                }
            }
        }
    }}

    for (bullet, hit_e, angle) in collisions.into_iter() {
        all_storages.delete(bullet);
        let (entities, mut collisions) = all_storages.borrow::<(EntitiesViewMut, ViewMut<Collision>)>();
        entities.add_component(&mut collisions, Collision { angle: angle }, hit_e);
    }
}

pub fn asteroid_collision(entities: EntitiesViewMut, transforms: View<Transform>, physicses: View<Physics>, asteroids: View<Asteroid>, players: View<Player>, mut collisions: ViewMut<Collision>) {
    let (p_entity, (p_transform, _, _)) = match (&transforms, &physicses, &players).iter().with_id().next() {
        Some(p) => p,
        _ => return,
    };

    for (transform, _, _) in (&transforms, &physicses, &asteroids).iter() {
        if transform.collides_with(*p_transform) {
            entities.add_component(&mut collisions, Collision { angle: 0f64 }, p_entity);
        }
    }
}

pub fn split_asteroids(mut all_storages: AllStoragesViewMut) {
    let mut deferred_delete = vec![];
    let mut deferred_create = vec![];

    {
        let (mut transforms, mut physicses, mut collisions, mut asteroids, mut rand, mut entities, mut renderables) = 
            all_storages.borrow::<(ViewMut<Transform>, ViewMut<Physics>, ViewMut<Collision>, ViewMut<Asteroid>, UniqueViewMut<StdRng>, EntitiesViewMut, ViewMut<Renderable>)>();

        for (entity, (transform, physics, collision, _)) in (&mut transforms, &mut physicses, &collisions, &asteroids).iter().with_id() {
            transform.r = transform.r / 1.5f64;

            if transform.r < 15f64 {
                deferred_delete.push(entity);
            } else {
                let mut new_physics = Physics { ..*physics };
                new_physics.angle = collision.angle + rand.gen_range(0f64, 140f64);
                physics.angle = collision.angle + rand.gen_range(0f64, 140f64);

                deferred_create.push((*transform, new_physics));
            }
        }

        for (transform, physics) in deferred_create.into_iter() {
            entities.add_entity((&mut transforms, &mut physicses, &mut asteroids, &mut renderables), (transform, physics, Asteroid {}, Renderable { color: Color::BLACK }));
        }

        let mut deferred = vec![];
        for id in (&collisions, &asteroids).iter_ids() {
            deferred.push(id);
        }
        for id in deferred.into_iter() {
            collisions.remove(id);
        }
    }

    deferred_delete.into_iter().for_each(|id| { all_storages.delete(id); });
}

pub fn player_collision(mut all_storages: AllStoragesViewMut) {
    let mut deferred_delete = vec![];
    
    {let (physicses, mut healths, mut collisions, players) =
        all_storages.borrow::<(View<Physics>, ViewMut<Health>, ViewMut<Collision>, View<Player>)>();

    let (id, (_, health, _, _)) = match (&physicses, &mut healths, &mut collisions, &players).iter().with_id().next() {
        Some(p) => p,
        _ => return,
    };

    if health.iframe_count == 0 {
        health.hp -= 1;
        health.iframe_count = health.iframe_max;
        if health.hp <= 0 {
            deferred_delete.push(id);
        }
    }

    collisions.remove(id);}

    for id in deferred_delete.into_iter() {
        all_storages.delete(id);
    }
}

pub fn iframe_counter(mut healths: ViewMut<Health>) {
    for health in (&mut healths)
        .iter()
        .filter(|health| health.iframe_count > 0) {
        health.iframe_count -= 1;
    }
}

pub fn player_input(mut entities: EntitiesViewMut, game: UniqueViewMut<AsteroidGame>, mut transforms: ViewMut<Transform>, players: View<Player>, mut physicses: ViewMut<Physics>, mut renderables: ViewMut<Renderable>, mut bullets: ViewMut<Bullet>) {
    let mut transform = match (&mut transforms, &players).iter().next() {
        Some(p) => p.0,
        _ => return,
    };

    let speed = 5f64;
    if game.move_left {
        transform.x -= speed;
    }
    if game.move_right {
        transform.x += speed;
    }
    if game.move_up {
        transform.y -= speed;
    }
    if game.move_down {
        transform.y += speed;
    }

    let transform = transform.clone();
    if game.lmb_down {
        entities.add_entity((&mut transforms, &mut physicses, &mut renderables, &mut bullets),         
        (Transform {
            x: transform.x,
            y: transform.y,
            r: 6f64,
            ..Transform::default()
        },
        Physics {
            speed: 10f64,
            accel: 1f64,
            angle: game.shoot_angle,
            ..Physics::default()
        },
        Renderable {
            color: Color::rgba(0.02, 0.24, 0.81, 0.5),
        },
        Bullet::new(Team::Player)
        ));
    }
}