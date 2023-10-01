#![feature(iter_array_chunks)]

use psimple::Simple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;

const SAMPLE_RATE: f32 = 44100.0; // 44.1 kHz
const BUFFER_SIZE: usize = 1024;
const BUFFER_SIZE_HALF: usize = BUFFER_SIZE / 2;

fn note(duration_ms: usize, freq: f32) -> impl Iterator<Item = f32> {
    let samples_per_note = (SAMPLE_RATE * duration_ms as f32 / 1000.0) as usize;
    let mut sample_count = 0;

    let phase_increment = 2.0 * std::f32::consts::PI * freq / SAMPLE_RATE;
    let mut phase: f32 = 0.0;

    std::iter::from_fn(move || {
        if sample_count < samples_per_note {
            let sample = phase.sin();
            phase += phase_increment;
            if phase >= 2.0 * std::f32::consts::PI {
                phase -= 2.0 * std::f32::consts::PI;
            }
            sample_count += 1;
            Some(sample)
        } else {
            None
        }
    })
}

fn main() {
    let spec = Spec {
        format: Format::F32le,
        channels: 2,
        rate: SAMPLE_RATE as _,
    };
    assert!(spec.is_valid());

    let s = Simple::new(
        None,                // Use the default server
        "reject synth",      // Our application’s name
        Direction::Playback, // We want a playback stream
        None,                // Use the default device
        "synth",             // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None,                // Use default buffering attributes
    )
    .unwrap();
    let a440_left = note(5_000, 440.);
    let a440_right = note(5_000, 440.);
    let mut buffer = [0f32; BUFFER_SIZE];
    for chunks in a440_left.zip(a440_right).array_chunks::<BUFFER_SIZE_HALF>() {
        for (i, (left, right)) in chunks.iter().enumerate() {
            buffer[i * 2] = *left;
            buffer[i * 2 + 1] = *right;
        }
        s.write(as_u8_slice(&buffer)).unwrap();
    }
}

fn as_u8_slice<T>(input: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            input.as_ptr() as *const u8,
            input.len() * std::mem::size_of::<T>(),
        )
    }
}
