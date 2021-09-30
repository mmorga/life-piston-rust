extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

mod game_of_life;
mod timing_buffer;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::Size;
use game_of_life::{Cell, Universe};
use timing_buffer::TimingBuffer;

const WINDOW_SIZE: Size = Size {
    width: 1000.0,
    height: 1000.0,
};

const GRID_SIZE: u32 = 256;

const BG_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const FG_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub struct ViewOfLife<'a> {
    gl: GlGraphics, // OpenGL drawing backend.
    glyph_cache: GlyphCache<'a>,
    fps: TimingBuffer,
    square: graphics::types::Rectangle,
    cell_width: f64,
    offset: f64,
    universe: Universe,
}

impl ViewOfLife<'_> {
    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.fps.add_time(args.ext_dt);

        let msg = format!("fps: {0:.2}", self.fps.avg());
        let square = self.square;
        let glyph_cache = &mut self.glyph_cache;
        let offset = self.offset;
        let changed_cells = &self.universe.changed_cells;
        let cell_width = self.cell_width;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BG_COLOR, gl);

            // Draw the live cells
            for (x, y, cell) in changed_cells.iter() {
                let dx = *x as f64 * cell_width;
                let dy = *y as f64 * cell_width;
                let transform = c.transform.trans(offset, offset).trans(dx, dy);

                // Draw a box rotating around the middle of the screen.
                let color = match cell {
                    Cell::Alive => FG_COLOR,
                    Cell::Dead => BG_COLOR,
                };
                rectangle(color, square, transform, gl);
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
        self.universe.clear_changed_cells();
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.universe.update();
    }

    pub fn new(opengl: OpenGL, window_size: Size, width: u32, height: u32) -> ViewOfLife<'static> {
        let texture_settings = TextureSettings::new();
        let glyph_cache = GlyphCache::new(
            "/System/Library/Fonts/Supplemental/Futura.ttc",
            (),
            texture_settings,
        )
        .unwrap();

        let cell_width = (window_size.width - 30.0) / width as f64;

        ViewOfLife {
            gl: GlGraphics::new(opengl),
            glyph_cache,
            fps: TimingBuffer::new(100),
            cell_width,
            square: graphics::rectangle::square(0.0, 0.0, cell_width),
            offset: (window_size.width - (cell_width * width as f64)) / 2.0,
            universe: Universe::new(width, height),
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
    let mut app = ViewOfLife::new(opengl, WINDOW_SIZE, GRID_SIZE, GRID_SIZE);

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
