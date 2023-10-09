#![feature(iter_array_chunks)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(array_windows)]

use dsl::{Accidental, Harmony, Instruction, Key, Note, NotePitch, Scale, ABC};
use psimple::Simple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;
use r#macro::m;

const SAMPLE_RATE: f32 = 44100.0; // 44.1 kHz
const BUFFER_SIZE: usize = 1024;
const BUFFER_SIZE_HALF: usize = BUFFER_SIZE / 2;

mod songs {
    use super::*;

    #[allow(dead_code)]
    pub fn fairy() -> Vec<Instruction> {
        m! {
            bpm 90
            key G
            scale minor

            2,1,-1#,1,
            1,-1,-2,-1,
            -1,-2,-3#,-2
            -2,-3,-4#,-3

            2,1,-1#,1,
            3,2,1#,2
            4,3,2,3
            2,1,-1,-2
        }
    }

    pub fn kalm() -> Vec<Instruction> {
        m! {
            bpm 70
            key E
            scale minor

            III: ~4 ~3 ~2 ~3 , 4 ~-3 ~-1

            i: ~2, ~1, 1~.

            iv: ~1 ~5 ~4 ~3 , VII: 4. ~~5 ~~6#

            v: ~7 ~5 , 5~.

            VI: ~5 ~6, ~5  ~4 , vi: 3. ~4 // should add 7th

            // the iv change actually happens half a beat earlier, but we don't have a way to express that yet...
            III: ~5 ~-3 , ~-2, ~-1 , i: ~2 ~1 , 1. iv: ~1

            ~2 ~3 , 4~. i: ~3

            // VII here should also have 7
            ~4 ~5 VII: 6~.
        }
    }
}

fn main() {
    let pulse = init_pulse();
    let mut ctx = SongContext::default();
    let mut buffer = [0f32; BUFFER_SIZE];
    let song = songs::kalm();
    for chunk in ctx.play(&song).array_chunks::<BUFFER_SIZE_HALF>() {
        for (i, &note) in chunk.iter().enumerate() {
            buffer[i * 2] = note;
            buffer[i * 2 + 1] = note;
        }
        pulse.write(as_u8_slice(&buffer)).unwrap();
    }
}

const ATTACK_MS: usize = 10;

// volume is between 0 and 1
fn freqs_to_samples<'a>(
    duration_ms: usize,
    freqs: impl IntoIterator<Item = f32> + ExactSizeIterator,
    volume: f32,
    phase: f32,
) -> (impl Iterator<Item = f32> + 'a, f32) {
    let num_samples_per_note = (SAMPLE_RATE * duration_ms as f32 / 1000.0) as usize;
    let num_samples_in_attack = (SAMPLE_RATE * ATTACK_MS as f32 / 1000.0) as usize;

    let freqs_len = freqs.len();
    // Create phase increments for each frequency in the chord
    let phase_increments: Vec<f32> = freqs
        .into_iter()
        .map(|freq| 2.0 * std::f32::consts::PI * freq / SAMPLE_RATE)
        .collect();

    // Initialize phases for each frequency in the chord
    let mut phases: Vec<f32> = vec![phase; freqs_len];

    // Calculate the average phase increment for ending_phase calculation
    let avg_phase_increment = phase_increments.iter().sum::<f32>() / freqs_len as f32;

    let ending_phase = phase + avg_phase_increment * num_samples_per_note as f32;

    // Normalize phase to [0, 2π]
    let ending_phase_normalized = ending_phase % (2.0 * std::f32::consts::PI);

    let samples_iter = (0..num_samples_per_note).map(move |i| {
        let mut chord_sample: f32 = 0.0;

        for (p, &incr) in phases.iter_mut().zip(&phase_increments) {
            chord_sample += p.sin();
            *p += incr;
            if *p >= 2.0 * std::f32::consts::PI {
                *p -= 2.0 * std::f32::consts::PI;
            }
        }

        // Average the sample value for all notes in the chord
        let sample = chord_sample / freqs_len as f32;

        let attack_envelope = (i as f32 / num_samples_in_attack as f32).min(1.);
        let release_envelope =
            ((num_samples_per_note - i) as f32 / num_samples_in_attack as f32).min(1.);

        sample * volume * attack_envelope * release_envelope
    });

    (samples_iter, ending_phase_normalized)
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
        Scale::Major => [2, 2, 1, 2, 2, 2, 1],
        Scale::Minor => [2, 1, 2, 2, 1, 2, 2],
    }
}

