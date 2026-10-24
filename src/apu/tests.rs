// Bit groups in literals follow register fields, e.g. 0b10_1_1_0111 is duty, halt, constant, volume.
#![allow(clippy::unusual_byte_groupings)]

use super::*;
use pulse::LENGTH_TABLE;

/// Sets pulse 1 to the given duty, constant volume 15, timer period 8, and a long note.
fn playing_pulse(apu: &mut Apu, duty: u8) {
    apu.write_register(0x4000, (duty << 6) | 0x0F);
    apu.write_register(0x4002, 0x08);
    apu.write_register(0x4003, 0x08); // length index 1: 254
}

/// One sequencer step takes (period + 1) APU cycles, each two CPU cycles.
fn next_step(apu: &mut Apu) {
    for _ in 0..(apu.pulse1.timer_period + 1) * 2 {
        apu.tick();
    }
}

fn waveform(apu: &mut Apu) -> Vec<u8> {
    (0..8)
        .map(|_| {
            next_step(apu);
            apu.pulse1.output() / 15
        })
        .collect()
}

#[test]
fn register_0_sets_duty_halt_and_volume() {
    let mut apu = Apu::new();
    apu.write_register(0x4000, 0b10_1_1_0111);
    assert_eq!(apu.pulse1.duty, 2);
    assert!(apu.pulse1.halt_length);
    assert_eq!(apu.pulse1.volume, 7);
}

#[test]
fn registers_2_and_3_set_the_11_bit_timer_period() {
    let mut apu = Apu::new();
    apu.write_register(0x4002, 0xAB);
    apu.write_register(0x4003, 0b00000_101);
    assert_eq!(apu.pulse1.timer_period, 0x5AB);
}

#[test]
fn register_3_loads_length_from_table() {
    let mut apu = Apu::new();
    apu.write_register(0x4003, 0b00011_000);
    assert_eq!(apu.pulse1.length, LENGTH_TABLE[3]);
    assert_eq!(apu.pulse1.length, 2);
}

#[test]
fn register_3_restarts_the_sequencer() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 2);
    next_step(&mut apu);
    apu.write_register(0x4003, 0x08);
    assert_eq!(apu.pulse1.step, 0);
}

#[test]
fn pulse_2_has_its_own_registers() {
    let mut apu = Apu::new();
    apu.write_register(0x4004, 0b01_0_0_0011);
    apu.write_register(0x4006, 0x34);
    apu.write_register(0x4007, 0x0A);
    assert_eq!(apu.pulse2.duty, 1);
    assert_eq!(apu.pulse2.volume, 3);
    assert_eq!(apu.pulse2.timer_period, 0x234);
    assert_eq!(apu.pulse1.timer_period, 0);
}

#[test]
fn duty_waveforms() {
    // The sequencer counts down from 0, so table entries come out as 7, 6, ..., 0.
    let expected = [
        [0, 0, 0, 0, 0, 0, 1, 0],
        [0, 0, 0, 0, 0, 1, 1, 0],
        [0, 0, 0, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 0, 0, 1],
    ];
    for (duty, wave) in expected.iter().enumerate() {
        let mut apu = Apu::new();
        playing_pulse(&mut apu, duty as u8);
        assert_eq!(waveform(&mut apu), wave, "duty {duty}");
    }
}

#[test]
fn timer_steps_every_period_plus_one_apu_cycles() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 0);
    apu.write_register(0x4002, 20);

    // Timer starts at 0, so the first step happens on the first APU cycle.
    apu.tick();
    apu.tick();
    assert_eq!(apu.pulse1.step, 7);
    for _ in 0..20 * 2 {
        apu.tick();
    }
    assert_eq!(apu.pulse1.step, 7, "one APU cycle short");
    apu.tick();
    apu.tick();
    assert_eq!(apu.pulse1.step, 6);
}

#[test]
fn timer_only_clocks_on_every_other_cpu_cycle() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 0);
    apu.tick();
    assert_eq!(apu.pulse1.step, 0);
    apu.tick();
    assert_eq!(apu.pulse1.step, 7);
}

#[test]
fn length_counter_counts_down_to_zero_and_silences() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 3); // step 0 of duty 3 is high
    apu.write_register(0x4003, 0b00011_000); // length 2
    assert_eq!(apu.pulse1.output(), 15);
    apu.pulse1.clock_length();
    assert_eq!(apu.pulse1.length, 1);
    apu.pulse1.clock_length();
    assert_eq!(apu.pulse1.output(), 0);
    apu.pulse1.clock_length();
    assert_eq!(apu.pulse1.length, 0, "stops at zero");
}

#[test]
fn halt_flag_freezes_length_counter() {
    let mut apu = Apu::new();
    apu.write_register(0x4000, 0x20);
    apu.write_register(0x4003, 0x08);
    apu.pulse1.clock_length();
    assert_eq!(apu.pulse1.length, 254);
}

#[test]
fn output_is_volume_when_waveform_is_high() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 3);
    apu.write_register(0x4000, 0b11_0_0_1001);
    assert_eq!(apu.pulse1.output(), 9);
}

#[test]
fn periods_below_8_are_silenced() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 3);
    apu.write_register(0x4002, 0x07);
    assert_eq!(apu.pulse1.output(), 0);
    apu.write_register(0x4002, 0x08);
    assert_eq!(apu.pulse1.output(), 15);
}
