#![feature(iter_array_chunks)]

use psimple::Simple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;
use r#macro::m;

const SAMPLE_RATE: f32 = 44100.0; // 44.1 kHz
const BUFFER_SIZE: usize = 1024;
const BUFFER_SIZE_HALF: usize = BUFFER_SIZE / 2;

// volume is between 0 and 1
fn note(duration_ms: usize, freq: f32, volume: f32) -> impl Iterator<Item = f32> {
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
            Some(sample * volume)
        } else {
            None
        }
    })
}

fn main() {
    let insts = m! {
      bpm 90
      key G
      scale min

      // these are scale degrees
      // and quarter notes
      2,1,0#,1,
      1,0,-1,0,
    };

    let pulse = init_pulse();
    let a440 = note(5_000, 440., 1.);
    let mut buffer = [0f32; BUFFER_SIZE];
    for chunks in a440.array_chunks::<BUFFER_SIZE_HALF>() {
        for (i, &note) in chunks.iter().enumerate() {
            buffer[i * 2] = note;
            buffer[i * 2 + 1] = note;
        }
        pulse.write(as_u8_slice(&buffer)).unwrap();
    }
}

fn init_pulse() -> Simple {
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
    s
}

fn as_u8_slice<T>(input: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            input.as_ptr() as *const u8,
            input.len() * std::mem::size_of::<T>(),
        )
    }
}
