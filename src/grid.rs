use std::collections::HashMap;
use std::thread;
use std::time::SystemTime;

use core_affinity;
use fastrand;
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
        let d = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Duration since UNIX_EPOCH failed");
        fastrand::seed(d.as_micros() as u64);

        let cells = (0..width * height).map(|_| fastrand::bool()).collect();

        let mut connections = vec![[0; NUM_CONNECTIONS]; width * height];

        let mut diffs = if SQUARE_MODE {
            // The diffs around a square (for distance = 1 it's the 8 squares around and itself)
            (-MAX_DISTANCE_TO_CONNECT..=MAX_DISTANCE_TO_CONNECT)
                .flat_map(|dx| {
                    (-MAX_DISTANCE_TO_CONNECT..=MAX_DISTANCE_TO_CONNECT).map(move |dy| (dx, dy))
                })
                .collect()
        } else {
            vec![(0, 1), (1, 0), (0, -1), (-1, 0)]
        };
        fastrand::shuffle(&mut diffs);

        for x in 0..width {
            for y in 0..height {
                if SHUFFLE_DIFFS {
                    fastrand::shuffle(&mut diffs);
                }

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
            update_table.insert(i, fastrand::bool());

            while (i == 0 && update_table[&i] == true)
                || (i == (1 << NUM_CONNECTIONS) - 1 && update_table[&i] == false)
            {
                update_table.remove(&i);
                update_table.insert(i, fastrand::bool());
            }
        }

        // Epilepsy fix
        // update_table.insert(0, false);
        // update_table.insert((1 << NUM_CONNECTIONS) - 1, true);

        println!();
        for i in 0..(1 << NUM_CONNECTIONS) {
            print!("{}: ", i);
            print!("{:0>1$b}", i, NUM_CONNECTIONS);

            println!(" -> {}", update_table[&i] as usize);
        }
        println!();

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

    pub fn toggle_cell(&mut self, pos: Vec2) {
        let x = pos.x as usize;
        let y = pos.y as usize;

        self.cells[x + y * self.width] = !self.cells[x + y * self.width];
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

        self.cells = new_cells;
    }

    pub fn randomize_cells(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = fastrand::bool();
        }
    }

    pub fn randomize_table(&mut self) {
        for (key, val) in self.update_table.iter_mut() {
            *val = fastrand::bool();

            while (*key == 0 && *val == true)
                || (*key == (1 << NUM_CONNECTIONS) - 1 && *val == false)
            {
                *val = fastrand::bool();
            }
        }
    }

    // other is None for normal drawing, otherwise the pixels that are different will be drawn in blue
    pub fn to_image_compared_to(&self, other: Option<&Self>, draw_mode: DrawMode) -> Image {
        let mut image = Image::gen_image_color(
            self.width as u16,
            self.height as u16,
            Color::new(0., 0., 0., 0.),
        );
        let img_ref = &mut image;

        let num_cpus = num_cpus::get().min(MAX_CORES);
        let core_ids = core_affinity::get_core_ids().expect("Could not get core IDs");
        let cells_per_cpu = (self.cells.len() as f32 / num_cpus as f32).ceil() as usize;
        let cell_chunks_lens = self
            .cells
            .chunks(cells_per_cpu)
            .map(|x| x.len())
            .collect::<Vec<_>>();

        thread::scope(|x| unsafe {
            let self_ptr = self as *const _ as usize;
            let other_ptr = match other {
                None => 0,
                Some(_) => other.unwrap() as *const _ as usize,
            };
            let image = img_ref as *const _ as usize;

            let mut i = 0;
            for cpu in 0..num_cpus {
                let end_i = i + cell_chunks_lens[cpu];
                let core_id = core_ids[cpu];

                x.spawn(move || {
                    core_affinity::set_for_current(core_id);
                    let slf = &*(self_ptr as *const Self);
                    let other: Option<&Grid> = match other {
                        None => None,
                        Some(_) => Some(&*(other_ptr as *const Self)),
                    };

                    let image = &mut *(image as *mut Image);

                    while i < end_i {
                        let x = i % slf.width;
                        let y = i / slf.width;

                        let cell = slf.cells[i];
                        let other_cell = match other {
                            None => cell,
                            Some(other) => other.cells[i],
                        };

                        let color = if cell != other_cell {
                            Color::new(0., 0., 1., 1.)
                        } else {
                            match draw_mode {
                                DrawMode::Normal => {
                                    if cell {
                                        WHITE
                                    } else {
                                        BLACK
                                    }
                                }

                                DrawMode::Changes => {
                                    let num_changes = slf.num_changes[i];

                                    let red = if num_changes == 0 { 1. } else { 0. };
                                    let green = if num_changes > 0 {
                                        0.5 + num_changes as f32 / slf.changes_len as f32
                                    } else {
                                        0.
                                    };

                                    Color::new(red, green, 0., 1.)
                                }
                            }
                        };

                        image.set_pixel(x as u32, y as u32, color);

                        i += 1;
                    }
                });

                i += cell_chunks_lens[cpu];
            }
        });

        image
    }

    pub fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            connections: self.connections.clone(),
            update_table: self.update_table.clone(),
            width: self.width,
            height: self.height,
            changes_len: self.changes_len,
            num_changes: self.num_changes.clone(),
        }
    }
}
