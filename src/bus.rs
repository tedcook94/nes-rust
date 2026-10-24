use crate::apu::Apu;
use crate::cartridge::Mapper;
use crate::controller::Controller;
use crate::ppu::Ppu;

pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, byte: u8);
}

pub struct CpuBus {
    ram: [u8; 0x800],
    ppu: Ppu,
    apu: Apu,
    mapper: Box<dyn Mapper>,
    dma_pending: bool,
    controller: Controller,
}

impl CpuBus {
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        CpuBus {
            ram: [0; 0x800],
            ppu: Ppu::new(),
            apu: Apu::new(),
            mapper,
            dma_pending: false,
            controller: Controller::new(),
        }
    }

    pub fn take_dma(&mut self) -> bool {
        std::mem::take(&mut self.dma_pending)
    }

    pub fn tick_ppu(&mut self, cpu_cycles: u16) -> bool {
        for _ in 0..u32::from(cpu_cycles) * 3 {
            self.ppu.tick(&*self.mapper);
        }
        self.ppu.take_nmi()
    }

    pub fn tick_apu(&mut self, cpu_cycles: u16) {
        for _ in 0..cpu_cycles {
            self.apu.tick();
        }
    }

    pub fn set_sample_rate(&mut self, rate: u32) {
        self.apu.set_sample_rate(rate);
    }

    pub fn take_samples(&mut self) -> Vec<f32> {
        self.apu.take_samples()
    }

    /// No device drives IRQ yet; the MMC3 mapper will.
    pub fn irq_line(&self) -> bool {
        false
    }

    pub fn set_buttons(&mut self, buttons: u8) {
        self.controller.set_buttons(buttons);
    }

    pub fn take_frame_ready(&mut self) -> bool {
        self.ppu.take_frame_ready()
    }

    pub fn frame(&self) -> &[u8] {
        self.ppu.frame()
    }
}

impl Bus for CpuBus {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[to_ram_address(address)],
            0x2000..=0x3FFF => self.ppu.read_register(ppu_register(address), &*self.mapper),
            // Upper bits come from the data bus; $40 is what most games see.
            0x4015 => self.apu.read_status(),
            0x4016 => 0x40 | self.controller.read(),
            0x4000..=0x4017 => 0, // reserved for APU and controller 2
            0x4018..=0x401F => 0, // unused test registers
            0x4020..=0xFFFF => self.mapper.cpu_read(address),
        }
    }

    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[to_ram_address(address)] = byte,
            0x2000..=0x3FFF => {
                self.ppu
                    .write_register(ppu_register(address), byte, &mut *self.mapper)
            }
            0x4014 => self.oam_dma(byte),
            0x4016 => self.controller.write(byte),
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu.write_register(address, byte),
            0x4018..=0x401F => (), // unused test registers
            0x4020..=0xFFFF => self.mapper.cpu_write(address, byte),
        }
    }
}

impl CpuBus {
    fn oam_dma(&mut self, page: u8) {
        let start = u16::from(page) << 8;
        for offset in 0..0x100 {
            let value = self.read(start + offset);
            self.ppu.write_register(0x2004, value, &mut *self.mapper);
        }
        self.dma_pending = true;
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
