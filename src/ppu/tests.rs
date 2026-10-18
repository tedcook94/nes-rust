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

static IDLE_MAPPER: TestMapper = TestMapper {
    chr: [0; 0x2000],
    mirroring: Mirroring::Horizontal,
};

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

// Registers

fn set_address(ppu: &mut Ppu, mapper: &mut TestMapper, address: u16) {
    let [low, high] = address.to_le_bytes();
    ppu.write_register(0x2006, high, mapper);
    ppu.write_register(0x2006, low, mapper);
}

#[test]
fn ppuaddr_sets_high_byte_then_low_byte() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2006, 0x21, &mut mapper);
    ppu.write_register(0x2006, 0x08, &mut mapper);
    assert_eq!(ppu.v, 0x2108, "v = {:#06X}", ppu.v);
}

#[test]
fn ppuaddr_drops_top_two_bits() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0xFF00);
    assert_eq!(ppu.v, 0x3F00, "v = {:#06X}", ppu.v);
}

#[test]
fn ppuaddr_third_write_starts_over_at_high_byte() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x2108);
    ppu.write_register(0x2006, 0x3F, &mut mapper);
    assert_eq!(ppu.v, 0x3F08, "v = {:#06X}", ppu.v);
}

#[test]
fn ppustatus_read_resets_write_toggle() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2006, 0x21, &mut mapper);
    ppu.read_register(0x2002, &mapper);
    ppu.write_register(0x2006, 0x23, &mut mapper);
    ppu.write_register(0x2006, 0x45, &mut mapper);
    assert_eq!(ppu.v, 0x2345, "v = {:#06X}", ppu.v);
}

#[test]
fn ppudata_write_goes_to_current_address() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x2042);
    ppu.write_register(0x2007, 0x99, &mut mapper);
    assert_eq!(ppu.read(0x2042, &mapper), 0x99);
}

#[test]
fn ppudata_writes_increment_by_one() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x2000);
    for value in [0x11, 0x22, 0x33] {
        ppu.write_register(0x2007, value, &mut mapper);
    }
    assert_eq!(ppu.read(0x2000, &mapper), 0x11);
    assert_eq!(ppu.read(0x2001, &mapper), 0x22);
    assert_eq!(ppu.read(0x2002, &mapper), 0x33);
    assert_eq!(ppu.v, 0x2003);
}

#[test]
fn ppudata_writes_increment_by_32_when_ctrl_bit_2_set() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2000, 0b0000_0100, &mut mapper);
    set_address(&mut ppu, &mut mapper, 0x2000);
    ppu.write_register(0x2007, 0x11, &mut mapper);
    ppu.write_register(0x2007, 0x22, &mut mapper);
    assert_eq!(ppu.read(0x2000, &mapper), 0x11);
    assert_eq!(ppu.read(0x2020, &mapper), 0x22);
    assert_eq!(ppu.read(0x2001, &mapper), 0x00);
    assert_eq!(ppu.v, 0x2040);
}

#[test]
fn ppudata_write_reaches_pattern_tables_through_mapper() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x0123);
    ppu.write_register(0x2007, 0x99, &mut mapper);
    assert_eq!(mapper.chr[0x0123], 0x99);
}

#[test]
fn ppudata_read_is_delayed_by_one() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write(0x2000, 0x11, &mut mapper);
    ppu.write(0x2001, 0x22, &mut mapper);

    set_address(&mut ppu, &mut mapper, 0x2000);
    assert_eq!(ppu.read_register(0x2007, &mapper), 0x00, "stale buffer");
    assert_eq!(ppu.read_register(0x2007, &mapper), 0x11);
    assert_eq!(ppu.read_register(0x2007, &mapper), 0x22);
}

#[test]
fn ppudata_reads_increment_address() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x2000);
    ppu.read_register(0x2007, &mapper);
    ppu.read_register(0x2007, &mapper);
    assert_eq!(ppu.v, 0x2002);
}

#[test]
fn ppudata_reads_increment_by_32_when_ctrl_bit_2_set() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2000, 0b0000_0100, &mut mapper);
    set_address(&mut ppu, &mut mapper, 0x2000);
    ppu.read_register(0x2007, &mapper);
    assert_eq!(ppu.v, 0x2020);
}

