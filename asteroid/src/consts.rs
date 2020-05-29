pub mod layers {
    pub const PLAYER: u64 = 1;
    pub const ENEMY: u64 = 1 << 1;
    pub const ASTEROID: u64 = 1 << 2;
    pub const BULLET_PLAYER: u64 = 1 << 3;
    pub const BULLET_ENEMY: u64 = 1 << 4;
    pub const WALL: u64 = 1 << 5;
}

pub mod draw_layers {
    pub const ASTEROID: f32 = 2.0;
    pub const WALL: f32 = 1.0;
    pub const PLAYER: f32 = 0.0;
    pub const ENEMY: f32 = -1.0;
    pub const BULLET: f32 = -2.0; 
}

pub mod textures {
    pub const ASTEROID: &'static str = "asteroid";
    pub const SQUARE: &'static str = "square";
}