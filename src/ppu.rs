use crate::cartridge::{Mapper, Mirroring};

pub struct Ppu {
    nametables: [u8; 0x800],
    palettes: [u8; 0x20],
    ctrl: u8,
    v: u16,
    w: bool,
    read_buffer: u8,
}

const CTRL_INCREMENT_32: u8 = 0b0000_0100;

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            nametables: [0; 0x800],
            palettes: [0; 0x20],
            ctrl: 0,
            v: 0,
            w: false,
            read_buffer: 0,
        }
    }

    pub fn read_register(&mut self, register: u16, mapper: &dyn Mapper) -> u8 {
        match register {
            0x2002 => {
                self.w = false;
                0
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
            0x2000 => self.ctrl = value,
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

#[cfg(test)]
mod tests;
