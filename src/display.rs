use sdl2::pixels::Color;
use sdl2::rect::Rect;

const DISPLAY_SCALE_FACTOR: u32 = 10;

const DISPLAY_WIDTH: u32 = 64;
const DISPLAY_HEIGHT: u32 = 32;

pub struct Display {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    disp_buffer: [bool; (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize],
}

impl Display {
    pub fn init() -> Display {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "rust-sdl2 demo",
                DISPLAY_WIDTH * DISPLAY_SCALE_FACTOR,
                DISPLAY_HEIGHT * DISPLAY_SCALE_FACTOR,
            )
            .position_centered()
            .build()
            .expect("ERROR: Unable to initialize SDL2 video-subsystem. Exiting...");

        let mut canvas = window
            .into_canvas()
            .build()
            .expect("ERROR: Unable to create canvas in SDL2-window. Exiting...");

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Display {
            canvas: canvas,
            disp_buffer: [false; (DISPLAY_HEIGHT * DISPLAY_WIDTH) as usize],
        }
    }

    pub fn blend_sprite(
        &mut self,
        x_coord: u16,
        y_coord: u16,
        height: u16,
        start_adress: u16,
        memory: &[u8],
    ) {
        // TODO: maybe change disp_buffer to [u8; _]???
        for y in 0..height {
            for x in 0..8 {
                self.disp_buffer[((y + y_coord) * DISPLAY_WIDTH as u16 + x_coord + x) as usize] =
                    memory[(start_adress + y) as usize] & (1 << x) != 0;
            }
        }
    }

    pub fn draw(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);

        // TODO: calling draw_rect for every white is a waste. use draw texture or something...
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.disp_buffer[(y * DISPLAY_WIDTH + x) as usize] {
                    self.canvas
                        .draw_rect(Rect::new(
                            (x * DISPLAY_SCALE_FACTOR) as i32,
                            (y * DISPLAY_SCALE_FACTOR) as i32,
                            DISPLAY_SCALE_FACTOR,
                            DISPLAY_SCALE_FACTOR,
                        ))
                        .expect("ERROR: Could not draw pixel. Exiting...");
                }
            }
        }

        self.canvas.present();
    }
}