const fn scale_descending(scale: Scale) -> [i8; 7] {
    match scale {
        Scale::Major => [-1, -2, -2, -2, -1, -2, -2],
        Scale::Minor => [-2, -1, -2, -2, -2, -1, -1],
    }
}

fn scale_degree_to_semitones(scale: Scale, degree: i8) -> i8 {
    if degree == 0 {
        panic!("scale degree cannot be 0, it doesn't make sense")
    }
    if degree == 1 {
        return 0;
    }
    let (scale, num_to_take) = if degree > 0 {
        (scale_ascending(scale), (degree - 1) as usize)
    } else {
        (scale_descending(scale), -(degree) as usize)
    };
    scale.iter().cycle().take(num_to_take).sum()
}

struct SongContext {
    bpm: u16,
    key: Key,
    scale: Scale,
    phase: f32,
    harmony: Option<Harmony>,
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
        Self {
            bpm,
            key,
            scale,
            phase: 0.,
            harmony: None,
        }
    }

    fn chord_freqs(&self) -> Vec<f32> {
        let mut freqs = vec![];
        if let Some(harmony) = self.harmony {
            // self.key
            // self.scale
            // harmony.degree
            // harmony.scale
            let num_semitones_to_chord_base = scale_degree_to_semitones(self.scale, harmony.degree);
            let freq_of_base_of_chord =
                shift_up_by_interval(self.freq_of_tonic(), num_semitones_to_chord_base);
            // 1
            freqs.push(freq_of_base_of_chord);
            // 3
            let semitones_to_3 = scale_degree_to_semitones(harmony.scale, 3);
            let freq_of_3 = shift_up_by_interval(freq_of_base_of_chord, semitones_to_3);
            freqs.push(freq_of_3);
            // 5
            let semitones_to_5 = scale_degree_to_semitones(harmony.scale, 5);
            let freq_of_5 = shift_up_by_interval(freq_of_base_of_chord, semitones_to_5);
            freqs.push(freq_of_5);
        }
        freqs
    }

    fn render_note(&mut self, n: Note) -> impl Iterator<Item = f32> {
        let freq = self.pitch_to_freq(n.pitch);
        let quarter_duration = 60_000 / self.bpm as usize;
        let duration_ms =
            quarter_duration * n.duration.numerator as usize / n.duration.denominator as usize;

        let mut freqs = self.chord_freqs();
        freqs.push(freq);

        let (samples, ending_phase) =
            freqs_to_samples(duration_ms, freqs.into_iter(), 1., self.phase);
        self.phase = ending_phase;
        samples
    }

    fn pitch_to_freq(&mut self, pitch: NotePitch) -> f32 {
        match pitch.enum_ {
            dsl::NotePitchEnum::ScaleDegree(degree) => {
                let offset = scale_degree_to_semitones(self.scale, degree);
                let offset = match pitch.accidental {
                    dsl::Accidental::Natural => offset,
                    dsl::Accidental::Sharp => offset + 1,
                    dsl::Accidental::Flat => offset - 1,
                };
                shift_up_by_interval(self.freq_of_tonic(), offset)
            }
        }
    }

    fn freq_of_tonic(&self) -> f32 {
        to_freq(self.key.abc, self.key.accidental)
    }

    fn play<'a>(&'a mut self, instrs: &'a [Instruction]) -> impl Iterator<Item = f32> + 'a {
        let mut skip_to_note_index = None;
        'outer: for i in 0..instrs.len() {
            if matches!(instrs[i], Instruction::SkipToNote) {
                for j in (i + 1)..instrs.len() {
                    if matches!(instrs[j], Instruction::PlayNote { .. }) {
                        skip_to_note_index = Some(j);
                        break 'outer;
                    }
                }
                panic!("skip to note must be followed by a note")
            }
        }

        instrs
            .iter()
            .enumerate()
            .flat_map(move |(i, inst)| match inst {
                Instruction::SetBPM(bpm) => {
                    self.bpm = *bpm;
                    None
                }
                Instruction::SetKey(key) => {
                    self.key = *key;
                    None
                }
                Instruction::SetScale(scale) => {
                    self.scale = *scale;
                    None
                }
                Instruction::PlayNote(note) => {
                    if let Some(skip_to_note_index) = skip_to_note_index {
                        if i < skip_to_note_index {
                            return None;
                        }
                    }
                    Some(self.render_note(*note))
                }
                Instruction::SkipToNote => None,
                Instruction::SetHarmony(harmony) => {
                    self.harmony = Some(*harmony);
                    None
                }
            })
            .flatten()
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
