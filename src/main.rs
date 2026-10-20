//! Runs a ROM in a window: `cargo run --release -- path/to/game.nes`

use minifb::{Key, Scale, Window, WindowOptions};
use nes::console::Console;
use nes::controller::button;
use nes::ppu::palette::SYSTEM_PALETTE;
use nes::ppu::{HEIGHT, WIDTH};

fn main() {
    let path = std::env::args().nth(1).expect("usage: nes <rom.nes>");
    let rom = std::fs::read(&path).unwrap_or_else(|e| panic!("can't read {path}: {e}"));
    let mut console = Console::new(&rom).unwrap_or_else(|e| panic!("bad ROM: {e:?}"));

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new(&path, WIDTH, HEIGHT, options)
        .unwrap_or_else(|e| panic!("can't open window: {e}"));
    window.set_target_fps(60);

    // Keyboard → NES buttons. Arrow keys move; Z/X are B/A; Enter/Right Shift are Start/Select.
    let key_map = [
        (Key::X, button::A),
        (Key::Z, button::B),
        (Key::RightShift, button::SELECT),
        (Key::Enter, button::START),
        (Key::Up, button::UP),
        (Key::Down, button::DOWN),
        (Key::Left, button::LEFT),
        (Key::Right, button::RIGHT),
    ];

    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let buttons = key_map
            .iter()
            .filter(|(key, _)| window.is_key_down(*key))
            .fold(0, |held, (_, button)| held | button);
        console.set_buttons(buttons);
        console.step_frame();
        for (pixel, &value) in buffer.iter_mut().zip(console.frame()) {
            *pixel = SYSTEM_PALETTE[usize::from(value)];
        }
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("window update failed");
    }
}
