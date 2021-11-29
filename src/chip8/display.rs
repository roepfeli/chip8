use sdl2::pixels::Color;
use sdl2::rect::Rect;

const DISPLAY_SCALE_FACTOR: u32 = 10;

const DISPLAY_WIDTH: u32 = 64;
const DISPLAY_HEIGHT: u32 = 32;

// TODO: add flag to indicate change in disp_buffer: only draw if there was a change

pub struct Display {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    disp_buffer: [bool; (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize],
}

impl Display {
    pub fn init(sdl_context: sdl2::Sdl) -> Display {
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
            .accelerated()
            .build()
            .expect("ERROR: Unable to create canvas in SDL2-window. Exiting...");

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        Display {
            canvas: canvas,
            disp_buffer: [false; (DISPLAY_HEIGHT * DISPLAY_WIDTH) as usize],
        }
    }

    pub fn clear_screen(&mut self) {
        self.disp_buffer.map(|_| false);
    }

    pub fn blend_sprite(
        &mut self,
        x_coord: u8,
        y_coord: u8,
        height: u8,
        start_adress: u16,
        memory: &[u8],
    ) -> bool {
        // TODO: maybe change disp_buffer to [u8; _]???
        let x_coord = x_coord as usize;
        let y_coord = y_coord as usize;
        let height = height as usize;
        let start_adress = start_adress as usize;

        let mut was_turned_off = false;

        for y in 0..height as usize {
            for x in 0..8usize {
                let actual_x = (x + x_coord) % DISPLAY_WIDTH as usize;
                let actual_y = (y + y_coord) % DISPLAY_HEIGHT as usize;
                let result = self.disp_buffer[actual_y * DISPLAY_WIDTH as usize + actual_x]
                    ^ (memory[start_adress + y] & (128 >> x) != 0);
                self.disp_buffer[actual_y * DISPLAY_WIDTH as usize + actual_x] = result;
                if !result {
                    was_turned_off = true;
                }
            }
        }

        was_turned_off
    }

    pub fn draw(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);

        // TODO: calling draw_rect for every white is a waste. use draw texture or something...
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.disp_buffer[(y * DISPLAY_WIDTH + x) as usize] {
                    let rect = Rect::new(
                        (x * DISPLAY_SCALE_FACTOR) as i32,
                        (y * DISPLAY_SCALE_FACTOR) as i32,
                        DISPLAY_SCALE_FACTOR,
                        DISPLAY_SCALE_FACTOR,
                    );

                    self.canvas
                        .fill_rect(rect)
                        .expect("ERROR: Could not fill rectangle");

                    self.canvas
                        .draw_rect(rect)
                        .expect("ERROR: Could not draw pixel. Exiting...");
                }
            }
        }

        self.canvas.present();
    }
}
