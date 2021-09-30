extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::Size;

const WINDOW_SIZE: Size = Size {
    width: 1000.0,
    height: 1000.0,
};

const GRID_SIZE: u32 = 256;

pub struct TimingBuffer {
    buffer: Vec<f64>,
    size: usize,
}

impl TimingBuffer {
    pub fn new(size: usize) -> TimingBuffer {
        TimingBuffer {
            buffer: Vec::with_capacity(size),
            size,
        }
    }

    pub fn avg(&self) -> f64 {
        let avg = 1.0 / (self.buffer.iter().sum::<f64>() / self.buffer.len() as f64);
        avg
    }

    pub fn update(&mut self, timing: f64) -> f64 {
        if self.buffer.is_empty() {
            for _i in 1..self.size {
                self.buffer.push(timing);
            }
        } else {
            self.buffer.remove(0);
            self.buffer.push(timing);
        }

        let avg = self.avg();
        avg
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

pub struct Universe<'a> {
    gl: GlGraphics, // OpenGL drawing backend.
    glyph_cache: GlyphCache<'a>,
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    window_size: Size,
    fps: TimingBuffer,
    ups: TimingBuffer,
}

impl Universe<'_> {
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
        let cell_width = (self.window_size.width - 30.0)/ self.width as f64;
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

        let glyph_cache = &mut self.glyph_cache;

        let msg = format!("fps: {0:.2} ups: {1:.2}", self.fps.update(args.ext_dt), self.ups.avg());

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BG_COLOR, gl);

            // Draw the live cells
            for (x, y) in cells.iter() {
                let transform = c.transform.trans(offset, offset).trans(*x, *y);

                // Draw a box rotating around the middle of the screen.
                rectangle(FG_COLOR, square, transform, gl);
            }

            // Draw the fps calculation
            text::Text::new_color([0.0, 0.5, 0.0, 1.0], 16)
                .draw(
                    &msg,
                    glyph_cache,
                    &DrawState::default(),
                    c.transform.trans(10.0, 15.0),
                    gl,
                )
                .unwrap();
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.ups.update(args.dt);
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

    pub fn new(opengl: OpenGL, window_size: Size) -> Universe<'static> {
        let width = GRID_SIZE;
        let height = GRID_SIZE;
        let texture_settings = TextureSettings::new();
        let glyph_cache = GlyphCache::new(
            "/System/Library/Fonts/Supplemental/Futura.ttc",
            (),
            texture_settings,
        )
        .unwrap();

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
            glyph_cache,
            width,
            height,
            cells,
            window_size,
            fps: TimingBuffer::new(100),
            ups: TimingBuffer::new(100),
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

    let mut event_settings = EventSettings::new();
    event_settings.ups = 30; // max number of updates per second
    println!("event_settings: {:?}", event_settings);
    // {
    //     max_fps: DEFAULT_MAX_FPS,
    //     ups: DEFAULT_UPS,
    //     swap_buffers: true,
    //     bench_mode: false,
    //     lazy: false,
    //     ups_reset: DEFAULT_UPS_RESET,
    // }
    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
