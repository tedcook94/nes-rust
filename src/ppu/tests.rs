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
    assert_eq!(ppu.v, 0x2108, "v only updates on the second write");
    ppu.write_register(0x2006, 0x00, &mut mapper);
    assert_eq!(ppu.v, 0x3F00, "v = {:#06X}", ppu.v);
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

fn render_second_frame(ppu: &mut Ppu, mapper: &TestMapper) {
    render(ppu, mapper);
    tick_to_with(ppu, mapper, 0, 0);
    render(ppu, mapper);
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

    // The nametable bits reach v on the pre-render line, so they apply from the next frame.
    render_second_frame(&mut ppu, &mapper);
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

// Sprite rendering

fn set_sprite_palettes(ppu: &mut Ppu, mapper: &mut TestMapper) {
    for palette in 0..4u16 {
        for color in 1..4u16 {
            // Sprite palette p, color c → value $p(c+4), distinct from background values.
            ppu.write(
                0x3F10 + palette * 4 + color,
                0x10 * palette as u8 + 4 + color as u8,
                mapper,
            );
        }
    }
}

fn enable_sprites(ppu: &mut Ppu, mapper: &mut TestMapper) {
    ppu.write_register(0x2001, ppu.mask | 0b0001_0000, mapper);
}

fn set_sprite(ppu: &mut Ppu, index: usize, y: u8, tile: u8, attributes: u8, x: u8) {
    ppu.oam[index * 4..index * 4 + 4].copy_from_slice(&[y, tile, attributes, x]);
}

// Sprite tests use tile 1 (see `mapper_with_test_tile`) and start with every
// sprite parked off-screen at Y = $FF.
fn sprite_ppu() -> (Ppu, TestMapper) {
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    ppu.oam = [0xFF; 0x100];
    set_palettes(&mut ppu, &mut mapper);
    set_sprite_palettes(&mut ppu, &mut mapper);
    enable_sprites(&mut ppu, &mut mapper);
    (ppu, mapper)
}

#[test]
fn sprite_is_drawn_one_scanline_below_its_y() {
    let (mut ppu, mapper) = sprite_ppu();
    set_sprite(&mut ppu, 0, 10, 0x01, 0, 20);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 21, 10), 0x0F, "row Y itself is empty");
    assert_eq!(pixel(&ppu, 21, 11), 0x05);
    assert_eq!(pixel(&ppu, 21, 18), 0x07, "last row");
    assert_eq!(pixel(&ppu, 21, 19), 0x0F);
}

#[test]
fn sprite_uses_its_palette() {
    let (mut ppu, mapper) = sprite_ppu();
    set_sprite(&mut ppu, 0, 10, 0x01, 0b0000_0010, 20);

    render(&mut ppu, &mapper);
    let row: Vec<u8> = (20..28).map(|x| pixel(&ppu, x, 11)).collect();
    assert_eq!(row, [0x0F, 0x25, 0x26, 0x27, 0x0F, 0x25, 0x26, 0x27]);
}

#[test]
fn sprite_color_zero_is_transparent() {
    let (mut ppu, mut mapper) = sprite_ppu();
    ppu.write(0x2000 + 32 + 2, 0x01, &mut mapper); // background tile 1 at (16, 8)
    enable_background(&mut ppu, &mut mapper);
    ppu.write_register(0x2001, 0b0001_1000, &mut mapper);
    set_sprite(&mut ppu, 0, 7, 0x01, 0, 16);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 16, 8), 0x0F, "both transparent: backdrop");
    assert_eq!(pixel(&ppu, 17, 8), 0x05, "sprite over background");
}

#[test]
fn sprite_flips_horizontally() {
    let (mut ppu, mapper) = sprite_ppu();
    set_sprite(&mut ppu, 0, 10, 0x01, 0b0100_0000, 20);

    render(&mut ppu, &mapper);
    let row: Vec<u8> = (20..28).map(|x| pixel(&ppu, x, 11)).collect();
    assert_eq!(row, [0x07, 0x06, 0x05, 0x0F, 0x07, 0x06, 0x05, 0x0F]);
}

