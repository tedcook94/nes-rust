//! Runs a ROM in a window: `cargo run --release -- path/to/game.nes`

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use minifb::{Key, Scale, Window, WindowOptions};
use nes::console::Console;
use nes::controller::button;
use nes::ppu::palette::SYSTEM_PALETTE;
use nes::ppu::{HEIGHT, WIDTH};

fn main() {
    let path = std::env::args().nth(1).expect("usage: nes <rom.nes>");
    let rom = std::fs::read(&path).unwrap_or_else(|e| panic!("can't read {path}: {e}"));
    let mut console = Console::new(&rom).unwrap_or_else(|e| panic!("bad ROM: {e:?}"));

    let queue = Arc::new(Mutex::new(VecDeque::new()));
    // Dropping the stream stops playback, so it has to live as long as the loop.
    let (_stream, sample_rate) = start_audio(Arc::clone(&queue));
    console.set_sample_rate(sample_rate);
    let mut filter = HighPass::default();
    // The window runs at 60 fps but the NES at about 60.1, so samples slowly pile up.
    // Capping the queue bounds the lag to about 3 frames.
    let frame_samples = sample_rate as usize / 60;
    let max_queued = frame_samples * 3;
    let target_queued = frame_samples * 2;

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new(&path, WIDTH, HEIGHT, options)
        .unwrap_or_else(|e| panic!("can't open window: {e}"));
    window.set_target_fps(60);

    // Keyboard → NES buttons. Arrow keys move; Z/X are B/A; Enter/Right Shift are Start/Select.
    let key_map = [
        (Key::X, button::A),
        (Key::Z, button::B),
        (Key::RightShift, button::SELECT),
        (Key::Enter, button::START),
        (Key::Up, button::UP),
        (Key::Down, button::DOWN),
        (Key::Left, button::LEFT),
        (Key::Right, button::RIGHT),
    ];

    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let buttons = key_map
            .iter()
            .filter(|(key, _)| window.is_key_down(*key))
            .fold(0, |held, (_, button)| held | button);
        console.set_buttons(buttons);
        console.step_frame();
        let samples = console.take_samples();
        let mut queued = queue.lock().unwrap();
        queued.extend(samples.into_iter().map(|s| filter.apply(s)));
        trim_queue(&mut queued, max_queued, target_queued);
        drop(queued);
        for (pixel, &value) in buffer.iter_mut().zip(console.frame()) {
            *pixel = SYSTEM_PALETTE[usize::from(value)];
        }
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("window update failed");
    }
}

/// Opens the default output device and plays samples from `queue` until the
/// returned stream is dropped.
fn start_audio(queue: Arc<Mutex<VecDeque<f32>>>) -> (cpal::Stream, u32) {
    let device = cpal::default_host()
        .default_output_device()
        .expect("no audio output device");
    let config: cpal::StreamConfig = device
        .default_output_config()
        .expect("no default audio config")
        .into();
    let channels = usize::from(config.channels);
    let sample_rate = config.sample_rate;

    // Runs on cpal's audio thread whenever the device needs more samples.
    let mut last = 0.0;
    let fill = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut queue = queue.lock().unwrap();
        for frame in data.chunks_mut(channels) {
            // Repeating the last sample on underrun is quieter than dropping to 0.
            last = queue.pop_front().unwrap_or(last);
            frame.fill(last);
        }
    };
    let stream = device
        .build_output_stream(config, fill, |e| eprintln!("audio error: {e}"), None)
        .expect("can't open audio stream");
    stream.play().expect("can't start audio");
    (stream, sample_rate)
}

/// Once `queue` holds more than `max` samples, drops the oldest down to `target`.
fn trim_queue(queue: &mut VecDeque<f32>, max: usize, target: usize) {
    if queue.len() > max {
        queue.drain(..queue.len() - target);
    }
}

/// Removes the DC offset: the mixer outputs 0.0..=1.0, so silence would sit at a
/// constant level instead of 0. The NES itself has a similar filter at about 90 Hz.
#[derive(Default)]
struct HighPass {
    previous_in: f32,
    previous_out: f32,
}

impl HighPass {
    fn apply(&mut self, sample: f32) -> f32 {
        let out = sample - self.previous_in + 0.996 * self.previous_out;
        self.previous_in = sample;
        self.previous_out = out;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_queue_leaves_short_queues_alone() {
        let mut queue: VecDeque<f32> = (0..10).map(|n| n as f32).collect();
        trim_queue(&mut queue, 10, 5);
        assert_eq!(queue.len(), 10);
    }

    #[test]
    fn trim_queue_drops_oldest_down_to_target() {
        let mut queue: VecDeque<f32> = (0..11).map(|n| n as f32).collect();
        trim_queue(&mut queue, 10, 5);
        assert_eq!(queue, [6.0, 7.0, 8.0, 9.0, 10.0]);
    }
}
