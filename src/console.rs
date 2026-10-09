use crate::bus::CpuBus;
use crate::cartridge::{Cartridge, RomError, create_mapper};
use crate::cpu::{Cpu, CpuState};

pub struct Console {
    cpu: Cpu,
    bus: CpuBus,
}

impl Console {
    pub fn new(rom: &[u8]) -> Result<Self, RomError> {
        let cartridge = Cartridge::parse(rom)?;

        let mapper = create_mapper(cartridge)?;

        let mut console = Console {
            cpu: Cpu::new(),
            bus: CpuBus::new(mapper),
        };
        console.cpu.reset(&mut console.bus);
        Ok(console)
    }

    pub fn step(&mut self) -> u8 {
        self.cpu.step(&mut self.bus)
    }

    pub fn jump_to(&mut self, address: u16) {
        self.cpu.set_pc(address);
    }

    pub fn cpu_state(&self) -> CpuState {
        self.cpu.state()
    }
}

#[cfg(test)]
mod tests;