#[test]
fn palette_read_is_not_delayed() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write(0x3F01, 0x2A, &mut mapper);

    set_address(&mut ppu, &mut mapper, 0x3F01);
    assert_eq!(ppu.read_register(0x2007, &mapper), 0x2A);
}

#[test]
fn palette_read_fills_buffer_from_nametable_underneath() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write(0x3F01, 0x2A, &mut mapper);
    ppu.write(0x2F01, 0x77, &mut mapper);

    set_address(&mut ppu, &mut mapper, 0x3F01);
    ppu.read_register(0x2007, &mapper);
    set_address(&mut ppu, &mut mapper, 0x2000);
    assert_eq!(ppu.read_register(0x2007, &mapper), 0x77);
}

#[test]
fn ppudata_address_wraps_after_3fff() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x3FFF);
    ppu.write_register(0x2007, 0x2A, &mut mapper);
    assert_eq!(ppu.v, 0x0000, "v = {:#06X}", ppu.v);
}

#[test]
fn ppuctrl_write_does_not_change_address() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x2108);
    ppu.write_register(0x2000, 0xFF, &mut mapper);
    assert_eq!(ppu.v, 0x2108);
}

// Timing

const DOTS_PER_FRAME: u32 = 341 * 262;

fn tick_to(ppu: &mut Ppu, scanline: u16, dot: u16) {
    while (ppu.scanline, ppu.dot) != (scanline, dot) {
        ppu.tick(&IDLE_MAPPER);
    }
}

fn read_status(ppu: &mut Ppu) -> u8 {
    ppu.read_register(0x2002, &TestMapper::new(Mirroring::Horizontal))
}

#[test]
fn dot_wraps_to_next_scanline_after_340() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 0, 340);
    ppu.tick(&IDLE_MAPPER);
    assert_eq!((ppu.scanline, ppu.dot), (1, 0));
}

#[test]
fn frame_is_341_dots_by_262_scanlines() {
    let mut ppu = Ppu::new();

    for _ in 0..DOTS_PER_FRAME {
        ppu.tick(&IDLE_MAPPER);
    }
    assert_eq!((ppu.scanline, ppu.dot), (0, 0));
}

#[test]
fn vblank_starts_at_scanline_241_dot_1() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 241, 1);
    assert!(
        !ppu.vblank,
        "not yet: the flag is set while processing dot 1"
    );
    ppu.tick(&IDLE_MAPPER);
    assert!(ppu.vblank);
}

#[test]
fn vblank_ends_on_pre_render_scanline() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 261, 1);
    assert!(ppu.vblank);
    ppu.tick(&IDLE_MAPPER);
    assert!(!ppu.vblank);
}

#[test]
fn vblank_repeats_every_frame() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 241, 2);
    read_status(&mut ppu);
    for _ in 0..DOTS_PER_FRAME {
        ppu.tick(&IDLE_MAPPER);
    }
    assert!(ppu.vblank);
}

// PPUSTATUS

#[test]
fn ppustatus_reports_vblank_in_bit_7() {
    let mut ppu = Ppu::new();

    assert_eq!(read_status(&mut ppu) & 0x80, 0);
    tick_to(&mut ppu, 241, 2);
    assert_eq!(read_status(&mut ppu) & 0x80, 0x80);
}

#[test]
fn ppustatus_read_clears_vblank() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 241, 2);
    read_status(&mut ppu);
    assert_eq!(read_status(&mut ppu) & 0x80, 0);
}

// NMI

#[test]
fn vblank_raises_nmi_when_enabled() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write_register(0x2000, 0x80, &mut mapper);

    tick_to(&mut ppu, 241, 2);
    assert!(ppu.take_nmi());
}

#[test]
fn vblank_does_not_raise_nmi_when_disabled() {
    let mut ppu = Ppu::new();

    tick_to(&mut ppu, 241, 2);
    assert!(!ppu.take_nmi());
}

