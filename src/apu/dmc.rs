/// Timer periods in CPU cycles.
const RATES: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

/// Delta modulation channel: plays 1-bit delta-encoded samples read from PRG.
pub(super) struct Dmc {
    irq_enabled: bool,
    pub(super) irq_pending: bool,
    pub(super) looping: bool,
    timer_period: u16,
    timer: u16,
    pub(super) level: u8,
    sample_address: u16,
    sample_length: u16,
    pub(super) address: u16,
    pub(super) bytes_remaining: u16,
    pub(super) buffer: Option<u8>,
    shift: u8,
    bits_remaining: u8,
    silent: bool,
}

impl Dmc {
    pub(super) fn new() -> Self {
        Dmc {
            irq_enabled: false,
            irq_pending: false,
            looping: false,
            timer_period: RATES[0],
            timer: 0,
            level: 0,
            sample_address: 0xC000,
            sample_length: 1,
            address: 0xC000,
            bytes_remaining: 0,
            buffer: None,
            shift: 0,
            bits_remaining: 8,
            silent: true,
        }
    }

    /// `register` is 0-3, relative to $4010.
    pub(super) fn write(&mut self, register: u16, value: u8) {
        match register {
            0 => {
                self.irq_enabled = value & 0x80 != 0;
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
                self.looping = value & 0x40 != 0;
                self.timer_period = RATES[usize::from(value & 0x0F)];
            }
            1 => self.level = value & 0x7F,
            2 => self.sample_address = 0xC000 + u16::from(value) * 64,
            3 => self.sample_length = u16::from(value) * 16 + 1,
            _ => unreachable!("DMC register {register}"),
        }
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.irq_pending = false;
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.restart();
        }
    }

    fn restart(&mut self) {
        self.address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }

    /// The address the memory reader wants next, if its buffer is empty.
    pub(super) fn fetch_address(&self) -> Option<u16> {
        (self.buffer.is_none() && self.bytes_remaining > 0).then_some(self.address)
    }

    pub(super) fn fill(&mut self, byte: u8) {
        self.buffer = Some(byte);
        // The address wraps from $FFFF to $8000, not $0000.
        self.address = self.address.checked_add(1).unwrap_or(0x8000);
        self.bytes_remaining -= 1;
        if self.bytes_remaining == 0 {
            if self.looping {
                self.restart();
            } else if self.irq_enabled {
                self.irq_pending = true;
            }
        }
    }

    /// Clocked every CPU cycle, since `RATES` is in CPU cycles.
    pub(super) fn clock_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = self.timer_period - 1;
        if !self.silent {
            if self.shift & 1 != 0 {
                if self.level <= 125 {
                    self.level += 2;
                }
            } else if self.level >= 2 {
                self.level -= 2;
            }
        }
        self.shift >>= 1;
        self.bits_remaining -= 1;
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            match self.buffer.take() {
                Some(byte) => {
                    self.shift = byte;
                    self.silent = false;
                }
                None => self.silent = true,
            }
        }
    }

    pub(super) fn output(&self) -> u8 {
        self.level
    }
}
