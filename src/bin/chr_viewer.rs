//! Shows a cartridge's pattern tables: `cargo run --bin chr_viewer -- path/to/game.nes`

use minifb::{Key, Scale, Window, WindowOptions};
use nes::cartridge::{Cartridge, create_mapper};
use nes::ppu::tile::decode_tile;

// Two 128x128 pattern tables ($0000 and $1000) side by side, 16x16 tiles each.
const WIDTH: usize = 256;
const HEIGHT: usize = 128;

// Color indexes 0-3 as grayscale, since palettes aren't applied here.
const SHADES: [u32; 4] = [0x000000, 0x555555, 0xAAAAAA, 0xFFFFFF];

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: chr_viewer <rom.nes>");
    let rom = std::fs::read(&path).unwrap_or_else(|e| panic!("can't read {path}: {e}"));
    let cartridge = Cartridge::parse(&rom).unwrap_or_else(|e| panic!("bad ROM: {e:?}"));
    let mapper = create_mapper(cartridge).unwrap_or_else(|e| panic!("bad ROM: {e:?}"));

    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    for tile_index in 0..512 {
        let mut bytes = [0u8; 16];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = mapper.ppu_read((tile_index * 16 + i) as u16);
        }

        let table = tile_index / 256;
        let tile_x = table * 16 + tile_index % 16;
        let tile_y = (tile_index % 256) / 16;
        for (y, row) in decode_tile(&bytes).iter().enumerate() {
            for (x, &color) in row.iter().enumerate() {
                let pixel = (tile_y * 8 + y) * WIDTH + tile_x * 8 + x;
                buffer[pixel] = SHADES[usize::from(color)];
            }
        }
    }

    let options = WindowOptions {
        scale: Scale::X4,
        ..WindowOptions::default()
    };
    let mut window = Window::new(&format!("CHR: {path}"), WIDTH, HEIGHT, options)
        .unwrap_or_else(|e| panic!("can't open window: {e}"));
    window.set_target_fps(30);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("window update failed");
    }
}
