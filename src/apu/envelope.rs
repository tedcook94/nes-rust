/// Volume control shared by the pulse and noise channels: either a constant
/// volume or a sawtooth that decays from 15 to 0.
pub(super) struct Envelope {
    pub(super) start: bool,
    /// Doubles as the length counter halt flag.
    pub(super) looping: bool,
    pub(super) constant: bool,
    /// Constant volume, or the decay period when not constant.
    pub(super) value: u8,
    divider: u8,
    pub(super) decay: u8,
}

impl Envelope {
    pub(super) fn new() -> Self {
        Envelope {
            start: false,
            looping: false,
            constant: false,
            value: 0,
            divider: 0,
            decay: 0,
        }
    }

    /// Takes the low 6 bits of the channel's first register.
    pub(super) fn write(&mut self, value: u8) {
        self.looping = value & 0x20 != 0;
        self.constant = value & 0x10 != 0;
        self.value = value & 0x0F;
    }

    pub(super) fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.value;
        } else if self.divider == 0 {
            self.divider = self.value;
            if self.decay > 0 {
                self.decay -= 1;
            } else if self.looping {
                self.decay = 15;
            }
        } else {
            self.divider -= 1;
        }
    }

    pub(super) fn volume(&self) -> u8 {
        if self.constant {
            self.value
        } else {
            self.decay
        }
    }
}
