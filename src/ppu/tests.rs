use super::*;

// A minimal mapper: 8KB of writable CHR and a chosen mirroring.
struct TestMapper {
    chr: [u8; 0x2000],
    mirroring: Mirroring,
}

impl TestMapper {
    fn new(mirroring: Mirroring) -> Self {
        TestMapper {
            chr: [0; 0x2000],
            mirroring,
        }
    }
}

impl Mapper for TestMapper {
    fn cpu_read(&self, _address: u16) -> u8 {
        0
    }
    fn cpu_write(&mut self, _address: u16, _value: u8) {}
    fn ppu_read(&self, address: u16) -> u8 {
        self.chr[usize::from(address)]
    }
    fn ppu_write(&mut self, address: u16, value: u8) {
        self.chr[usize::from(address)] = value;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// Pattern tables

#[test]
fn pattern_table_reads_go_to_mapper() {
    let ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    mapper.chr[0x0000] = 0x11;
    mapper.chr[0x1FFF] = 0x22;

    assert_eq!(ppu.read(0x0000, &mapper), 0x11);
    assert_eq!(ppu.read(0x1FFF, &mapper), 0x22);
}

#[test]
fn pattern_table_writes_go_to_mapper() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x1234, 0x99, &mut mapper);
    assert_eq!(mapper.chr[0x1234], 0x99);
}

#[test]
fn pattern_table_writes_do_not_touch_nametables() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x0000, 0x99, &mut mapper);
    assert_eq!(ppu.read(0x2000, &mapper), 0x00);
}

// Nametables

#[test]
fn nametable_reads_back_written_value() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x2042, 0x99, &mut mapper);
    assert_eq!(ppu.read(0x2042, &mapper), 0x99);
}

#[test]
fn nametable_writes_do_not_reach_mapper() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x2000, 0x99, &mut mapper);
    assert!(mapper.chr.iter().all(|&b| b == 0));
}

// (address written, addresses that must share it, address that must not)
fn assert_mirroring(mirroring: Mirroring, write: u16, shared: u16, separate: u16) {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(mirroring);

    ppu.write(write, 0x99, &mut mapper);
    assert_eq!(
        ppu.read(shared, &mapper),
        0x99,
        "{mirroring:?}: {write:#06X} should mirror to {shared:#06X}"
    );
    assert_eq!(
        ppu.read(separate, &mapper),
        0x00,
        "{mirroring:?}: {write:#06X} should not mirror to {separate:#06X}"
    );
}

#[test]
fn horizontal_mirroring_shares_left_and_right_tables() {
    // A ($2000) = B ($2400), C ($2800) = D ($2C00)
    assert_mirroring(Mirroring::Horizontal, 0x2005, 0x2405, 0x2805);
    assert_mirroring(Mirroring::Horizontal, 0x2805, 0x2C05, 0x2005);
}

#[test]
fn vertical_mirroring_shares_top_and_bottom_tables() {
    // A ($2000) = C ($2800), B ($2400) = D ($2C00)
    assert_mirroring(Mirroring::Vertical, 0x2005, 0x2805, 0x2405);
    assert_mirroring(Mirroring::Vertical, 0x2405, 0x2C05, 0x2005);
}

#[test]
fn nametable_last_byte_of_each_table_stays_in_that_table() {
    // $23FF is the last byte of table A; it must not spill into table B.
    assert_mirroring(Mirroring::Vertical, 0x23FF, 0x2BFF, 0x27FF);
}

#[test]
fn nametables_mirror_into_3000_range() {
    // $3000-$3EFF is a copy of $2000-$2EFF.
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Vertical);

    ppu.write(0x2123, 0x99, &mut mapper);
    assert_eq!(ppu.read(0x3123, &mapper), 0x99);
    ppu.write(0x3E00, 0x77, &mut mapper);
    assert_eq!(ppu.read(0x2E00, &mapper), 0x77);
}

#[test]
fn mirroring_is_read_from_mapper_on_each_access() {
    // MMC3 can switch mirroring at runtime, so it must not be cached.
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x2005, 0x99, &mut mapper);
    assert_eq!(ppu.read(0x2405, &mapper), 0x99);
    mapper.mirroring = Mirroring::Vertical;
    assert_eq!(ppu.read(0x2405, &mapper), 0x00);
    assert_eq!(ppu.read(0x2805, &mapper), 0x99);
}

// Palettes

#[test]
fn palette_reads_back_written_value() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x3F01, 0x2A, &mut mapper);
    ppu.write(0x3F1F, 0x15, &mut mapper);
    assert_eq!(ppu.read(0x3F01, &mapper), 0x2A);
    assert_eq!(ppu.read(0x3F1F, &mapper), 0x15);
}

#[test]
fn palette_repeats_every_32_bytes() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x3F05, 0x2A, &mut mapper);
    assert_eq!(ppu.read(0x3F25, &mapper), 0x2A);
    assert_eq!(ppu.read(0x3FE5, &mapper), 0x2A);
}

#[test]
fn sprite_palette_entry_zero_mirrors_background() {
    for (sprite, background) in [
        (0x3F10, 0x3F00),
        (0x3F14, 0x3F04),
        (0x3F18, 0x3F08),
        (0x3F1C, 0x3F0C),
    ] {
        let mut ppu = Ppu::new();
        let mut mapper = TestMapper::new(Mirroring::Horizontal);

        ppu.write(sprite, 0x2A, &mut mapper);
        assert_eq!(ppu.read(background, &mapper), 0x2A, "write {sprite:#06X}");
        ppu.write(background, 0x15, &mut mapper);
        assert_eq!(ppu.read(sprite, &mapper), 0x15, "write {background:#06X}");
    }
}

#[test]
fn other_sprite_palette_entries_are_separate() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x3F11, 0x2A, &mut mapper);
    assert_eq!(ppu.read(0x3F01, &mapper), 0x00);
}

#[test]
fn palette_writes_do_not_touch_nametables() {
    // $3F00 is above the nametable mirror range; it must not alias $2F00.
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x3F00, 0x2A, &mut mapper);
    assert_eq!(ppu.read(0x2F00, &mapper), 0x00);
}

// Address range

#[test]
fn addresses_above_3fff_mirror_down() {
    // The PPU address bus is 14 bits wide.
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write(0x6042, 0x99, &mut mapper);
    assert_eq!(ppu.read(0x2042, &mapper), 0x99);
}
