use super::*;
use crate::cartridge::test_rom::{PRG_UNIT, build_rom};
use crate::cartridge::{Cartridge, create_mapper};

fn create_bus_with_rom(rom: &[u8]) -> CpuBus {
    CpuBus::new(create_mapper(Cartridge::parse(rom).unwrap()).unwrap())
}

// A bus with a 16KB NROM cartridge whose PRG is all $AA.
fn create_bus() -> CpuBus {
    create_bus_with_rom(&build_rom(1, 1, 0, 0))
}

#[test]
fn ram_reads_back_written_value() {
    let mut bus = create_bus();

    bus.write(0x0042, 0x99);
    assert_eq!(bus.read(0x0042), 0x99);
}

#[test]
fn ram_starts_zeroed() {
    let mut bus = create_bus();

    assert_eq!(bus.read(0x0000), 0x00);
    assert_eq!(bus.read(0x07FF), 0x00);
}

#[test]
fn ram_write_is_visible_at_all_mirrors() {
    let mut bus = create_bus();

    bus.write(0x0042, 0x99);
    for address in [0x0042, 0x0842, 0x1042, 0x1842] {
        assert_eq!(bus.read(address), 0x99, "read at {address:#06X}");
    }
}

#[test]
fn ram_write_to_mirror_is_visible_at_base() {
    let mut bus = create_bus();

    bus.write(0x1842, 0x99);
    assert_eq!(bus.read(0x0042), 0x99);
}

#[test]
fn ram_last_mirror_address_maps_to_last_ram_byte() {
    let mut bus = create_bus();

    bus.write(0x1FFF, 0x99);
    assert_eq!(bus.read(0x07FF), 0x99);
}

#[test]
fn ram_first_and_last_bytes_are_distinct() {
    // Catches a mirror mask that's too narrow (e.g. & 0x00FF).
    let mut bus = create_bus();

    bus.write(0x0000, 0x11);
    bus.write(0x07FF, 0x22);
    bus.write(0x0100, 0x33);
    assert_eq!(bus.read(0x0000), 0x11);
    assert_eq!(bus.read(0x07FF), 0x22);
    assert_eq!(bus.read(0x0100), 0x33);
}

#[test]
fn ram_does_not_extend_past_mirrors() {
    // $2000 is a PPU register, not RAM: a write to $0000 must not appear there.
    let mut bus = create_bus();

    bus.write(0x0000, 0x99);
    assert_ne!(bus.read(0x2000), 0x99);
}

// Stubbed ranges

// One address from the start, middle and end of each non-RAM range.
const STUBBED_ADDRESSES: [u16; 10] = [
    0x2000, 0x2007, 0x3FFF, // PPU registers and mirrors
    0x4000, 0x4014, 0x4017, // APU and I/O
    0x4018, 0x401F, // unused test-mode registers
    0x4020, 0x6000, // cartridge space below PRG (NROM has nothing here)
];

#[test]
fn stubbed_ranges_read_zero() {
    let mut bus = create_bus();

    for address in STUBBED_ADDRESSES {
        assert_eq!(bus.read(address), 0x00, "read at {address:#06X}");
    }
}

#[test]
fn stubbed_ranges_ignore_writes() {
    let mut bus = create_bus();

    for address in STUBBED_ADDRESSES {
        bus.write(address, 0x99);
        assert_eq!(bus.read(address), 0x00, "read at {address:#06X}");
    }
}

#[test]
fn stubbed_range_writes_do_not_touch_ram() {
    // Catches applying the RAM mask to every address: $2042 & $07FF = $0042.
    let mut bus = create_bus();

    for address in [0x2042, 0x4042, 0x6042] {
        bus.write(address, 0x99);
    }
    assert_eq!(bus.read(0x0042), 0x00);
}

// PPU register mirroring

#[test]
fn ppu_register_maps_base_registers_to_themselves() {
    for address in 0x2000..=0x2007 {
        assert_eq!(ppu_register(address), address, "{address:#06X}");
    }
}

#[test]
fn ppu_register_mirrors_every_eight_bytes() {
    let cases = [
        (0x2008, 0x2000),
        (0x200F, 0x2007),
        (0x3456, 0x2006),
        (0x3FF8, 0x2000),
        (0x3FFF, 0x2007),
    ];
    for (address, expected) in cases {
        assert_eq!(
            ppu_register(address),
            expected,
            "{address:#06X} → {:#06X}",
            ppu_register(address)
        );
    }
}

// Cartridge

#[test]
fn reads_prg_from_cartridge() {
    let mut rom = build_rom(2, 1, 0, 0);
    rom[16] = 0x11; // PRG offset 0
    rom[16 + 2 * PRG_UNIT - 1] = 0x22; // last PRG byte
    let mut bus = create_bus_with_rom(&rom);

    assert_eq!(bus.read(0x8000), 0x11);
    assert_eq!(bus.read(0xFFFF), 0x22);
}

#[test]
fn cartridge_reads_are_mapped_by_mapper() {
    // 16KB NROM mirrors: $C000 reads the same PRG byte as $8000.
    let mut rom = build_rom(1, 1, 0, 0);
    rom[16] = 0x11;
    let mut bus = create_bus_with_rom(&rom);

    assert_eq!(bus.read(0xC000), 0x11);
}

#[test]
fn writes_to_prg_do_not_change_it() {
    let mut bus = create_bus();

    bus.write(0x8000, 0x99);
    assert_eq!(bus.read(0x8000), 0xAA);
}

#[test]
fn ram_writes_do_not_reach_cartridge() {
    let mut bus = create_bus();

    bus.write(0x0000, 0x99);
    assert_eq!(bus.read(0x8000), 0xAA);
}
