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

// Clear flags
#[test]
fn clc_clears_carry_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x18]);
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0xFF & !Status::CARRY);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cld_clears_decimal_mode_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD8]);
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0xFF & !Status::DECIMAL_MODE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cli_clears_interrupt_disable_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x58]);
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0xFF & !Status::INTERRUPT_DISABLE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn clv_clears_overflow_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xB8]);
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0xFF & !Status::OVERFLOW);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

// Decrement
#[test]
fn dec_zero_page_decrements_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC6, 0x10]);
    cpu.registers.a = 0x11;
    cpu.status = Status(0xFF);
    bus.0[0x10] = 0x43;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x42, "$10 = {:#04X}", bus.0[0x10]);
    assert_eq!(cpu.registers.a, 0x11); // decoy: DEC must not touch A
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dec_wraps_around_and_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC6, 0x10]);
    cpu.status = Status(0x00);
    bus.0[0x10] = 0x00;

    cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0xFF, "$10 = {:#04X}", bus.0[0x10]);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dec_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC6, 0x10]);
    cpu.status = Status(0x00);
    bus.0[0x10] = 0x01;

    cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x00, "$10 = {:#04X}", bus.0[0x10]);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dec_zero_page_x_decrements_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD6, 0x10]);
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x43;
    bus.0[0x10] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0x42, "$14 = {:#04X}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn dec_zero_page_x_with_wraparound_decrements_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD6, 0xFF]);
    cpu.registers.x = 0x02;
    bus.0[0x01] = 0x43;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0x42, "$01 = {:#04X}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cycles, 6);
}

#[test]
fn dec_absolute_decrements_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xCE, 0x34, 0x12]);
    bus.0[0x1234] = 0x43;
    bus.0[0x3412] = 0xEE; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x42, "$1234 = {:#04X}", bus.0[0x1234]);
    assert_eq!(bus.0[0x3412], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn dec_absolute_x_decrements_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xDE, 0x00, 0x12]);
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0x43;
    bus.0[0x1200] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0x42, "$1204 = {:#04X}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn dec_absolute_x_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xDE, 0xFF, 0x12]);
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0x43;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x42, "$1300 = {:#04X}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn dex_decrements_x() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xCA]);
    cpu.registers.x = 0x11;
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x10, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dex_wraps_around() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xCA]);
    cpu.registers.x = 0x00;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0xFF, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dex_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xCA]);
    cpu.registers.x = 0x01;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x00, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dey_decrements_y() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x88]);
    cpu.registers.y = 0x11;
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x10, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dey_wraps_around() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x88]);
    cpu.registers.y = 0x00;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0xFF, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn dey_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x88]);
    cpu.registers.y = 0x01;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x00, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

// Increment
#[test]
fn inc_zero_page_increments_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE6, 0x10]);
    cpu.registers.a = 0x11;
    cpu.status = Status(0xFF);
    bus.0[0x10] = 0x41;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x42, "$10 = {:#04X}", bus.0[0x10]);
    assert_eq!(cpu.registers.a, 0x11); // decoy: INC must not touch A
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn inc_wraps_around_and_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE6, 0x10]);
    cpu.status = Status(0x00);
    bus.0[0x10] = 0xFF;

    cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x00, "$10 = {:#04X}", bus.0[0x10]);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn inc_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE6, 0x10]);
    cpu.status = Status(0x00);
    bus.0[0x10] = 0x7F;

    cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x80, "$10 = {:#04X}", bus.0[0x10]);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn inc_zero_page_x_increments_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xF6, 0x10]);
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x41;
    bus.0[0x10] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0x42, "$14 = {:#04X}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn inc_zero_page_x_with_wraparound_increments_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xF6, 0xFF]);
    cpu.registers.x = 0x02;
    bus.0[0x01] = 0x41;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0x42, "$01 = {:#04X}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cycles, 6);
}

#[test]
fn inc_absolute_increments_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xEE, 0x34, 0x12]);
    bus.0[0x1234] = 0x41;
    bus.0[0x3412] = 0xEE; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x42, "$1234 = {:#04X}", bus.0[0x1234]);
    assert_eq!(bus.0[0x3412], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn inc_absolute_x_increments_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xFE, 0x00, 0x12]);
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0x41;
    bus.0[0x1200] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0x42, "$1204 = {:#04X}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn inc_absolute_x_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xFE, 0xFF, 0x12]);
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0x41;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x42, "$1300 = {:#04X}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn inx_increments_x() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE8]);
    cpu.registers.x = 0x10;
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x11, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn inx_wraps_around() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE8]);
    cpu.registers.x = 0xFF;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x00, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn inx_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE8]);
    cpu.registers.x = 0x7F;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x80, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn iny_increments_y() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC8]);
    cpu.registers.y = 0x10;
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x11, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn iny_wraps_around() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC8]);
    cpu.registers.y = 0xFF;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x00, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn iny_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC8]);
    cpu.registers.y = 0x7F;
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x80, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
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

