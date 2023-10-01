#![feature(iter_array_chunks)]
#![feature(anonymous_lifetime_in_impl_trait)]

use dsl::{Accidental, Inst, Key, Note, Scale, ABC};
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

struct SongContext {
    bpm: u16,
    key: Key,
    scale: Scale,
}

impl SongContext {
    fn default() -> Self {
        Self::new(
            120,
            Key {
                abc: ABC::C,
                accidental: Accidental::Natural,
            },
            Scale::Major,
        )
    }

    fn new(bpm: u16, key: Key, scale: Scale) -> Self {
        Self { bpm, key, scale }
    }

    fn render_note(&self, n: Note) -> impl Iterator<Item = f32> {
        let freq = match n.pitch.enum_ {
            dsl::NotePitchEnum::ScaleDegree(degree) => {
                let degree = degree;
                let key = self.key.abc as i8;
                let scale = match self.scale {
                    Scale::Major => [0, 2, 4, 5, 7, 9, 11],
                    Scale::Minor => [0, 2, 3, 5, 7, 8, 10],
                };
                let scale_degree = scale[(degree + key) as usize % scale.len()];
                440.0 * 2.0f32.powf((scale_degree as f32) / 12.0)
            }
        };
        let duration_ms = match n.duration {
            dsl::Duration::Quarter => 60_000 / self.bpm as usize,
        };
        note(duration_ms, freq, 1.)
    }

    fn play_instructions<'a>(
        &'a mut self,
        instrs: impl Iterator<Item = &'a Inst> + 'a,
    ) -> impl Iterator<Item = f32> + 'a {
        instrs
            .flat_map(move |inst| match inst {
                Inst::BPM(bpm) => {
                    self.bpm = *bpm;
                    None
                }
                Inst::Key(key) => {
                    self.key = *key;
                    None
                }
                Inst::Scale(scale) => {
                    self.scale = *scale;
                    None
                }
                Inst::Note(note) => Some(self.render_note(*note)),
            })
            .flatten()
    }
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
