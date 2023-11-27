use std::io::{stdout, Write};

use macroquad::prelude::*;

mod config;
mod grid;

use config::{GRID_SIZE, WINDOW_SIZE_PX};
use grid::{DrawMode, Grid};

fn window_config() -> Conf {
    Conf {
        window_title: "Particle Life".to_owned(),
        window_width: WINDOW_SIZE_PX.x.round() as i32,
        window_height: WINDOW_SIZE_PX.y.round() as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut stdout = stdout();

    let mut grid = Grid::new(GRID_SIZE.x as usize, GRID_SIZE.y as usize);
    let mut draw_mode = DrawMode::Changes;

    let mut paused = false;

    loop {
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }

        if is_key_pressed(KeyCode::M) {
            match draw_mode {
                DrawMode::Normal => draw_mode = DrawMode::Changes,
                DrawMode::Changes => draw_mode = DrawMode::Normal,
            }
        }

        if is_key_pressed(KeyCode::N) {
            grid = Grid::new(GRID_SIZE.x as usize, GRID_SIZE.y as usize);
        }

        if is_key_pressed(KeyCode::T) {
            grid.randomize_table();
        }

        if is_key_pressed(KeyCode::C) {
            grid.randomize_cells();
        }

        if !paused {
            grid.update();
        }

        // clear_background(BLACK);

        let image = grid.to_image(draw_mode);

        let texture = Texture2D::from_image(&image);

        draw_texture_ex(
            &texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(WINDOW_SIZE_PX),
                ..Default::default()
            },
        );

        // Print FPS every second
        if get_time() % 1. < get_frame_time() as f64 {
            print!("\rFPS: {}", get_fps());
            stdout.flush().expect("Stdout flush failed");
        }

        next_frame().await
    }
}