// Set flags
#[test]
fn sec_sets_carry_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x38]);
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0x00 | Status::CARRY);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn sed_sets_decimal_mode_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xF8]);
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0x00 | Status::DECIMAL_MODE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn sei_sets_interrupt_disable_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x78]);
    cpu.status = Status(0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0x00 | Status::INTERRUPT_DISABLE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
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
    assert_eq!(bus.0[0x10], 0x01, "$10 = {:#04X}", bus.0[0x10]);
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

// STX
#[test]
fn stx_zero_page_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x86, 0x10]);
    cpu.registers.x = 0x01;
    cpu.status.set(Status::ZERO, true);
    cpu.status.set(Status::NEGATIVE, true);
    let status = cpu.status.0;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x01, "$10 = {:#04X}", bus.0[0x10]);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn stx_zero_page_y_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x96, 0x10]);
    cpu.registers.x = 0x42;
    cpu.registers.y = 0x04;
    cpu.status.set(Status::ZERO, true);
    cpu.status.set(Status::NEGATIVE, true);
    let status = cpu.status.0;
    bus.0[0x10] = 0xEE; // decoy: base address without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0x42, "$14 = {:#04X}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn stx_zero_page_y_with_wraparound_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x96, 0xFF]);
    cpu.registers.x = 0x42;
    cpu.registers.y = 0x02;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0x42, "$01 = {:#04X}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn stx_absolute_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x8E, 0x34, 0x12]);
    cpu.registers.x = 0x42;
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

// STY
#[test]
fn sty_zero_page_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x84, 0x10]);
    cpu.registers.y = 0x01;
    cpu.status.set(Status::ZERO, true);
    cpu.status.set(Status::NEGATIVE, true);
    let status = cpu.status.0;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x01, "$10 = {:#04X}", bus.0[0x10]);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn sty_zero_page_x_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x94, 0x10]);
    cpu.registers.y = 0x42;
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
fn sty_zero_page_x_with_wraparound_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x94, 0xFF]);
    cpu.registers.y = 0x42;
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
fn sty_absolute_stores_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x8C, 0x34, 0x12]);
    cpu.registers.y = 0x42;
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

// Transfers
#[test]
fn tax_transfers_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xAA]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x42, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tax_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xAA]);
    cpu.registers.a = 0x00;
    cpu.registers.x = 0xEE;
    cpu.status.set(Status::NEGATIVE, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x00);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tax_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xAA]);
    cpu.registers.a = 0x80;
    cpu.status.set(Status::ZERO, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x80);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tay_transfers_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA8]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x11; // decoy: X must not change
    cpu.registers.y = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x42, "Y = {:#04X}", cpu.registers.y);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn tay_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA8]);
    cpu.registers.a = 0x00;
    cpu.registers.y = 0xEE;
    cpu.status.set(Status::NEGATIVE, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x00);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tay_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xA8]);
    cpu.registers.a = 0x80;
    cpu.status.set(Status::ZERO, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.y, 0x80);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn txa_transfers_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x8A]);
    cpu.registers.x = 0x42;
    cpu.registers.y = 0x11; // decoy: Y must not be the source
    cpu.registers.a = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x42, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.x, 0x42);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn txa_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x8A]);
    cpu.registers.x = 0x00;
    cpu.registers.a = 0xEE;
    cpu.status.set(Status::NEGATIVE, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn txa_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x8A]);
    cpu.registers.x = 0x80;
    cpu.status.set(Status::ZERO, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x80);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tya_transfers_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x98]);
    cpu.registers.y = 0x42;
    cpu.registers.x = 0x11; // decoy: X must not be the source
    cpu.registers.a = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x42, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.y, 0x42);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn tya_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x98]);
    cpu.registers.y = 0x00;
    cpu.registers.a = 0xEE;
    cpu.status.set(Status::NEGATIVE, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tya_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x98]);
    cpu.registers.y = 0x80;
    cpu.status.set(Status::ZERO, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x80);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tsx_transfers_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBA]);
    cpu.registers.sp = 0x42;
    cpu.registers.x = 0xEE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x42, "X = {:#04X}", cpu.registers.x);
    assert_eq!(cpu.registers.sp, 0x42);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn tsx_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBA]);
    cpu.registers.sp = 0x00;
    cpu.registers.x = 0xEE;
    cpu.status.set(Status::NEGATIVE, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0x00);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn tsx_sets_negative_flag() {
    // Power-on SP is $FD, which has bit 7 set.
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xBA]);
    cpu.status.set(Status::ZERO, true);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.x, 0xFD);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn txs_transfers_value() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x9A]);
    cpu.registers.x = 0x42;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.sp, 0x42, "SP = {:#04X}", cpu.registers.sp);
    assert_eq!(cpu.registers.x, 0x42);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn txs_does_not_set_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x9A]);
    cpu.registers.x = 0x00;
    cpu.status.set(Status::NEGATIVE, true);
    let status = cpu.status.0;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.sp, 0x00);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn txs_does_not_set_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x9A]);
    cpu.registers.x = 0x80;
    cpu.status.set(Status::ZERO, true);
    let status = cpu.status.0;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.sp, 0x80);
    assert_eq!(cpu.status.0, status);
}

// AND

