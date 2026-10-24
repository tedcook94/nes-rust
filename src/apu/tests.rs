// Bit groups in literals follow register fields, e.g. 0b10_1_1_0111 is duty, halt, constant, volume.
#![allow(clippy::unusual_byte_groupings)]

use super::*;
use pulse::LENGTH_TABLE;

/// Sets pulse 1 to the given duty, constant volume 15, timer period 8, and a long note.
fn playing_pulse(apu: &mut Apu, duty: u8) {
    apu.write_register(0x4000, (duty << 6) | 0x1F);
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
    assert!(apu.pulse1.envelope.looping);
    assert!(apu.pulse1.envelope.constant);
    assert_eq!(apu.pulse1.envelope.value, 7);
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
    apu.write_register(0x4004, 0b01_0_1_0011);
    apu.write_register(0x4006, 0x34);
    apu.write_register(0x4007, 0x0A);
    assert_eq!(apu.pulse2.duty, 1);
    assert_eq!(apu.pulse2.envelope.value, 3);
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
    apu.write_register(0x4000, 0x30);
    apu.write_register(0x4003, 0x08);
    apu.pulse1.clock_length();
    assert_eq!(apu.pulse1.length, 254);
}

#[test]
fn output_is_volume_when_waveform_is_high() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 3);
    apu.write_register(0x4000, 0b11_0_1_1001);
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

fn tick_n(apu: &mut Apu, cycles: u32) {
    for _ in 0..cycles {
        apu.tick();
    }
}

// Envelope

#[test]
fn envelope_start_sets_decay_to_15() {
    let mut apu = Apu::new();
    apu.write_register(0x4000, 0x02); // decay mode, period 2
    apu.write_register(0x4003, 0x08);
    assert!(apu.pulse1.envelope.start);
    apu.pulse1.envelope.clock();
    assert!(!apu.pulse1.envelope.start);
    assert_eq!(apu.pulse1.envelope.decay, 15);
}

#[test]
fn envelope_decays_every_period_plus_one_clocks() {
    let mut apu = Apu::new();
    apu.write_register(0x4000, 0x02);
    apu.write_register(0x4003, 0x08);
    let envelope = &mut apu.pulse1.envelope;
    envelope.clock();
    let decays: Vec<u8> = (0..6)
        .map(|_| {
            envelope.clock();
            envelope.decay
        })
        .collect();
    assert_eq!(decays, [15, 15, 14, 14, 14, 13]);
}

#[test]
fn envelope_stops_at_zero_without_loop() {
    let mut apu = Apu::new();
    apu.write_register(0x4000, 0x00);
    apu.write_register(0x4003, 0x08);
    for _ in 0..20 {
        apu.pulse1.envelope.clock();
    }
    assert_eq!(apu.pulse1.envelope.decay, 0);
}

#[test]
fn envelope_loop_wraps_zero_to_15() {
    let mut apu = Apu::new();
    apu.write_register(0x4000, 0x20);
    apu.write_register(0x4003, 0x08);
    for _ in 0..16 {
        apu.pulse1.envelope.clock();
    }
    assert_eq!(apu.pulse1.envelope.decay, 0);
    apu.pulse1.envelope.clock();
    assert_eq!(apu.pulse1.envelope.decay, 15);
}

#[test]
fn output_uses_decay_level_when_not_constant() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 3);
    apu.write_register(0x4000, 0b11_0_0_0000);
    apu.pulse1.envelope.clock();
    apu.pulse1.envelope.clock();
    assert_eq!(apu.pulse1.output(), 14);
}

// Sweep

#[test]
fn register_1_sets_sweep_fields() {
    let mut apu = Apu::new();
    apu.write_register(0x4001, 0b1_101_1_011);
    let sweep = &apu.pulse1.sweep;
    assert!(sweep.enabled);
    assert_eq!(sweep.period, 5);
    assert!(sweep.negate);
    assert_eq!(sweep.shift, 3);
    assert!(sweep.reload);
}

#[test]
fn sweep_target_adds_shifted_period() {
    let mut apu = Apu::new();
    apu.write_register(0x4002, 0x00);
    apu.write_register(0x4003, 0x01); // period $100
    apu.write_register(0x4001, 0b1_000_0_010);
    assert_eq!(apu.pulse1.sweep_target(), 0x140);
}

#[test]
fn sweep_negate_differs_between_pulse_channels() {
    let mut apu = Apu::new();
    apu.write_register(0x4003, 0x01);
    apu.write_register(0x4001, 0b1_000_1_010);
    apu.write_register(0x4007, 0x01);
    apu.write_register(0x4005, 0b1_000_1_010);
    assert_eq!(apu.pulse1.sweep_target(), 0xBF, "ones' complement");
    assert_eq!(apu.pulse2.sweep_target(), 0xC0, "two's complement");
}