#[test]
fn take_nmi_clears_it() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write_register(0x2000, 0x80, &mut mapper);

    tick_to(&mut ppu, 241, 2);
    assert!(ppu.take_nmi());
    assert!(!ppu.take_nmi());
}

#[test]
fn nmi_is_raised_once_per_frame() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write_register(0x2000, 0x80, &mut mapper);

    tick_to(&mut ppu, 241, 2);
    ppu.take_nmi();
    tick_to(&mut ppu, 260, 0);
    assert!(!ppu.take_nmi());
}

#[test]
fn enabling_nmi_during_vblank_raises_it_immediately() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    tick_to(&mut ppu, 245, 0);
    ppu.write_register(0x2000, 0x80, &mut mapper);
    assert!(ppu.take_nmi());
}

#[test]
fn rewriting_enabled_nmi_during_vblank_does_not_raise_again() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write_register(0x2000, 0x80, &mut mapper);

    tick_to(&mut ppu, 245, 0);
    ppu.take_nmi();
    ppu.write_register(0x2000, 0x80, &mut mapper);
    assert!(!ppu.take_nmi());
}

#[test]
fn enabling_nmi_outside_vblank_does_not_raise_it() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    tick_to(&mut ppu, 100, 0);
    ppu.write_register(0x2000, 0x80, &mut mapper);
    assert!(!ppu.take_nmi());
}

// Background rendering

fn render(ppu: &mut Ppu, mapper: &TestMapper) {
    tick_to_with(ppu, mapper, 241, 2);
}

fn tick_to_with(ppu: &mut Ppu, mapper: &TestMapper, scanline: u16, dot: u16) {
    while (ppu.scanline, ppu.dot) != (scanline, dot) {
        ppu.tick(mapper);
    }
}

fn pixel(ppu: &Ppu, x: usize, y: usize) -> u8 {
    ppu.frame()[y * WIDTH + x]
}

// Tile 1: row 0 is colors 0,1,2,3,0,1,2,3; every other row is color 3.
fn mapper_with_test_tile(base: usize) -> TestMapper {
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    let tile = base + 16;
    mapper.chr[tile] = 0b0101_0101;
    mapper.chr[tile + 8] = 0b0011_0011;
    for y in 1..8 {
        mapper.chr[tile + y] = 0xFF;
        mapper.chr[tile + 8 + y] = 0xFF;
    }
    mapper
}

fn set_palettes(ppu: &mut Ppu, mapper: &mut TestMapper) {
    ppu.write(0x3F00, 0x0F, mapper); // backdrop
    for palette in 0..4u16 {
        for color in 1..4u16 {
            // Background palette p, color c → value $pc, so each is distinct.
            ppu.write(
                0x3F00 + palette * 4 + color,
                0x10 * palette as u8 + color as u8,
                mapper,
            );
        }
    }
}

fn enable_background(ppu: &mut Ppu, mapper: &mut TestMapper) {
    ppu.write_register(0x2001, 0b0000_1000, mapper);
}

#[test]
fn background_disabled_fills_frame_with_backdrop() {
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000, 0x01, &mut mapper);

    render(&mut ppu, &mapper);
    assert!(ppu.frame().iter().all(|&p| p == 0x0F));
}

#[test]
fn tile_colors_map_through_palette() {
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    render(&mut ppu, &mapper);
    let row: Vec<u8> = (0..8).map(|x| pixel(&ppu, x, 0)).collect();
    assert_eq!(row, [0x0F, 0x01, 0x02, 0x03, 0x0F, 0x01, 0x02, 0x03]);
    assert_eq!(pixel(&ppu, 0, 7), 0x03);
}

#[test]
fn color_zero_uses_backdrop_not_palette_entry_zero() {
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x3F04, 0x2A, &mut mapper); // palette 1 entry 0: unused for rendering
    ppu.write(0x23C0, 0b0000_0001, &mut mapper); // top-left quadrant uses palette 1
    ppu.write(0x2000, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 0, 0), 0x0F);
    assert_eq!(pixel(&ppu, 1, 0), 0x11);
}