#[test]
fn and_immediate_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x29, 0xAA]);
    cpu.registers.a = 0xCC;
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn and_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x29, 0xF0]);
    cpu.registers.a = 0x0F;
    cpu.status = Status(0x00);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00, "A = {:#04X}", cpu.registers.a);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn and_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x29, 0x80]);
    cpu.registers.a = 0xFF;
    cpu.status = Status(Status::ZERO);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x80, "A = {:#04X}", cpu.registers.a);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn and_zero_page_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x25, 0x10]);
    cpu.registers.a = 0xCC;
    bus.0[0x10] = 0xAA;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn and_zero_page_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x35, 0x10]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0xAA;
    bus.0[0x10] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn and_absolute_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x2D, 0x34, 0x12]);
    cpu.registers.a = 0xCC;
    bus.0[0x1234] = 0xAA;
    bus.0[0x3412] = 0xFF; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn and_absolute_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x3D, 0x00, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn and_absolute_x_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x3D, 0xFF, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn and_absolute_y_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x39, 0x00, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.y = 0x04;
    bus.0[0x1204] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: base address without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn and_absolute_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x39, 0xFF, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.y = 0x01;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn and_indirect_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x21, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 + X → $14 → pointer $1234
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x34;
    bus.0[0x15] = 0x12;
    bus.0[0x1234] = 0xAA;
    bus.0[0x10] = 0x00; // decoy pointer without X → $2000
    bus.0[0x11] = 0x20;
    bus.0[0x2000] = 0xFF;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn and_indirect_y_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x31, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 → pointer $1234 + Y → $1238
    cpu.registers.x = 0x02; // should be ignored
    cpu.registers.y = 0x04;
    bus.0[0x10] = 0x34;
    bus.0[0x11] = 0x12;
    bus.0[0x1238] = 0xAA;
    bus.0[0x1234] = 0xFF; // decoy: pointer without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn and_indirect_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x31, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 → pointer $12FF + Y → $1300
    cpu.registers.y = 0x01;
    bus.0[0x10] = 0xFF;
    bus.0[0x11] = 0x12;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x88, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

// ORA

#[test]
fn ora_immediate_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x09, 0xAA]);
    cpu.registers.a = 0xCC;
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn ora_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x09, 0x00]);
    cpu.registers.a = 0x00;
    cpu.status = Status(0x00);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00, "A = {:#04X}", cpu.registers.a);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ora_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x09, 0x80]);
    cpu.registers.a = 0x00;
    cpu.status = Status(Status::ZERO);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x80, "A = {:#04X}", cpu.registers.a);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn ora_zero_page_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x05, 0x10]);
    cpu.registers.a = 0xCC;
    bus.0[0x10] = 0xAA;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn ora_zero_page_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x15, 0x10]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0xAA;
    bus.0[0x10] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn ora_absolute_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x0D, 0x34, 0x12]);
    cpu.registers.a = 0xCC;
    bus.0[0x1234] = 0xAA;
    bus.0[0x3412] = 0xFF; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn ora_absolute_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x1D, 0x00, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn ora_absolute_x_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x1D, 0xFF, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn ora_absolute_y_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x19, 0x00, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.y = 0x04;
    bus.0[0x1204] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: base address without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn ora_absolute_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x19, 0xFF, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.y = 0x01;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn ora_indirect_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x01, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 + X → $14 → pointer $1234
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x34;
    bus.0[0x15] = 0x12;
    bus.0[0x1234] = 0xAA;
    bus.0[0x10] = 0x00; // decoy pointer without X → $2000
    bus.0[0x11] = 0x20;
    bus.0[0x2000] = 0xFF;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn ora_indirect_y_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x11, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 → pointer $1234 + Y → $1238
    cpu.registers.x = 0x02; // should be ignored
    cpu.registers.y = 0x04;
    bus.0[0x10] = 0x34;
    bus.0[0x11] = 0x12;
    bus.0[0x1238] = 0xAA;
    bus.0[0x1234] = 0xFF; // decoy: pointer without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn ora_indirect_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x11, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 → pointer $12FF + Y → $1300
    cpu.registers.y = 0x01;
    bus.0[0x10] = 0xFF;
    bus.0[0x11] = 0x12;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xEE, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

// EOR

#[test]
fn eor_immediate_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x49, 0xAA]);
    cpu.registers.a = 0xCC;
    cpu.status = Status(0xFF);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn eor_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x49, 0xAA]);
    cpu.registers.a = 0xAA;
    cpu.status = Status(0x00);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00, "A = {:#04X}", cpu.registers.a);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn eor_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x49, 0x80]);
    cpu.registers.a = 0x00;
    cpu.status = Status(Status::ZERO);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x80, "A = {:#04X}", cpu.registers.a);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
}

#[test]
fn eor_zero_page_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x45, 0x10]);
    cpu.registers.a = 0xCC;
    bus.0[0x10] = 0xAA;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn eor_zero_page_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x55, 0x10]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0xAA;
    bus.0[0x10] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn eor_absolute_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x4D, 0x34, 0x12]);
    cpu.registers.a = 0xCC;
    bus.0[0x1234] = 0xAA;
    bus.0[0x3412] = 0xFF; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn eor_absolute_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x5D, 0x00, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn eor_absolute_x_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x5D, 0xFF, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn eor_absolute_y_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x59, 0x00, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.y = 0x04;
    bus.0[0x1204] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: base address without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn eor_absolute_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x59, 0xFF, 0x12]);
    cpu.registers.a = 0xCC;
    cpu.registers.y = 0x01;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn eor_indirect_x_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x41, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 + X → $14 → pointer $1234
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x34;
    bus.0[0x15] = 0x12;
    bus.0[0x1234] = 0xAA;
    bus.0[0x10] = 0x00; // decoy pointer without X → $2000
    bus.0[0x11] = 0x20;
    bus.0[0x2000] = 0xFF;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn eor_indirect_y_combines_with_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x51, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 → pointer $1234 + Y → $1238
    cpu.registers.x = 0x02; // should be ignored
    cpu.registers.y = 0x04;
    bus.0[0x10] = 0x34;
    bus.0[0x11] = 0x12;
    bus.0[0x1238] = 0xAA;
    bus.0[0x1234] = 0xFF; // decoy: pointer without Y

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn eor_indirect_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x51, 0x10]);
    cpu.registers.a = 0xCC;
    // $10 → pointer $12FF + Y → $1300
    cpu.registers.y = 0x01;
    bus.0[0x10] = 0xFF;
    bus.0[0x11] = 0x12;
    bus.0[0x1300] = 0xAA;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x66, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

