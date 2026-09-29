use super::*;
use crate::bus::Bus;

#[test]
fn new_resets_cpu() {
    // No cartridge yet, so the reset vector reads $0000 and reset costs 7 cycles.
    let console = Console::new();

    assert_eq!(console.cpu.cycle_count(), 7);
}

#[test]
fn step_returns_cycles_for_that_step() {
    // Every byte reads $00 (BRK), which takes 7 cycles.
    let mut console = Console::new();

    assert_eq!(console.step(), 7);
    assert_eq!(console.step(), 7);
}

#[test]
fn step_advances_cpu_cycle_count() {
    let mut console = Console::new();

    console.step();
    assert_eq!(console.cpu.cycle_count(), 7 + 7);
}

#[test]
fn step_uses_console_ram() {
    // BRK pushes PC and P onto the stack, which lives in CpuBus RAM at $01xx.
    let mut console = Console::new();

    console.step();
    assert_ne!(console.bus.read(0x01FB), 0x00, "pushed P should be in RAM");
}
