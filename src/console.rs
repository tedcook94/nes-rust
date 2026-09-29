use crate::bus::CpuBus;
use crate::cpu::Cpu;

pub struct Console {
    cpu: Cpu,
    bus: CpuBus,
}

impl Console {
    fn new() -> Self {
        let mut console = Console {
            cpu: Cpu::new(),
            bus: CpuBus::new(),
        };
        console.cpu.reset(&mut console.bus);
        console
    }

    fn step(&mut self) -> u8 {
        self.cpu.step(&mut self.bus)
    }
}

#[cfg(test)]
mod tests;
