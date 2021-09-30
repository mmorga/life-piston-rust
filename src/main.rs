extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::Size;

const WINDOW_SIZE: Size = Size {
    width: 1000.0,
    height: 1000.0,
};

const GRID_SIZE: u32 = 256;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

pub struct Universe {
    gl: GlGraphics, // OpenGL drawing backend.
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    window_size: Size,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    fn cell_width(&self) -> f64 {
        let cell_width = self.window_size.width / self.width as f64;
        cell_width
    }

    fn live_cells(&self) -> Vec<(f64, f64)> {
        let mut cells = Vec::new();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];

                if cell == Cell::Alive {
                    cells.push((
                        col as f64 * self.cell_width(),
                        row as f64 * self.cell_width(),
                    ));
                }
            }
        }
        cells
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BG_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const FG_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, self.cell_width());

        let cells = self.live_cells();

        let offset = (self.window_size.width - (self.cell_width() * self.width as f64)) / 2.0;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BG_COLOR, gl);

            // let text_trans = c.transform.trans(0.0, 0.0);
            // text(FG_COLOR, 12, "Game of Life", cache, text_trans, gl);
            for (x, y) in cells.iter() {
                let transform = c.transform.trans(offset, offset).trans(*x, *y);

                // Draw a box rotating around the middle of the screen.
                rectangle(FG_COLOR, square, transform, gl);
            }
        });
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbors
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbors
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbors dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbors
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn new(opengl: OpenGL, window_size: Size) -> Universe {
        let width = GRID_SIZE;
        let height = GRID_SIZE;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            gl: GlGraphics::new(opengl),
            width,
            height,
            cells,
            window_size,
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", WINDOW_SIZE)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = Universe::new(opengl, WINDOW_SIZE);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