// BIT

#[test]
fn bit_zero_page_sets_zero_negative_and_overflow() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x24, 0x10]);
    cpu.registers.a = 0x0F;
    cpu.status = Status(Status::CARRY); // decoy: BIT must not touch C
    bus.0[0x10] = 0xC0;

    let cycles = cpu.step(&mut bus);
    assert_eq!(
        cpu.status.0,
        Status::CARRY | Status::ZERO | Status::NEGATIVE | Status::OVERFLOW
    );
    assert_eq!(cpu.registers.a, 0x0F); // result of A & M is discarded
    assert_eq!(bus.0[0x10], 0xC0);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn bit_zero_page_clears_zero_negative_and_overflow() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x24, 0x10]);
    cpu.registers.a = 0xFF;
    cpu.status = Status(0xFF);
    bus.0[0x10] = 0x3F;

    cpu.step(&mut bus);
    assert_eq!(
        cpu.status.0,
        0xFF & !(Status::ZERO | Status::NEGATIVE | Status::OVERFLOW)
    );
    assert_eq!(cpu.registers.a, 0xFF);
}

#[test]
fn bit_takes_negative_and_overflow_from_memory_not_result() {
    // A & M = $00, but M has bits 7 and 6 set.
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x24, 0x10]);
    cpu.registers.a = 0x3F;
    cpu.status = Status(0x00);
    bus.0[0x10] = 0xC0;

    cpu.step(&mut bus);
    assert!(cpu.status.is_set(Status::ZERO));
    assert!(cpu.status.is_set(Status::NEGATIVE));
    assert!(cpu.status.is_set(Status::OVERFLOW));
}

#[test]
fn bit_absolute_tests_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x2C, 0x34, 0x12]);
    cpu.registers.a = 0x40;
    cpu.status = Status(0x00);
    bus.0[0x1234] = 0x40;
    bus.0[0x3412] = 0x80; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert!(!cpu.status.is_set(Status::ZERO));
    assert!(!cpu.status.is_set(Status::NEGATIVE));
    assert!(cpu.status.is_set(Status::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x40);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

// Compares

// C, Z and N are the only flags a compare may change.
const CZN: u8 = Status::CARRY | Status::ZERO | Status::NEGATIVE;

// CMP

#[test]
fn cmp_greater_sets_carry() {
    // $42 - $10 = $32
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC9, 0x10]);
    cpu.registers.a = 0x42;
    let expected = Status::CARRY;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.a, 0x42); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cmp_equal_sets_carry_and_zero() {
    // $42 - $42 = $00
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC9, 0x42]);
    cpu.registers.a = 0x42;
    let expected = Status::CARRY | Status::ZERO;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.a, 0x42); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cmp_less_clears_carry_and_sets_negative() {
    // $10 - $42 = $CE
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC9, 0x42]);
    cpu.registers.a = 0x10;
    let expected = Status::NEGATIVE;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.a, 0x10); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cmp_negative_comes_from_result_not_comparison() {
    // $01 - $FF = $02: less, but bit 7 clear
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC9, 0xFF]);
    cpu.registers.a = 0x01;
    let expected = 0;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.a, 0x01); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cmp_greater_with_result_bit_7_set() {
    // $FF - $01 = $FE: greater, but bit 7 set
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC9, 0x01]);
    cpu.registers.a = 0xFF;
    let expected = Status::CARRY | Status::NEGATIVE;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.a, 0xFF); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

// CPX

#[test]
fn cpx_greater_sets_carry() {
    // $42 - $10 = $32
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE0, 0x10]);
    cpu.registers.x = 0x42;
    let expected = Status::CARRY;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.x, 0x42); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpx_equal_sets_carry_and_zero() {
    // $42 - $42 = $00
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE0, 0x42]);
    cpu.registers.x = 0x42;
    let expected = Status::CARRY | Status::ZERO;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.x, 0x42); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpx_less_clears_carry_and_sets_negative() {
    // $10 - $42 = $CE
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE0, 0x42]);
    cpu.registers.x = 0x10;
    let expected = Status::NEGATIVE;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.x, 0x10); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpx_negative_comes_from_result_not_comparison() {
    // $01 - $FF = $02: less, but bit 7 clear
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE0, 0xFF]);
    cpu.registers.x = 0x01;
    let expected = 0;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.x, 0x01); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpx_greater_with_result_bit_7_set() {
    // $FF - $01 = $FE: greater, but bit 7 set
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE0, 0x01]);
    cpu.registers.x = 0xFF;
    let expected = Status::CARRY | Status::NEGATIVE;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.x, 0xFF); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

// CPY

#[test]
fn cpy_greater_sets_carry() {
    // $42 - $10 = $32
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC0, 0x10]);
    cpu.registers.y = 0x42;
    let expected = Status::CARRY;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.y, 0x42); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpy_equal_sets_carry_and_zero() {
    // $42 - $42 = $00
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC0, 0x42]);
    cpu.registers.y = 0x42;
    let expected = Status::CARRY | Status::ZERO;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.y, 0x42); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpy_less_clears_carry_and_sets_negative() {
    // $10 - $42 = $CE
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC0, 0x42]);
    cpu.registers.y = 0x10;
    let expected = Status::NEGATIVE;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.y, 0x10); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpy_negative_comes_from_result_not_comparison() {
    // $01 - $FF = $02: less, but bit 7 clear
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC0, 0xFF]);
    cpu.registers.y = 0x01;
    let expected = 0;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.y, 0x01); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn cpy_greater_with_result_bit_7_set() {
    // $FF - $01 = $FE: greater, but bit 7 set
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC0, 0x01]);
    cpu.registers.y = 0xFF;
    let expected = Status::CARRY | Status::NEGATIVE;
    // C/Z/N start opposite to the expected result; other flags are set as decoys.
    cpu.status = Status((!expected & CZN) | !CZN);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, expected | !CZN, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.y, 0xFF); // compare must not store the result
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