#[test]
fn sweep_updates_period_when_divider_reaches_zero() {
    let mut apu = Apu::new();
    apu.write_register(0x4003, 0x01);
    apu.write_register(0x4001, 0b1_001_0_001); // period 1, shift 1
    apu.pulse1.clock_sweep(); // reload, divider 0 so it also sweeps
    assert_eq!(apu.pulse1.timer_period, 0x180);
    apu.pulse1.clock_sweep();
    assert_eq!(apu.pulse1.timer_period, 0x180, "divider was reloaded to 1");
    apu.pulse1.clock_sweep();
    assert_eq!(apu.pulse1.timer_period, 0x240);
}

#[test]
fn sweep_does_nothing_with_shift_0_or_disabled() {
    let mut apu = Apu::new();
    apu.write_register(0x4003, 0x01);
    apu.write_register(0x4001, 0b1_000_0_000);
    apu.pulse1.clock_sweep();
    assert_eq!(apu.pulse1.timer_period, 0x100);
    apu.write_register(0x4001, 0b0_000_0_001);
    apu.pulse1.clock_sweep();
    assert_eq!(apu.pulse1.timer_period, 0x100);
}

#[test]
fn sweep_target_over_7ff_mutes_even_when_disabled() {
    let mut apu = Apu::new();
    playing_pulse(&mut apu, 3);
    apu.write_register(0x4002, 0xFF);
    apu.write_register(0x4003, 0x0D); // period $5FF, length index 1
    apu.write_register(0x4001, 0b0_000_0_010);
    assert_eq!(apu.pulse1.output(), 15, "target $77E");
    apu.write_register(0x4001, 0b0_000_0_001);
    assert_eq!(apu.pulse1.output(), 0, "target $8FE");
    apu.write_register(0x4001, 0b0_000_0_000);
    assert_eq!(apu.pulse1.output(), 0, "shift 0 still mutes: target $BFE");
}

// Frame counter

#[test]
fn four_step_mode_clocks_envelopes_each_quarter() {
    let mut apu = Apu::new();
    apu.write_register(0x4000, 0x00); // decay period 0: one step per clock
    apu.write_register(0x4003, 0x08);
    tick_n(&mut apu, STEP_1 - 1);
    assert!(apu.pulse1.envelope.start);
    tick_n(&mut apu, 1);
    assert_eq!(apu.pulse1.envelope.decay, 15);
    tick_n(&mut apu, STEP_2 - STEP_1);
    assert_eq!(apu.pulse1.envelope.decay, 14);
    tick_n(&mut apu, STEP_3 - STEP_2);
    assert_eq!(apu.pulse1.envelope.decay, 13);
    tick_n(&mut apu, FOUR_STEP_END - STEP_3);
    assert_eq!(apu.pulse1.envelope.decay, 12);
    tick_n(&mut apu, STEP_1);
    assert_eq!(apu.pulse1.envelope.decay, 11, "sequence restarts");
}

#[test]
fn four_step_mode_clocks_length_on_steps_2_and_4() {
    let mut apu = Apu::new();
    apu.write_register(0x4003, 0x08); // 254
    tick_n(&mut apu, STEP_2 - 1);
    assert_eq!(apu.pulse1.length, 254);
    tick_n(&mut apu, 1);
    assert_eq!(apu.pulse1.length, 253);
    tick_n(&mut apu, FOUR_STEP_END - STEP_2);
    assert_eq!(apu.pulse1.length, 252);
}

#[test]
fn five_step_mode_has_a_longer_sequence() {
    let mut apu = Apu::new();
    apu.write_register(0x4003, 0x08);
    apu.write_register(0x4017, 0x80);
    assert_eq!(
        apu.pulse1.length, 253,
        "writing 5-step mode clocks immediately"
    );
    tick_n(&mut apu, STEP_2);
    assert_eq!(apu.pulse1.length, 252);
    tick_n(&mut apu, FOUR_STEP_END - STEP_2);
    assert_eq!(apu.pulse1.length, 252, "nothing at the 4-step end");
    tick_n(&mut apu, FIVE_STEP_END - FOUR_STEP_END);
    assert_eq!(apu.pulse1.length, 251);
}

#[test]
fn writing_4017_resets_the_sequence() {
    let mut apu = Apu::new();
    apu.write_register(0x4003, 0x08);
    tick_n(&mut apu, STEP_2 - 10);
    apu.write_register(0x4017, 0x00);
    tick_n(&mut apu, 10);
    assert_eq!(apu.pulse1.length, 254);
}