#[test]
fn sprite_flips_vertically() {
    // Tile row 0 (the mixed row) moves to the bottom.
    let (mut ppu, mapper) = sprite_ppu();
    set_sprite(&mut ppu, 0, 10, 0x01, 0b1000_0000, 20);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 20, 11), 0x07);
    assert_eq!(pixel(&ppu, 20, 18), 0x0F);
    assert_eq!(pixel(&ppu, 21, 18), 0x05);
}

#[test]
fn sprite_behind_background_shows_only_through_transparent_pixels() {
    // Background tile 1 at (16, 8); sprite tile 1 behind it, shifted one pixel right.
    let (mut ppu, mut mapper) = sprite_ppu();
    ppu.write(0x2000 + 32 + 2, 0x01, &mut mapper);
    ppu.write_register(0x2001, 0b0001_1000, &mut mapper);
    set_sprite(&mut ppu, 0, 7, 0x01, 0b0010_0000, 17);

    render(&mut ppu, &mapper);
    // Background row: 0,1,2,3,0,... from x=16. Sprite row: 0,1,2,3,0,... from x=17.
    assert_eq!(
        pixel(&ppu, 18, 8),
        0x02,
        "background opaque: background wins"
    );
    assert_eq!(
        pixel(&ppu, 20, 8),
        0x07,
        "background transparent: sprite shows"
    );
}

#[test]
fn lower_oam_index_wins_overlap() {
    let (mut ppu, mapper) = sprite_ppu();
    set_sprite(&mut ppu, 0, 10, 0x01, 0b0000_0001, 20);
    set_sprite(&mut ppu, 1, 10, 0x01, 0b0000_0010, 20);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 21, 11), 0x15);
}

#[test]
fn higher_index_sprite_shows_through_lower_index_transparent_pixels() {
    let (mut ppu, mapper) = sprite_ppu();
    set_sprite(&mut ppu, 0, 10, 0x01, 0b0000_0001, 20);
    set_sprite(&mut ppu, 1, 10, 0x01, 0b0000_0010, 19);

    render(&mut ppu, &mapper);
    // Pixel 20 is transparent in sprite 0 but color 1 in sprite 1.
    assert_eq!(pixel(&ppu, 20, 11), 0x25);
}

#[test]
fn front_sprite_behind_background_still_hides_later_sprites() {
    // Hardware quirk: sprite 0 claims the pixel, then its priority hides it behind
    // the background, so sprite 1 (in front) never gets drawn there.
    let (mut ppu, mut mapper) = sprite_ppu();
    ppu.write(0x2000 + 32 + 2, 0x01, &mut mapper);
    ppu.write_register(0x2001, 0b0001_1000, &mut mapper);
    set_sprite(&mut ppu, 0, 7, 0x01, 0b0010_0000, 16);
    set_sprite(&mut ppu, 1, 7, 0x01, 0b0000_0010, 16);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 17, 8), 0x01);
}

#[test]
fn ctrl_bit_3_selects_sprite_pattern_table() {
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0x1000);
    ppu.oam = [0xFF; 0x100];
    set_sprite_palettes(&mut ppu, &mut mapper);
    enable_sprites(&mut ppu, &mut mapper);
    set_sprite(&mut ppu, 0, 10, 0x01, 0, 20);
    ppu.write_register(0x2000, 0b0000_1000, &mut mapper);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 21, 11), 0x05);
}

#[test]
fn sprite_8x16_uses_two_tiles_and_tile_bit_0_for_table() {
    // Tile $03 in 8x16 mode: table $1000, tiles $02 (top) and $03 (bottom).
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    mapper.chr[0x1000 + 0x02 * 16] = 0xFF; // top tile row 0: color 1
    mapper.chr[0x1000 + 0x03 * 16 + 8] = 0xFF; // bottom tile row 0: color 2
    ppu.oam = [0xFF; 0x100];
    set_sprite_palettes(&mut ppu, &mut mapper);
    enable_sprites(&mut ppu, &mut mapper);
    ppu.write_register(0x2000, 0b0010_0000, &mut mapper);
    set_sprite(&mut ppu, 0, 10, 0x03, 0, 20);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 20, 11), 0x05);
    assert_eq!(pixel(&ppu, 20, 19), 0x06);
}