// CMP addressing modes

#[test]
fn cmp_zero_page_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC5, 0x10]);
    cpu.registers.a = 0x42;
    bus.0[0x10] = 0x42;

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x10], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn cmp_zero_page_x_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD5, 0x10]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x42;
    bus.0[0x10] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x14], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn cmp_absolute_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xCD, 0x34, 0x12]);
    cpu.registers.a = 0x42;
    bus.0[0x1234] = 0x42;
    bus.0[0x3412] = 0xFF; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1234], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn cmp_absolute_x_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xDD, 0x00, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0x42;
    bus.0[0x1200] = 0xFF; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1204], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn cmp_absolute_x_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xDD, 0xFF, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0x42;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1300], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn cmp_absolute_y_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD9, 0x00, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.y = 0x04;
    bus.0[0x1204] = 0x42;
    bus.0[0x1200] = 0xFF; // decoy: base address without Y

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1204], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn cmp_absolute_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD9, 0xFF, 0x12]);
    cpu.registers.a = 0x42;
    cpu.registers.y = 0x01;
    bus.0[0x1300] = 0x42;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1300], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn cmp_indirect_x_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC1, 0x10]);
    cpu.registers.a = 0x42;
    // $10 + X → $14 → pointer $1234
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x34;
    bus.0[0x15] = 0x12;
    bus.0[0x1234] = 0x42;
    bus.0[0x10] = 0x00; // decoy pointer without X → $2000
    bus.0[0x11] = 0x20;
    bus.0[0x2000] = 0xFF;

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1234], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn cmp_indirect_y_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD1, 0x10]);
    cpu.registers.a = 0x42;
    // $10 → pointer $1234 + Y → $1238
    cpu.registers.x = 0x02; // should be ignored
    cpu.registers.y = 0x04;
    bus.0[0x10] = 0x34;
    bus.0[0x11] = 0x12;
    bus.0[0x1238] = 0x42;
    bus.0[0x1234] = 0xFF; // decoy: pointer without Y

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1238], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn cmp_indirect_y_with_page_crossed_adds_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xD1, 0x10]);
    cpu.registers.a = 0x42;
    // $10 → pointer $12FF + Y → $1300
    cpu.registers.y = 0x01;
    bus.0[0x10] = 0xFF;
    bus.0[0x11] = 0x12;
    bus.0[0x1300] = 0x42;
    bus.0[0x1200] = 0xFF; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(bus.0[0x1300], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

// CPX addressing modes

#[test]
fn cpx_zero_page_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xE4, 0x10]);
    cpu.registers.x = 0x42;
    bus.0[0x10] = 0x42;

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.x, 0x42);
    assert_eq!(bus.0[0x10], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn cpx_absolute_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xEC, 0x34, 0x12]);
    cpu.registers.x = 0x42;
    bus.0[0x1234] = 0x42;
    bus.0[0x3412] = 0xFF; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.x, 0x42);
    assert_eq!(bus.0[0x1234], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

// CPY addressing modes

#[test]
fn cpy_zero_page_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xC4, 0x10]);
    cpu.registers.y = 0x42;
    bus.0[0x10] = 0x42;

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.y, 0x42);
    assert_eq!(bus.0[0x10], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn cpy_absolute_compares_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0xCC, 0x34, 0x12]);
    cpu.registers.y = 0x42;
    bus.0[0x1234] = 0x42;
    bus.0[0x3412] = 0xFF; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    // Equal values set C and Z; a decoy read of $FF would clear both.
    assert!(cpu.status.is_set(Status::CARRY));
    assert!(cpu.status.is_set(Status::ZERO));
    assert_eq!(cpu.registers.y, 0x42);
    assert_eq!(bus.0[0x1234], 0x42); // compare must not write memory
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

// ASL

