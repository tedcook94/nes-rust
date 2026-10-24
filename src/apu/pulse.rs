use super::LENGTH_TABLE;
use super::envelope::Envelope;

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 25% inverted
];

pub(super) struct Sweep {
    pub(super) enabled: bool,
    pub(super) period: u8,
    pub(super) negate: bool,
    pub(super) shift: u8,
    pub(super) reload: bool,
    pub(super) divider: u8,
}

pub(super) struct Pulse {
    /// Pulse 1 negates with ones' complement (subtracting one extra); pulse 2 with two's.
    ones_complement: bool,
    pub(super) duty: u8,
    pub(super) step: u8,
    pub(super) envelope: Envelope,
    pub(super) sweep: Sweep,
    pub(super) timer_period: u16,
    pub(super) timer: u16,
    pub(super) length: u8,
}

impl Pulse {
    pub(super) fn new(ones_complement: bool) -> Self {
        Pulse {
            ones_complement,
            duty: 0,
            step: 0,
            envelope: Envelope::new(),
            sweep: Sweep {
                enabled: false,
                period: 0,
                negate: false,
                shift: 0,
                reload: false,
                divider: 0,
            },
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
                self.envelope.write(value);
            }
            1 => {
                self.sweep.enabled = value & 0x80 != 0;
                self.sweep.period = (value >> 4) & 0x07;
                self.sweep.negate = value & 0x08 != 0;
                self.sweep.shift = value & 0x07;
                self.sweep.reload = true;
            }
            2 => self.timer_period = (self.timer_period & 0x0700) | u16::from(value),
            3 => {
                self.timer_period = (self.timer_period & 0x00FF) | (u16::from(value & 0x07) << 8);
                self.length = LENGTH_TABLE[usize::from(value >> 3)];
                self.step = 0;
                self.envelope.start = true;
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

    pub(super) fn clock_length(&mut self) {
        if !self.envelope.looping && self.length > 0 {
            self.length -= 1;
        }
    }

    pub(super) fn clock_sweep(&mut self) {
        if self.sweep.divider == 0 && self.sweep.enabled && self.sweep.shift > 0 && !self.muted() {
            self.timer_period = self.sweep_target();
        }
        if self.sweep.divider == 0 || self.sweep.reload {
            self.sweep.divider = self.sweep.period;
            self.sweep.reload = false;
        } else {
            self.sweep.divider -= 1;
        }
    }

    pub(super) fn sweep_target(&self) -> u16 {
        let change = self.timer_period >> self.sweep.shift;
        if self.sweep.negate {
            let extra = u16::from(self.ones_complement);
            self.timer_period.saturating_sub(change + extra)
        } else {
            self.timer_period + change
        }
    }

    /// The sweep unit mutes the channel even when disabled, so it can't sweep out of range.
    fn muted(&self) -> bool {
        // Periods under 8 would be ultrasonic.
        self.timer_period < 8 || self.sweep_target() > 0x7FF
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "read by the mixer in a later commit")
    )]
    pub(super) fn output(&self) -> u8 {
        if self.length == 0 || self.muted() {
            return 0;
        }
        DUTY_TABLE[usize::from(self.duty)][usize::from(self.step)] * self.envelope.volume()
    }
}
