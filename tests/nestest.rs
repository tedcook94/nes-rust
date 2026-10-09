//! Runs nestest.nes and compares CPU state against the reference log.

use nes::console::Console;

const ROM: &[u8] = include_bytes!("roms/nestest.nes");

#[test]
fn nestest_starts_at_automation_entry_point() {
    // nestest.log line 1, without the instruction bytes, disassembly and PPU columns.
    let mut console = Console::new(ROM).expect("nestest.nes should load");

    console.jump_to(0xC000);
    assert_eq!(
        console.cpu_state().to_string(),
        "C000 A:00 X:00 Y:00 P:24 SP:FD CYC:7"
    );
}
