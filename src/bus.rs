use crate::cartridge::Mapper;

pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, byte: u8);
}

pub struct CpuBus {
    ram: [u8; 0x800],
    mapper: Box<dyn Mapper>,
}

impl CpuBus {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        CpuBus {
            ram: [0; 0x800],
            mapper,
        }
    }
}

impl Bus for CpuBus {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[to_ram_address(address)],
            0x2000..=0x3FFF => 0, // reserved for PPU
            0x4000..=0x4017 => 0, // reserved for APU and I/O
            0x4018..=0x401F => 0, // unused test registers
            0x4020..=0xFFFF => self.mapper.cpu_read(address),
        }
    }

    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[to_ram_address(address)] = byte,
            0x2000..=0x3FFF => (), // reserved for PPU
            0x4000..=0x4017 => (), // reserved for APU and I/O
            0x4018..=0x401F => (), // unused test registers
            0x4020..=0xFFFF => self.mapper.cpu_write(address, byte),
        }
    }
}

fn to_ram_address(address: u16) -> usize {
    usize::from(address & 0x07FF) // drop bits 11 and 12 to wrap after 2KB
}

fn ppu_register(address: u16) -> u16 {
    0x2000 + (address & 0x0007)
}

#[cfg(test)]
mod tests;
