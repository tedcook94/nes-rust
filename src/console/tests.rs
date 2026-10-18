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

// PPU timing

const STEP_LIMIT: usize = 100_000;

fn step_until_pc(console: &mut Console, pc: u16) -> bool {
    (0..STEP_LIMIT).any(|_| {
        console.step();
        console.cpu_state().pc == pc
    })
}

#[test]
fn program_waiting_for_vblank_continues_after_it() {
    // $C000: BIT $2002 / BPL $C000 / LDA #$42
    let program = [0x2C, 0x02, 0x20, 0x10, 0xFB, 0xA9, 0x42];
    let mut console = Console::new(&build_nrom_with_program(&program)).unwrap();

    assert!(
        step_until_pc(&mut console, 0xC005),
        "never left the vblank loop"
    );
    // Vblank starts 241 * 341 + 1 dots after reset's 7 cycles, so about cycle 27,401.
    // The loop polls every 7 cycles, so the exit lands up to one iteration later.
    let cycles = console.cpu_state().cycles;
    assert!((27_401..=27_416).contains(&cycles), "cycles = {cycles}");
}

#[test]
fn vblank_nmi_runs_handler() {
    // $C000: LDA #$80 / STA $2000 / JMP $C005. NMI handler at $C010: LDX #$42 / RTI.
    let mut program = [0xEA; 0x13];
    program[..8].copy_from_slice(&[0xA9, 0x80, 0x8D, 0x00, 0x20, 0x4C, 0x05, 0xC0]);
    program[0x10..].copy_from_slice(&[0xA2, 0x42, 0x40]);
    let mut rom = build_nrom_with_program(&program);
    rom[16 + 0x3FFA] = 0x10; // NMI vector → $C010
    rom[16 + 0x3FFB] = 0xC0;
    let mut console = Console::new(&rom).unwrap();

    assert!(step_until_pc(&mut console, 0xC012), "NMI handler never ran");
    assert_eq!(console.cpu_state().x, 0x42);
    assert!(
        step_until_pc(&mut console, 0xC005),
        "RTI didn't return to the loop"
    );
}

#[test]
fn no_nmi_when_disabled() {
    // $C000: JMP $C000. NMI vector → $C010 (LDX #$42), which must never run.
    let mut program = [0xEA; 0x12];
    program[..3].copy_from_slice(&[0x4C, 0x00, 0xC0]);
    program[0x10..].copy_from_slice(&[0xA2, 0x42]);
    let mut rom = build_nrom_with_program(&program);
    rom[16 + 0x3FFA] = 0x10;
    rom[16 + 0x3FFB] = 0xC0;
    let mut console = Console::new(&rom).unwrap();

    assert!(!step_until_pc(&mut console, 0xC012));
}

// Frames

#[test]
fn step_frame_runs_one_frame() {
    // A frame is 89,342 dots, about 29,781 CPU cycles.
    let mut console = Console::new(&build_nrom_with_program(&[0x4C, 0x00, 0xC0])).unwrap();

    console.step_frame();
    let first = console.cpu_state().cycles;
    console.step_frame();
    let second = console.cpu_state().cycles;
    assert!(
        (29_775..=29_785).contains(&(second - first)),
        "frame took {} cycles",
        second - first
    );
}

#[test]
fn frame_shows_backdrop_written_by_program() {
    // LDA #$3F / STA $2006 / LDA #$00 / STA $2006 / LDA #$21 / STA $2007 / JMP self
    let program = [
        0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, 0xA9, 0x21, 0x8D, 0x07, 0x20,
        0x4C, 0x0F, 0xC0,
    ];
    let mut console = Console::new(&build_nrom_with_program(&program)).unwrap();

    console.step_frame();
    assert_eq!(console.frame().len(), 256 * 240);
    assert!(console.frame().iter().all(|&p| p == 0x21));
}