#[test]
fn sprite_8x16_vertical_flip_swaps_tiles() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    mapper.chr[0x02 * 16] = 0xFF;
    mapper.chr[0x03 * 16 + 8] = 0xFF;
    ppu.oam = [0xFF; 0x100];
    set_sprite_palettes(&mut ppu, &mut mapper);
    enable_sprites(&mut ppu, &mut mapper);
    ppu.write_register(0x2000, 0b0010_0000, &mut mapper);
    set_sprite(&mut ppu, 0, 10, 0x02, 0b1000_0000, 20);

    render(&mut ppu, &mapper);
    assert_eq!(
        pixel(&ppu, 20, 26),
        0x05,
        "top tile row 0 is now the last row"
    );
    assert_eq!(pixel(&ppu, 20, 18), 0x06, "bottom tile row 0 is now row 7");
}

#[test]
fn sprites_disabled_are_not_drawn() {
    let (mut ppu, mut mapper) = sprite_ppu();
    ppu.write_register(0x2001, 0, &mut mapper);
    set_sprite(&mut ppu, 0, 10, 0x01, 0, 20);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 21, 11), 0x0F);
}

#[test]
fn sprite_is_clipped_at_right_and_bottom_edges() {
    let (mut ppu, mapper) = sprite_ppu();
    set_sprite(&mut ppu, 0, 235, 0x01, 0, 252);

    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 253, 236), 0x05);
    assert_eq!(pixel(&ppu, 255, 239), 0x07);
}

// Per-scanline rendering

#[test]
fn line_is_drawn_at_dot_256() {
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    tick_to_with(&mut ppu, &mapper, 0, 256);
    assert_eq!(pixel(&ppu, 1, 0), 0x00, "not drawn yet");
    ppu.tick(&mapper);
    assert_eq!(pixel(&ppu, 1, 0), 0x01);
}

#[test]
fn register_change_between_lines_affects_later_lines_only() {
    // Switching the background pattern table after line 3 changes lines 4+.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);

    tick_to_with(&mut ppu, &mapper, 4, 0);
    ppu.write_register(0x2000, 0b0001_0000, &mut mapper);
    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 0, 3), 0x03, "line 3 used table $0000");
    assert_eq!(pixel(&ppu, 0, 4), 0x0F, "line 4 used the blank table $1000");
}

// Sprite-0 hit

fn read_hit(ppu: &mut Ppu) -> bool {
    read_status(ppu) & 0x40 != 0
}

// Background tile 1 at (16, 8) and sprite 0 (tile 1) at the same spot.
fn sprite_zero_ppu() -> (Ppu, TestMapper) {
    let (mut ppu, mut mapper) = sprite_ppu();
    ppu.write(0x2000 + 32 + 2, 0x01, &mut mapper);
    ppu.write_register(0x2001, 0b0001_1000, &mut mapper);
    set_sprite(&mut ppu, 0, 7, 0x01, 0, 16);
    (ppu, mapper)
}

#[test]
fn sprite_zero_hit_when_opaque_pixels_overlap() {
    let (mut ppu, mapper) = sprite_zero_ppu();

    tick_to_with(&mut ppu, &mapper, 8, 257);
    assert!(read_hit(&mut ppu));
}

#[test]
fn sprite_zero_hit_is_set_on_the_overlapping_line() {
    let (mut ppu, mapper) = sprite_zero_ppu();

    tick_to_with(&mut ppu, &mapper, 7, 340);
    assert!(!read_hit(&mut ppu), "line before the sprite");
    tick_to_with(&mut ppu, &mapper, 8, 257);
    assert!(read_hit(&mut ppu));
}

#[test]
fn sprite_zero_hit_stays_set_until_pre_render() {
    let (mut ppu, mapper) = sprite_zero_ppu();

    tick_to_with(&mut ppu, &mapper, 250, 0);
    assert!(read_hit(&mut ppu));
    assert!(read_hit(&mut ppu), "a $2002 read doesn't clear it");
    tick_to_with(&mut ppu, &mapper, 261, 2);
    assert!(!read_hit(&mut ppu));
}

