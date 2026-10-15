use crate::cartridge::{Mapper, Mirroring};

pub struct Ppu {
    nametables: [u8; 0x800],
    palettes: [u8; 0x20],
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            nametables: [0; 0x800],
            palettes: [0; 0x20],
        }
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
