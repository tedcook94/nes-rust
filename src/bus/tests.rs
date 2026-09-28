use super::*;

#[test]
fn ram_reads_back_written_value() {
    let mut bus = CpuBus::new();

    bus.write(0x0042, 0x99);
    assert_eq!(bus.read(0x0042), 0x99);
}

#[test]
fn ram_starts_zeroed() {
    let mut bus = CpuBus::new();

    assert_eq!(bus.read(0x0000), 0x00);
    assert_eq!(bus.read(0x07FF), 0x00);
}

#[test]
fn ram_write_is_visible_at_all_mirrors() {
    let mut bus = CpuBus::new();

    bus.write(0x0042, 0x99);
    for address in [0x0042, 0x0842, 0x1042, 0x1842] {
        assert_eq!(bus.read(address), 0x99, "read at {address:#06X}");
    }
}

#[test]
fn ram_write_to_mirror_is_visible_at_base() {
    let mut bus = CpuBus::new();

    bus.write(0x1842, 0x99);
    assert_eq!(bus.read(0x0042), 0x99);
}

#[test]
fn ram_last_mirror_address_maps_to_last_ram_byte() {
    let mut bus = CpuBus::new();

    bus.write(0x1FFF, 0x99);
    assert_eq!(bus.read(0x07FF), 0x99);
}

#[test]
fn ram_first_and_last_bytes_are_distinct() {
    // Catches a mirror mask that's too narrow (e.g. & 0x00FF).
    let mut bus = CpuBus::new();

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
    let mut bus = CpuBus::new();

    bus.write(0x0000, 0x99);
    assert_ne!(bus.read(0x2000), 0x99);
}
