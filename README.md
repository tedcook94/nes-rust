# nes-rust

A NES emulator written in Rust as a learning project. It runs *Super Mario Bros.* and *Super Mario Bros. 3* with full sound.

## Running

### From a release

1. Download the archive for your platform from the [Releases](https://github.com/tedcook94/nes-rust/releases) page and extract it.
2. Run `nes` from a terminal with the path to an iNES (`.nes`) ROM:

   ```sh
   # Linux and macOS
   ./nes path/to/game.nes

   # Windows (PowerShell or Command Prompt)
   .\nes.exe path\to\game.nes
   ```

   On Windows you can also drag a `.nes` file onto `nes.exe`.

The macOS binaries aren't signed, so macOS blocks them the first time. To allow them, run this in the extracted folder:

```sh
xattr -d com.apple.quarantine nes chr_viewer
```

### From source

```sh
cargo run --release -- path/to/game.nes
```

On Linux, building needs the ALSA development package (`libasound2-dev` on Debian/Ubuntu, `alsa-lib-devel` on Fedora).

A pattern table viewer is also included (`chr_viewer` in the release archives):

```sh
cargo run --release --bin chr_viewer -- path/to/game.nes
```

### Controls

| NES | Keyboard |
|---|---|
| D-pad | Arrow keys |
| A | X |
| B | Z |
| Start | Enter |
| Select | Right Shift |
| Quit | Escape |

## What's supported

- **CPU:** all 151 official 6502 opcodes with cycle counts, NMI, and IRQ. Verified against the `nestest` reference log.
- **PPU:** background and sprites (8×8 and 8×16), scrolling with mid-frame splits, sprite-0 hit, left-edge masking, OAM DMA, and NTSC frame timing.
- **APU:** both pulse channels (envelope, sweep), triangle, noise, and DMC sample playback with its IRQ, the frame counter, and the nonlinear mixer.
- **Mappers:** 0 (NROM) and 4 (MMC3, including the scanline IRQ).
- **Cartridges:** iNES format, with CHR ROM or CHR RAM, and 8KB PRG RAM on MMC3.
- **Frontend:** a 2× scaled window, keyboard input for one controller, and audio output. Frames are paced at the NES's 60.1 fps, with the audio sample rate adjusted slightly to stay in sync with the sound card.

## Limitations

- **Mappers:** only 0 and 4. Games on MMC1 (*Zelda*, *Metroid*), UxROM (*Mega Man*), and others won't load.
- **Rendering is per scanline,** not per dot. Mid-line register changes aren't visible, and MMC3 scanlines are clocked at a fixed dot instead of watching PPU address line A12.
- **Missing PPU details:** the 8-sprites-per-line limit and sprite overflow flag, and the odd-frame dot skip.
- **Missing APU details:** the frame counter IRQ. DMC fetches always stall the CPU 4 cycles.
- **CPU:** no unofficial opcodes, and the one-instruction delay when `CLI`, `SEI`, or `PLP` change the interrupt flag isn't modeled.
- **No PAL support, second controller, save states, battery saves, configurable keys, or gamepad input.**

## Development

```sh
cargo test                     # unit tests plus the nestest CPU trace
cargo clippy --all-targets
```

The code is split by hardware component: `cpu`, `ppu`, `apu`, `cartridge` (with one module per mapper), `bus` (the CPU memory map), and `console`, which ties them together. `src/main.rs` is the windowed frontend.

## References

- [NESdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- [6502 instruction reference](https://www.nesdev.org/obelisk-6502-guide/reference.html)
