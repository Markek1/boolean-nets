use std::io::{stdout, Write};

use macroquad::prelude::*;

mod config;
mod grid;

use config::{GRID_SIZE, WINDOW_SIZE_PX};
use grid::{DrawMode, Grid};

fn window_config() -> Conf {
    Conf {
        window_title: "Boolean Nets".to_owned(),
        window_width: WINDOW_SIZE_PX.x.round() as i32,
        window_height: WINDOW_SIZE_PX.y.round() as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut stdout = stdout();

    let mut grid = Grid::new(GRID_SIZE.x as usize, GRID_SIZE.y as usize);
    let mut comparison_grid: Option<Grid> = None;
    let mut draw_mode = DrawMode::Changes;

    let mut paused = false;

    let mut generations_per_frame = 1;

    loop {
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }

        if is_key_pressed(KeyCode::Key1) {
            draw_mode = DrawMode::Normal;
        }

        if is_key_pressed(KeyCode::Key2) {
            draw_mode = DrawMode::Changes;
        }

        if is_key_pressed(KeyCode::N) {
            grid = Grid::new(GRID_SIZE.x as usize, GRID_SIZE.y as usize);
            comparison_grid = None;
        }

        if is_key_pressed(KeyCode::T) {
            grid.randomize_table();
        }

        if is_key_pressed(KeyCode::C) {
            grid.randomize_cells();
        }

        if is_key_pressed(KeyCode::Q) {
            generations_per_frame = 1;
        }

        if is_key_pressed(KeyCode::W) {
            generations_per_frame = (generations_per_frame - 1).max(1);
        }

        if is_key_released(KeyCode::E) {
            generations_per_frame += 1;
        }

        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let cell_pos = Vec2::new(
                (mouse_pos.0 / WINDOW_SIZE_PX.x * GRID_SIZE.x).floor(),
                (mouse_pos.1 / WINDOW_SIZE_PX.y * GRID_SIZE.y).floor(),
            );

            if let None = comparison_grid {
                comparison_grid = Some(grid.clone());
            }
            comparison_grid.as_mut().unwrap().toggle_cell(cell_pos);
        }

        if !paused {
            for _ in 0..generations_per_frame {
                grid.update();
                if let Some(ref mut cg) = comparison_grid {
                    cg.update();
                }
            }
        }

        let image = grid.to_image_compared_to(
            match comparison_grid {
                None => None,
                Some(ref cg) => Some(&cg),
            },
            draw_mode,
        );

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
            print!(
                "\rFPS: {} gens per frame: {} ",
                get_fps(),
                generations_per_frame
            );
            stdout.flush().expect("Stdout flush failed");
        }

        next_frame().await
    }
}
