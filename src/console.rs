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

    pub fn step(&mut self) -> u16 {
        let mut cycles = u16::from(self.cpu.step(&mut self.bus));
        if self.bus.take_dma() {
            // DMA halts the CPU for 513 cycles, plus one to align when it starts on an odd cycle.
            let stall = 513 + u16::from(self.cpu.state().cycles % 2 == 1);
            self.cpu.stall(stall);
            cycles += stall;
        }
        self.bus.tick_apu(cycles);
        if self.bus.tick_ppu(cycles) {
            self.cpu.trigger_nmi();
        }
        cycles
    }

    pub fn set_buttons(&mut self, buttons: u8) {
        self.bus.set_buttons(buttons);
    }

    /// Sets the output rate for `take_samples`; defaults to 44.1 kHz.
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.bus.set_sample_rate(rate);
    }

    /// Audio samples in 0.0..=1.0 produced since the last call.
    pub fn take_samples(&mut self) -> Vec<f32> {
        self.bus.take_samples()
    }

    pub fn step_frame(&mut self) {
        loop {
            self.step();
            if self.bus.take_frame_ready() {
                return;
            }
        }
    }

    pub fn frame(&self) -> &[u8] {
        self.bus.frame()
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