#[test]
fn no_sprite_zero_hit_over_transparent_background() {
    let (mut ppu, mut mapper) = sprite_zero_ppu();
    ppu.write(0x2000 + 32 + 2, 0x00, &mut mapper);

    render(&mut ppu, &mapper);
    assert!(!read_hit(&mut ppu));
}

#[test]
fn no_sprite_zero_hit_from_other_sprites() {
    let (mut ppu, mapper) = sprite_zero_ppu();
    ppu.oam[0..4].copy_from_slice(&[0xFF; 4]);
    set_sprite(&mut ppu, 1, 7, 0x01, 0, 16);

    render(&mut ppu, &mapper);
    assert!(!read_hit(&mut ppu));
}

#[test]
fn sprite_zero_hit_counts_even_when_behind_background() {
    let (mut ppu, mapper) = sprite_zero_ppu();
    ppu.oam[2] = 0b0010_0000;

    render(&mut ppu, &mapper);
    assert!(read_hit(&mut ppu));
}

#[test]
fn no_sprite_zero_hit_when_background_disabled() {
    let (mut ppu, mut mapper) = sprite_zero_ppu();
    ppu.write_register(0x2001, 0b0001_0000, &mut mapper);

    render(&mut ppu, &mapper);
    assert!(!read_hit(&mut ppu));
}

#[test]
fn no_sprite_zero_hit_when_sprites_disabled() {
    let (mut ppu, mut mapper) = sprite_zero_ppu();
    ppu.write_register(0x2001, 0b0000_1000, &mut mapper);

    render(&mut ppu, &mapper);
    assert!(!read_hit(&mut ppu));
}

#[test]
fn no_sprite_zero_hit_at_x_255() {
    // Only one sprite pixel is on screen at x = 255, and it's opaque (flipped row 0
    // starts with color 3). The background tile in column 31 is opaque there too.
    let (mut ppu, mut mapper) = sprite_ppu();
    ppu.write(0x2000 + 32 + 31, 0x01, &mut mapper);
    ppu.write_register(0x2001, 0b0001_1000, &mut mapper);
    set_sprite(&mut ppu, 0, 7, 0x01, 0b0100_0000, 255);

    render(&mut ppu, &mapper);
    assert!(!read_hit(&mut ppu));
}

// Scroll registers

#[test]
fn ppuctrl_sets_nametable_bits_of_t() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2000, 0b0000_0011, &mut mapper);
    assert_eq!(ppu.t, 0x0C00, "t = {:#06X}", ppu.t);
}

#[test]
fn ppuscroll_first_write_sets_coarse_and_fine_x() {
    // $7D = 01111_101: coarse X 15, fine X 5.
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2005, 0x7D, &mut mapper);
    assert_eq!(ppu.t, 0x000F, "t = {:#06X}", ppu.t);
    assert_eq!(ppu.fine_x, 5);
}

#[test]
fn ppuscroll_second_write_sets_coarse_and_fine_y() {
    // $5E = 01011_110: coarse Y 11, fine Y 6.
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2005, 0x00, &mut mapper);
    ppu.write_register(0x2005, 0x5E, &mut mapper);
    assert_eq!(ppu.t, 0x6160, "t = {:#06X}", ppu.t);
}

#[test]
fn ppuscroll_and_ppuaddr_share_the_write_toggle() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2005, 0x08, &mut mapper);
    ppu.write_register(0x2006, 0x34, &mut mapper); // second write: low byte
    assert_eq!(ppu.t & 0x00FF, 0x34);
    assert!(!ppu.w);
}

#[test]
fn ppuscroll_does_not_change_v() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    set_address(&mut ppu, &mut mapper, 0x2108);
    ppu.write_register(0x2005, 0xFF, &mut mapper);
    ppu.write_register(0x2005, 0xFF, &mut mapper);
    assert_eq!(ppu.v, 0x2108);
}

