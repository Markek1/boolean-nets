use std::collections::HashMap;
use std::thread;

use ::rand;
use ::rand::seq::SliceRandom;
use core_affinity;
use macroquad::prelude::*;
use num_cpus;

use crate::config::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrawMode {
    Normal,
    Changes,
}

pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<bool>,
    connections: Vec<[usize; NUM_CONNECTIONS]>,
    update_table: HashMap<usize, bool>,

    changes_len: usize,
    num_changes: Vec<usize>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = (0..width * height)
            .map(|_| rand::random::<bool>())
            .collect();

        let mut connections = vec![[0; NUM_CONNECTIONS]; width * height];

        // The diffs around a square (for distance = 1 it's the 8 squares around)
        let mut diffs: Vec<(i32, i32)> = (-MAX_DISTANCE_TO_CONNECT..=MAX_DISTANCE_TO_CONNECT)
            .flat_map(|dx| {
                (-MAX_DISTANCE_TO_CONNECT..=MAX_DISTANCE_TO_CONNECT).map(move |dy| (dx, dy))
            })
            .collect();

        for x in 0..width {
            for y in 0..height {
                diffs.shuffle(&mut rand::thread_rng());

                let mut num_taken = 0;

                for (dx, dy) in diffs.iter() {
                    if num_taken as i32 >= NUM_CONNECTIONS as i32 {
                        break;
                    }

                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        connections[x + y * width][num_taken] = (nx + ny * width as i32) as usize;
                        num_taken += 1;
                    }
                }
            }
        }

        let mut update_table = HashMap::new();

        for i in 0..(1 << NUM_CONNECTIONS) {
            let mut possible = rand::random::<bool>();

            // Epilepsy fix #1
            while i == 0 && possible {
                possible = rand::random::<bool>();
            }
            // Epilepsy fix #2
            while i == (1 << NUM_CONNECTIONS) - 1 && !possible {
                possible = rand::random::<bool>();
            }

            update_table.insert(i, possible);
        }

        Self {
            cells,
            connections,
            update_table,
            width,
            height,
            changes_len: 20,
            num_changes: vec![0; width * height],
        }
    }

    pub fn update(&mut self) {
        let mut new_cells = self.cells.clone();

        let num_cpus = num_cpus::get().min(MAX_CORES);
        let core_ids = core_affinity::get_core_ids().expect("Could not get core IDs");
        let cells_per_cpu = (self.cells.len() as f32 / num_cpus as f32).ceil() as usize;
        let cell_chunks_lens = self
            .cells
            .chunks_mut(cells_per_cpu)
            .map(|x| x.len())
            .collect::<Vec<_>>();

        thread::scope(|x| unsafe {
            let self_ptr = self as *const _ as usize;
            let new_cells = new_cells.as_mut_ptr() as usize;

            let mut i = 0;
            for cpu in 0..num_cpus {
                let end_i = i + cell_chunks_lens[cpu];
                let core_id = core_ids[cpu];

                x.spawn(move || {
                    core_affinity::set_for_current(core_id);
                    let slf = &mut *(self_ptr as *mut Self);

                    let new_cells = new_cells as *mut bool;

                    while i < end_i {
                        let mut new_val = 0;

                        for connection in slf.connections[i].iter() {
                            new_val <<= 1;
                            new_val |= slf.cells[*connection] as usize;
                        }

                        let old = *new_cells.add(i);
                        let new = slf.update_table[&new_val];

                        if old != new {
                            slf.num_changes[i] = (slf.num_changes[i] + 2).min(slf.changes_len);
                        }
                        slf.num_changes[i] = slf.num_changes[i].saturating_sub(1);

                        *new_cells.add(i) = new;

                        i += 1;
                    }
                });

                i += cell_chunks_lens[cpu];
            }
        });

        // for i in 0..self.cells.len() {
        //     let mut new_val = 0;

        //     for connection in self.connections[i].iter() {
        //         new_val <<= 1;
        //         new_val |= self.cells[*connection] as usize;
        //     }

        //     let old = new_cells[i];
        //     let new = self.update_table[&new_val];

        //     if old != new {
        //         self.num_changes[i] = (self.num_changes[i] + 2).min(self.changes_len);
        //     }
        //     self.num_changes[i] = self.num_changes[i].saturating_sub(1);

        //     new_cells[i] = new;
        // }

        self.cells = new_cells;
    }

    pub fn draw(&self, draw_mode: DrawMode) {
        let cell_size = Vec2::new(
            WINDOW_SIZE_PX.x / self.width as f32,
            WINDOW_SIZE_PX.y / self.height as f32,
        );

        for x in 0..self.width {
            for y in 0..self.height {
                let cell = self.cells[x + y * self.width];

                let color = match draw_mode {
                    DrawMode::Normal => {
                        if cell {
                            WHITE
                        } else {
                            BLACK
                        }
                    }

                    DrawMode::Changes => {
                        let num_changes = self.num_changes[x + y * self.width];

                        let red = if num_changes == 0 { 1. } else { 0. };
                        let green = if num_changes > 0 {
                            0.5 + num_changes as f32 / self.changes_len as f32
                        } else {
                            0.
                        };

                        Color::new(red, green, 0., 1.)
                    }
                };

                draw_rectangle(
                    x as f32 * cell_size.x,
                    y as f32 * cell_size.y,
                    cell_size.x,
                    cell_size.y,
                    color,
                );
            }
        }
    }
}
