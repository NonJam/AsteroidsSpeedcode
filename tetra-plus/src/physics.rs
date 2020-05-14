use shipyard::*;

use crate::components::Transform;

pub fn physics_workload(world: &mut World) -> &'static str {
    let name = "Physics";
    
    world.add_workload(name)
        .with_system(system!(collision_system))
        .build();

    name
}

pub fn collision_system(transforms: View<Transform>, mut colliders: ViewMut<CollisionBody>) {
    let mut collected: Vec<(&Transform, &mut CollisionBody)> = (&transforms, &mut colliders).iter().collect();

    let len = collected.len();

    for i in 1..len {
        let (left, right) = collected.split_at_mut(i);
        let body1 = left.last_mut().unwrap();
        for body2 in right.iter_mut() {
            let c1 = &mut body1.1.colliders[0];
            let c2 = &mut body2.1.colliders[0];
            
            if collider_overlaps_collider(body1.0, c1, body2.0, c2) {
                let collision = Collision::new(c1.shape.clone(), c1.collides_with, c1.collision_layer,
                    c2.shape.clone(), c2.collides_with, c2.collision_layer);
                
                if c1.collides_with & c2.collision_layer > 0 {
                    c1.overlapping.push(collision.clone());
                }

                if c2.collides_with & c1.collision_layer > 0 {
                    c2.overlapping.push(collision.clone());
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Collision {
    pub shape1: CollisionShape,
    pub collides_with1: u64,
    pub collision_layer1: u64,

    pub shape2: CollisionShape,
    pub collides_with2: u64,
    pub collision_layer2: u64,
}

impl Collision {
    pub fn new(shape1: CollisionShape, collides_with1: u64, collision_layer1: u64,
        shape2: CollisionShape, collides_with2: u64, collision_layer2: u64) -> Self {
        Collision {
            shape1,
            collides_with1,
            collision_layer1,
            shape2,
            collides_with2,
            collision_layer2,
        }
    }
}

#[derive(Clone)]
pub struct CollisionBody {
    pub colliders: Vec<Collider>,
}

impl CollisionBody {
    pub fn new(collider: Collider) -> Self {
        CollisionBody {
            colliders: vec![collider],
        }
    }
}

#[derive(Clone)]
pub struct Collider {
    pub shape: CollisionShape,
    pub collision_layer: u64,
    pub collides_with: u64,

    pub overlapping: Vec<Collision>,
}

impl Collider {
    pub fn circle(radius: f64, collision_layer: u64, collides_with: u64) -> Self {
        Collider {
            shape: CollisionShape::Circle(radius),
            collides_with,
            collision_layer,

            overlapping: vec![],
        }
    }
}

#[derive(Clone)]
pub enum CollisionShape {
    Circle(f64),
    //Composite(Vec<CollisionShape>),
}

pub fn collider_overlaps_collider(t1: &Transform, c1: &Collider, t2: &Transform, c2: &Collider) -> bool {
    match c1.shape {
        CollisionShape::Circle(r) => circle_overlaps_collider(t1, r, t2, c2),
    }
}

pub fn circle_overlaps_collider(t1: &Transform, c1: f64, t2: &Transform, c2: &Collider) -> bool {
    match c2.shape {
        CollisionShape::Circle(r) => circle_overlaps_circle(t1, c1, t2, r),
    }
}

pub fn circle_overlaps_circle(t1: &Transform, c1: f64, t2: &Transform, c2: f64) -> bool {
    let x = (t1.x - t2.x).abs();
    let y = (t1.y - t2.y).abs();
    x * x + y * y <= (c1 + c2) * (c1 + c2)
}