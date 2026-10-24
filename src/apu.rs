mod pulse;

use pulse::Pulse;

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    /// Pulse timers run at half the CPU rate, so they only clock on every other cycle.
    odd_cycle: bool,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            pulse1: Pulse::new(),
            pulse2: Pulse::new(),
            odd_cycle: false,
        }
    }

    /// Advances one CPU cycle.
    pub fn tick(&mut self) {
        if self.odd_cycle {
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
        }
        self.odd_cycle = !self.odd_cycle;
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x4000..=0x4003 => self.pulse1.write(address - 0x4000, value),
            0x4004..=0x4007 => self.pulse2.write(address - 0x4004, value),
            _ => {}
        }
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
