mod envelope;
mod pulse;

use pulse::Pulse;

// Frame counter steps, in CPU cycles since the sequence started.
const STEP_1: u32 = 7457;
const STEP_2: u32 = 14913;
const STEP_3: u32 = 22371;
const FOUR_STEP_END: u32 = 29829;
const FIVE_STEP_END: u32 = 37281;

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    /// Pulse timers run at half the CPU rate, so they only clock on every other cycle.
    odd_cycle: bool,
    frame_cycle: u32,
    five_step: bool,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            odd_cycle: false,
            frame_cycle: 0,
            five_step: false,
        }
    }

    /// Advances one CPU cycle.
    pub fn tick(&mut self) {
        if self.odd_cycle {
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
        }
        self.odd_cycle = !self.odd_cycle;
        self.clock_frame_counter();
    }

    fn clock_frame_counter(&mut self) {
        self.frame_cycle += 1;
        let end = if self.five_step {
            FIVE_STEP_END
        } else {
            FOUR_STEP_END
        };
        match self.frame_cycle {
            STEP_1 | STEP_3 => self.quarter_frame(),
            STEP_2 => {
                self.quarter_frame();
                self.half_frame();
            }
            // In 5-step mode, step 4 (29829) clocks nothing.
            cycle if cycle == end => {
                self.quarter_frame();
                self.half_frame();
                self.frame_cycle = 0;
            }
            _ => {}
        }
    }

    fn quarter_frame(&mut self) {
        self.pulse1.envelope.clock();
        self.pulse2.envelope.clock();
    }

    fn half_frame(&mut self) {
        self.pulse1.clock_length();
        self.pulse2.clock_length();
        self.pulse1.clock_sweep();
        self.pulse2.clock_sweep();
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x4000..=0x4003 => self.pulse1.write(address - 0x4000, value),
            0x4004..=0x4007 => self.pulse2.write(address - 0x4004, value),
            0x4017 => {
                // Bit 6 inhibits the frame IRQ; IRQs arrive with milestone 8.
                self.five_step = value & 0x80 != 0;
                // Hardware resets 3-4 cycles later; the difference is inaudible.
                self.frame_cycle = 0;
                if self.five_step {
                    self.quarter_frame();
                    self.half_frame();
                }
            }
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
