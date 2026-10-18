use crate::cartridge::{Mapper, Mirroring};
use tile::decode_row;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct Ppu {
    nametables: [u8; 0x800],
    palettes: [u8; 0x20],
    ctrl: u8,
    mask: u8,
    v: u16,
    w: bool,
    read_buffer: u8,
    vblank: bool,
    nmi_pending: bool,
    dot: u16,
    scanline: u16,
    frame: Box<[u8; WIDTH * HEIGHT]>,
    frame_ready: bool,
}

const CTRL_NAMETABLE: u8 = 0b0000_0011;
const CTRL_INCREMENT_32: u8 = 0b0000_0100;
const CTRL_BACKGROUND_TABLE: u8 = 0b0001_0000;
const MASK_SHOW_BACKGROUND: u8 = 0b0000_1000;
const CTRL_NMI_ENABLE: u8 = 0b1000_0000;
const STATUS_VBLANK: u8 = 0b1000_0000;

const DOTS_PER_SCANLINE: u16 = 341;
const SCANLINES_PER_FRAME: u16 = 262;
const VBLANK_SCANLINE: u16 = 241;
const PRE_RENDER_SCANLINE: u16 = 261;

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            nametables: [0; 0x800],
            palettes: [0; 0x20],
            ctrl: 0,
            mask: 0,
            v: 0,
            w: false,
            read_buffer: 0,
            vblank: false,
            nmi_pending: false,
            dot: 0,
            scanline: 0,
            frame: Box::new([0; WIDTH * HEIGHT]),
            frame_ready: false,
        }
    }

    /// Advances one dot. The PPU runs three dots per CPU cycle.
    pub fn tick(&mut self, mapper: &dyn Mapper) {
        if self.dot == 1 {
            match self.scanline {
                VBLANK_SCANLINE => {
                    // Rendering the whole frame at once can't show mid-frame changes such
                    // as scroll splits; per-scanline rendering comes with scrolling.
                    self.render_background(mapper);
                    self.frame_ready = true;
                    self.vblank = true;
                    if self.ctrl & CTRL_NMI_ENABLE != 0 {
                        self.nmi_pending = true;
                    }
                }
                PRE_RENDER_SCANLINE => self.vblank = false,
                _ => {}
            }
        }

        self.dot += 1;
        if self.dot == DOTS_PER_SCANLINE {
            self.dot = 0;
            self.scanline = (self.scanline + 1) % SCANLINES_PER_FRAME;
        }
    }

    pub fn take_frame_ready(&mut self) -> bool {
        std::mem::take(&mut self.frame_ready)
    }

    pub fn frame(&self) -> &[u8] {
        &self.frame[..]
    }

    fn render_background(&mut self, mapper: &dyn Mapper) {
        let backdrop = self.read(0x3F00, mapper) & 0x3F;
        if self.mask & MASK_SHOW_BACKGROUND == 0 {
            self.frame.fill(backdrop);
            return;
        }

        let nametable = 0x2000 + u16::from(self.ctrl & CTRL_NAMETABLE) * 0x400;
        let pattern_table = if self.ctrl & CTRL_BACKGROUND_TABLE != 0 {
            0x1000
        } else {
            0
        };
        for row in 0..30u16 {
            for column in 0..32u16 {
                let tile = u16::from(self.read(nametable + row * 32 + column, mapper));
                let attribute = self.read(nametable + 0x3C0 + (row / 4) * 8 + column / 4, mapper);
                // Each attribute byte covers 4x4 tiles as four 2x2 quadrants.
                let shift = (row % 4) / 2 * 4 + (column % 4) / 2 * 2;
                let palette = u16::from((attribute >> shift) & 0x03);

                for y in 0..8 {
                    let address = pattern_table + tile * 16 + y;
                    let colors =
                        decode_row(self.read(address, mapper), self.read(address + 8, mapper));
                    for (x, &color) in colors.iter().enumerate() {
                        let value = if color == 0 {
                            backdrop
                        } else {
                            self.read(0x3F00 + palette * 4 + u16::from(color), mapper) & 0x3F
                        };
                        let pixel = usize::from(row * 8 + y) * WIDTH + usize::from(column * 8) + x;
                        self.frame[pixel] = value;
                    }
                }
            }
        }
    }

    /// Returns whether an NMI was raised since the last call, and clears it.
    pub fn take_nmi(&mut self) -> bool {
        std::mem::take(&mut self.nmi_pending)
    }

    pub fn read_register(&mut self, register: u16, mapper: &dyn Mapper) -> u8 {
        match register {
            0x2002 => {
                self.w = false;
                if std::mem::take(&mut self.vblank) {
                    STATUS_VBLANK
                } else {
                    0
                }
            }
            0x2007 => {
                let address = self.v;
                let value = self.read(address, mapper);
                let result = if address & 0x3FFF >= 0x3F00 {
                    // Palette reads skip the buffer, but it still refills from the nametable below.
                    self.read_buffer = self.read(address - 0x1000, mapper);
                    value
                } else {
                    std::mem::replace(&mut self.read_buffer, value)
                };
                self.increment_v();
                result
            }
            _ => 0, // write-only registers; open bus is not modeled
        }
    }

    pub fn write_register(&mut self, register: u16, value: u8, mapper: &mut dyn Mapper) {
        match register {
            0x2000 => {
                let was_enabled = self.ctrl & CTRL_NMI_ENABLE != 0;
                self.ctrl = value;
                // Enabling NMI during vblank fires one immediately.
                if !was_enabled && value & CTRL_NMI_ENABLE != 0 && self.vblank {
                    self.nmi_pending = true;
                }
            }
            0x2001 => self.mask = value,
            0x2006 => {
                if self.w {
                    self.v = (self.v & 0xFF00) | u16::from(value);
                } else {
                    // The PPU address bus is 14 bits.
                    self.v = (u16::from(value & 0x3F) << 8) | (self.v & 0x00FF);
                }
                self.w = !self.w;
            }
            0x2007 => {
                self.write(self.v, value, mapper);
                self.increment_v();
            }
            _ => {}
        }
    }

    fn increment_v(&mut self) {
        let step = if self.ctrl & CTRL_INCREMENT_32 != 0 {
            32
        } else {
            1
        };
        self.v = self.v.wrapping_add(step) & 0x3FFF;
    }

    fn read(&self, address: u16, mapper: &dyn Mapper) -> u8 {
        let address = address & 0x3FFF;
        match address {
            0x0000..=0x1FFF => mapper.ppu_read(address),
            0x2000..=0x3EFF => self.nametables[nametable_index(address, mapper.mirroring())],
            0x3F00..=0x3FFF => self.palettes[palette_index(address)],
            _ => unreachable!("address was masked to 14 bits"),
        }
    }

    fn write(&mut self, address: u16, value: u8, mapper: &mut dyn Mapper) {
        let address = address & 0x3FFF;
        match address {
            0x0000..=0x1FFF => mapper.ppu_write(address, value),
            0x2000..=0x3EFF => {
                self.nametables[nametable_index(address, mapper.mirroring())] = value;
            }
            0x3F00..=0x3FFF => self.palettes[palette_index(address)] = value,
            _ => unreachable!("address was masked to 14 bits"),
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

fn nametable_index(address: u16, mirroring: Mirroring) -> usize {
    let offset = (address - 0x2000) & 0x0FFF; // $3000-$3EFF mirrors $2000-$2EFF
    let table = offset / 0x400;
    let within_table = offset % 0x400;
    let physical_table = match mirroring {
        Mirroring::Horizontal => table / 2,
        Mirroring::Vertical => table % 2,
    };
    usize::from(physical_table * 0x400 + within_table)
}

fn palette_index(address: u16) -> usize {
    let index = usize::from(address & 0x1F);
    match index {
        0x10 | 0x14 | 0x18 | 0x1C => index - 0x10,
        _ => index,
    }
}

pub mod palette;
pub mod tile;

#[cfg(test)]
mod tests;
