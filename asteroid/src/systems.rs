use vermarine_lib::{
    shipyard::*,
    tetra::{
        graphics::Color,
        math::Vec2,
    },
    physics::{
        PhysicsBody,
        CollisionBody,
        Collider,
        CollisionShape,
        Transform,
        world::{
            PhysicsWorld,
        },
    },
    rendering::{
        Sprite,
    },
};

use rand::rngs::StdRng;
use rand::Rng;

use crate::{
    components::*,
    layers,
    textures,
    AsteroidGame,
    draw_layers,
};

pub fn spawn_asteroids(
    mut entities: EntitiesViewMut,
    mut rand: UniqueViewMut<StdRng>,
    mut game: UniqueViewMut<AsteroidGame>,
    mut physicses: ViewMut<Physics>,
    mut sprites: ViewMut<Sprite>,
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

        let transform = Transform::new(x as f64, y as f64);
        let mut angle = transform.get_angle_to(640f64, 360f64);
        angle += rand.gen_range(-22f64, 22f64);
        let speed = rand.gen_range(5f64, 10f64);

        let asteroid = entities.add_entity(
            (
                &mut physicses,
                &mut sprites,
                &mut asteroids,
            ),
            (
                Physics {
                    speed,
                    angle,
                    ..Physics::default()
                },
                create_sprite(textures::ASTEROID, radius, Color::rgb(0.3, 0.3, 0.3), draw_layers::ASTEROID),
                Asteroid {},

            ),
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            asteroid, 
            transform,
            CollisionBody::from_sensor(Collider::circle(
                radius,
                layers::ASTEROID,
                layers::BULLET_PLAYER,
            )),
        );
    }
}

pub fn apply_physics(mut physics_bodies: ViewMut<PhysicsBody>, mut physicses: ViewMut<Physics>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    physics_world.sync(&mut physics_bodies);

    for (id, (_, physics)) in (&physics_bodies, &mut physicses).iter().with_id() {
        if physics.apply_auto == false {
            continue;
        }

        physics.speed += physics.accel;
        physics.angle += physics.curve;

        let input = Vec2::new(
            physics.dx + physics.angle.to_radians().sin() * physics.speed,
            physics.dy - physics.angle.to_radians().cos() * physics.speed,
        );

        if input != Vec2::zero() {
            physics_world.move_body(id, input);
        }
    }
}

pub fn move_player_bullets(
    mut physics_bodies: ViewMut<PhysicsBody>,
    mut physics_world: UniqueViewMut<PhysicsWorld>,
    mut physicses: ViewMut<Physics>,
    bullets: View<Bullet>,
) {
    physics_world.sync(&mut physics_bodies);

    for (id, (_, physics, bullet)) in (&physics_bodies, &mut physicses, &bullets).iter().with_id() {
        if physics.apply_auto || bullet.team != Team::Player {
            continue;
        }

        physics.speed += physics.accel;
        physics.angle += physics.curve;

        let input = Vec2::new(
            physics.dx + physics.angle.to_radians().sin() * physics.speed,
            physics.dy - physics.angle.to_radians().cos() * physics.speed,
        );

        let mut collisions = physics_world.move_body_and_collide(id, input);

        if let Some(collision) = collisions.pop() {
            let reflected = input.reflected(collision.normal);

            if reflected.x.is_nan() || reflected.y.is_nan() {
                continue;
            }

            physics.dx = reflected.x;
            physics.dy = reflected.y;
            physics.angle = 0.0;
            physics.speed = 0.0;
            physics.accel = 0.0;
            physics.curve = 0.0;
        }
    } 
}

pub fn wrap_asteroids(mut physics_bodies: ViewMut<PhysicsBody>, asteroids: View<Asteroid>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    physics_world.sync(&mut physics_bodies);
    
    for (id, _) in (&physics_bodies, &asteroids).iter().with_id() {
        let (asteroid, collision_body) = physics_world.parts(id);
        let r = collision_body.sensors[0].shape.get_width() / 2.0;

        // Wrap X
        if asteroid.x > 1280f64 + r {
            let x = -(asteroid.x - 1280f64);
            physics_world.move_body_to_x(id, x);
        } else if asteroid.x < -r {
            let x = 1280f64 + (-asteroid.x);
            physics_world.move_body_to_x(id, x);
        }

        let asteroid = physics_world.transform(id);
        // Wrap Y
        if asteroid.y > 720f64 + r {
            let y = -(asteroid.y - 720f64);
            physics_world.move_body_to_y(id, y);
        } else if asteroid.y < 0f64 - r {
            let y = 720f64 + (-asteroid.y);
            physics_world.move_body_to_y(id, y);
        }
    }
}

