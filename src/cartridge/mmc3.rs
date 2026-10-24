use super::{Mapper, Mirroring};

const PRG_BANK: usize = 0x2000;
const CHR_BANK: usize = 0x0400;

/// Mapper 4. PRG is switched in 8KB banks, CHR in 1KB banks.
pub(super) struct Mmc3 {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
    chr_is_ram: bool,
    prg_ram: [u8; 0x2000],
    /// R0-R7, selected by the low 3 bits of $8000.
    registers: [u8; 8],
    selected: usize,
    /// $8000 bit 6: swaps which of $8000 and $C000 is switchable.
    prg_mode: bool,
    /// $8000 bit 7: swaps the 2KB and 1KB CHR halves.
    chr_inversion: bool,
    mirroring: Mirroring,
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mmc3 {
    pub(super) fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let chr_is_ram = chr_rom.is_empty();
        Mmc3 {
            prg_rom,
            chr: if chr_is_ram { vec![0; 0x2000] } else { chr_rom },
            chr_is_ram,
            prg_ram: [0; 0x2000],
            registers: [0; 8],
            selected: 0,
            prg_mode: false,
            chr_inversion: false,
            mirroring,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    fn prg_bank_count(&self) -> usize {
        self.prg_rom.len() / PRG_BANK
    }

    /// Bank number for each 8KB slot at $8000, $A000, $C000, $E000.
    fn prg_bank(&self, slot: usize) -> usize {
        let second_last = self.prg_bank_count() - 2;
        let r6 = usize::from(self.registers[6]);
        let r7 = usize::from(self.registers[7]);
        match (slot, self.prg_mode) {
            (0, false) | (2, true) => r6,
            (0, true) | (2, false) => second_last,
            (1, _) => r7,
            _ => self.prg_bank_count() - 1,
        }
    }

    /// Bank number for each 1KB slot at $0000, $0400, ..., $1C00.
    fn chr_bank(&self, slot: usize) -> usize {
        // Inversion swaps the two 4KB halves.
        let slot = if self.chr_inversion { slot ^ 4 } else { slot };
        let r = |i: usize| usize::from(self.registers[i]);
        match slot {
            // R0 and R1 are 2KB banks; the low bit is ignored.
            0 => r(0) & !1,
            1 => r(0) | 1,
            2 => r(1) & !1,
            3 => r(1) | 1,
            _ => r(slot - 2),
        }
    }

    fn chr_index(&self, address: u16) -> usize {
        let address = usize::from(address & 0x1FFF);
        let bank = self.chr_bank(address / CHR_BANK);
        (bank * CHR_BANK + address % CHR_BANK) % self.chr.len()
    }
}

impl Mapper for Mmc3 {
    fn cpu_read(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => self.prg_ram[usize::from(address - 0x6000)],
            0x8000..=0xFFFF => {
                let offset = usize::from(address - 0x8000);
                let bank = self.prg_bank(offset / PRG_BANK) % self.prg_bank_count();
                self.prg_rom[bank * PRG_BANK + offset % PRG_BANK]
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        let even = address & 1 == 0;
        match address {
            // Write protection ($A001) is ignored; SMB3 never relies on it.
            0x6000..=0x7FFF => self.prg_ram[usize::from(address - 0x6000)] = value,
            0x8000..=0x9FFF if even => {
                self.selected = usize::from(value & 0x07);
                self.prg_mode = value & 0x40 != 0;
                self.chr_inversion = value & 0x80 != 0;
            }
            0x8000..=0x9FFF => self.registers[self.selected] = value,
            0xA000..=0xBFFF if even => {
                self.mirroring = if value & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
            }
            0xC000..=0xDFFF if even => self.irq_latch = value,
            0xC000..=0xDFFF => self.irq_reload = true,
            // Disabling also acknowledges a pending IRQ.
            0xE000..=0xFFFF if even => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            0xE000..=0xFFFF => self.irq_enabled = true,
            _ => {}
        }
    }

    fn ppu_read(&self, address: u16) -> u8 {
        self.chr[self.chr_index(address)]
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        if self.chr_is_ram {
            let index = self.chr_index(address);
            self.chr[index] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn clock_scanline(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }
}

#[cfg(test)]
mod tests;
