mod display;

fn main() {
    // initialize the display-module
    let mut display = display::Display::init();

    let fake_mem = [0xff, 0xff, 0xff, 0xff];
    let height = fake_mem.len();
    let start_adress = 0;

    let x_coord = 0;
    let y_coord = 0;

    display.blend_sprite(x_coord, y_coord, height as u16, start_adress, &fake_mem);
    display.draw();

    std::thread::sleep(std::time::Duration::new(10, 0))
}