#[test]
fn asl_accumulator_shifts_bit_7_into_carry() {
    // 1000_0001 << 1 → 0000_0010, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x0A]);
    cpu.registers.a = 0x81;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x02, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, Status::CARRY, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn asl_accumulator_clears_carry() {
    // 0100_0001 << 1 → 1000_0010, C = 0; old C is ignored
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x0A]);
    cpu.registers.a = 0x41;
    cpu.status = Status(Status::CARRY);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x82, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, Status::NEGATIVE, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn asl_accumulator_sets_zero_flag() {
    // 1000_0000 << 1 → 0000_0000, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x0A]);
    cpu.registers.a = 0x80;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00, "A = {:#010b}", cpu.registers.a);
    assert_eq!(
        cpu.status.0,
        Status::CARRY | Status::ZERO,
        "P = {:#010b}",
        cpu.status.0
    );
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn asl_zero_page_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x06, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    bus.0[0x10] = 0x41;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x82, "0x10 = {:#010b}", bus.0[0x10]);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn asl_zero_page_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x16, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x41;
    bus.0[0x10] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0x82, "0x14 = {:#010b}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn asl_zero_page_x_with_wraparound_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x16, 0xFF]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x02;
    bus.0[0x01] = 0x41;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0x82, "0x01 = {:#010b}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn asl_absolute_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x0E, 0x34, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    bus.0[0x1234] = 0x41;
    bus.0[0x3412] = 0xEE; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x82, "0x1234 = {:#010b}", bus.0[0x1234]);
    assert_eq!(bus.0[0x3412], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn asl_absolute_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x1E, 0x00, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0x41;
    bus.0[0x1200] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0x82, "0x1204 = {:#010b}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn asl_absolute_x_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x1E, 0xFF, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0x41;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x82, "0x1300 = {:#010b}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

// LSR

#[test]
fn lsr_accumulator_shifts_bit_0_into_carry() {
    // 1000_0001 >> 1 → 0100_0000, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x4A]);
    cpu.registers.a = 0x81;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x40, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, Status::CARRY, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn lsr_accumulator_clears_carry() {
    // 1000_0010 >> 1 → 0100_0001, C = 0; old C is ignored
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x4A]);
    cpu.registers.a = 0x82;
    cpu.status = Status(Status::CARRY);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x41, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, 0, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn lsr_accumulator_sets_zero_flag() {
    // 0000_0001 >> 1 → 0000_0000, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x4A]);
    cpu.registers.a = 0x01;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00, "A = {:#010b}", cpu.registers.a);
    assert_eq!(
        cpu.status.0,
        Status::CARRY | Status::ZERO,
        "P = {:#010b}",
        cpu.status.0
    );
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn lsr_zero_page_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x46, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    bus.0[0x10] = 0x82;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x41, "0x10 = {:#010b}", bus.0[0x10]);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn lsr_zero_page_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x56, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x82;
    bus.0[0x10] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0x41, "0x14 = {:#010b}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn lsr_zero_page_x_with_wraparound_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x56, 0xFF]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x02;
    bus.0[0x01] = 0x82;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0x41, "0x01 = {:#010b}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn lsr_absolute_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x4E, 0x34, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    bus.0[0x1234] = 0x82;
    bus.0[0x3412] = 0xEE; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x41, "0x1234 = {:#010b}", bus.0[0x1234]);
    assert_eq!(bus.0[0x3412], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn lsr_absolute_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x5E, 0x00, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0x82;
    bus.0[0x1200] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0x41, "0x1204 = {:#010b}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn lsr_absolute_x_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x5E, 0xFF, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(0);
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0x82;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x41, "0x1300 = {:#010b}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

// ROL

#[test]
fn rol_accumulator_shifts_bit_7_into_carry() {
    // 1000_0001 rol, C=0 → 0000_0010, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x2A]);
    cpu.registers.a = 0x81;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x02, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, Status::CARRY, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn rol_accumulator_rotates_carry_into_bit_0() {
    // 0100_0001 rol, C=1 → 1000_0011, C = 0
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x2A]);
    cpu.registers.a = 0x41;
    cpu.status = Status(Status::CARRY);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x83, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, Status::NEGATIVE, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn rol_accumulator_sets_zero_flag() {
    // 1000_0000 rol, C=0 → 0000_0000, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x2A]);
    cpu.registers.a = 0x80;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00, "A = {:#010b}", cpu.registers.a);
    assert_eq!(
        cpu.status.0,
        Status::CARRY | Status::ZERO,
        "P = {:#010b}",
        cpu.status.0
    );
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn rol_zero_page_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x26, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    bus.0[0x10] = 0x41;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0x83, "0x10 = {:#010b}", bus.0[0x10]);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn rol_zero_page_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x36, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x41;
    bus.0[0x10] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0x83, "0x14 = {:#010b}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn rol_zero_page_x_with_wraparound_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x36, 0xFF]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x02;
    bus.0[0x01] = 0x41;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0x83, "0x01 = {:#010b}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn rol_absolute_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x2E, 0x34, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    bus.0[0x1234] = 0x41;
    bus.0[0x3412] = 0xEE; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0x83, "0x1234 = {:#010b}", bus.0[0x1234]);
    assert_eq!(bus.0[0x3412], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn rol_absolute_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x3E, 0x00, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0x41;
    bus.0[0x1200] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0x83, "0x1204 = {:#010b}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn rol_absolute_x_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x3E, 0xFF, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0x41;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0x83, "0x1300 = {:#010b}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

// ROR

#[test]
fn ror_accumulator_shifts_bit_0_into_carry() {
    // 1000_0001 ror, C=0 → 0100_0000, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x6A]);
    cpu.registers.a = 0x81;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x40, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, Status::CARRY, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn ror_accumulator_rotates_carry_into_bit_7() {
    // 1000_0010 ror, C=1 → 1100_0001, C = 0
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x6A]);
    cpu.registers.a = 0x82;
    cpu.status = Status(Status::CARRY);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0xC1, "A = {:#010b}", cpu.registers.a);
    assert_eq!(cpu.status.0, Status::NEGATIVE, "P = {:#010b}", cpu.status.0);
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn ror_accumulator_sets_zero_flag() {
    // 0000_0001 ror, C=0 → 0000_0000, C = 1
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x6A]);
    cpu.registers.a = 0x01;
    cpu.status = Status(0);
    bus.0[0x00] = 0xEE; // decoy: accumulator mode must not touch memory

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00, "A = {:#010b}", cpu.registers.a);
    assert_eq!(
        cpu.status.0,
        Status::CARRY | Status::ZERO,
        "P = {:#010b}",
        cpu.status.0
    );
    assert_eq!(bus.0[0x00], 0xEE);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 2);
    assert_eq!(cpu.cycle_count, 2);
}

