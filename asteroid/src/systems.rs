use vermarine_lib::*;

use rand::rngs::StdRng;
use rand::Rng;
use shipyard::*;
use tetra::graphics::Color;
use tetra::math::Vec2;

use crate::components::*;
use crate::layers;
use crate::AsteroidGame;

pub fn spawn_asteroids(
    mut entities: EntitiesViewMut,
    mut rand: UniqueViewMut<StdRng>,
    mut game: UniqueViewMut<AsteroidGame>,
    mut physicses: ViewMut<Physics>,
    mut renderables: ViewMut<Renderable>,
    mut asteroids: ViewMut<Asteroid>,
    mut physics_bodies: ViewMut<PhysicsBody>,
    mut physics_world: UniqueViewMut<PhysicsWorld>,
) {
    game.asteroid_timer += 1;
    while game.asteroid_timer > 100 {
        physics_world.sync(&mut physics_bodies);

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

        let bullet = entities.add_entity(
            (
                &mut physicses,
                &mut renderables,
                &mut asteroids,
            ),
            (
                Physics {
                    speed,
                    angle,
                    ..Physics::default()
                },
                Renderable::new_sprite("asteroid", Color::BLACK),
                Asteroid {},

            ),
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            &bullet, 
            transform,
            CollisionBody::new(Collider::circle(
                transform.r,
                layers::ASTEROID,
                layers::BULLET_PLAYER | layers::PLAYER,
            )),
        );
    }
}

pub fn apply_physics(mut physics_bodies: ViewMut<PhysicsBody>, mut physicses: ViewMut<Physics>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    physics_world.sync(&mut physics_bodies);

    for (body, physics) in (&physics_bodies, &mut physicses).iter() {
        physics.speed += physics.accel;
        physics.angle += physics.curve;

        let input = Vec2::new(
            physics.dx + physics.angle.to_radians().sin() * physics.speed,
            physics.dy - physics.angle.to_radians().cos() * physics.speed,
        );
        body.move_body(&mut physics_world, &input);
    }
}

