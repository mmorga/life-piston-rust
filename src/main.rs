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
use game_of_life::{Universe};
use timing_buffer::TimingBuffer;

// Laptop size 2880 x 1800 (half is 1440x900)
const WINDOW_SIZE: Size = Size {
    width: 1440.0,
    height: 900.0,
};

pub struct ViewOfLife<'a> {
    gl: GlGraphics, // OpenGL drawing backend.
    glyph_cache: GlyphCache<'a>,
    fps: TimingBuffer,
    square: graphics::types::Rectangle,
    cell_size: f64,
    offset_x: f64,
    offset_y: f64,
    universe: Universe,
}

impl ViewOfLife<'_> {
    const BG_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const FG_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    // updates square, offset, cell_width for current screen size
    // we want the universe to be displayed with square boxes
    fn calculate (&mut self, args: &RenderArgs) {
        let u_width = self.universe.width as f64;
        let u_height = self.universe.height as f64;
        let w_width = args.draw_size[0] as f64 / 2.0;
        let w_height = args.draw_size[1] as f64 / 2.0;
        let top_margin = 30.0; // Space for FPS message
        let cell_width = w_width / u_width;
        let cell_height = (w_height - top_margin) / u_height;
        self.cell_size = cell_width.min(cell_height);
        self.offset_x = (w_width - (self.cell_size * u_width)) / 2.0; // left margin
        self.offset_y = (w_height - (self.cell_size * u_height)) / 2.0 + top_margin; // top margin
        self.square = graphics::rectangle::square(0.0, 0.0, self.cell_size);
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.fps.add_time(args.ext_dt);

        let msg = format!("fps: {0:.2}", self.fps.avg());
        self.calculate(args);
        let square = self.square;
        let glyph_cache = &mut self.glyph_cache;
        let offset_x = self.offset_x;
        let offset_y = self.offset_y;
        let live_cells = &self.universe.live_cells;
        let cell_size = self.cell_size;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(Self::BG_COLOR, gl);

            // Draw the live cells
            for (x, y) in live_cells.iter() {
                let dx = *x as f64 * cell_size;
                let dy = *y as f64 * cell_size;
                let transform = c.transform.trans(offset_x, offset_y).trans(dx, dy);

                rectangle(Self::FG_COLOR, square, transform, gl);
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

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.universe.update();
    }

    pub fn new(opengl: OpenGL, width: u32, height: u32) -> ViewOfLife<'static> {
        let texture_settings = TextureSettings::new();
        let glyph_cache = GlyphCache::new(
            "/System/Library/Fonts/Supplemental/Futura.ttc",
            (),
            texture_settings,
        )
        .unwrap();

        ViewOfLife {
            gl: GlGraphics::new(opengl),
            glyph_cache,
            fps: TimingBuffer::new(100),
            cell_size: 10.0,
            square: graphics::rectangle::square(0.0, 0.0, 10.0),
            offset_x: 0.0,
            offset_y: 0.0,
            universe: Universe::new(width, height),
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("game-of-life", WINDOW_SIZE)
        .graphics_api(opengl)
        .exit_on_esc(true)
        // .fullscreen(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = ViewOfLife::new(opengl, WINDOW_SIZE.width as u32 / 2, WINDOW_SIZE.height as u32 / 2);

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
