use crate::{bus::Bus, cpu::AddressingMode::*};

pub struct Cpu {
    registers: Registers,
    status: Status,
    cycle_count: u64,
}

#[derive(Debug)]
enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            registers: Registers {
                a: 0,
                x: 0,
                y: 0,
                sp: 0xFD,
                pc: 0,
            },
            status: Status(Status::INTERRUPT_DISABLE | Status::UNUSED),
            cycle_count: 0,
        }
    }

    fn step<T: Bus>(&mut self, bus: &mut T) -> u8 {
        let opcode = self.fetch_byte(bus);
        let cycles = match opcode {
            // LDA
            0xA9 => {
                let _ = self.lda(bus, Immediate);
                2
            }
            0xA5 => {
                let _ = self.lda(bus, ZeroPage);
                3
            }
            0xB5 => {
                let _ = self.lda(bus, ZeroPageX);
                4
            }
            0xAD => {
                let _ = self.lda(bus, Absolute);
                4
            }
            0xBD => {
                let page_crossed = self.lda(bus, AbsoluteX);
                if page_crossed { 5 } else { 4 }
            }
            0xB9 => {
                let page_crossed = self.lda(bus, AbsoluteY);
                if page_crossed { 5 } else { 4 }
            }
            0xA1 => {
                let _ = self.lda(bus, IndirectX);
                6
            }
            0xB1 => {
                let page_crossed = self.lda(bus, IndirectY);
                if page_crossed { 6 } else { 5 }
            }
            // LDX
            0xA2 => {
                let _ = self.ldx(bus, Immediate);
                2
            }
            0xA6 => {
                let _ = self.ldx(bus, ZeroPage);
                3
            }
            0xB6 => {
                let _ = self.ldx(bus, ZeroPageY);
                4
            }
            0xAE => {
                let _ = self.ldx(bus, Absolute);
                4
            }
            0xBE => {
                let page_crossed = self.ldx(bus, AbsoluteY);
                if page_crossed { 5 } else { 4 }
            }
            // LDY
            0xA0 => {
                let _ = self.ldy(bus, Immediate);
                2
            }
            0xA4 => {
                let _ = self.ldy(bus, ZeroPage);
                3
            }
            0xB4 => {
                let _ = self.ldy(bus, ZeroPageX);
                4
            }
            0xAC => {
                let _ = self.ldy(bus, Absolute);
                4
            }
            0xBC => {
                let page_crossed = self.ldy(bus, AbsoluteX);
                if page_crossed { 5 } else { 4 }
            }
            // STA
            0x85 => {
                self.store(bus, ZeroPage, self.registers.a);
                3
            }
            0x95 => {
                self.store(bus, ZeroPageX, self.registers.a);
                4
            }
            0x8D => {
                self.store(bus, Absolute, self.registers.a);
                4
            }
            0x9D => {
                self.store(bus, AbsoluteX, self.registers.a);
                5
            }
            0x99 => {
                self.store(bus, AbsoluteY, self.registers.a);
                5
            }
            0x81 => {
                self.store(bus, IndirectX, self.registers.a);
                6
            }
            0x91 => {
                self.store(bus, IndirectY, self.registers.a);
                6
            }
            // STX
            0x86 => {
                self.store(bus, ZeroPage, self.registers.x);
                3
            }
            0x96 => {
                self.store(bus, ZeroPageY, self.registers.x);
                4
            }
            0x8E => {
                self.store(bus, Absolute, self.registers.x);
                4
            }
            // STY
            0x84 => {
                self.store(bus, ZeroPage, self.registers.y);
                3
            }
            0x94 => {
                self.store(bus, ZeroPageX, self.registers.y);
                4
            }
            0x8C => {
                self.store(bus, Absolute, self.registers.y);
                4
            }
            // Transfers
            0xAA => {
                self.registers.x = self.registers.a;
                self.status.set_zero_and_negative(self.registers.x);
                2
            }
            0xA8 => {
                self.registers.y = self.registers.a;
                self.status.set_zero_and_negative(self.registers.y);
                2
            }
            0xBA => {
                self.registers.x = self.registers.sp;
                self.status.set_zero_and_negative(self.registers.x);
                2
            }
            0x8A => {
                self.registers.a = self.registers.x;
                self.status.set_zero_and_negative(self.registers.a);
                2
            }
            0x9A => {
                self.registers.sp = self.registers.x;
                // TXS  is the only transfer that doesn't set flags
                2
            }
            0x98 => {
                self.registers.a = self.registers.y;
                self.status.set_zero_and_negative(self.registers.a);
                2
            }
            _ => panic!("unimplemented opcode {:#04X}", opcode),
        };
        self.cycle_count += u64::from(cycles);
        cycles
    }

    fn take_pc(&mut self) -> u16 {
        let address = self.registers.pc;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        address
    }

    fn fetch_byte<T: Bus>(&mut self, bus: &mut T) -> u8 {
        let byte = bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }

    fn fetch_word<T: Bus>(&mut self, bus: &mut T) -> u16 {
        u16::from_le_bytes([self.fetch_byte(bus), self.fetch_byte(bus)])
    }

    fn get_indexed_address<T: Bus>(&mut self, bus: &mut T, index: u8) -> (u16, bool) {
        let base = self.fetch_word(bus);
        let address = base.wrapping_add(u16::from(index));
        (address, base & 0xFF00 != address & 0xFF00)
    }

    fn get_address_by_mode<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) -> (u16, bool) {
        match mode {
            Immediate => (self.take_pc(), false),
            ZeroPage => (u16::from(self.fetch_byte(bus)), false),
            ZeroPageX => (
                u16::from(self.fetch_byte(bus).wrapping_add(self.registers.x)),
                false,
            ),
            ZeroPageY => (
                u16::from(self.fetch_byte(bus).wrapping_add(self.registers.y)),
                false,
            ),
            Absolute => (self.fetch_word(bus), false),
            AbsoluteX => self.get_indexed_address(bus, self.registers.x),
            AbsoluteY => self.get_indexed_address(bus, self.registers.y),
            IndirectX => {
                let nn = self.fetch_byte(bus);
                (
                    u16::from_le_bytes([
                        bus.read(u16::from(nn.wrapping_add(self.registers.x))),
                        bus.read(u16::from(nn.wrapping_add(self.registers.x).wrapping_add(1))),
                    ]),
                    false,
                )
            }
            IndirectY => {
                let nn = self.fetch_byte(bus);
                let pointer = u16::from_le_bytes([
                    bus.read(u16::from(nn)),
                    bus.read(u16::from(nn.wrapping_add(1))),
                ]);
                let indexed_pointer = pointer.wrapping_add(u16::from(self.registers.y));
                (
                    indexed_pointer,
                    pointer & 0xFF00 != indexed_pointer & 0xFF00,
                )
            }
        }
    }

    fn read_operand<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) -> (u8, bool) {
        let (address, page_crossed) = self.get_address_by_mode(bus, mode);
        (bus.read(address), page_crossed)
    }

    fn lda<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) -> bool {
        let (operand, page_crossed) = self.read_operand(bus, mode);
        self.registers.a = operand;
        self.status.set_zero_and_negative(operand);
        page_crossed
    }

    fn ldx<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) -> bool {
        let (operand, page_crossed) = self.read_operand(bus, mode);
        self.registers.x = operand;
        self.status.set_zero_and_negative(operand);
        page_crossed
    }

    fn ldy<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) -> bool {
        let (operand, page_crossed) = self.read_operand(bus, mode);
        self.registers.y = operand;
        self.status.set_zero_and_negative(operand);
        page_crossed
    }

    fn store<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode, value: u8) {
        let (address, _) = self.get_address_by_mode(bus, mode);
        bus.write(address, value);
    }
}

struct Registers {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
}

struct Status(u8);

impl Status {
    const CARRY: u8 = 0b0000_0001;
    const ZERO: u8 = 0b0000_0010;
    const INTERRUPT_DISABLE: u8 = 0b0000_0100;
    const DECIMAL_MODE: u8 = 0b0000_1000;
    const BREAK_COMMAND: u8 = 0b0001_0000;
    const OVERFLOW: u8 = 0b0100_0000;
    const NEGATIVE: u8 = 0b1000_0000;
    const UNUSED: u8 = 0b0010_0000;

    fn set(&mut self, flag: u8, val: bool) {
        if val { self.0 |= flag } else { self.0 &= !flag }
    }

    fn is_set(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn set_zero_and_negative(&mut self, value: u8) {
        self.set(Status::ZERO, value == 0x00);
        self.set(Status::NEGATIVE, value & 0x80 != 0); // check bit 7 (sign bit)
    }
}

#[cfg(test)]
mod tests;