pub fn wrap_asteroids(bodies: View<PhysicsBody>, asteroids: View<Asteroid>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    for (body, _) in (&bodies, &asteroids).iter() {
        let asteroid = body.transform_mut(&mut physics_world);

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

pub fn wrap_player(mut physics_bodies: ViewMut<PhysicsBody>, players: View<Player>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    physics_world.sync(&mut physics_bodies);

    for (body, _) in (&physics_bodies, &players).iter() {
        let player = body.transform_mut(&mut physics_world);

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

    {
        let (mut physics_bodies, mut physics_world) = all_storages.borrow::<(ViewMut<PhysicsBody>, UniqueViewMut<PhysicsWorld>)>();
        physics_world.sync(&mut physics_bodies);

        for (e, _) in (&physics_bodies).iter().with_id().filter(|(_, body)| {
            let transform = body.transform(&mut physics_world);
            transform.x < -1000f64
                || transform.x > 2280f64
                || transform.y < -1000f64
                || transform.y > 1720f64
        }) {
            deferred.push(e);
        }
    }

    for e in deferred.into_iter() {
        all_storages.delete(e);
    }
}

pub fn spawn_spinners(
    mut entities: EntitiesViewMut,
    mut rand: UniqueViewMut<StdRng>,
    mut game: UniqueViewMut<AsteroidGame>,
    players: View<Player>,
    mut physicses: ViewMut<Physics>,
    mut spinners: ViewMut<Spinner>,
    mut renderables: ViewMut<Renderable>,
    mut physics_bodies: ViewMut<PhysicsBody>,
    mut physics_world: UniqueViewMut<PhysicsWorld>,
) {
    game.spinner_timer += 1;
    while game.spinner_timer > 400 {
        physics_world.sync(&mut physics_bodies);

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

        let body = match (&physics_bodies, &players).iter().next() {
            Some(p) => p.0.clone(),
            _ => return,
        };
        let player = body.transform(&mut physics_world);

        let transform = Transform::new(x as f64, y as f64, radius);
        let angle = transform.get_angle_to(player.x, player.y);
        let spinner = entities.add_entity(
            (
                &mut spinners,
                &mut physicses,
                &mut renderables,
            ),
            (
                Spinner { angle, cooldown: 0 },
                Physics {
                    accel: 0.18f64,
                    angle,
                    ..Physics::default()
                },
                Renderable::new_sprite("asteroid", Color::BLACK),
            ),
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            &spinner, 
            transform, 
            CollisionBody::new(Collider::circle(transform.r, layers::ENEMY, layers::PLAYER)),
    )
    }
}

pub fn shoot_spinners(
    mut entities: EntitiesViewMut,
    mut spinners: ViewMut<Spinner>,
    mut bullets: ViewMut<Bullet>,
    mut physicses: ViewMut<Physics>,
    mut renderables: ViewMut<Renderable>,
    mut physics_bodies: ViewMut<PhysicsBody>,
    mut physics_world: UniqueViewMut<PhysicsWorld>,
) {
    let mut deferred = vec![];
    physics_world.sync(&mut physics_bodies);

    for (body, spinner) in (&physics_bodies, &mut spinners).iter() {
        let transform = body.transform(&mut physics_world);

        if spinner.cooldown > 0 {
            spinner.cooldown -= 1;
        } else {
            for i in 0..4 {
                spinner.cooldown = 4;

                spinner.angle += 4f64;

                deferred.push(((
                    Bullet { team: Team::Ast },
                    Physics {
                        speed: 3f64,
                        angle: spinner.angle + i as f64 * 90f64,
                        ..Physics::default()
                    },
                    Renderable::new_sprite("asteroid", Color::rgb(0.8, 0.0, 0.0)),
                ), (
                    Transform {
                        r: 7.5,
                        ..*transform
                    },
                    CollisionBody::new(Collider::circle(7.5, layers::BULLET_ENEMY, layers::PLAYER)),
                ),));
            }
        }
    }

    for data in deferred.into_iter() {
        let bullet = entities.add_entity(
            (
                &mut bullets,
                &mut physicses,
                &mut renderables,
            ),
            data.0,
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            &bullet, 
            (data.1).0, 
            (data.1).1,
        );
    }
}

pub fn iframe_counter(mut healths: ViewMut<Health>) {
    for health in (&mut healths)
        .iter()
        .filter(|health| health.iframe_count > 0)
    {
        health.iframe_count -= 1;
    }
}

pub fn player_input(
    mut entities: EntitiesViewMut,
    game: UniqueViewMut<AsteroidGame>,
    mut physics_world: UniqueViewMut<PhysicsWorld>,
    mut physics_bodies: ViewMut<PhysicsBody>,
    players: View<Player>,
    mut physicses: ViewMut<Physics>,
    mut renderables: ViewMut<Renderable>,
    mut bullets: ViewMut<Bullet>,
) {
    physics_world.sync(&mut physics_bodies);
    
    let body = match (&physics_bodies, &players).iter().next() {
        Some(p) => p.0,
        _ => return,
    };

    let speed = 5f64;
    let mut input = Vec2::new(0.0, 0.0);
    if game.move_left {
        input.x -= speed;
    }
    if game.move_right {
        input.x += speed;
    }
    if game.move_up {
        input.y -= speed;
    }
    if game.move_down {
        input.y += speed;
    }

    body.move_body_and_collide(&mut physics_world, &input);
    let transform = physics_world.transform(body);

    let transform = transform.clone();
    if game.lmb_down {
        let bullet = entities.add_entity(
            (
                &mut physicses,
                &mut renderables,
                &mut bullets,
            ),
            (
                Physics {
                    speed: 10f64,
                    accel: 1f64,
                    angle: game.shoot_angle,
                    ..Physics::default()
                },
                Renderable::new_sprite("asteroid", Color::rgb(0.02, 0.24, 0.81)),
                Bullet::new(Team::Player),
            ),
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            &bullet, 
            Transform {
                x: transform.x,
                y: transform.y,
                r: 6f64,
                ..Transform::default()
            },
            CollisionBody::new(Collider::circle(
                6f64,
                layers::BULLET_PLAYER,
                layers::ASTEROID | layers::ENEMY,
            )),
        );
    }
}

pub fn player_damage(mut all_storages: AllStoragesViewMut) {
    let mut kill = vec![];
    let outter_health;
    let outter_id;

    {
        let (mut collision_bodies, mut healths, players, mut physics_world) =
            all_storages.borrow::<(ViewMut<PhysicsBody>, ViewMut<Health>, View<Player>, UniqueViewMut<PhysicsWorld>)>();

        physics_world.sync(&mut collision_bodies);

        let (id, body, health) = match (&mut collision_bodies, &players, &mut healths)
            .iter()
            .with_id()
            .next()
        {
            Some((id, (b, _, hp))) => (id, b.collider(&mut physics_world), hp),
            _ => return,
        };

        if health.iframe_count > 0 {
            return;
        }

        for collision in body.colliders[0].overlapping.iter() {
            println!("collision");
            if collision.collision_layer2 & layers::ASTEROID > 0 {
                health.hp -= 1;
                health.iframe_count = health.iframe_max;
                break;
            } else if collision.collision_layer2 & layers::ENEMY > 0 {
                health.hp -= 1;
                health.iframe_count = health.iframe_max;
                break;
            } else if collision.collision_layer2 & layers::BULLET_ENEMY > 0 {
                health.hp -= 1;
                health.iframe_count = health.iframe_max;
                kill.push(collision.entity2);
                break;
            }
        }

        outter_health = health.clone();
        outter_id = id.clone();
    }

    if outter_health.hp <= 0 {
        all_storages.delete(outter_id);
    }

    for id in kill.into_iter() {
        all_storages.delete(id);
    }
}

pub fn asteroid_damage(mut all_storages: AllStoragesViewMut) {
    let mut create = vec![];
    let mut kill = vec![];

    {
        let (mut rand, mut physics_bodies, asteroids, mut physicses, mut physics_world) = all_storages
            .borrow::<(
                UniqueViewMut<StdRng>,
                ViewMut<PhysicsBody>,
                View<Asteroid>,
                ViewMut<Physics>,
                UniqueViewMut<PhysicsWorld>,
            )>();

        physics_world.sync(&mut physics_bodies);
        

        for (id, (physics_body, physics, _)) in
            (&mut physics_bodies, &mut physicses, &asteroids)
                .iter()
                .with_id()
        {
            let (transform, body) = physics_body.parts_mut(&mut physics_world);

            let overlapping = &mut body.colliders[0].overlapping;
            for collision in overlapping.clone().iter() {
                if collision.collision_layer2 & layers::BULLET_PLAYER > 0 {
                    kill.push(collision.entity2);

                    transform.r /= 1.5f64;

                    match body.colliders[0].shape {
                        CollisionShape::Circle(r) => {
                            body.colliders[0].shape = CollisionShape::Circle(r / 1.5f64)
                        }
                        _ => {}
                    }

                    if transform.r < 15f64 {
                        kill.push(id);
                    } else {
                        let mut new_physics = Physics { ..*physics };
                        let angle = collision
                            .transform2
                            .get_angle_to(collision.transform1.x, collision.transform1.y);
                        new_physics.angle = angle + rand.gen_range(0f64, 140f64);
                        physics.angle = angle + rand.gen_range(0f64, 140f64);

                        let collision_body = CollisionBody::from_body(&body);

                        create.push((*transform, new_physics, collision_body));
                    }
                }
            }
        }
    }

    {
        let (
            mut entities,
            mut asteroids,
            mut renderables,
            mut physicses,
            mut physics_bodies,
            mut physics_world,
        ) = all_storages.borrow::<(
            EntitiesViewMut,
            ViewMut<Asteroid>,
            ViewMut<Renderable>,
            ViewMut<Physics>,
            ViewMut<PhysicsBody>,
            UniqueViewMut<PhysicsWorld>,
        )>();
        for (transform, physics, collision_body) in create.into_iter() {
            let splitted = entities.add_entity(
                (
                    &mut asteroids,
                    &mut renderables,
                    &mut physicses,
                ),
                (
                    Asteroid {},
                    Renderable::new_sprite("asteroid", Color::BLACK),
                    physics,
                ),
            );

            physics_world.create_body(
                &mut entities,
                &mut physics_bodies,
                &splitted,
                transform,
                collision_body,
            );
        }
    }

    for id in kill.into_iter() {
        all_storages.delete(id);
    }
}
