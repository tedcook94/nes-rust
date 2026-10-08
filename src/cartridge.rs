#[derive(Debug, PartialEq)]
enum RomError {
    BadMagic,
    Trainer,
    Truncated,
}

#[derive(Debug, PartialEq)]
enum Mirroring {
    Horizontal,
    Vertical,
}

struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8,
    mirroring: Mirroring,
}

impl Cartridge {
    fn parse(bytes: &[u8]) -> Result<Cartridge, RomError> {
        if bytes.len() < 16 {
            return Err(RomError::Truncated);
        }
        if bytes[0..4] != *b"NES\x1A" {
            return Err(RomError::BadMagic);
        }

        let prg_units = bytes[4];
        let chr_units = bytes[5];
        let flags_6 = bytes[6];
        let flags_7 = bytes[7];

        if flags_6 & 0b0000_0100 != 0 {
            return Err(RomError::Trainer);
        }

        let prg_len = usize::from(prg_units) * 0x4000; // 16KB per unit
        let chr_len = usize::from(chr_units) * 0x2000; // 8KB per unit

        if bytes.len() < (16 + prg_len + chr_len) {
            return Err(RomError::Truncated);
        }

        let chr_start = 16 + prg_len;
        let mapper = (flags_7 & 0b1111_0000) | (flags_6 >> 4); // top 4 bits of 7 are high nibble, top 4 of 6 are low

        Ok(Cartridge {
            prg_rom: bytes[16..chr_start].to_vec(),
            chr_rom: bytes[chr_start..chr_start + chr_len].to_vec(),
            mapper,
            mirroring: if flags_6 & 0x01 != 0 {
                Mirroring::Vertical
            } else {
                Mirroring::Horizontal
            },
        })
    }
}

#[cfg(test)]
mod tests;
