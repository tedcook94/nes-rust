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
    // Frames are paced by a timer for smooth video. The sound card's clock differs slightly
    // from ours, so the sample rate is nudged to keep about 3 frames of audio queued.
    let target_queued = sample_rate as usize / 60 * 3;
    let frame_period = std::time::Duration::from_secs_f64(1.0 / NES_FPS);
    let mut deadline = std::time::Instant::now();

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new(&path, WIDTH, HEIGHT, options)
        .unwrap_or_else(|e| panic!("can't open window: {e}"));
    window.set_target_fps(0);

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
        let now = std::time::Instant::now();
        if now < deadline {
            std::thread::sleep(deadline - now);
        }
        deadline += frame_period;
        // After a long stall (e.g. dragging the window), resync instead of racing to catch up.
        if deadline < std::time::Instant::now() {
            deadline = std::time::Instant::now() + frame_period;
        }
        let queued = queue.lock().unwrap().len();
        console.set_sample_rate(adjusted_rate(sample_rate, queued, target_queued));
        console.step_frame();
        let samples = console.take_samples();
        queue
            .lock()
            .unwrap()
            .extend(samples.into_iter().map(|s| filter.apply(s)));
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

const NES_FPS: f64 = 60.0988;

/// Raises the emulated sample rate when the queue is short and lowers it when long,
/// by at most 0.5%, so the queue settles at `target` without audible pitch change.
fn adjusted_rate(base: u32, queued: usize, target: usize) -> u32 {
    let error = (target as f64 - queued as f64) / target as f64;
    let factor = 1.0 + (error * 0.005).clamp(-0.005, 0.005);
    (f64::from(base) * factor).round() as u32
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
    fn rate_unchanged_at_target() {
        assert_eq!(adjusted_rate(48_000, 2_400, 2_400), 48_000);
    }

    #[test]
    fn rate_rises_when_queue_is_short() {
        assert_eq!(adjusted_rate(48_000, 1_200, 2_400), 48_120);
    }

    #[test]
    fn rate_falls_when_queue_is_long() {
        assert_eq!(adjusted_rate(48_000, 3_600, 2_400), 47_880);
    }

    #[test]
    fn rate_change_is_clamped() {
        assert_eq!(adjusted_rate(48_000, 0, 2_400), 48_240);
        assert_eq!(adjusted_rate(48_000, 24_000, 2_400), 47_760);
    }
}
