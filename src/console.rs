use crate::bus::CpuBus;
use crate::cartridge::{Cartridge, RomError, create_mapper};
use crate::cpu::Cpu;

pub struct Console {
    cpu: Cpu,
    bus: CpuBus,
}

impl Console {
    fn new(rom: &[u8]) -> Result<Self, RomError> {
        let cartridge = Cartridge::parse(rom)?;

        let mapper = create_mapper(cartridge)?;

        let mut console = Console {
            cpu: Cpu::new(),
            bus: CpuBus::new(mapper),
        };
        console.cpu.reset(&mut console.bus);
        Ok(console)
    }

    fn step(&mut self) -> u8 {
        self.cpu.step(&mut self.bus)
    }
}

#[cfg(test)]
mod tests;
