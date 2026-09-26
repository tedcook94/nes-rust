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

    fn lda<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) -> bool {
        let (address, page_crossed) = self.get_address_by_mode(bus, mode);
        let operand = bus.read(address);
        self.registers.a = operand;
        self.status.set_zero_and_negative(operand);
        page_crossed
    }

    fn ldx<T: Bus>(&mut self, bus: &mut T, mode: AddressingMode) -> bool {
        let (address, page_crossed) = self.get_address_by_mode(bus, mode);
        let operand = bus.read(address);
        self.registers.x = operand;
        self.status.set_zero_and_negative(operand);
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
mod tests {
    use super::*;

    struct TestBus([u8; 0x10000]);

    impl TestBus {
        fn new() -> Self {
            TestBus([0; 0x10000])
        }
    }

    impl Bus for TestBus {
        fn read(&mut self, address: u16) -> u8 {
            self.0[address as usize]
        }
        fn write(&mut self, address: u16, byte: u8) {
            self.0[address as usize] = byte
        }
    }

    const STARTING_ADDRESS: u16 = 0x8000;

    fn create_test_cpu_and_bus(bytes_to_write: &[u8]) -> (Cpu, TestBus) {
        let mut cpu = Cpu::new();
        cpu.registers.pc = STARTING_ADDRESS;
        let mut bus = TestBus::new();
        for (i, &v) in bytes_to_write.iter().enumerate() {
            bus.0[(STARTING_ADDRESS as usize) + i] = v;
        }
        (cpu, bus)
    }

    #[test]
    fn cpu_powers_on_in_proper_state() {
        let cpu = Cpu::new();
        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.x, 0);
        assert_eq!(cpu.registers.y, 0);
        assert_eq!(cpu.registers.sp, 0xFD);
        assert_eq!(cpu.registers.pc, 0);
        assert_eq!(cpu.status.0, 0x24);
        assert_eq!(cpu.cycle_count, 0);
    }

    // Status
    #[test]
    fn set_carry_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        assert_eq!(status.0, Status::CARRY);
    }

    #[test]
    fn set_zero_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::ZERO, true);
        assert!(status.is_set(Status::ZERO));
        assert_eq!(status.0, Status::ZERO);
    }

    #[test]
    fn set_interrupt_disable_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::INTERRUPT_DISABLE, true);
        assert!(status.is_set(Status::INTERRUPT_DISABLE));
        assert_eq!(status.0, Status::INTERRUPT_DISABLE);
    }

    #[test]
    fn set_decimal_mode_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::DECIMAL_MODE, true);
        assert!(status.is_set(Status::DECIMAL_MODE));
        assert_eq!(status.0, Status::DECIMAL_MODE);
    }

    #[test]
    fn set_break_command_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::BREAK_COMMAND, true);
        assert!(status.is_set(Status::BREAK_COMMAND));
        assert_eq!(status.0, Status::BREAK_COMMAND);
    }

    #[test]
    fn set_overflow_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::OVERFLOW, true);
        assert!(status.is_set(Status::OVERFLOW));
        assert_eq!(status.0, Status::OVERFLOW);
    }

    #[test]
    fn set_negative_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::NEGATIVE, true);
        assert!(status.is_set(Status::NEGATIVE));
        assert_eq!(status.0, Status::NEGATIVE);
    }

    #[test]
    fn set_can_clear_flag() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        status.set(Status::CARRY, false);
        assert!(!status.is_set(Status::CARRY));
    }

    #[test]
    fn setting_flag_is_idempotent() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
    }

    #[test]
    fn clearing_flag_is_idempotent() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        status.set(Status::CARRY, false);
        assert!(!status.is_set(Status::CARRY));
        status.set(Status::CARRY, false);
        assert!(!status.is_set(Status::CARRY));
    }

    #[test]
    fn can_clear_all_used_flags() {
        let mut status = Status(0b1111_1111);
        status.set(Status::CARRY, false);
        status.set(Status::ZERO, false);
        status.set(Status::INTERRUPT_DISABLE, false);
        status.set(Status::DECIMAL_MODE, false);
        status.set(Status::BREAK_COMMAND, false);
        status.set(Status::OVERFLOW, false);
        status.set(Status::NEGATIVE, false);
        assert_eq!(status.0, Status::UNUSED);
    }

    // LDA
    #[test]
    fn lda_immediate_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA9, 0x42]);
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x42, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycle_count, 2);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_immediate_sets_zero_flag() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA9, 0x00]);
        cpu.registers.a = 0xFF;
        cpu.status.set(Status::NEGATIVE, true);

        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x00, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycle_count, 2);
        assert!(cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_immediate_sets_negative_flag() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA9, 0x80]);
        cpu.registers.a = 0xFF;
        cpu.status.set(Status::ZERO, true);

        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x80, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycle_count, 2);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_zero_page_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA5, 0x10]);
        bus.0[0x10] = 0x01;
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x01, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.cycle_count, 3);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_zero_page_x_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB5, 0x10]);
        cpu.registers.x = 0x01;
        bus.0[0x11] = 0x01;
        bus.0[0x10] = 0x11; // decoy value at provided address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x01, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_zero_page_x_with_wraparound_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB5, 0xFF]);
        cpu.registers.x = 0x01;
        bus.0[0x00] = 0x01;
        bus.0[0xFF] = 0x11; // decoy value at provided address
        bus.0[0x100] = 0x10; // decoy value at address when adding as u16
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x01, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_absolute_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xAD, 0x10, 0x01]);
        bus.0[0x0110] = 0x11;
        bus.0[0x1001] = 0xFF; // decoy value at inverted address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_absolute_x_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBD, 0x10, 0x01]);
        cpu.registers.x = 0x01;
        bus.0[0x0111] = 0x11;
        bus.0[0x0110] = 0xFF; // decoy value at provided address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_absolute_x_with_page_crossed_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBD, 0xFF, 0x01]);
        cpu.registers.x = 0x01;
        bus.0[0x0200] = 0x11;
        bus.0[0x0100] = 0xFF; // decoy value at mis-added address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_absolute_x_wraps_around_address_space() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBD, 0xFF, 0xFF]);
        cpu.registers.x = 0x01;
        bus.0[0x0000] = 0x11;
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_absolute_y_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB9, 0x10, 0x01]);
        cpu.registers.y = 0x01;
        bus.0[0x0111] = 0x11;
        bus.0[0x0110] = 0xFF; // decoy value at provided address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_absolute_y_with_page_crossed_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB9, 0xFF, 0x01]);
        cpu.registers.y = 0x01;
        bus.0[0x0200] = 0x11;
        bus.0[0x0100] = 0xFF; // decoy value at mis-added address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_absolute_y_wraps_around_address_space() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB9, 0xFF, 0xFF]);
        cpu.registers.y = 0x01;
        bus.0[0x0000] = 0x11;
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_indirect_x_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA1, 0x10]);
        cpu.registers.x = 0x01;
        bus.0[0x10] = 0x01;
        bus.0[0x11] = 0x34;
        bus.0[0x12] = 0x12;
        bus.0[0x1234] = 0x11;
        bus.0[0x3401] = 0xFF; // decoy value at provided pointer address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 6);
        assert_eq!(cpu.cycle_count, 6);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_indirect_x_pointer_wraps() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA1, 0xFF]);
        cpu.registers.x = 0x00;
        bus.0[0xFF] = 0x34;
        bus.0[0x00] = 0x12;
        bus.0[0x1234] = 0x11;
        bus.0[0x0100] = 0xFF; // decoy value at malformed pointer address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 6);
        assert_eq!(cpu.cycle_count, 6);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_indirect_y_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB1, 0x10]);
        cpu.registers.y = 0x01;
        bus.0[0x10] = 0x34;
        bus.0[0x11] = 0x12;
        bus.0[0x1235] = 0x11;
        bus.0[0x1234] = 0x01; // decoy value at provided pointer address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_indirect_y_with_page_crossed_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB1, 0x10]);
        cpu.registers.y = 0x01;
        bus.0[0x10] = 0xFF;
        bus.0[0x11] = 0x00;
        bus.0[0x0100] = 0x11;
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 6);
        assert_eq!(cpu.cycle_count, 6);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn lda_indirect_y_pointer_wraps() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB1, 0xFF]);
        cpu.registers.y = 0x01;
        bus.0[0xFF] = 0x34;
        bus.0[0x00] = 0x12;
        bus.0[0x1235] = 0x11;
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.a, 0x11, "A = {:#04X}", cpu.registers.a);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    // LDX
    #[test]
    fn ldx_immediate_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA2, 0x42]);
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x42, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycle_count, 2);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_immediate_sets_zero_flag() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA2, 0x00]);
        cpu.registers.x = 0xFF;
        cpu.status.set(Status::NEGATIVE, true);

        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x00, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycle_count, 2);
        assert!(cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_immediate_sets_negative_flag() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA2, 0x80]);
        cpu.registers.x = 0xFF;
        cpu.status.set(Status::ZERO, true);

        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x80, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.cycle_count, 2);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_zero_page_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA6, 0x10]);
        bus.0[0x10] = 0x01;
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x01, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.cycle_count, 3);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_zero_page_y_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB6, 0x10]);
        cpu.registers.y = 0x01;
        bus.0[0x11] = 0x01;
        bus.0[0x10] = 0x11; // decoy value at provided address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x01, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_zero_page_y_with_wraparound_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB6, 0xFF]);
        cpu.registers.y = 0x01;
        bus.0[0x00] = 0x01;
        bus.0[0xFF] = 0x11; // decoy value at provided address
        bus.0[0x100] = 0x10; // decoy value at address when adding as u16
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x01, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_absolute_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xAE, 0x10, 0x01]);
        bus.0[0x0110] = 0x11;
        bus.0[0x1001] = 0xFF; // decoy value at inverted address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x11, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_absolute_y_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBE, 0x10, 0x01]);
        cpu.registers.y = 0x01;
        bus.0[0x0111] = 0x11;
        bus.0[0x0110] = 0xFF; // decoy value at provided address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x11, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.cycle_count, 4);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_absolute_y_with_page_crossed_loads_value() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBE, 0xFF, 0x01]);
        cpu.registers.y = 0x01;
        bus.0[0x0200] = 0x11;
        bus.0[0x0100] = 0xFF; // decoy value at mis-added address
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x11, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }

    #[test]
    fn ldx_absolute_y_wraps_around_address_space() {
        let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBE, 0xFF, 0xFF]);
        cpu.registers.y = 0x01;
        bus.0[0x0000] = 0x11;
        let cycles = cpu.step(&mut bus);
        assert_eq!(cpu.registers.x, 0x11, "X = {:#04X}", cpu.registers.x);
        assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.cycle_count, 5);
        assert!(!cpu.status.is_set(Status::ZERO));
        assert!(!cpu.status.is_set(Status::NEGATIVE));
    }
}
