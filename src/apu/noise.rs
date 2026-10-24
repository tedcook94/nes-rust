use super::LENGTH_TABLE;
use super::envelope::Envelope;

/// Timer periods in CPU cycles.
const PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub(super) struct Noise {
    pub(super) envelope: Envelope,
    /// Short mode taps bit 6 instead of bit 1, giving a 93-step metallic tone.
    pub(super) short_mode: bool,
    pub(super) timer_period: u16,
    timer: u16,
    pub(super) shift: u16,
    pub(super) length: u8,
}

impl Noise {
    pub(super) fn new() -> Self {
        Noise {
            envelope: Envelope::new(),
            short_mode: false,
            timer_period: PERIODS[0],
            timer: 0,
            shift: 1,
            length: 0,
        }
    }

    /// `register` is 0-3, relative to $400C.
    pub(super) fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => self.envelope.write(value),
            1 => {}
            2 => {
                self.short_mode = value & 0x80 != 0;
                self.timer_period = PERIODS[usize::from(value & 0x0F)];
            }
            3 => {
                self.length = LENGTH_TABLE[usize::from(value >> 3)];
                self.envelope.start = true;
            }
            _ => unreachable!("noise register {register}"),
        }
    }

    /// Clocked every CPU cycle, since `PERIODS` is in CPU cycles.
    pub(super) fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period - 1;
            self.clock_shift();
        } else {
            self.timer -= 1;
        }
    }

    pub(super) fn clock_shift(&mut self) {
        let tap = if self.short_mode { 6 } else { 1 };
        let feedback = (self.shift ^ (self.shift >> tap)) & 1;
        self.shift = (self.shift >> 1) | (feedback << 14);
    }

    pub(super) fn clock_length(&mut self) {
        if !self.envelope.looping && self.length > 0 {
            self.length -= 1;
        }
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "read by the mixer in a later commit")
    )]
    pub(super) fn output(&self) -> u8 {
        if self.length == 0 || self.shift & 1 != 0 {
            return 0;
        }
        self.envelope.volume()
    }
}
