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
    Indirect,
    IndirectX,
    IndirectY,
    Accumulator,
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
                self.logical(bus, Immediate, ops::and);
                2
            }
            0x25 => {
                self.logical(bus, ZeroPage, ops::and);
                3
            }
            0x35 => {
                self.logical(bus, ZeroPageX, ops::and);
                4
            }
            0x2D => {
                self.logical(bus, Absolute, ops::and);
                4
            }
            0x3D => {
                let page_crossed = self.logical(bus, AbsoluteX, ops::and);
                if page_crossed { 5 } else { 4 }
            }
            0x39 => {
                let page_crossed = self.logical(bus, AbsoluteY, ops::and);
                if page_crossed { 5 } else { 4 }
            }
            0x21 => {
                self.logical(bus, IndirectX, ops::and);
                6
            }
            0x31 => {
                let page_crossed = self.logical(bus, IndirectY, ops::and);
                if page_crossed { 6 } else { 5 }
            }
            // ASL
            0x0A => {
                self.shift(bus, Accumulator, ops::asl);
                2
            }
            0x06 => {
                self.shift(bus, ZeroPage, ops::asl);
                5
            }
            0x16 => {
                self.shift(bus, ZeroPageX, ops::asl);
                6
            }
            0x0E => {
                self.shift(bus, Absolute, ops::asl);
                6
            }
            0x1E => {
                self.shift(bus, AbsoluteX, ops::asl);
                7
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
                self.modify(bus, ZeroPage, ops::dec);
                5
            }
            0xD6 => {
                self.modify(bus, ZeroPageX, ops::dec);
                6
            }
            0xCE => {
                self.modify(bus, Absolute, ops::dec);
                6
            }
            0xDE => {
                self.modify(bus, AbsoluteX, ops::dec);
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
                self.logical(bus, Immediate, ops::eor);
                2
            }
            0x45 => {
                self.logical(bus, ZeroPage, ops::eor);
                3
            }
            0x55 => {
                self.logical(bus, ZeroPageX, ops::eor);
                4
            }
            0x4D => {
                self.logical(bus, Absolute, ops::eor);
                4
            }
            0x5D => {
                let page_crossed = self.logical(bus, AbsoluteX, ops::eor);
                if page_crossed { 5 } else { 4 }
            }
            0x59 => {
                let page_crossed = self.logical(bus, AbsoluteY, ops::eor);
                if page_crossed { 5 } else { 4 }
            }
            0x41 => {
                self.logical(bus, IndirectX, ops::eor);
                6
            }
            0x51 => {
                let page_crossed = self.logical(bus, IndirectY, ops::eor);
                if page_crossed { 6 } else { 5 }
            }
            // INC
            0xE6 => {
                self.modify(bus, ZeroPage, ops::inc);
                5
            }
            0xF6 => {
                self.modify(bus, ZeroPageX, ops::inc);
                6
            }
            0xEE => {
                self.modify(bus, Absolute, ops::inc);
                6
            }
            0xFE => {
                self.modify(bus, AbsoluteX, ops::inc);
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
            // JMP
            0x4C => {
                let (address, _) = self.get_address_by_mode(bus, Absolute);
                self.registers.pc = address;
                3
            }
            0x6C => {
                let (address, _) = self.get_address_by_mode(bus, Indirect);
                self.registers.pc = address;
                5
            }
            // JSR
            0x20 => {
                let (address, _) = self.get_address_by_mode(bus, Absolute);
                self.push_word(bus, self.registers.pc.wrapping_sub(1));
                self.registers.pc = address;
                6
            }
            // LDA
            0xA9 => {
                self.load(bus, Immediate, ops::lda);
                2
            }
            0xA5 => {
                self.load(bus, ZeroPage, ops::lda);
                3
            }
            0xB5 => {
                self.load(bus, ZeroPageX, ops::lda);
                4
            }
            0xAD => {
                self.load(bus, Absolute, ops::lda);
                4
            }
            0xBD => {
                let page_crossed = self.load(bus, AbsoluteX, ops::lda);
                if page_crossed { 5 } else { 4 }
            }
            0xB9 => {
                let page_crossed = self.load(bus, AbsoluteY, ops::lda);
                if page_crossed { 5 } else { 4 }
            }
            0xA1 => {
                self.load(bus, IndirectX, ops::lda);
                6
            }
            0xB1 => {
                let page_crossed = self.load(bus, IndirectY, ops::lda);
                if page_crossed { 6 } else { 5 }
            }
            // LDX
            0xA2 => {
                self.load(bus, Immediate, ops::ldx);
                2
            }
            0xA6 => {
                self.load(bus, ZeroPage, ops::ldx);
                3
            }
            0xB6 => {
                self.load(bus, ZeroPageY, ops::ldx);
                4
            }
            0xAE => {
                self.load(bus, Absolute, ops::ldx);
                4
            }
            0xBE => {
                let page_crossed = self.load(bus, AbsoluteY, ops::ldx);
                if page_crossed { 5 } else { 4 }
            }
            // LDY
            0xA0 => {
                self.load(bus, Immediate, ops::ldy);
                2
            }
            0xA4 => {
                self.load(bus, ZeroPage, ops::ldy);
                3
            }
            0xB4 => {
                self.load(bus, ZeroPageX, ops::ldy);
                4
            }
            0xAC => {
                self.load(bus, Absolute, ops::ldy);
                4
            }
            0xBC => {
                let page_crossed = self.load(bus, AbsoluteX, ops::ldy);
                if page_crossed { 5 } else { 4 }
            }
            // LSR
            0x4A => {
                self.shift(bus, Accumulator, ops::lsr);
                2
            }
            0x46 => {
                self.shift(bus, ZeroPage, ops::lsr);
                5
            }
            0x56 => {
                self.shift(bus, ZeroPageX, ops::lsr);
                6
            }
            0x4E => {
                self.shift(bus, Absolute, ops::lsr);
                6
            }
            0x5E => {
                self.shift(bus, AbsoluteX, ops::lsr);
                7
            }
            // ORA
            0x09 => {
                self.logical(bus, Immediate, ops::ora);
                2
            }
            0x05 => {
                self.logical(bus, ZeroPage, ops::ora);
                3
            }
            0x15 => {
                self.logical(bus, ZeroPageX, ops::ora);
                4
            }
            0x0D => {
                self.logical(bus, Absolute, ops::ora);
                4
            }
            0x1D => {
                let page_crossed = self.logical(bus, AbsoluteX, ops::ora);
                if page_crossed { 5 } else { 4 }
            }
            0x19 => {
                let page_crossed = self.logical(bus, AbsoluteY, ops::ora);
                if page_crossed { 5 } else { 4 }
            }
            0x01 => {
                self.logical(bus, IndirectX, ops::ora);
                6
            }
            0x11 => {
                let page_crossed = self.logical(bus, IndirectY, ops::ora);
                if page_crossed { 6 } else { 5 }
            }
            // PHA
            0x48 => {
                self.push(bus, self.registers.a);
                3
            }
            // PHP
            0x08 => {
                self.push(bus, self.status.0 | Status::BREAK_COMMAND | Status::UNUSED);
                3
            }
            // PLA
            0x68 => {
                self.registers.a = self.pull(bus);
                self.status.set_zero_and_negative(self.registers.a);
                4
            }
            // PLP
            0x28 => {
                let pulled = self.pull(bus);
                self.status.0 =
                    (pulled & !(Status::BREAK_COMMAND | Status::UNUSED)) | Status::UNUSED;
                4
            }
            // ROL
            0x2A => {
                self.shift(bus, Accumulator, ops::rol);
                2
            }
            0x26 => {
                self.shift(bus, ZeroPage, ops::rol);
                5
            }
            0x36 => {
                self.shift(bus, ZeroPageX, ops::rol);
                6
            }
            0x2E => {
                self.shift(bus, Absolute, ops::rol);
                6
            }
            0x3E => {
                self.shift(bus, AbsoluteX, ops::rol);
                7
            }
            // ROR
            0x6A => {
                self.shift(bus, Accumulator, ops::ror);
                2
            }
            0x66 => {
                self.shift(bus, ZeroPage, ops::ror);
                5
            }
            0x76 => {
                self.shift(bus, ZeroPageX, ops::ror);
                6
            }
            0x6E => {
                self.shift(bus, Absolute, ops::ror);
                6
            }
            0x7E => {
                self.shift(bus, AbsoluteX, ops::ror);
                7
            }
            // RTS
            0x60 => {
                self.registers.pc = self.pull_word(bus).wrapping_add(1);
                6
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

    fn push_word<T: Bus>(&mut self, bus: &mut T, word: u16) {
        let [low, high] = word.to_le_bytes();
        self.push(bus, high);
        self.push(bus, low);
    }

    fn pull_word<T: Bus>(&mut self, bus: &mut T) -> u16 {
        let low = self.pull(bus);
        let high = self.pull(bus);
        u16::from_le_bytes([low, high])
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
            Indirect => {
                let pointer = self.fetch_word(bus);
                let [low, high] = pointer.to_le_bytes();
                (
                    u16::from_le_bytes([
                        bus.read(pointer),
                        // 6502 bug: high byte never crosses a page
                        bus.read(u16::from_le_bytes([low.wrapping_add(1), high])),
                    ]),
                    false,
                )
            }
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
            Accumulator => {
                panic!("accumulator not supported by get_address_by_mode")
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

    fn shift<T: Bus>(
        &mut self,
        bus: &mut T,
        mode: AddressingMode,
        f: impl FnOnce(u8, bool) -> (u8, bool),
    ) {
        let old_carry = self.status.is_set(Status::CARRY);
        let (value, carry) = match mode {
            Accumulator => {
                let (value, carry) = f(self.registers.a, old_carry);
                self.registers.a = value;
                (value, carry)
            }
            _ => {
                let (address, _) = self.get_address_by_mode(bus, mode);
                let (value, carry) = f(bus.read(address), old_carry);
                bus.write(address, value);
                (value, carry)
            }
        };
        self.status.set(Status::CARRY, carry);
        self.status.set_zero_and_negative(value);
    }

    fn push<T: Bus>(&mut self, bus: &mut T, value: u8) {
        bus.write(0x0100 + u16::from(self.registers.sp), value);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
    }

    fn pull<T: Bus>(&mut self, bus: &mut T) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        bus.read(0x0100 + u16::from(self.registers.sp))
    }
}

pub struct Registers {
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

mod ops;

#[cfg(test)]
mod tests;
