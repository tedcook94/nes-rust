use crate::cartridge::{Mapper, Mirroring};
use tile::decode_row;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct Ppu {
    nametables: [u8; 0x800],
    palettes: [u8; 0x20],
    ctrl: u8,
    mask: u8,
    oam: [u8; 0x100],
    oam_address: u8,
    /// Current VRAM address. While rendering, its bits hold the scroll position:
    /// `yyy NN YYYYY XXXXX` (fine Y, nametable, coarse Y, coarse X).
    v: u16,
    /// Scroll and address writes land here first, then get copied into `v`.
    t: u16,
    /// Fine X scroll, 0-7.
    fine_x: u8,
    w: bool,
    read_buffer: u8,
    vblank: bool,
    nmi_pending: bool,
    dot: u16,
    scanline: u16,
    frame: Box<[u8; WIDTH * HEIGHT]>,
    sprite_zero_hit: bool,
    frame_ready: bool,
}

const CTRL_NAMETABLE: u8 = 0b0000_0011;
const CTRL_INCREMENT_32: u8 = 0b0000_0100;
const CTRL_SPRITE_TABLE: u8 = 0b0000_1000;
const CTRL_BACKGROUND_TABLE: u8 = 0b0001_0000;
const CTRL_SPRITE_8X16: u8 = 0b0010_0000;
const MASK_SHOW_BACKGROUND: u8 = 0b0000_1000;
const MASK_SHOW_SPRITES: u8 = 0b0001_0000;

const SPRITE_PALETTE: u8 = 0b0000_0011;
const SPRITE_BEHIND_BACKGROUND: u8 = 0b0010_0000;
const SPRITE_FLIP_HORIZONTAL: u8 = 0b0100_0000;
const SPRITE_FLIP_VERTICAL: u8 = 0b1000_0000;
const CTRL_NMI_ENABLE: u8 = 0b1000_0000;
const STATUS_SPRITE_ZERO_HIT: u8 = 0b0100_0000;
const STATUS_VBLANK: u8 = 0b1000_0000;

