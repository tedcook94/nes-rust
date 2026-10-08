use super::*;

const PRG_UNIT: usize = 0x4000; // 16KB
const CHR_UNIT: usize = 0x2000; // 8KB

// Builds an iNES file in memory. PRG bytes are filled with 0xAA, CHR with 0xBB.
fn build_rom(prg_units: u8, chr_units: u8, flags6: u8, flags7: u8) -> Vec<u8> {
    let mut rom = vec![b'N', b'E', b'S', 0x1A, prg_units, chr_units, flags6, flags7];
    rom.resize(16, 0);
    rom.resize(16 + usize::from(prg_units) * PRG_UNIT, 0xAA);
    rom.resize(rom.len() + usize::from(chr_units) * CHR_UNIT, 0xBB);
    rom
}

#[test]
fn parses_prg_and_chr_sizes() {
    // SMB1 layout: 32KB PRG, 8KB CHR.
    let cartridge = Cartridge::parse(&build_rom(2, 1, 0, 0)).unwrap();

    assert_eq!(cartridge.prg_rom.len(), 2 * PRG_UNIT);
    assert_eq!(cartridge.chr_rom.len(), CHR_UNIT);
}

#[test]
fn slices_prg_and_chr_at_correct_offsets() {
    let mut rom = build_rom(1, 1, 0, 0);
    rom[16] = 0x11; // first PRG byte
    rom[16 + PRG_UNIT - 1] = 0x22; // last PRG byte
    rom[16 + PRG_UNIT] = 0x33; // first CHR byte
    let last = rom.len() - 1;
    rom[last] = 0x44; // last CHR byte

    let cartridge = Cartridge::parse(&rom).unwrap();
    assert_eq!(cartridge.prg_rom[0], 0x11);
    assert_eq!(cartridge.prg_rom[PRG_UNIT - 1], 0x22);
    assert_eq!(cartridge.chr_rom[0], 0x33);
    assert_eq!(cartridge.chr_rom[CHR_UNIT - 1], 0x44);
}

#[test]
fn zero_chr_units_means_no_chr_rom() {
    // CHR size 0 means the cartridge uses CHR-RAM instead.
    let cartridge = Cartridge::parse(&build_rom(1, 0, 0, 0)).unwrap();

    assert!(cartridge.chr_rom.is_empty());
}

#[test]
fn parses_mapper_from_both_nibbles() {
    // Low nibble from flags 6 bits 4-7, high nibble from flags 7 bits 4-7.
    // Low bits of each flag byte are set as decoys.
    let cartridge = Cartridge::parse(&build_rom(1, 1, 0x1B, 0x4F)).unwrap();

    assert_eq!(cartridge.mapper, 0x41);
}

#[test]
fn parses_mapper_zero() {
    let cartridge = Cartridge::parse(&build_rom(1, 1, 0x01, 0x00)).unwrap();

    assert_eq!(cartridge.mapper, 0);
}

#[test]
fn parses_horizontal_mirroring() {
    let cartridge = Cartridge::parse(&build_rom(1, 1, 0x00, 0)).unwrap();

    assert_eq!(cartridge.mirroring, Mirroring::Horizontal);
}

#[test]
fn parses_vertical_mirroring() {
    // Other flag-6 bits are set as decoys; only bit 0 matters.
    let cartridge = Cartridge::parse(&build_rom(1, 1, 0xF1, 0)).unwrap();

    assert_eq!(cartridge.mirroring, Mirroring::Vertical);
}

#[test]
fn rejects_bad_magic() {
    let mut rom = build_rom(1, 1, 0, 0);
    rom[3] = 0x00;

    assert_eq!(Cartridge::parse(&rom).err(), Some(RomError::BadMagic));
}

#[test]
fn rejects_file_shorter_than_header() {
    assert_eq!(
        Cartridge::parse(&[b'N', b'E', b'S', 0x1A]).err(),
        Some(RomError::Truncated)
    );
}

#[test]
fn rejects_file_shorter_than_declared_prg() {
    let mut rom = build_rom(2, 0, 0, 0);
    rom.truncate(16 + PRG_UNIT);

    assert_eq!(Cartridge::parse(&rom).err(), Some(RomError::Truncated));
}

#[test]
fn rejects_file_shorter_than_declared_chr() {
    let mut rom = build_rom(1, 1, 0, 0);
    rom.pop();

    assert_eq!(Cartridge::parse(&rom).err(), Some(RomError::Truncated));
}

#[test]
fn rejects_trainer() {
    // Flag 6 bit 2: a 512-byte trainer precedes PRG. Rare; not supported.
    let cartridge = Cartridge::parse(&build_rom(1, 1, 0x04, 0));

    assert_eq!(cartridge.err(), Some(RomError::Trainer));
}
