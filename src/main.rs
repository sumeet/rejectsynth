use psimple::Simple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;

fn main() {
    const SAMPLE_RATE: f32 = 44100.0; // 44.1 kHz
    const FREQUENCY: f32 = 440.0; // Frequency of A4 note in Hz
    const DURATION: f32 = 8.0; // Duration in seconds
    const BUFFER_SIZE: usize = 1024; // Fixed buffer size

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

    let samples_per_note = (SAMPLE_RATE * DURATION) as usize;
    let mut sample_count = 0;

    let mut buf = [0.0f32; BUFFER_SIZE];
    let phase_increment = 2.0 * std::f32::consts::PI * FREQUENCY / SAMPLE_RATE;
    let mut phase: f32 = 0.0;

    'main: loop {
        for i in 0..BUFFER_SIZE {
            if sample_count < samples_per_note {
                buf[i] = phase.sin();
                phase += phase_increment;
                if phase >= 2.0 * std::f32::consts::PI {
                    phase -= 2.0 * std::f32::consts::PI;
                }
                sample_count += 1;
            } else {
                break 'main;
            }
        }

        if sample_count >= samples_per_note {
            sample_count = 0; // Reset for the next note
        }

        // Now you can send `buf` to PulseAudio for playback
        s.write(as_u8_slice(&buf)).unwrap();
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
