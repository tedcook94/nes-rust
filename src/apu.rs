mod dmc;
mod envelope;
mod noise;
mod pulse;
mod triangle;

use dmc::Dmc;
use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, //
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

/// NTSC CPU clock in Hz.
const CPU_RATE: f64 = 1_789_773.0;

// Frame counter steps, in CPU cycles since the sequence started.
const STEP_1: u32 = 7457;
const STEP_2: u32 = 14913;
const STEP_3: u32 = 22371;
const FOUR_STEP_END: u32 = 29829;
const FIVE_STEP_END: u32 = 37281;

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    /// Pulse timers run at half the CPU rate, so they only clock on every other cycle.
    odd_cycle: bool,
    frame_cycle: u32,
    five_step: bool,
    /// CPU cycles per output sample; fractional, so samples land on an uneven grid.
    cycles_per_sample: f64,
    sample_clock: f64,
    sample_sum: f32,
    sample_count: u32,
    samples: Vec<f32>,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            odd_cycle: false,
            frame_cycle: 0,
            five_step: false,
            cycles_per_sample: CPU_RATE / 44_100.0,
            sample_clock: 0.0,
            sample_sum: 0.0,
            sample_count: 0,
            samples: Vec::new(),
        }
    }

    /// Advances one CPU cycle.
    pub fn tick(&mut self) {
        if self.odd_cycle {
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
        }
        self.odd_cycle = !self.odd_cycle;
        self.triangle.clock_timer();
        self.noise.clock_timer();
        self.dmc.clock_timer();
        self.clock_frame_counter();
        self.collect_sample();
    }

    /// The PRG address the DMC needs read next; answer with `dmc_fill`.
    pub fn dmc_fetch_address(&self) -> Option<u16> {
        self.dmc.fetch_address()
    }

    pub fn dmc_fill(&mut self, byte: u8) {
        self.dmc.fill(byte);
    }

    pub fn set_sample_rate(&mut self, rate: u32) {
        self.cycles_per_sample = CPU_RATE / f64::from(rate);
    }

    /// Hands over the samples produced since the last call.
    pub fn take_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    /// Averages every CPU cycle's output into one sample. Averaging acts as a crude
    /// low-pass filter, which keeps high tones from aliasing into noise.
    fn collect_sample(&mut self) {
        self.sample_sum += self.sample();
        self.sample_count += 1;
        self.sample_clock += 1.0;
        if self.sample_clock >= self.cycles_per_sample {
            self.sample_clock -= self.cycles_per_sample;
            self.samples
                .push(self.sample_sum / self.sample_count as f32);
            self.sample_sum = 0.0;
            self.sample_count = 0;
        }
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
        self.noise.envelope.clock();
        self.triangle.clock_linear();
    }

    fn half_frame(&mut self) {
        self.pulse1.clock_length();
        self.pulse2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();
        self.pulse1.clock_sweep();
        self.pulse2.clock_sweep();
    }

    /// Bits 0-3: whether each channel's length counter is nonzero. DMC and IRQ bits stay 0.
    pub fn read_status(&self) -> u8 {
        u8::from(self.pulse1.length > 0)
            | u8::from(self.pulse2.length > 0) << 1
            | u8::from(self.triangle.length > 0) << 2
            | u8::from(self.noise.length > 0) << 3
    }

    /// Mixes the channels into one sample in 0.0..=1.0, using the NESdev formulas
    /// that approximate the hardware's nonlinear resistor DAC.
    pub fn sample(&self) -> f32 {
        let pulse = f32::from(self.pulse1.output() + self.pulse2.output());
        let pulse_out = if pulse == 0.0 {
            0.0
        } else {
            95.88 / (8128.0 / pulse + 100.0)
        };
        let tnd = f32::from(self.triangle.output()) / 8227.0
            + f32::from(self.noise.output()) / 12241.0
            + f32::from(self.dmc.output()) / 22638.0;
        let tnd_out = if tnd == 0.0 {
            0.0
        } else {
            159.79 / (1.0 / tnd + 100.0)
        };
        pulse_out + tnd_out
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x4000..=0x4003 => self.pulse1.write(address - 0x4000, value),
            0x4004..=0x4007 => self.pulse2.write(address - 0x4004, value),
            0x4008..=0x400B => self.triangle.write(address - 0x4008, value),
            0x400C..=0x400F => self.noise.write(address - 0x400C, value),
            0x4015 => {
                self.pulse1.set_enabled(value & 0x01 != 0);
                self.pulse2.set_enabled(value & 0x02 != 0);
                self.triangle.set_enabled(value & 0x04 != 0);
                self.dmc.set_enabled(value & 0x10 != 0);
                self.noise.set_enabled(value & 0x08 != 0);
            }
            0x4010..=0x4013 => self.dmc.write(address - 0x4010, value),
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
