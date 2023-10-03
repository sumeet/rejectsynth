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

fn to_freq(abc: ABC, accidental: Accidental) -> f32 {
    let abc = match abc {
        ABC::A => 0,
        ABC::B => 2,
        ABC::C => 3,
        ABC::D => 5,
        ABC::E => 7,
        ABC::F => 8,
        ABC::G => 10,
    };
    let accidental = match accidental {
        Accidental::Natural => 0,
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
    };
    let degree = abc + accidental;
    440.0 * 2.0f32.powf(degree as f32 / 12.0)
}

fn shift_up_by_interval(freq: f32, interval: i8) -> f32 {
    freq * 2.0f32.powf(interval as f32 / 12.0)
}

const fn scale_ascending(scale: Scale) -> [i8; 7] {
    match scale {
        Scale::Major => [0, 2, 2, 1, 2, 2, 2],
        Scale::Minor => [0, 2, 1, 2, 2, 1, 2],
    }
}

const fn scale_descending(scale: Scale) -> [i8; 7] {
    match scale {
        Scale::Major => [0, -2, -2, -1, -2, -2, -2],
        Scale::Minor => [0, -2, -1, -2, -2, -1, -2],
    }
}

fn scale_degree_to_semitones(scale: Scale, degree: i8) -> i8 {
    if degree == 0 {
        panic!(
            "scale degree cannot be 0, it doesn't make sense. 1 or -1 would stay in the same place"
        )
    }
    if degree == 1 {
        return 0;
    }
    let (scale, num_to_take) = if degree > 0 {
        (scale_ascending(scale), degree as usize)
    } else {
        (scale_descending(scale), -(degree - 1) as usize)
    };
    scale.iter().cycle().take(num_to_take).sum()
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
                let offset = dbg!(scale_degree_to_semitones(self.scale, dbg!(degree)));
                let offset = match n.pitch.accidental {
                    dsl::Accidental::Natural => offset,
                    dsl::Accidental::Sharp => offset + 1,
                    dsl::Accidental::Flat => offset - 1,
                };
                shift_up_by_interval(to_freq(self.key.abc, self.key.accidental), offset)
            }
        };
        let duration_ms = match n.duration {
            dsl::Duration::Quarter => 60_000 / self.bpm as usize,
        };
        note(duration_ms, freq, 1.)
    }

    fn play<'a>(
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
    let pulse = init_pulse();

    let mut ctx = SongContext::default();

    let insts = m! {
      bpm 90
      key G
      scale myx

      2,1,-1#,1,
      1,-1,-2,-3,
    };

    let mut buffer = [0f32; BUFFER_SIZE];
    for chunks in ctx.play(insts.iter()).array_chunks::<BUFFER_SIZE_HALF>() {
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
