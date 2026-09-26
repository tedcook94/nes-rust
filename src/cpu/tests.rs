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
    bus.0[0x10] = 0x11; // decoy: base address without X
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
    bus.0[0xFF] = 0x11; // decoy: base address without X
    bus.0[0x100] = 0x10; // decoy: address if added as u16
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
    bus.0[0x1001] = 0xFF; // decoy: byte-swapped address
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
    bus.0[0x0110] = 0xFF; // decoy: base address without X
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
    bus.0[0x0100] = 0xFF; // decoy: address if high byte doesn't carry
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
    bus.0[0x0110] = 0xFF; // decoy: base address without Y
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
    bus.0[0x0100] = 0xFF; // decoy: address if high byte doesn't carry
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
    // $10 + X → $11 → pointer $1234
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA1, 0x10]);
    cpu.registers.x = 0x01;
    bus.0[0x10] = 0x01;
    bus.0[0x11] = 0x34;
    bus.0[0x12] = 0x12;
    bus.0[0x1234] = 0x11;
    bus.0[0x3401] = 0xFF; // decoy: address from pointer without X
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
    // pointer low at $FF, high at $00 → $1234
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA1, 0xFF]);
    cpu.registers.x = 0x00;
    bus.0[0xFF] = 0x34;
    bus.0[0x00] = 0x12;
    bus.0[0x1234] = 0x11;
    bus.0[0x0100] = 0xFF; // wrong high byte if pointer doesn't wrap
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
    // $10 → pointer $1234 + Y → $1235
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB1, 0x10]);
    cpu.registers.y = 0x01;
    bus.0[0x10] = 0x34;
    bus.0[0x11] = 0x12;
    bus.0[0x1235] = 0x11;
    bus.0[0x1234] = 0x01; // decoy: pointer without Y
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
    // $10 → pointer $00FF + Y → $0100
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
    // pointer low at $FF, high at $00 → $1234 + Y → $1235
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
    bus.0[0x10] = 0x11; // decoy: base address without Y
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
    bus.0[0xFF] = 0x11; // decoy: base address without Y
    bus.0[0x100] = 0x10; // decoy: address if added as u16
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
    bus.0[0x1001] = 0xFF; // decoy: byte-swapped address
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
    bus.0[0x0110] = 0xFF; // decoy: base address without Y
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
    bus.0[0x0100] = 0xFF; // decoy: address if high byte doesn't carry
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

// LDY
#[test]
fn ldy_immediate_loads_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA0, 0x42]);
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x42, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA0, 0x00]);
    cpu.registers.y = 0xFF;
    cpu.status.set(Status::NEGATIVE, true);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x00, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_immediate_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA0, 0x80]);
    cpu.registers.y = 0xFF;
    cpu.status.set(Status::ZERO, true);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x80, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_zero_page_loads_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA4, 0x10]);
    bus.0[0x10] = 0x01;
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x01, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_zero_page_x_loads_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB4, 0x10]);
    cpu.registers.x = 0x01;
    bus.0[0x11] = 0x01;
    bus.0[0x10] = 0x11; // decoy: base address without X
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x01, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_zero_page_x_with_wraparound_loads_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB4, 0xFF]);
    cpu.registers.x = 0x01;
    bus.0[0x00] = 0x01;
    bus.0[0xFF] = 0x11; // decoy: base address without X
    bus.0[0x100] = 0x10; // decoy: address if added as u16
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x01, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_absolute_loads_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xAC, 0x10, 0x01]);
    bus.0[0x0110] = 0x11;
    bus.0[0x1001] = 0xFF; // decoy: byte-swapped address
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x11, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_absolute_x_loads_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBC, 0x10, 0x01]);
    cpu.registers.x = 0x01;
    bus.0[0x0111] = 0x11;
    bus.0[0x0110] = 0xFF; // decoy: base address without X
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x11, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_absolute_x_with_page_crossed_loads_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBC, 0xFF, 0x01]);
    cpu.registers.x = 0x01;
    bus.0[0x0200] = 0x11;
    bus.0[0x0100] = 0xFF; // decoy: address if high byte doesn't carry
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x11, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ldy_absolute_x_wraps_around_address_space() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBC, 0xFF, 0xFF]);
    cpu.registers.x = 0x01;
    bus.0[0x0000] = 0x11;
    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x11, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

