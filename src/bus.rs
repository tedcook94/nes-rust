pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, byte: u8);
}

struct CpuBus {
    ram: [u8; 0x800],
}

impl CpuBus {
    fn new() -> Self {
        CpuBus { ram: [0; 0x800] }
    }
}

impl Bus for CpuBus {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[to_ram_address(address)],
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[to_ram_address(address)] = byte,
            _ => {}
        }
    }
}

fn to_ram_address(address: u16) -> usize {
    usize::from(address & 0x07FF) // drop bits 11 and 12 to wrap after 2KB
}

#[cfg(test)]
mod tests;
