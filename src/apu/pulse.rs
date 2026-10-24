const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 25% inverted
];

pub(super) const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, //
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub(super) struct Pulse {
    pub(super) duty: u8,
    pub(super) step: u8,
    pub(super) halt_length: bool,
    pub(super) volume: u8,
    pub(super) timer_period: u16,
    pub(super) timer: u16,
    pub(super) length: u8,
}

impl Pulse {
    pub(super) fn new() -> Self {
        Pulse {
            duty: 0,
            step: 0,
            halt_length: false,
            volume: 0,
            timer_period: 0,
            timer: 0,
            length: 0,
        }
    }

    /// `register` is 0-3, relative to the channel's first address.
    pub(super) fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => {
                self.duty = value >> 6;
                self.halt_length = value & 0x20 != 0;
                self.volume = value & 0x0F;
            }
            1 => {} // sweep: not yet
            2 => self.timer_period = (self.timer_period & 0x0700) | u16::from(value),
            3 => {
                self.timer_period = (self.timer_period & 0x00FF) | (u16::from(value & 0x07) << 8);
                self.length = LENGTH_TABLE[usize::from(value >> 3)];
                self.step = 0;
            }
            _ => unreachable!("pulse register {register}"),
        }
    }

    pub(super) fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            // The hardware sequencer counts down: 0, 7, 6, ..., 1.
            self.step = (self.step + 7) % 8;
        } else {
            self.timer -= 1;
        }
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "clocked by the frame counter in the next commit")
    )]
    pub(super) fn clock_length(&mut self) {
        if !self.halt_length && self.length > 0 {
            self.length -= 1;
        }
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "read by the mixer in a later commit")
    )]
    pub(super) fn output(&self) -> u8 {
        // Periods under 8 would be ultrasonic; the hardware silences them.
        if self.length == 0 || self.timer_period < 8 {
            return 0;
        }
        DUTY_TABLE[usize::from(self.duty)][usize::from(self.step)] * self.volume
    }
}