// STA
#[test]
fn sta_zero_page_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x85, 0x10]);
    cpu.registers.a = 0x01;
    cpu.status.set(Status::ZERO, true);
    cpu.status.set(Status::NEGATIVE, true);
    let status = cpu.status.0;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x01, "Address 0x10 = {:#04X}", bus.0[0x10]);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn sta_zero_page_x_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x95, 0x10]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x04;
    cpu.status.set(Status::ZERO, true);
    cpu.status.set(Status::NEGATIVE, true);
    let status = cpu.status.0;
    bus.0[0x10] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0x42, "$14 = {:#04X}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn sta_zero_page_x_with_wraparound_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x95, 0xFF]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x02;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0x42, "$01 = {:#04X}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn sta_absolute_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x8D, 0x34, 0x12]);
    cpu.registers.a = 0x42;
    cpu.status.set(Status::ZERO, true);
    cpu.status.set(Status::NEGATIVE, true);
    let status = cpu.status.0;
    bus.0[0x3412] = 0xEE; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x42, "$1234 = {:#04X}", bus.0[0x1234]);
    assert_eq!(bus.0[0x3412], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn sta_absolute_x_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x9D, 0x00, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x04;
    bus.0[0x1200] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0x42, "$1204 = {:#04X}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn sta_absolute_x_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x9D, 0xFF, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x01;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x42, "$1300 = {:#04X}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn sta_absolute_y_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x99, 0x00, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.y = 0x04;
    bus.0[0x1200] = 0xEE; // decoy: base address without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0x42, "$1204 = {:#04X}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn sta_absolute_y_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x99, 0xFF, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.y = 0x01;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x42, "$1300 = {:#04X}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn sta_indirect_x_stores_value() {
    // $10 + X → $14 → pointer $1234
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x81, 0x10]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x34;
    bus.0[0x15] = 0x12;
    bus.0[0x10] = 0x00; // decoy pointer without X → $2000
    bus.0[0x11] = 0x20;
    bus.0[0x2000] = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x42, "$1234 = {:#04X}", bus.0[0x1234]);
    assert_eq!(bus.0[0x2000], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn sta_indirect_x_pointer_wraps() {
    // pointer low at $FF, high at $00 → $1234
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x81, 0xFF]);
    cpu.registers.a = 0x42;
    bus.0[0xFF] = 0x34;
    bus.0[0x00] = 0x12;
    bus.0[0x0100] = 0x20; // wrong high byte if pointer doesn't wrap → $2034
    bus.0[0x2034] = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x42, "$1234 = {:#04X}", bus.0[0x1234]);
    assert_eq!(bus.0[0x2034], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn sta_indirect_y_stores_value() {
    // $10 → pointer $1234 + Y → $1238
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x91, 0x10]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x02; // should be ignored
    cpu.registers.y = 0x04;
    bus.0[0x10] = 0x34;
    bus.0[0x11] = 0x12;
    bus.0[0x1234] = 0xEE; // decoy: pointer without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1238], 0x42, "$1238 = {:#04X}", bus.0[0x1238]);
    assert_eq!(bus.0[0x1234], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn sta_indirect_y_with_page_crossed_has_no_extra_cycle() {
    // $10 → pointer $12FF + Y → $1300
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x91, 0x10]);
    cpu.registers.a = 0x42;
    cpu.registers.y = 0x01;
    bus.0[0x10] = 0xFF;
    bus.0[0x11] = 0x12;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x42, "$1300 = {:#04X}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn sta_indirect_y_pointer_wraps() {
    // pointer low at $FF, high at $00 → $1234
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x91, 0xFF]);
    cpu.registers.a = 0x42;
    bus.0[0xFF] = 0x34;
    bus.0[0x00] = 0x12;
    bus.0[0x0100] = 0x20; // wrong high byte if pointer doesn't wrap → $2034
    bus.0[0x2034] = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x42, "$1234 = {:#04X}", bus.0[0x1234]);
    assert_eq!(bus.0[0x2034], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}
