use super::{Mapper, Mirroring};

pub(super) struct Nrom {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
    chr_is_ram: bool,
    mirroring: Mirroring,
}

impl Nrom {
    pub(super) fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let chr_is_ram = chr_rom.is_empty();
        Nrom {
            prg_rom,
            chr: if chr_is_ram { vec![0; 0x2000] } else { chr_rom },
            chr_is_ram,
            mirroring,
        }
    }
}

impl Mapper for Nrom {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xFFFF => self.prg_rom[usize::from(address - 0x8000) % self.prg_rom.len()],
            _ => 0,
        }
    }

    fn cpu_write(&mut self, _address: u16, _value: u8) {}

    fn ppu_read(&self, address: u16) -> u8 {
        self.chr[usize::from(address) % self.chr.len()]
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        if !self.chr_is_ram {
            return;
        }

        let index = usize::from(address) % self.chr.len();
        self.chr[index] = value;
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests;
