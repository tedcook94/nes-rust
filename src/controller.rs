/// Buttons in the order the controller shifts them out.
pub mod button {
    pub const A: u8 = 1 << 0;
    pub const B: u8 = 1 << 1;
    pub const SELECT: u8 = 1 << 2;
    pub const START: u8 = 1 << 3;
    pub const UP: u8 = 1 << 4;
    pub const DOWN: u8 = 1 << 5;
    pub const LEFT: u8 = 1 << 6;
    pub const RIGHT: u8 = 1 << 7;
}

pub struct Controller {
    buttons: u8,
    shift: u8,
    strobe: bool,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            buttons: 0,
            shift: 0,
            strobe: false,
        }
    }

    pub fn set_buttons(&mut self, buttons: u8) {
        self.buttons = buttons;
        if self.strobe {
            self.shift = buttons;
        }
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value & 1 != 0;
        if self.strobe {
            self.shift = self.buttons;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.strobe {
            return self.buttons & 1;
        }
        let bit = self.shift & 1;
        // After 8 reads, official controllers return 1.
        self.shift = (self.shift >> 1) | 0x80;
        bit
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
