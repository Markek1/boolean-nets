use macroquad::prelude::*;

pub static WINDOW_SIZE_PX: Vec2 = Vec2::from_array([1920., 1200.]);

pub const GRID_SIZE: Vec2 = Vec2::from_array([1920., 1200.]);
pub const NUM_CONNECTIONS: usize = 3;
pub static SQUARE_MODE: bool = true;
pub static MAX_DISTANCE_TO_CONNECT: i32 = 3; // Has no effect if square mode is flase
pub static SHUFFLE_DIFFS: bool = true;

pub static MAX_CORES: usize = 16;