// Bit groups within `v` and `t`.
const COARSE_X: u16 = 0x001F;
const COARSE_Y: u16 = 0x03E0;
const NAMETABLE_X: u16 = 0x0400;
const NAMETABLE_Y: u16 = 0x0800;
const FINE_Y: u16 = 0x7000;
const HORIZONTAL_BITS: u16 = NAMETABLE_X | COARSE_X;
const VERTICAL_BITS: u16 = FINE_Y | NAMETABLE_Y | COARSE_Y;

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
            oam: [0; 0x100],
            oam_address: 0,
            v: 0,
            t: 0,
            fine_x: 0,
            w: false,
            read_buffer: 0,
            vblank: false,
            nmi_pending: false,
            dot: 0,
            scanline: 0,
            frame: Box::new([0; WIDTH * HEIGHT]),
            sprite_zero_hit: false,
            frame_ready: false,
        }
    }

    /// Advances one dot. The PPU runs three dots per CPU cycle.
    pub fn tick(&mut self, mapper: &dyn Mapper) {
        if self.dot == 1 {
            match self.scanline {
                VBLANK_SCANLINE => {
                    self.frame_ready = true;
                    self.vblank = true;
                    if self.ctrl & CTRL_NMI_ENABLE != 0 {
                        self.nmi_pending = true;
                    }
                }
                PRE_RENDER_SCANLINE => {
                    self.vblank = false;
                    self.sprite_zero_hit = false;
                }
                _ => {}
            }
        }
        let visible = usize::from(self.scanline) < HEIGHT;
        // Each visible line is drawn in one go once its pixels would have been output,
        // so register writes between lines (such as scroll splits) take effect.
        if self.dot == 256 && visible {
            self.render_scanline(self.scanline, mapper);
        }
        if self.rendering_enabled() && (visible || self.scanline == PRE_RENDER_SCANLINE) {
            match self.dot {
                256 => self.increment_y(),
                257 => self.v = (self.v & !HORIZONTAL_BITS) | (self.t & HORIZONTAL_BITS),
                280..=304 if self.scanline == PRE_RENDER_SCANLINE => {
                    self.v = (self.v & !VERTICAL_BITS) | (self.t & VERTICAL_BITS);
                }
                _ => {}
            }
        }

        self.dot += 1;
        if self.dot == DOTS_PER_SCANLINE {
            self.dot = 0;
            self.scanline = (self.scanline + 1) % SCANLINES_PER_FRAME;
        }
    }

    fn rendering_enabled(&self) -> bool {
        self.mask & (MASK_SHOW_BACKGROUND | MASK_SHOW_SPRITES) != 0
    }

    /// Moves `v` down one pixel row, wrapping into the next nametable after row 29.
    fn increment_y(&mut self) {
        if self.v & FINE_Y != FINE_Y {
            self.v += 0x1000;
            return;
        }
        self.v &= !FINE_Y;
        let coarse_y = (self.v & COARSE_Y) >> 5;
        let coarse_y = match coarse_y {
            29 => {
                self.v ^= NAMETABLE_Y;
                0
            }
            // Rows 30-31 hold attributes; scrolling into them wraps without switching tables.
            31 => 0,
            _ => coarse_y + 1,
        };
        self.v = (self.v & !COARSE_Y) | (coarse_y << 5);
    }

    pub fn take_frame_ready(&mut self) -> bool {
        std::mem::take(&mut self.frame_ready)
    }

    pub fn frame(&self) -> &[u8] {
        &self.frame[..]
    }

    fn render_scanline(&mut self, y: u16, mapper: &dyn Mapper) {
        let backdrop = self.read(0x3F00, mapper) & 0x3F;
        let line = usize::from(y) * WIDTH;
        self.frame[line..line + WIDTH].fill(backdrop);

        let background_opaque = self.render_background_line(y, mapper);
        self.render_sprite_line(y, &background_opaque, mapper);
    }

    /// Draws one line of background and returns which of its pixels are opaque.
    fn render_background_line(&mut self, y: u16, mapper: &dyn Mapper) -> [bool; WIDTH] {
        let mut opaque = [false; WIDTH];
        if self.mask & MASK_SHOW_BACKGROUND == 0 {
            return opaque;
        }

        let pattern_table = if self.ctrl & CTRL_BACKGROUND_TABLE != 0 {
            0x1000
        } else {
            0
        };
        let fine_y = (self.v & FINE_Y) >> 12;
        // 33 tiles cover 256 pixels at any fine X offset.
        let mut line = [0u8; WIDTH + 8];
        let mut line_opaque = [false; WIDTH + 8];
        let mut v = self.v;
        for column in 0..33 {
            let tile = u16::from(self.read(0x2000 | (v & 0x0FFF), mapper));
            let attribute_address =
                0x23C0 | (v & (NAMETABLE_X | NAMETABLE_Y)) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
            let attribute = self.read(attribute_address, mapper);
            // Each attribute byte covers 4x4 tiles as four 2x2 quadrants.
            let shift = ((v >> 4) & 0x04) | (v & 0x02);
            let palette = u16::from((attribute >> shift) & 0x03);

            let address = pattern_table + tile * 16 + fine_y;
            let colors = decode_row(self.read(address, mapper), self.read(address + 8, mapper));
            for (offset, &color) in colors.iter().enumerate() {
                if color == 0 {
                    continue;
                }
                let x = column * 8 + offset;
                line[x] = self.read(0x3F00 + palette * 4 + u16::from(color), mapper) & 0x3F;
                line_opaque[x] = true;
            }

            v = if v & COARSE_X == COARSE_X {
                (v & !COARSE_X) ^ NAMETABLE_X
            } else {
                v + 1
            };
        }

        let start = usize::from(self.fine_x);
        let row = usize::from(y) * WIDTH;
        for x in 0..WIDTH {
            if line_opaque[start + x] {
                self.frame[row + x] = line[start + x];
                opaque[x] = true;
            }
        }
        opaque
    }

    fn render_sprite_line(
        &mut self,
        y: u16,
        background_opaque: &[bool; WIDTH],
        mapper: &dyn Mapper,
    ) {
        if self.mask & MASK_SHOW_SPRITES == 0 {
            return;
        }

        let height: u16 = if self.ctrl & CTRL_SPRITE_8X16 != 0 {
            16
        } else {
            8
        };
        // The lowest-index opaque sprite pixel claims each screen pixel, even when its
        // priority then hides it behind the background.
        let mut claimed = [false; WIDTH];
        let (sprites, _) = self.oam.as_chunks::<4>();
        for (index, &[sprite_y, tile, attributes, sprite_x]) in sprites.iter().enumerate() {
            // Sprites appear one scanline below their OAM Y.
            let Some(row) = y
                .checked_sub(u16::from(sprite_y) + 1)
                .filter(|&r| r < height)
            else {
                continue;
            };
            let palette = u16::from(attributes & SPRITE_PALETTE);
            let tile_row = if attributes & SPRITE_FLIP_VERTICAL != 0 {
                height - 1 - row
            } else {
                row
            };
            let address = self.sprite_row_address(tile, tile_row);
            let mut colors = decode_row(self.read(address, mapper), self.read(address + 8, mapper));
            if attributes & SPRITE_FLIP_HORIZONTAL != 0 {
                colors.reverse();
            }

            for (offset, &color) in colors.iter().enumerate() {
                let x = usize::from(sprite_x) + offset;
                if color == 0 || x >= WIDTH {
                    continue;
                }
                // Hardware never reports a hit at x = 255.
                if index == 0 && background_opaque[x] && x != 255 {
                    self.sprite_zero_hit = true;
                }
                if std::mem::replace(&mut claimed[x], true) {
                    continue;
                }
                let behind = attributes & SPRITE_BEHIND_BACKGROUND != 0;
                if behind && background_opaque[x] {
                    continue;
                }
                self.frame[usize::from(y) * WIDTH + x] =
                    self.read(0x3F10 + palette * 4 + u16::from(color), mapper) & 0x3F;
            }
        }
    }

    /// Pattern table address of one 8-pixel row of a sprite. 8x16 sprites take their
    /// pattern table from tile bit 0 and use two consecutive tiles.
    fn sprite_row_address(&self, tile: u8, row: u16) -> u16 {
        if self.ctrl & CTRL_SPRITE_8X16 != 0 {
            let table = u16::from(tile & 1) * 0x1000;
            let top_tile = u16::from(tile & 0xFE);
            table + (top_tile + row / 8) * 16 + row % 8
        } else {
            let table = if self.ctrl & CTRL_SPRITE_TABLE != 0 {
                0x1000
            } else {
                0
            };
            table + u16::from(tile) * 16 + row
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
                let mut status = 0;
                if std::mem::take(&mut self.vblank) {
                    status |= STATUS_VBLANK;
                }
                if self.sprite_zero_hit {
                    status |= STATUS_SPRITE_ZERO_HIT;
                }
                status
            }
            0x2004 => self.oam[usize::from(self.oam_address)],
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
                self.t = (self.t & !(NAMETABLE_X | NAMETABLE_Y))
                    | (u16::from(value & CTRL_NAMETABLE) << 10);
                // Enabling NMI during vblank fires one immediately.
                if !was_enabled && value & CTRL_NMI_ENABLE != 0 && self.vblank {
                    self.nmi_pending = true;
                }
            }
            0x2001 => self.mask = value,
            0x2003 => self.oam_address = value,
            0x2004 => {
                self.oam[usize::from(self.oam_address)] = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            0x2005 => {
                if self.w {
                    let coarse_y = u16::from(value >> 3) << 5;
                    let fine_y = u16::from(value & 0x07) << 12;
                    self.t = (self.t & !(COARSE_Y | FINE_Y)) | coarse_y | fine_y;
                } else {
                    self.t = (self.t & !COARSE_X) | u16::from(value >> 3);
                    self.fine_x = value & 0x07;
                }
                self.w = !self.w;
            }
            0x2006 => {
                if self.w {
                    self.t = (self.t & 0xFF00) | u16::from(value);
                    self.v = self.t;
                } else {
                    // The top bit of t is cleared along with the two bits above 14-bit addresses.
                    self.t = (u16::from(value & 0x3F) << 8) | (self.t & 0x00FF);
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
