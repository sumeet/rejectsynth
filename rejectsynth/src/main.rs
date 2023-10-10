#![allow(special_module_name)]
#![feature(iter_array_chunks)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(array_windows)]

use psimple::Simple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;

mod lib;

const BUFFER_SIZE: usize = 1024;
const BUFFER_SIZE_HALF: usize = BUFFER_SIZE / 2;

fn main() {
    let pulse = init_pulse();
    let mut ctx = lib::SongContext::default();
    let mut buffer = [0f32; BUFFER_SIZE];
    let song = lib::songs::kalm();
    for chunk in ctx.play(&song).array_chunks::<BUFFER_SIZE_HALF>() {
        for (i, &note) in chunk.iter().enumerate() {
            buffer[i * 2] = note;
            buffer[i * 2 + 1] = note;
        }
        pulse.write(as_u8_slice(&buffer)).unwrap();
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

fn init_pulse() -> Simple {
    let spec = Spec {
        format: Format::F32le,
        channels: 2,
        rate: lib::SAMPLE_RATE as _,
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