#[test]
fn ppuaddr_second_write_copies_t_to_v() {
    // A mid-frame $2006 write pair is how some games jump the scroll.
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);

    ppu.write_register(0x2000, 0b0000_0010, &mut mapper);
    ppu.write_register(0x2006, 0x04, &mut mapper);
    assert_eq!(
        ppu.t & 0x0C00,
        0x0400,
        "high byte replaces the nametable bits"
    );
    ppu.write_register(0x2006, 0x56, &mut mapper);
    assert_eq!(ppu.v, 0x0456);
}

// v during rendering

#[test]
fn increment_y_steps_fine_y_then_coarse_y() {
    let mut ppu = Ppu::new();

    ppu.v = 0x6000; // fine Y 6
    ppu.increment_y();
    assert_eq!(ppu.v, 0x7000);
    ppu.increment_y();
    assert_eq!(ppu.v, 0x0020, "fine Y wraps, coarse Y + 1");
}

#[test]
fn increment_y_wraps_row_29_into_next_nametable() {
    let mut ppu = Ppu::new();

    ppu.v = 0x7000 | (29 << 5);
    ppu.increment_y();
    assert_eq!(ppu.v, 0x0800, "v = {:#06X}", ppu.v);
}

#[test]
fn increment_y_wraps_row_31_without_switching_nametable() {
    let mut ppu = Ppu::new();

    ppu.v = 0x7000 | (31 << 5);
    ppu.increment_y();
    assert_eq!(ppu.v, 0x0000, "v = {:#06X}", ppu.v);
}

#[test]
fn horizontal_bits_reload_from_t_at_dot_257() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    enable_background(&mut ppu, &mut mapper);

    tick_to_with(&mut ppu, &mapper, 5, 257);
    ppu.t = 0x041F;
    ppu.v = 0x0000;
    ppu.tick(&mapper);
    assert_eq!(ppu.v, 0x041F, "v = {:#06X}", ppu.v);
}

#[test]
fn vertical_bits_reload_from_t_on_pre_render_line() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    enable_background(&mut ppu, &mut mapper);

    tick_to_with(&mut ppu, &mapper, 261, 279);
    ppu.t = 0x7BE0;
    ppu.v = 0x0000;
    tick_to_with(&mut ppu, &mapper, 261, 305);
    assert_eq!(ppu.v, 0x7BE0, "v = {:#06X}", ppu.v);
}

#[test]
fn v_is_not_touched_while_rendering_is_disabled() {
    let mut ppu = Ppu::new();
    let mapper = TestMapper::new(Mirroring::Horizontal);
    ppu.v = 0x2108;
    ppu.t = 0x7FFF;

    tick_to_with(&mut ppu, &mapper, 261, 340);
    assert_eq!(ppu.v, 0x2108);
}

// Scrolled output

fn set_scroll(ppu: &mut Ppu, mapper: &mut TestMapper, x: u8, y: u8) {
    ppu.write_register(0x2005, x, mapper);
    ppu.write_register(0x2005, y, mapper);
}

#[test]
fn fine_x_scroll_shifts_pixels_left() {
    // Tile 1 row 0 is 0,1,2,3,0,1,2,3. Scrolling 3 pixels starts at color 3.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2000, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    set_scroll(&mut ppu, &mut mapper, 3, 0);

    render_second_frame(&mut ppu, &mapper);
    let row: Vec<u8> = (0..5).map(|x| pixel(&ppu, x, 0)).collect();
    assert_eq!(row, [0x03, 0x0F, 0x01, 0x02, 0x03]);
    assert_eq!(pixel(&ppu, 5, 0), 0x0F, "tile 1 ended");
}

#[test]
fn coarse_x_scroll_moves_tiles_left() {
    // Tile at column 5 appears at column 3 with a 16-pixel scroll.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2005, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    set_scroll(&mut ppu, &mut mapper, 16, 0);

    render_second_frame(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 25, 0), 0x01);
}

#[test]
fn x_scroll_continues_into_next_nametable() {
    // Vertical mirroring: $2400 is the table to the right. Scrolling 8 pixels
    // brings its first tile into the last column.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    mapper.mirroring = Mirroring::Vertical;
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2400, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    set_scroll(&mut ppu, &mut mapper, 8, 0);

    render_second_frame(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 249, 0), 0x01);
}

