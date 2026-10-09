//! Runs nestest.nes and compares CPU state against the reference log.

use nes::console::Console;

const ROM: &[u8] = include_bytes!("roms/nestest.nes");
const LOG: &str = include_str!("roms/nestest.log");

const OFFICIAL_OPCODE_LINES: usize = 5003;

fn expected_state(line: &str) -> String {
    let pc = &line[0..4];
    let registers_start = line.find("A:").expect("log line should contain A:");
    let registers_end = line.find(" PPU:").expect("log line should contain PPU:");
    let cycles_start = line.find("CYC:").expect("log line should contain CYC:");
    format!(
        "{pc} {} {}",
        &line[registers_start..registers_end],
        &line[cycles_start..]
    )
}

fn start_nestest() -> Console {
    let mut console = Console::new(ROM).expect("nestest.nes should load");
    console.jump_to(0xC000); // automation entry point: runs every test without input
    console
}

#[test]
fn expected_state_strips_unused_columns() {
    let line = "C000  4C F5 C5  JMP $C5F5                       A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7";

    assert_eq!(expected_state(line), "C000 A:00 X:00 Y:00 P:24 SP:FD CYC:7");
}

#[test]
fn nestest_starts_at_automation_entry_point() {
    let console = start_nestest();

    assert_eq!(
        console.cpu_state().to_string(),
        expected_state(LOG.lines().next().unwrap())
    );
}

#[test]
fn nestest_official_opcodes_match_log() {
    let mut console = start_nestest();
    let lines: Vec<&str> = LOG.lines().take(OFFICIAL_OPCODE_LINES).collect();

    for (index, line) in lines.iter().enumerate() {
        let expected = expected_state(line);
        let actual = console.cpu_state().to_string();
        if actual != expected {
            let previous = index.checked_sub(1).map_or("(none)", |i| lines[i]);
            panic!(
                "mismatch at nestest.log line {}\n\
                 previous instruction: {previous}\n\
                 expected: {expected}\n\
                 actual:   {actual}",
                index + 1,
            );
        }
        console.step();
    }
}
