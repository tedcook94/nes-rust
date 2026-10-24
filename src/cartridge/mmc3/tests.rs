use super::*;

/// 16 PRG banks (128KB) and 64 CHR banks (64KB); each bank is filled with its own number.
fn mmc3() -> Mmc3 {
    let prg = (0..16u8).flat_map(|b| [b; PRG_BANK]).collect();
    let chr = (0..64u8).flat_map(|b| [b; CHR_BANK]).collect();
    Mmc3::new(prg, chr, Mirroring::Horizontal)
}

fn set_register(mapper: &mut Mmc3, register: u8, value: u8) {
    mapper.cpu_write(0x8000, register);
    mapper.cpu_write(0x8001, value);
}

fn prg_slots(mapper: &Mmc3) -> [u8; 4] {
    [0x8000, 0xA000, 0xC000, 0xE000].map(|a| mapper.cpu_read(a))
}

fn chr_slots(mapper: &Mmc3) -> [u8; 8] {
    std::array::from_fn(|i| mapper.ppu_read(i as u16 * 0x400))
}

#[test]
fn prg_mode_0_switches_8000_and_fixes_c000() {
    let mut mapper = mmc3();
    set_register(&mut mapper, 6, 3);
    set_register(&mut mapper, 7, 5);
    assert_eq!(prg_slots(&mapper), [3, 5, 14, 15]);
}

#[test]
fn prg_mode_1_swaps_8000_and_c000() {
    let mut mapper = mmc3();
    set_register(&mut mapper, 6, 3);
    set_register(&mut mapper, 7, 5);
    mapper.cpu_write(0x8000, 0x40);
    assert_eq!(prg_slots(&mapper), [14, 5, 3, 15]);
}

#[test]
fn prg_bank_numbers_wrap_to_rom_size() {
    let mut mapper = mmc3();
    set_register(&mut mapper, 6, 16 + 2);
    assert_eq!(mapper.cpu_read(0x8000), 2);
}

#[test]
fn bank_registers_mirror_across_their_range() {
    let mut mapper = mmc3();
    mapper.cpu_write(0x9FFE, 7);
    mapper.cpu_write(0x9FFF, 9);
    assert_eq!(mapper.cpu_read(0xA000), 9);
}

#[test]
fn chr_mode_0_has_2kb_banks_low() {
    let mut mapper = mmc3();
    for (register, bank) in [(0, 9), (1, 20), (2, 30), (3, 31), (4, 32), (5, 33)] {
        set_register(&mut mapper, register, bank);
    }
    // R0 and R1 ignore their low bit.
    assert_eq!(chr_slots(&mapper), [8, 9, 20, 21, 30, 31, 32, 33]);
}

#[test]
fn chr_inversion_swaps_halves() {
    let mut mapper = mmc3();
    for (register, bank) in [(0, 8), (1, 20), (2, 30), (3, 31), (4, 32), (5, 33)] {
        set_register(&mut mapper, register, bank);
    }
    mapper.cpu_write(0x8000, 0x80);
    assert_eq!(chr_slots(&mapper), [30, 31, 32, 33, 8, 9, 20, 21]);
}

#[test]
fn a000_sets_mirroring() {
    let mut mapper = mmc3();
    mapper.cpu_write(0xA000, 0);
    assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    mapper.cpu_write(0xA000, 1);
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
}

#[test]
fn odd_a000_address_does_not_set_mirroring() {
    let mut mapper = mmc3();
    mapper.cpu_write(0xA001, 0);
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
}

#[test]
fn prg_ram_reads_back() {
    let mut mapper = mmc3();
    mapper.cpu_write(0x6000, 0x12);
    mapper.cpu_write(0x7FFF, 0x34);
    assert_eq!(mapper.cpu_read(0x6000), 0x12);
    assert_eq!(mapper.cpu_read(0x7FFF), 0x34);
}

#[test]
fn chr_ram_is_writable_through_banks() {
    let mut mapper = Mmc3::new(vec![0; 4 * PRG_BANK], Vec::new(), Mirroring::Horizontal);
    set_register(&mut mapper, 2, 3);
    mapper.ppu_write(0x1000, 0x55);
    assert_eq!(mapper.ppu_read(0x1000), 0x55);
    assert_eq!(mapper.ppu_read(0x0000), 0x00);
}
