use tetra::graphics::{
    Color
};
use tetra::math::Vec2;
use vermarine_lib::{
    *,
    rendering:: {
        Sprite,
        draw_buffer::{
            DrawCommand,
        },
    },
};

//
// Health
#[derive(Clone, Debug)]
pub struct Health {
    pub max: i32,
    pub hp: i32,
    pub iframe_count: i32,
    pub iframe_max: i32,
    pub iframe_col: Option<Color>,
}

impl Default for Health {
    fn default() -> Self {
        Health {
            max: 0,
            hp: 0,
            iframe_count: 0,
            iframe_max: 0,
            iframe_col: None,
        }
    }
}

impl Health {
    pub fn new(max: i32, iframes: i32, iframe_col: Option<Color>) -> Self {
        Health {
            max: max,
            hp: max,
            iframe_count: 0,
            iframe_max: iframes,
            iframe_col: iframe_col,
        }
    }
}

//
// sprite creator

pub fn create_sprite(texture: &'static str, radius: f64, color: Color, draw_layer: f32) -> Sprite {
    let scale = (radius / 1024.0 * 2.0) as f32;
    Sprite::from_command(
        DrawCommand::new(texture)
        .scale(Vec2::new(scale, scale))
        .origin(Vec2::new(512.0, 512.0))
        .color(color)
        .draw_layer(draw_layer)
    )
}

//
// Physics

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Physics {
    pub apply_auto: bool,
    pub dx: f64,
    pub dy: f64,
    pub speed: f64,
    pub angle: f64,
    pub accel: f64,
    pub curve: f64,
}

impl Default for Physics {
    fn default() -> Self {
        Physics {
            apply_auto: true,
            dx: 0f64,
            dy: 0f64,
            speed: 0f64,
            angle: 0f64,
            accel: 0f64,
            curve: 0f64,
        }
    }
}

//
// Spinner
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spinner {
    pub angle: f64,
    pub cooldown: i32,
}

//
// Collision

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Collision {
    pub angle: f64,
}

//
// Bullet

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Team {
    Player,
    Ast,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bullet {
    pub team: Team,
    pub bounces: u32,
    pub bounce_limit: u32,
}

impl Bullet {
    pub fn new(team: Team) -> Self {
        Bullet { team: team, bounces: 0, bounce_limit: 3, }
    }
}

//
// Tags

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Asteroid;
