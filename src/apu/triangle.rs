use super::LENGTH_TABLE;

const SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, //
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

pub(super) struct Triangle {
    /// Halts the length counter and keeps reloading the linear counter.
    pub(super) control: bool,
    pub(super) linear_reload_value: u8,
    pub(super) linear_reload: bool,
    pub(super) linear: u8,
    pub(super) step: u8,
    pub(super) timer_period: u16,
    timer: u16,
    pub(super) length: u8,
}

impl Triangle {
    pub(super) fn new() -> Self {
        Triangle {
            control: false,
            linear_reload_value: 0,
            linear_reload: false,
            linear: 0,
            step: 0,
            timer_period: 0,
            timer: 0,
            length: 0,
        }
    }

    /// `register` is 0-3, relative to $4008.
    pub(super) fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => {
                self.control = value & 0x80 != 0;
                self.linear_reload_value = value & 0x7F;
            }
            1 => {}
            2 => self.timer_period = (self.timer_period & 0x0700) | u16::from(value),
            3 => {
                self.timer_period = (self.timer_period & 0x00FF) | (u16::from(value & 0x07) << 8);
                self.length = LENGTH_TABLE[usize::from(value >> 3)];
                self.linear_reload = true;
            }
            _ => unreachable!("triangle register {register}"),
        }
    }

    /// Runs at the full CPU rate, unlike the other channels.
    pub(super) fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            // Silencing freezes the sequencer instead of dropping output to 0, which avoids pops.
            if self.length > 0 && self.linear > 0 {
                self.step = (self.step + 1) % 32;
            }
        } else {
            self.timer -= 1;
        }
    }

    pub(super) fn clock_linear(&mut self) {
        if self.linear_reload {
            self.linear = self.linear_reload_value;
        } else if self.linear > 0 {
            self.linear -= 1;
        }
        if !self.control {
            self.linear_reload = false;
        }
    }

    pub(super) fn clock_length(&mut self) {
        if !self.control && self.length > 0 {
            self.length -= 1;
        }
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "read by the mixer in a later commit")
    )]
    pub(super) fn output(&self) -> u8 {
        SEQUENCE[usize::from(self.step)]
    }
}
