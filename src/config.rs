use macroquad::prelude::*;

pub static WINDOW_SIZE_PX: Vec2 = Vec2::from_array([1000., 1000.]);

pub const GRID_SIZE: Vec2 = Vec2::from_array([1000., 1000.]);
pub const NUM_CONNECTIONS: usize = 3;
// If equal to 1, can connect to 8 squares around
pub static MAX_DISTANCE_TO_CONNECT: i32 = 6;

pub static MAX_CORES: usize = 4;