#[test]
fn tile_is_placed_by_nametable_position() {
    // Tile 1 at column 5, row 3 → pixels (40..48, 24..32).
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000 + 3 * 32 + 5, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 41, 24), 0x01);
    assert_eq!(pixel(&ppu, 41, 23), 0x0F);
    assert_eq!(pixel(&ppu, 49, 24), 0x0F);
}

#[test]
fn attribute_quadrants_select_palettes() {
    // Attribute byte $23C0 covers tiles (0..4, 0..4):
    // bits 0-1 top-left, 2-3 top-right, 4-5 bottom-left, 6-7 bottom-right.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    for (column, row) in [(0, 0), (2, 0), (0, 2), (2, 2)] {
        ppu.write(0x2000 + row * 32 + column, 0x01, &mut mapper);
    }
    ppu.write(0x23C0, 0b11_10_01_00, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 1, 0), 0x01, "top-left: palette 0");
    assert_eq!(pixel(&ppu, 17, 0), 0x11, "top-right: palette 1");
    assert_eq!(pixel(&ppu, 1, 16), 0x21, "bottom-left: palette 2");
    assert_eq!(pixel(&ppu, 17, 16), 0x31, "bottom-right: palette 3");
}

#[test]
fn attribute_byte_is_chosen_per_4x4_tile_block() {
    // Tile at column 4 uses the second attribute byte, $23C1.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2004, 0x01, &mut mapper);
    ppu.write(0x23C1, 0b0000_0011, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 33, 0), 0x31);
}

#[test]
fn ctrl_bit_4_selects_pattern_table_at_1000() {
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0x1000);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 1, 0), 0x0F, "table $0000 tile 1 is blank");

    let mut ppu = Ppu::new();
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    ppu.write_register(0x2000, 0b0001_0000, &mut mapper);
    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 1, 0), 0x01);
}

#[test]
fn ctrl_bits_0_and_1_select_nametable() {
    // Vertical mirroring: $2400 is a separate table from $2000.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    mapper.mirroring = Mirroring::Vertical;
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2400, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    ppu.write_register(0x2000, 0b0000_0001, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 1, 0), 0x01);
}

#[test]
fn bottom_right_tile_is_rendered() {
    // Column 31, row 29 → pixels (248..256, 232..240).
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000 + 29 * 32 + 31, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 255, 239), 0x03);
}

#[test]
fn frame_is_ready_once_per_frame_at_vblank() {
    let mut ppu = Ppu::new();
    let mapper = TestMapper::new(Mirroring::Horizontal);

    tick_to_with(&mut ppu, &mapper, 241, 1);
    assert!(!ppu.take_frame_ready());
    ppu.tick(&mapper);
    assert!(ppu.take_frame_ready());
    assert!(!ppu.take_frame_ready());
}

#[test]
fn rendered_palette_values_use_low_6_bits() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.write(0x3F00, 0xC5, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 0, 0), 0x05);
}

// OAM

#[test]
fn oamdata_write_goes_to_oam_address() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2003, 0x10, &mut mapper);
    ppu.write_register(0x2004, 0x99, &mut mapper);
    assert_eq!(ppu.oam[0x10], 0x99);
}

#[test]
fn oamdata_writes_increment_address() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2003, 0x10, &mut mapper);
    ppu.write_register(0x2004, 0x11, &mut mapper);
    ppu.write_register(0x2004, 0x22, &mut mapper);
    assert_eq!(ppu.oam[0x10..0x12], [0x11, 0x22]);
    assert_eq!(ppu.oam_address, 0x12);
}

#[test]
fn oamdata_read_does_not_increment_address() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.oam[0x10] = 0x99;

    ppu.write_register(0x2003, 0x10, &mut mapper);
    assert_eq!(ppu.read_register(0x2004, &mapper), 0x99);
    assert_eq!(ppu.read_register(0x2004, &mapper), 0x99);
    assert_eq!(ppu.oam_address, 0x10);
}

#[test]
fn oam_address_wraps_after_ff() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2003, 0xFF, &mut mapper);
    ppu.write_register(0x2004, 0x11, &mut mapper);
    ppu.write_register(0x2004, 0x22, &mut mapper);
    assert_eq!(ppu.oam[0xFF], 0x11);
    assert_eq!(ppu.oam[0x00], 0x22);
}
