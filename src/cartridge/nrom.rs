use super::Mapper;

pub(super) struct Nrom {
    prg_rom: Vec<u8>,
}

impl Nrom {
    pub(super) fn new(prg_rom: Vec<u8>) -> Self {
        Nrom { prg_rom }
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
}

#[cfg(test)]
mod tests;