#[test]
fn ror_zero_page_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x66, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    bus.0[0x10] = 0x82;

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x10], 0xC1, "0x10 = {:#010b}", bus.0[0x10]);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn ror_zero_page_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x76, 0x10]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x04;
    bus.0[0x14] = 0x82;
    bus.0[0x10] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x14], 0xC1, "0x14 = {:#010b}", bus.0[0x14]);
    assert_eq!(bus.0[0x10], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn ror_zero_page_x_with_wraparound_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x76, 0xFF]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x02;
    bus.0[0x01] = 0x82;
    bus.0[0x0101] = 0xEE; // decoy: address if added as u16

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01], 0xC1, "0x01 = {:#010b}", bus.0[0x01]);
    assert_eq!(bus.0[0x0101], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 2);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn ror_absolute_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x6E, 0x34, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    bus.0[0x1234] = 0x82;
    bus.0[0x3412] = 0xEE; // decoy: byte-swapped address

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1234], 0xC1, "0x1234 = {:#010b}", bus.0[0x1234]);
    assert_eq!(bus.0[0x3412], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn ror_absolute_x_modifies_memory() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x7E, 0x00, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x04;
    bus.0[0x1204] = 0x82;
    bus.0[0x1200] = 0xEE; // decoy: base address without X

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1204], 0xC1, "0x1204 = {:#010b}", bus.0[0x1204]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

#[test]
fn ror_absolute_x_with_page_crossed_has_no_extra_cycle() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x7E, 0xFF, 0x12]);
    cpu.registers.a = 0x11; // decoy: memory mode must not touch A
    cpu.status = Status(Status::CARRY);
    cpu.registers.x = 0x01;
    bus.0[0x1300] = 0x82;
    bus.0[0x1200] = 0xEE; // decoy: address if high byte doesn't carry

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x1300], 0xC1, "0x1300 = {:#010b}", bus.0[0x1300]);
    assert_eq!(bus.0[0x1200], 0xEE);
    assert_eq!(cpu.registers.a, 0x11);
    assert!(!cpu.status.is_set(Status::CARRY));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 3);
    assert_eq!(cycles, 7);
    assert_eq!(cpu.cycle_count, 7);
}

// Stack

#[test]
fn pha_pushes_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x48]);
    cpu.registers.a = 0x42;
    cpu.status = Status(0xC3);
    bus.0[0x01FC] = 0xEE; // decoy: address if SP is decremented before writing

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01FD], 0x42, "$01FD = {:#04X}", bus.0[0x01FD]);
    assert_eq!(bus.0[0x01FC], 0xEE);
    assert_eq!(cpu.registers.sp, 0xFC);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.status.0, 0xC3);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn pha_wraps_stack_pointer() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x48]);
    cpu.registers.a = 0x42;
    cpu.registers.sp = 0x00;

    cpu.step(&mut bus);
    assert_eq!(bus.0[0x0100], 0x42);
    assert_eq!(cpu.registers.sp, 0xFF);
}

#[test]
fn pla_pulls_into_a() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x68]);
    cpu.registers.a = 0xEE;
    cpu.registers.sp = 0xFC;
    cpu.status = Status(0xFF);
    bus.0[0x01FD] = 0x42;
    bus.0[0x01FC] = 0x11; // decoy: address if SP is incremented after reading

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x42, "A = {:#04X}", cpu.registers.a);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert_eq!(cpu.status.0, 0xFF & !(Status::ZERO | Status::NEGATIVE));
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn pla_sets_zero_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x68]);
    cpu.registers.a = 0xEE;
    cpu.registers.sp = 0xFC;
    cpu.status = Status(0x00);
    bus.0[0x01FD] = 0x00;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x00);
    assert_eq!(cpu.status.0, Status::ZERO);
}

#[test]
fn pla_sets_negative_flag() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x68]);
    cpu.registers.sp = 0xFC;
    cpu.status = Status(0x00);
    bus.0[0x01FD] = 0x80;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x80);
    assert_eq!(cpu.status.0, Status::NEGATIVE);
}

#[test]
fn pla_wraps_stack_pointer() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x68]);
    cpu.registers.sp = 0xFF;
    bus.0[0x0100] = 0x42;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.sp, 0x00);
}

#[test]
fn pha_then_pla_round_trips() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x48, 0xA9, 0x00, 0x68]);
    cpu.registers.a = 0x42;

    cpu.step(&mut bus); // PHA
    cpu.step(&mut bus); // LDA #$00
    cpu.step(&mut bus); // PLA
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn php_pushes_status_with_break_and_unused_set() {
    // $C3 = N V - - - - Z C; pushed copy gains B (bit 4) and bit 5 → $F3
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x08]);
    cpu.status = Status(0xC3);

    let cycles = cpu.step(&mut bus);
    assert_eq!(bus.0[0x01FD], 0xF3, "$01FD = {:#010b}", bus.0[0x01FD]);
    assert_eq!(cpu.registers.sp, 0xFC);
    assert_eq!(cpu.status.0, 0xC3); // live P is unchanged
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn plp_pulls_status() {
    // $C3 has neither bit 4 nor 5; P keeps bit 5 set → $E3
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x28]);
    cpu.registers.sp = 0xFC;
    cpu.status = Status(Status::UNUSED);
    bus.0[0x01FD] = 0xC3;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0xE3, "P = {:#010b}", cpu.status.0);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert_eq!(cpu.registers.pc, STARTING_ADDRESS + 1);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.cycle_count, 4);
}

#[test]
fn plp_ignores_break_bit_when_set() {
    // $FF would set B; P ends with B clear and bit 5 set → $EF
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x28]);
    cpu.registers.sp = 0xFC;
    cpu.status = Status(Status::UNUSED);
    bus.0[0x01FD] = 0xFF;

    cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0xEF, "P = {:#010b}", cpu.status.0);
}

