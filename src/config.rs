use macroquad::prelude::*;

pub static WINDOW_SIZE_PX: Vec2 = Vec2::from_array([1920., 1200.]);

pub const GRID_SIZE: Vec2 = Vec2::from_array([1920., 1200.]);
pub const NUM_CONNECTIONS: usize = 3;
// If equal to 1, can connect to 9 squares around (including itself)
pub static MAX_DISTANCE_TO_CONNECT: i32 = 3;

pub static MAX_CORES: usize = 16;
