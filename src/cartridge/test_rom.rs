//! Builders for in-memory iNES files, shared by tests across modules.

pub const PRG_UNIT: usize = 0x4000; // 16KB
pub const CHR_UNIT: usize = 0x2000; // 8KB

/// Builds an iNES file in memory. PRG bytes are filled with 0xAA, CHR with 0xBB.
pub fn build_rom(prg_units: u8, chr_units: u8, flags6: u8, flags7: u8) -> Vec<u8> {
    let mut rom = vec![b'N', b'E', b'S', 0x1A, prg_units, chr_units, flags6, flags7];
    rom.resize(16, 0);
    rom.resize(16 + usize::from(prg_units) * PRG_UNIT, 0xAA);
    rom.resize(rom.len() + usize::from(chr_units) * CHR_UNIT, 0xBB);
    rom
}

/// Builds a 16KB NROM image with `program` at $C000 and the reset vector
/// pointing there. Unused PRG is filled with NOP ($EA).
pub fn build_nrom_with_program(program: &[u8]) -> Vec<u8> {
    let mut rom = build_rom(1, 1, 0, 0);
    let prg = &mut rom[16..16 + PRG_UNIT];
    prg.fill(0xEA);
    prg[..program.len()].copy_from_slice(program); // $C000 mirrors PRG offset 0
    prg[0x3FFC] = 0x00; // reset vector → $C000
    prg[0x3FFD] = 0xC0;
    rom
}