pub fn wrap_player(mut physics_bodies: ViewMut<PhysicsBody>, players: View<Player>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    physics_world.sync(&mut physics_bodies);

    for (id, _) in (&physics_bodies, &players).iter().with_id() {
        let (player, collision_body) = physics_world.parts(id);
        let r = collision_body.sensors[0].shape.get_width() / 2.0;

        // Wrap X
        if player.x > 1280f64 + r {
            let x = -(player.x - 1280f64);
            physics_world.move_body_to_x(id, x);
        } else if player.x < -r {
            let x = 1280f64 + (-player.x);
            physics_world.move_body_to_x(id, x);
        }

        let player = physics_world.transform(id);
        // Wrap Y
        if player.y > 720f64 + r {
            let y = -(player.y - 720f64);
            physics_world.move_body_to_y(id, y);
        } else if player.y < 0f64 - r {
            let y = 720f64 + (-player.y);
            physics_world.move_body_to_y(id, y);
        }
    }
}

pub fn destroy_offscreen(mut all_storages: AllStoragesViewMut) {
    let mut deferred = vec![];

    {
        let (mut physics_bodies, mut physics_world) = all_storages.borrow::<(ViewMut<PhysicsBody>, UniqueViewMut<PhysicsWorld>)>();
        physics_world.sync(&mut physics_bodies);

        for (e, _) in (&physics_bodies).iter().with_id().filter(|(e, _)| {
            let transform = physics_world.transform(*e);
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
    mut sprites: ViewMut<Sprite>,
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

        let body = match (&physics_bodies, &players).iter().with_id().next() {
            Some((id, _)) => id,
            _ => return,
        };
        let player = physics_world.transform(body);

        let transform = Transform::new(x as f64, y as f64);
        let angle = transform.get_angle_to(player.x, player.y);
        let spinner = entities.add_entity(
            (
                &mut spinners,
                &mut physicses,
                &mut sprites,
            ),
            (
                Spinner { angle, cooldown: 0 },
                Physics {
                    accel: 0.18f64,
                    angle,
                    ..Physics::default()
                },
                create_sprite(textures::ASTEROID, radius, Color::rgb(0.7, 0.0, 0.0), draw_layers::ENEMY),
            ),
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            spinner, 
            transform, 
            CollisionBody::from_sensor(Collider::circle(radius, layers::ENEMY, 0)),
    )
    }
}

pub fn shoot_spinners(
    mut entities: EntitiesViewMut,
    mut spinners: ViewMut<Spinner>,
    mut bullets: ViewMut<Bullet>,
    mut physicses: ViewMut<Physics>,
    mut sprites: ViewMut<Sprite>,
    mut physics_bodies: ViewMut<PhysicsBody>,
    mut physics_world: UniqueViewMut<PhysicsWorld>,
) {
    let mut deferred = vec![];
    physics_world.sync(&mut physics_bodies);

    for (id, (_, spinner)) in (&physics_bodies, &mut spinners).iter().with_id() {
        let transform = physics_world.transform(id);

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
                    create_sprite(textures::ASTEROID, 7.5, Color::rgb(0.8, 0.0, 0.0), draw_layers::BULLET),
                ), (
                    Transform {
                        ..*transform
                    },
                    CollisionBody::from_sensor(Collider::circle(7.5, layers::BULLET_ENEMY, layers::WALL)),
                ),));
            }
        }
    }

    for data in deferred.into_iter() {
        let bullet = entities.add_entity(
            (
                &mut bullets,
                &mut physicses,
                &mut sprites,
            ),
            data.0,
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            bullet, 
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
    mut sprites: ViewMut<Sprite>,
    mut bullets: ViewMut<Bullet>,
) {
    physics_world.sync(&mut physics_bodies);
    
    let body = match (&physics_bodies, &players).iter().with_id().next() {
        Some((id, _)) => id,
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

    if input != Vec2::zero() {
        physics_world.move_body_and_collide(body, input);
    }
    let transform = physics_world.transform(body);

    let transform = transform.clone();
    if game.lmb_down {
        let bullet = entities.add_entity(
            (
                &mut physicses,
                &mut sprites,
                &mut bullets,
            ),
            (
                Physics {
                    apply_auto: false,
                    speed: 10f64,
                    accel: 1f64,
                    angle: game.shoot_angle,
                    ..Physics::default()
                },
                create_sprite(textures::ASTEROID, 10.0, Color::rgb(0.02, 0.24, 0.81), draw_layers::BULLET),
                Bullet::new(Team::Player),
            ),
        );

        physics_world.create_body(
            &mut entities, 
            &mut physics_bodies, 
            bullet, 
            Transform {
                x: transform.x,
                y: transform.y,
                ..Transform::default()
            },
            CollisionBody::from_collider(
                Collider::half_extents(
                    10.0,
                    10.0,
                    layers::BULLET_PLAYER,
                    layers::WALL
                ),),
        );
    }
}

pub fn player_damage(mut all_storages: AllStoragesViewMut) {
    let mut kill = vec![];

    {
        let (mut collision_bodies, mut healths, players, mut sprites, mut physics_world) =
            all_storages.borrow::<(ViewMut<PhysicsBody>, ViewMut<Health>, View<Player>, ViewMut<Sprite>, UniqueViewMut<PhysicsWorld>)>();

        physics_world.sync(&mut collision_bodies);

        let (id, body, health, sprite) = match (&mut collision_bodies, &players, &mut healths, &mut sprites)
            .iter()
            .with_id()
            .next()
        {
            Some((id, (_, _, hp, sprite))) => (id, physics_world.collider(id), hp, sprite),
            _ => return,
        };

        if health.iframe_count > 0 {
            return;
        } else {
            sprite.0.color = Color::rgb(0.0, 1.0, 0.0);
        }

        for collision in body.sensors[0].overlapping.iter() {
            if collision.collision_layer2 & layers::ASTEROID > 0 {
                health.hp -= 1;
                health.iframe_count = health.iframe_max;
                sprite.0.color = Color::RED;
                break;
            } else if collision.collision_layer2 & layers::ENEMY > 0 {
                health.hp -= 1;
                health.iframe_count = health.iframe_max;
                sprite.0.color = Color::RED;
                break;
            } else if collision.collision_layer2 & layers::BULLET_ENEMY > 0 {
                health.hp -= 1;
                health.iframe_count = health.iframe_max;
                sprite.0.color = Color::RED;
                kill.push(collision.entity2);
                break;
            }
        }

        if health.hp <= 0 {
            kill.push(id.clone());
        }
    }

    for id in kill.into_iter() {
        all_storages.delete(id);
    }
}

pub fn asteroid_damage(mut all_storages: AllStoragesViewMut) {
    let mut create = vec![];
    let mut kill = vec![];

    {
        let (mut rand, mut physics_bodies, asteroids, mut sprites, mut physicses, mut physics_world) = all_storages
            .borrow::<(
                UniqueViewMut<StdRng>,
                ViewMut<PhysicsBody>,
                View<Asteroid>,
                ViewMut<Sprite>,
                ViewMut<Physics>,
                UniqueViewMut<PhysicsWorld>,
            )>();

        physics_world.sync(&mut physics_bodies);
        

        for (id, (_, physics, _, sprite)) in
            (&mut physics_bodies, &mut physicses, &asteroids, &mut sprites)
                .iter()
                .with_id()
        {
            let (transform, body) = physics_world.parts_mut(id);

            let overlapping = &mut body.sensors[0].overlapping;
            for collision in overlapping.clone().iter() {
                if collision.collision_layer2 & layers::BULLET_PLAYER > 0 {
                    kill.push(collision.entity2);

                    sprite.0.scale /= 1.5;

                    match body.sensors[0].shape {
                        CollisionShape::Circle(r) => {
                            body.sensors[0].shape = CollisionShape::Circle(r / 1.5)
                        }
                        _ => { }
                    }

                    if body.sensors[0].shape.get_width() / 2.0 < 15f64 {
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
            mut sprites,
            mut physicses,
            mut physics_bodies,
            mut physics_world,
        ) = all_storages.borrow::<(
            EntitiesViewMut,
            ViewMut<Asteroid>,
            ViewMut<Sprite>,
            ViewMut<Physics>,
            ViewMut<PhysicsBody>,
            UniqueViewMut<PhysicsWorld>,
        )>();
        for (transform, physics, collision_body) in create.into_iter() {
            let splitted = entities.add_entity(
                (
                    &mut asteroids,
                    &mut sprites,
                    &mut physicses,
                ),
                (
                    Asteroid {},
                    create_sprite(textures::ASTEROID, collision_body.sensors[0].shape.get_width() / 2.0, Color::rgb(0.3, 0.3, 0.3), draw_layers::ASTEROID),
                    physics,
                ),
            );

            physics_world.create_body(
                &mut entities,
                &mut physics_bodies,
                splitted,
                transform,
                collision_body,
            );
        }
    }

    for id in kill.into_iter() {
        all_storages.delete(id);
    }
}

pub fn destroy_bullets(mut all_storages: AllStoragesViewMut) {
    let mut to_kill = vec![];

    all_storages.run(|
        bullets: View<Bullet>,
        mut bodies: ViewMut<PhysicsBody>,
        world: UniqueViewMut<PhysicsWorld>,| {
            for (id, (_, _)) in (&bullets, &mut bodies).iter().with_id() {
                let body = world.collider(id);
                if let Some(sensor) = body.sensors.get(0) {
                    if sensor.overlapping.len() > 0 {
                        to_kill.push(id);
                    }
                }
            }
    });

    for id in to_kill.into_iter() {
        all_storages.delete(id);
    }
}