#[test]
fn fine_x_scroll_reveals_part_of_tile_33() {
    // With fine X 4, the last 4 screen pixels come from the 33rd tile.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    mapper.mirroring = Mirroring::Vertical;
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2400, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    set_scroll(&mut ppu, &mut mapper, 4, 0);

    render_second_frame(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 253, 0), 0x01);
}

#[test]
fn y_scroll_moves_rows_up() {
    // Scrolling down 10 pixels: tile row 1, pixel row 2 lands on screen line 0.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2020, 0x01, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    set_scroll(&mut ppu, &mut mapper, 0, 10);

    render_second_frame(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 0, 0), 0x03, "tile row 2 is all color 3");
    assert_eq!(pixel(&ppu, 0, 6), 0x0F, "tile ended");
}

#[test]
fn scrolled_attributes_follow_the_tile() {
    // Tile at column 4 uses attribute byte $23C1; with a 16-pixel scroll it's drawn
    // at column 2 but must keep palette 3.
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    ppu.write(0x2004, 0x01, &mut mapper);
    ppu.write(0x23C1, 0b0000_0011, &mut mapper);
    enable_background(&mut ppu, &mut mapper);
    set_scroll(&mut ppu, &mut mapper, 16, 0);

    render_second_frame(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 17, 0), 0x31);
}

#[test]
fn mid_frame_scroll_change_splits_the_screen() {
    // SMB1's pattern: status bar unscrolled, then a new X scroll after sprite-0 hit.
    // Lines 24 and 32 are both pixel row 0 of a tile (0,1,2,3,...).
    let mut ppu = Ppu::new();
    let mut mapper = mapper_with_test_tile(0);
    set_palettes(&mut ppu, &mut mapper);
    for row in 0..30 {
        ppu.write(0x2000 + row * 32, 0x01, &mut mapper);
    }
    enable_background(&mut ppu, &mut mapper);
    render_second_frame(&mut ppu, &mapper);

    tick_to_with(&mut ppu, &mapper, 0, 0);
    tick_to_with(&mut ppu, &mapper, 32, 0);
    set_scroll(&mut ppu, &mut mapper, 2, 0);
    render(&mut ppu, &mapper);
    assert_eq!(pixel(&ppu, 1, 24), 0x01, "above the split: unscrolled");
    assert_eq!(pixel(&ppu, 1, 32), 0x03, "below the split: scrolled by 2");
}

// Mapper scanline clock

fn count_scanline_clocks(ppu: &mut Ppu, mapper: &TestMapper) -> u32 {
    let mut clocks = 0;
    for _ in 0..341 * 262 {
        ppu.tick(mapper);
        clocks += u32::from(ppu.take_scanline_clock());
    }
    clocks
}

#[test]
fn scanline_clock_fires_on_visible_and_pre_render_lines() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    enable_background(&mut ppu, &mut mapper);
    assert_eq!(count_scanline_clocks(&mut ppu, &mapper), 241);
}

#[test]
fn scanline_clock_fires_at_dot_260() {
    let mut ppu = Ppu::new();
    let mut mapper = TestMapper::new(Mirroring::Horizontal);
    enable_background(&mut ppu, &mut mapper);
    tick_to_with(&mut ppu, &mapper, 10, 260);
    ppu.take_scanline_clock(); // from earlier lines
    tick_to_with(&mut ppu, &mapper, 11, 260);
    assert!(ppu.take_scanline_clock(), "line 10");
    assert!(!ppu.take_scanline_clock());
    ppu.tick(&mapper);
    assert!(ppu.take_scanline_clock());
    assert!(!ppu.take_scanline_clock(), "taking clears it");
}

#[test]
fn no_scanline_clock_while_rendering_is_disabled() {
    let mut ppu = Ppu::new();
    let mapper = TestMapper::new(Mirroring::Horizontal);
    assert_eq!(count_scanline_clocks(&mut ppu, &mapper), 0);
}
