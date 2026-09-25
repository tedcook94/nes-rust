# nes-rust

A NES emulator written in Rust as a learning project.

## Milestones

- [x] **0. Project skeleton**: Cargo project, module layout, `cargo test` passing.
- [ ] **1. CPU (6502 / 2A03)**: Registers, flags, addressing modes, and official opcodes, each with unit tests against a `TestBus`.
- [ ] **2. Bus & memory map**: 2KB RAM with mirroring, PPU and APU/IO register ranges, cartridge space.
- [ ] **3. Cartridge**: iNES header parsing, a `Mapper` trait, and mapper 0 (NROM).
- [ ] **4. nestest**: Compare the CPU trace against the `nestest.nes` reference log.
- [ ] **5. PPU**: Registers, VRAM and palettes, background rendering, sprites and OAM DMA, scrolling, NMI and sprite-0 hit, and frame timing.
- [ ] **6. Frontend & input**: Window, framebuffer, and controller input. **SMB1 playable (silent).**
- [ ] **7. APU**: Pulse, triangle, and noise channels, the frame counter, and audio output. **SMB1 complete.**
- [ ] **8. MMC3 (mapper 4)**: Bank switching and the scanline IRQ. **SMB3 playable.**

## References

- [NESdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- [6502 instruction reference](https://www.nesdev.org/obelisk-6502-guide/reference.html)
