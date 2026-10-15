use super::*;
use crate::cartridge::Mirroring;

// PRG where each byte holds the low byte of its own offset, plus markers at
// both ends so mirroring and linear mapping are easy to tell apart.
fn prg(len: usize) -> Vec<u8> {
    let mut prg: Vec<u8> = (0..len).map(|i| i as u8).collect();
    prg[0] = 0x11; // first byte
    prg[len - 1] = 0x22; // last byte
    prg
}

#[test]
fn reads_32kb_prg_linearly() {
    let nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Horizontal);

    assert_eq!(nrom.cpu_read(0x8000), 0x11);
    assert_eq!(nrom.cpu_read(0x8001), 0x01);
    assert_eq!(nrom.cpu_read(0xC000), 0x00); // offset $4000: low byte is $00
    assert_eq!(nrom.cpu_read(0xC001), 0x01);
    assert_eq!(nrom.cpu_read(0xFFFF), 0x22);
}

#[test]
fn second_half_of_32kb_prg_is_not_mirrored() {
    // Byte $4000 is distinct from byte $0000, so a mirroring bug shows up here.
    let mut data = prg(0x8000);
    data[0x4000] = 0x33;
    let nrom = Nrom::new(data, chr(), Mirroring::Horizontal);

    assert_eq!(nrom.cpu_read(0xC000), 0x33);
    assert_eq!(nrom.cpu_read(0x8000), 0x11);
}

#[test]
fn mirrors_16kb_prg() {
    // $8000-$BFFF and $C000-$FFFF both map to the same 16KB.
    let nrom = Nrom::new(prg(0x4000), chr(), Mirroring::Horizontal);

    assert_eq!(nrom.cpu_read(0x8000), 0x11);
    assert_eq!(nrom.cpu_read(0xC000), 0x11);
    assert_eq!(nrom.cpu_read(0xBFFF), 0x22);
    assert_eq!(nrom.cpu_read(0xFFFF), 0x22);
    assert_eq!(nrom.cpu_read(0x9234), nrom.cpu_read(0xD234));
}

#[test]
fn reset_vector_of_16kb_prg_comes_from_end_of_bank() {
    // nestest is 16KB: its vectors at $FFFA-$FFFF live at PRG $3FFA-$3FFF.
    let mut data = prg(0x4000);
    data[0x3FFC] = 0x00;
    data[0x3FFD] = 0xC0;
    let nrom = Nrom::new(data, chr(), Mirroring::Horizontal);

    assert_eq!(nrom.cpu_read(0xFFFC), 0x00);
    assert_eq!(nrom.cpu_read(0xFFFD), 0xC0);
}

#[test]
fn writes_to_prg_are_ignored() {
    let mut nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Horizontal);

    nrom.cpu_write(0x8000, 0x99);
    nrom.cpu_write(0xFFFF, 0x99);
    assert_eq!(nrom.cpu_read(0x8000), 0x11);
    assert_eq!(nrom.cpu_read(0xFFFF), 0x22);
}

#[test]
fn reads_below_prg_return_zero() {
    // $4020-$7FFF: NROM has no PRG-RAM or expansion hardware.
    let nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Horizontal);

    assert_eq!(nrom.cpu_read(0x4020), 0x00);
    assert_eq!(nrom.cpu_read(0x6000), 0x00);
    assert_eq!(nrom.cpu_read(0x7FFF), 0x00);
}

// CHR (PPU side)

// CHR where each byte holds the low byte of its own offset, with markers at both ends.
fn chr() -> Vec<u8> {
    let mut chr: Vec<u8> = (0..0x2000).map(|i| i as u8).collect();
    chr[0] = 0x33; // first byte
    chr[0x1FFF] = 0x44; // last byte
    chr
}

#[test]
fn ppu_reads_chr_rom() {
    let nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Horizontal);

    assert_eq!(nrom.ppu_read(0x0000), 0x33);
    assert_eq!(nrom.ppu_read(0x0001), 0x01);
    assert_eq!(nrom.ppu_read(0x1000), 0x00); // offset $1000: low byte is $00
    assert_eq!(nrom.ppu_read(0x1FFF), 0x44);
}

#[test]
fn ppu_and_cpu_reads_use_separate_data() {
    // CHR at PPU $0000 and PRG at CPU $8000 hold different markers.
    let nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Horizontal);

    assert_eq!(nrom.ppu_read(0x0000), 0x33);
    assert_eq!(nrom.cpu_read(0x8000), 0x11);
}

#[test]
fn ppu_writes_to_chr_rom_are_ignored() {
    let mut nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Horizontal);

    nrom.ppu_write(0x0000, 0x99);
    assert_eq!(nrom.ppu_read(0x0000), 0x33);
}

#[test]
fn chr_ram_when_cartridge_has_no_chr_rom() {
    // An empty CHR from the header means 8KB of writable CHR-RAM.
    let mut nrom = Nrom::new(prg(0x8000), Vec::new(), Mirroring::Horizontal);

    assert_eq!(nrom.ppu_read(0x0000), 0x00);
    nrom.ppu_write(0x0000, 0x99);
    nrom.ppu_write(0x1FFF, 0x77);
    assert_eq!(nrom.ppu_read(0x0000), 0x99);
    assert_eq!(nrom.ppu_read(0x1FFF), 0x77);
}

// Mirroring

#[test]
fn reports_horizontal_mirroring() {
    let nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Horizontal);

    assert_eq!(nrom.mirroring(), Mirroring::Horizontal);
}

#[test]
fn reports_vertical_mirroring() {
    let nrom = Nrom::new(prg(0x8000), chr(), Mirroring::Vertical);

    assert_eq!(nrom.mirroring(), Mirroring::Vertical);
}