#[test]
fn plp_keeps_unused_bit_when_clear() {
    // $00 would clear bit 5; P keeps it set → $20
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x28]);
    cpu.registers.sp = 0xFC;
    cpu.status = Status(0xFF);
    bus.0[0x01FD] = 0x00;

    cpu.step(&mut bus);
    assert_eq!(cpu.status.0, 0x20, "P = {:#010b}", cpu.status.0);
}

#[test]
fn php_then_plp_round_trips() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x08, 0x28]);
    cpu.status = Status(0xE3);

    cpu.step(&mut bus); // PHP
    cpu.status = Status(Status::UNUSED);
    cpu.step(&mut bus); // PLP
    assert_eq!(cpu.status.0, 0xE3);
    assert_eq!(cpu.registers.sp, 0xFD);
}

// JMP

#[test]
fn jmp_absolute_sets_pc() {
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x4C, 0x34, 0x12]);
    cpu.status = Status(0xC3);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1234, "PC = {:#06X}", cpu.registers.pc);
    assert_eq!(cpu.registers.sp, 0xFD); // JMP doesn't touch the stack
    assert_eq!(cpu.status.0, 0xC3);
    assert_eq!(cycles, 3);
    assert_eq!(cpu.cycle_count, 3);
}

#[test]
fn jmp_absolute_does_not_read_target() {
    // Absolute JMP uses the operand as the target; it must not dereference it.
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x4C, 0x34, 0x12]);
    bus.0[0x1234] = 0x78; // decoy: pointer bytes if JMP were indirect
    bus.0[0x1235] = 0x56;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1234, "PC = {:#06X}", cpu.registers.pc);
}

#[test]
fn jmp_indirect_sets_pc_from_pointer() {
    // $0120 → pointer bytes 34 12 → $1234
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x6C, 0x20, 0x01]);
    bus.0[0x0120] = 0x34;
    bus.0[0x0121] = 0x12;
    bus.0[0x0020] = 0x78; // decoy: pointer if only the low operand byte is used
    bus.0[0x0021] = 0x56;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1234, "PC = {:#06X}", cpu.registers.pc);
    assert_eq!(cycles, 5);
    assert_eq!(cpu.cycle_count, 5);
}

#[test]
fn jmp_indirect_page_wrap_bug() {
    // Pointer at $02FF: low byte from $02FF, high byte from $0200 (not $0300).
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x6C, 0xFF, 0x02]);
    bus.0[0x02FF] = 0x34;
    bus.0[0x0200] = 0x12;
    bus.0[0x0300] = 0x56; // decoy: high byte if the page carries

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1234, "PC = {:#06X}", cpu.registers.pc);
}

// JSR / RTS

#[test]
fn jsr_pushes_return_address_minus_one() {
    // JSR at $8000 is 3 bytes; the pushed address is $8002 (last byte of JSR).
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x20, 0x34, 0x12]);
    cpu.status = Status(0xC3);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1234, "PC = {:#06X}", cpu.registers.pc);
    assert_eq!(bus.0[0x01FD], 0x80, "high byte pushed first");
    assert_eq!(bus.0[0x01FC], 0x02, "low byte pushed second");
    assert_eq!(cpu.registers.sp, 0xFB);
    assert_eq!(cpu.status.0, 0xC3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn rts_pulls_return_address_plus_one() {
    // Stack holds $8002 (low at $01FC, high at $01FD); RTS resumes at $8003.
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x60]);
    cpu.registers.sp = 0xFB;
    cpu.status = Status(0xC3);
    bus.0[0x01FC] = 0x02;
    bus.0[0x01FD] = 0x80;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x8003, "PC = {:#06X}", cpu.registers.pc);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert_eq!(cpu.status.0, 0xC3);
    assert_eq!(cycles, 6);
    assert_eq!(cpu.cycle_count, 6);
}

#[test]
fn rts_pulls_low_byte_first() {
    // Distinct bytes catch swapped pull order: $1234 + 1, not $3412 + 1.
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x60]);
    cpu.registers.sp = 0xFB;
    bus.0[0x01FC] = 0x34;
    bus.0[0x01FD] = 0x12;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x1235, "PC = {:#06X}", cpu.registers.pc);
}

#[test]
fn rts_wraps_pc() {
    // Pulled $FFFF + 1 wraps to $0000.
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&[0x60]);
    cpu.registers.sp = 0xFB;
    bus.0[0x01FC] = 0xFF;
    bus.0[0x01FD] = 0xFF;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0000, "PC = {:#06X}", cpu.registers.pc);
}

#[test]
fn jsr_then_rts_returns_to_next_instruction() {
    // $8000: JSR $8010 / $8003: LDX #$42 / $8010: LDA #$11, RTS
    let mut program = [0xEA; 0x13];
    program[0x00..0x03].copy_from_slice(&[0x20, 0x10, 0x80]);
    program[0x03..0x05].copy_from_slice(&[0xA2, 0x42]);
    program[0x10..0x13].copy_from_slice(&[0xA9, 0x11, 0x60]);
    let (mut cpu, mut bus) = create_test_cpu_and_bus(&program);

    cpu.step(&mut bus); // JSR
    assert_eq!(cpu.registers.pc, 0x8010);
    cpu.step(&mut bus); // LDA #$11
    cpu.step(&mut bus); // RTS
    assert_eq!(cpu.registers.pc, 0x8003, "PC = {:#06X}", cpu.registers.pc);
    cpu.step(&mut bus); // LDX #$42
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x42);
    assert_eq!(cpu.registers.sp, 0xFD);
}
