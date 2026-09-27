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
            // AND
            0x29 => {
                self.logical(bus, Immediate, |a, m| a & m);
                2
            }
            0x25 => {
                self.logical(bus, ZeroPage, |a, m| a & m);
                3
            }
            0x35 => {
                self.logical(bus, ZeroPageX, |a, m| a & m);
                4
            }
            0x2D => {
                self.logical(bus, Absolute, |a, m| a & m);
                4
            }
            0x3D => {
                let page_crossed = self.logical(bus, AbsoluteX, |a, m| a & m);
                if page_crossed { 5 } else { 4 }
            }
            0x39 => {
                let page_crossed = self.logical(bus, AbsoluteY, |a, m| a & m);
                if page_crossed { 5 } else { 4 }
            }
            0x21 => {
                self.logical(bus, IndirectX, |a, m| a & m);
                6
            }
            0x31 => {
                let page_crossed = self.logical(bus, IndirectY, |a, m| a & m);
                if page_crossed { 6 } else { 5 }
            }
            // BIT
            0x24 => {
                self.bit(bus, ZeroPage);
                3
            }
            0x2C => {
                self.bit(bus, Absolute);
                4
            }
            // CLC
            0x18 => {
                self.status.set(Status::CARRY, false);
                2
            }
            // CLD
            0xD8 => {
                self.status.set(Status::DECIMAL_MODE, false);
                2
            }
            // CLI
            0x58 => {
                self.status.set(Status::INTERRUPT_DISABLE, false);
                2
            }
            // CLV
            0xB8 => {
                self.status.set(Status::OVERFLOW, false);
                2
            }
            // CMP
            0xC9 => {
                self.compare(bus, Immediate, self.registers.a);
                2
            }
            0xC5 => {
                self.compare(bus, ZeroPage, self.registers.a);
                3
            }
            0xD5 => {
                self.compare(bus, ZeroPageX, self.registers.a);
                4
            }
            0xCD => {
                self.compare(bus, Absolute, self.registers.a);
                4
            }
            0xDD => {
                let page_crossed = self.compare(bus, AbsoluteX, self.registers.a);
                if page_crossed { 5 } else { 4 }
            }
            0xD9 => {
                let page_crossed = self.compare(bus, AbsoluteY, self.registers.a);
                if page_crossed { 5 } else { 4 }
            }
            0xC1 => {
                self.compare(bus, IndirectX, self.registers.a);
                6
            }
            0xD1 => {
                let page_crossed = self.compare(bus, IndirectY, self.registers.a);
                if page_crossed { 6 } else { 5 }
            }
            // CPX
            0xE0 => {
                self.compare(bus, Immediate, self.registers.x);
                2
            }
            0xE4 => {
                self.compare(bus, ZeroPage, self.registers.x);
                3
            }
            0xEC => {
                self.compare(bus, Absolute, self.registers.x);
                4
            }
            // CPY
            0xC0 => {
                self.compare(bus, Immediate, self.registers.y);
                2
            }
            0xC4 => {
                self.compare(bus, ZeroPage, self.registers.y);
                3
            }
            0xCC => {
                self.compare(bus, Absolute, self.registers.y);
                4
            }
            // DEC
            0xC6 => {
                self.modify(bus, ZeroPage, |v| v.wrapping_sub(1));
                5
            }
            0xD6 => {
                self.modify(bus, ZeroPageX, |v| v.wrapping_sub(1));
                6
            }
            0xCE => {
                self.modify(bus, Absolute, |v| v.wrapping_sub(1));
                6
            }
            0xDE => {
                self.modify(bus, AbsoluteX, |v| v.wrapping_sub(1));
                7
            }
            // DEX
            0xCA => {
                self.registers.x = self.registers.x.wrapping_sub(1);
                self.status.set_zero_and_negative(self.registers.x);
                2
            }
            // DEY
            0x88 => {
                self.registers.y = self.registers.y.wrapping_sub(1);
                self.status.set_zero_and_negative(self.registers.y);
                2
            }
            // EOR
            0x49 => {
                self.logical(bus, Immediate, |a, m| a ^ m);
                2
            }
            0x45 => {
                self.logical(bus, ZeroPage, |a, m| a ^ m);
                3
            }
            0x55 => {
                self.logical(bus, ZeroPageX, |a, m| a ^ m);
                4
            }
            0x4D => {
                self.logical(bus, Absolute, |a, m| a ^ m);
                4
            }
            0x5D => {
                let page_crossed = self.logical(bus, AbsoluteX, |a, m| a ^ m);
                if page_crossed { 5 } else { 4 }
            }
            0x59 => {
                let page_crossed = self.logical(bus, AbsoluteY, |a, m| a ^ m);
                if page_crossed { 5 } else { 4 }
            }
            0x41 => {
                self.logical(bus, IndirectX, |a, m| a ^ m);
                6
            }
            0x51 => {
                let page_crossed = self.logical(bus, IndirectY, |a, m| a ^ m);
                if page_crossed { 6 } else { 5 }
            }
            // INC
            0xE6 => {
                self.modify(bus, ZeroPage, |v| v.wrapping_add(1));
                5
            }
            0xF6 => {
                self.modify(bus, ZeroPageX, |v| v.wrapping_add(1));
                6
            }
            0xEE => {
                self.modify(bus, Absolute, |v| v.wrapping_add(1));
                6
            }
            0xFE => {
                self.modify(bus, AbsoluteX, |v| v.wrapping_add(1));
                7
            }
            // INX
            0xE8 => {
                self.registers.x = self.registers.x.wrapping_add(1);
                self.status.set_zero_and_negative(self.registers.x);
                2
            }
            // INY
            0xC8 => {
                self.registers.y = self.registers.y.wrapping_add(1);
                self.status.set_zero_and_negative(self.registers.y);
                2
            }
            // LDA
            0xA9 => {
                self.load(bus, Immediate, |r, v| r.a = v);
                2
            }
            0xA5 => {
                self.load(bus, ZeroPage, |r, v| r.a = v);
                3
            }
            0xB5 => {
                self.load(bus, ZeroPageX, |r, v| r.a = v);
                4
            }
            0xAD => {
                self.load(bus, Absolute, |r, v| r.a = v);
                4
            }
            0xBD => {
                let page_crossed = self.load(bus, AbsoluteX, |r, v| r.a = v);
                if page_crossed { 5 } else { 4 }
            }
            0xB9 => {
                let page_crossed = self.load(bus, AbsoluteY, |r, v| r.a = v);
                if page_crossed { 5 } else { 4 }
            }
            0xA1 => {
                self.load(bus, IndirectX, |r, v| r.a = v);
                6
            }
            0xB1 => {
                let page_crossed = self.load(bus, IndirectY, |r, v| r.a = v);
                if page_crossed { 6 } else { 5 }
            }
            // LDX
            0xA2 => {
                self.load(bus, Immediate, |r, v| r.x = v);
                2
            }
            0xA6 => {
                self.load(bus, ZeroPage, |r, v| r.x = v);
                3
            }
            0xB6 => {
                self.load(bus, ZeroPageY, |r, v| r.x = v);
                4
            }
            0xAE => {
                self.load(bus, Absolute, |r, v| r.x = v);
                4
            }
            0xBE => {
                let page_crossed = self.load(bus, AbsoluteY, |r, v| r.x = v);
                if page_crossed { 5 } else { 4 }
            }
            // LDY
            0xA0 => {
                self.load(bus, Immediate, |r, v| r.y = v);
                2
            }
            0xA4 => {
                self.load(bus, ZeroPage, |r, v| r.y = v);
                3
            }
            0xB4 => {
                self.load(bus, ZeroPageX, |r, v| r.y = v);
                4
            }
            0xAC => {
                self.load(bus, Absolute, |r, v| r.y = v);
                4
            }
            0xBC => {
                let page_crossed = self.load(bus, AbsoluteX, |r, v| r.y = v);
                if page_crossed { 5 } else { 4 }
            }
            // ORA
            0x09 => {
                self.logical(bus, Immediate, |a, m| a | m);
                2
            }
            0x05 => {
                self.logical(bus, ZeroPage, |a, m| a | m);
                3
            }
            0x15 => {
                self.logical(bus, ZeroPageX, |a, m| a | m);
                4
            }
            0x0D => {
                self.logical(bus, Absolute, |a, m| a | m);
                4
            }
            0x1D => {
                let page_crossed = self.logical(bus, AbsoluteX, |a, m| a | m);
                if page_crossed { 5 } else { 4 }
            }
            0x19 => {
                let page_crossed = self.logical(bus, AbsoluteY, |a, m| a | m);
                if page_crossed { 5 } else { 4 }
            }
            0x01 => {
                self.logical(bus, IndirectX, |a, m| a | m);
                6
            }
            0x11 => {
                let page_crossed = self.logical(bus, IndirectY, |a, m| a | m);
                if page_crossed { 6 } else { 5 }
            }
            // SEC
            0x38 => {
                self.status.set(Status::CARRY, true);
                2
            }
            // SED
            0xF8 => {
                self.status.set(Status::DECIMAL_MODE, true);
                2
            }
            // SEI
            0x78 => {
                self.status.set(Status::INTERRUPT_DISABLE, true);
                2
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
            // TAX
            0xAA => {
                self.registers.x = self.registers.a;
                self.status.set_zero_and_negative(self.registers.x);
                2
            }
            // TAY
            0xA8 => {
                self.registers.y = self.registers.a;
                self.status.set_zero_and_negative(self.registers.y);
                2
            }
            // TSX
            0xBA => {
                self.registers.x = self.registers.sp;
                self.status.set_zero_and_negative(self.registers.x);
                2
            }
            // TXA
            0x8A => {
                self.registers.a = self.registers.x;
                self.status.set_zero_and_negative(self.registers.a);
                2
            }
            // TXS
            0x9A => {
                self.registers.sp = self.registers.x;
                // TXS is the only transfer that doesn't set flags
                2
            }
            // TYA
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

    fn load<T: Bus>(
        &mut self,
        bus: &mut T,
        mode: AddressingMode,
        f: impl FnOnce(&mut Registers, u8),
    ) -> bool {
        let (operand, page_crossed) = self.read_operand(bus, mode);
        f(&mut self.registers, operand);
        self.status.set_zero_and_negative(operand);
        page_crossed
    }

    fn store<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode, register: u8) {
        let (address, _) = self.get_address_by_mode(bus, mode);
        bus.write(address, register);
    }

    fn modify<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode, f: impl FnOnce(u8) -> u8) {
        let (address, _) = self.get_address_by_mode(bus, mode);
        let value = f(bus.read(address));
        bus.write(address, value);
        self.status.set_zero_and_negative(value);
    }

    fn logical<T: Bus>(
        &mut self,
        bus: &mut T,
        mode: AddressingMode,
        f: impl FnOnce(u8, u8) -> u8,
    ) -> bool {
        let (operand, page_crossed) = self.read_operand(bus, mode);
        self.registers.a = f(self.registers.a, operand);
        self.status.set_zero_and_negative(self.registers.a);
        page_crossed
    }

    fn bit<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) {
        let (operand, _) = self.read_operand(bus, mode);
        self.status
            .set(Status::ZERO, self.registers.a & operand == 0);
        self.status.set(Status::NEGATIVE, operand & 0x80 != 0); // check bit 7 (sign bit)
        self.status.set(Status::OVERFLOW, operand & 0x40 != 0); // check bit 6 (overflow bit)
    }

    fn compare<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode, register: u8) -> bool {
        let (operand, page_crossed) = self.read_operand(bus, mode);
        self.status.set(Status::CARRY, register >= operand);
        self.status
            .set_zero_and_negative(register.wrapping_sub(operand));
        page_crossed
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
