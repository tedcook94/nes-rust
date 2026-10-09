use super::*;
use crate::bus::Bus;
use crate::cartridge::test_rom::{build_nrom_with_program, build_rom};
use crate::cpu::CpuState;

#[test]
fn new_rejects_invalid_rom() {
    assert_eq!(Console::new(&[0x00; 16]).err(), Some(RomError::BadMagic));
}

#[test]
fn new_rejects_unsupported_mapper() {
    // Mapper 4 (MMC3): flags 6 high nibble = 4.
    assert_eq!(
        Console::new(&build_rom(1, 1, 0x40, 0)).err(),
        Some(RomError::UnsupportedMapper(4))
    );
}

#[test]
fn new_resets_cpu_to_cartridge_reset_vector() {
    let console = Console::new(&build_nrom_with_program(&[])).unwrap();

    assert_eq!(console.cpu.state().cycles, 7);
    assert_eq!(
        console.cpu.state().pc,
        0xC000,
        "PC = {:#06X}",
        console.cpu.state().pc
    );
}

#[test]
fn step_runs_code_from_cartridge() {
    // $C000: LDA #$42
    let mut console = Console::new(&build_nrom_with_program(&[0xA9, 0x42])).unwrap();

    let cycles = console.step();
    assert_eq!(cycles, 2);
    assert_eq!(
        console.cpu.state().pc,
        0xC002,
        "PC = {:#06X}",
        console.cpu.state().pc
    );
    assert_eq!(console.cpu.state().cycles, 7 + 2);
}

#[test]
fn step_returns_cycles_for_that_step() {
    // LDA #$42 (2), LDA $10 (3), LDA $1234 (4)
    let program = [0xA9, 0x42, 0xA5, 0x10, 0xAD, 0x34, 0x12];
    let mut console = Console::new(&build_nrom_with_program(&program)).unwrap();

    assert_eq!(console.step(), 2);
    assert_eq!(console.step(), 3);
    assert_eq!(console.step(), 4);
}

#[test]
fn program_can_write_and_read_ram() {
    // LDA #$42, STA $0200, LDA #$00, LDA $0A00 (mirror of $0200)
    let program = [0xA9, 0x42, 0x8D, 0x00, 0x02, 0xA9, 0x00, 0xAD, 0x00, 0x0A];
    let mut console = Console::new(&build_nrom_with_program(&program)).unwrap();

    for _ in 0..4 {
        console.step();
    }
    assert_eq!(console.bus.read(0x0200), 0x42);
    assert_eq!(
        console.cpu.state().a,
        0x42,
        "A = {:#04X}",
        console.cpu.state().a
    );
}

// jump_to

#[test]
fn jump_to_sets_pc() {
    let mut console = Console::new(&build_nrom_with_program(&[])).unwrap();

    console.jump_to(0xC123);
    assert_eq!(console.cpu_state().pc, 0xC123);
}

#[test]
fn jump_to_does_not_add_cycles() {
    // nestest.log line 1 expects CYC:7 at $C000, the same as right after reset.
    let mut console = Console::new(&build_nrom_with_program(&[])).unwrap();

    console.jump_to(0xC000);
    assert_eq!(console.cpu_state().cycles, 7);
}

#[test]
fn jump_to_leaves_other_state_unchanged() {
    let mut console = Console::new(&build_nrom_with_program(&[])).unwrap();
    let before = console.cpu_state();

    console.jump_to(0xC123);
    let after = console.cpu_state();
    assert_eq!(
        after,
        CpuState {
            pc: 0xC123,
            ..before
        }
    );
}

#[test]
fn step_after_jump_runs_code_at_new_address() {
    // $C000: NOP padding; $C010: LDA #$42
    let mut program = [0xEA; 0x12];
    program[0x10] = 0xA9;
    program[0x11] = 0x42;
    let mut console = Console::new(&build_nrom_with_program(&program)).unwrap();

    console.jump_to(0xC010);
    console.step();
    assert_eq!(console.cpu_state().a, 0x42);
    assert_eq!(console.cpu_state().pc, 0xC012);
}
