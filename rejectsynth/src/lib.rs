mod parser;

use std::collections::HashSet;
use std::ops::RangeInclusive;

use dsl::{Accidental, Harmony, Instruction, Key, Note, NotePitch, Scale, ABC};
use r#macro::m;
use wasm_bindgen::prelude::wasm_bindgen;

pub const SAMPLE_RATE: f32 = 44100.0; // 44.1 kHz

use crate::parser::SpannedInstruction;
pub use parser::grammar;

#[wasm_bindgen]
pub struct WasmSongIterator {
    ctx: SongContext,
    song: Vec<SpannedInstruction>,
    syntaxes: Vec<Syntax>,
    selection: Option<RangeInclusive<usize>>,
}

#[wasm_bindgen]
impl WasmSongIterator {
    #[wasm_bindgen]
    pub fn from_song_text(song_text: &str, l: Option<usize>, r: Option<usize>) -> Self {
        let parse_result = parse(song_text).unwrap();
        let instructions = parse_result
            .spanned_instructions
            .iter()
            .map(|spanned_instruction| spanned_instruction.instruction)
            .collect();
        let selection = match (l, r) {
            // TODO: this is weird, it works better when i subtract 1,
            // BUT the indices from vs code seem to be all right
            // maybe it's a problem with our spanning logic...?
            (Some(l), Some(r)) => Some(l - 1..=r - 1),
            _ => None,
        };
        Self {
            ctx: SongContext::default(instructions),
            song: parse_result.spanned_instructions,
            syntaxes: parse_result.syntaxes,
            selection,
        }
    }

    #[wasm_bindgen]
    pub fn is_done(&self) -> bool {
        self.ctx.is_done()
    }

    #[wasm_bindgen]
    pub fn play_next(&mut self) -> PlaybackResult {
        let samples = match self.ctx.current_instruction() {
            Instruction::PlayNote { .. } => {
                let spanned_instruction = &self.song[self.ctx.pc];
                if let Some(selection) = &self.selection {
                    if selection.contains(&spanned_instruction.l)
                        || selection.contains(&spanned_instruction.r)
                    {
                        self.ctx.iterate()
                    } else {
                        self.ctx.skip();
                        vec![]
                    }
                } else {
                    self.ctx.iterate()
                }
            }
            Instruction::SetBPM(_)
            | Instruction::SetKey(_)
            | Instruction::SetScale(_)
            | Instruction::SkipToNote
            | Instruction::SetHarmony(_) => self.ctx.iterate(),
        };

        let on_syntaxes = self
            .ctx
            .on_instructions
            .iter()
            .map(|&i| self.syntaxes[i].clone())
            .collect();

        PlaybackResult {
            samples,
            on_syntaxes,
        }
    }
}

#[wasm_bindgen]
pub struct PlaybackResult {
    samples: Vec<f32>,
    on_syntaxes: Vec<Syntax>,
}

#[wasm_bindgen]
impl PlaybackResult {
    #[wasm_bindgen(getter)]
    pub fn samples(&self) -> Vec<f32> {
        self.samples.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn on_syntaxes(&self) -> Vec<Syntax> {
        self.on_syntaxes.clone()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Syntax {
    pub line_no: usize,
    pub col_no: usize,
    pub len: usize,
    node_type: String,
}

#[wasm_bindgen]
impl Syntax {
    #[wasm_bindgen(getter)]
    pub fn node_type(&self) -> String {
        self.node_type.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_node_type(&mut self, s: &str) {
        self.node_type = s.to_string();
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook));
}

fn console_error_panic_hook(info: &std::panic::PanicInfo) {
    let msg = match info.payload().downcast_ref::<&str>() {
        Some(s) => *s,
        None => "Box<Any>",
    };
    let location = info.location().unwrap(); // The current implementation always returns Some
    let description = format!(
        "panic occurred in file '{}' at line {}: {}",
        location.file(),
        location.line(),
        msg
    );
    log(&description);
}

struct ParseResult {
    spanned_instructions: Vec<SpannedInstruction>,
    syntaxes: Vec<Syntax>,
}

fn parse(s: &str) -> Result<ParseResult, Box<dyn std::error::Error>> {
    let positions_of_line_breaks = s.match_indices('\n').map(|(i, _)| i).collect::<Vec<_>>();
    let spanned_instructions = grammar::song(s)?;

    let syntaxes = spanned_instructions
        .iter()
        .map(|spanned_instruction| {
            let instruction = &spanned_instruction.instruction;
            let (l, r) = (spanned_instruction.l, spanned_instruction.r);
            let line_no = positions_of_line_breaks
                .iter()
                .position(|&pos| pos > l)
                .unwrap_or(positions_of_line_breaks.len());
            let col_no = if line_no == 0 {
                l
            } else {
                l - positions_of_line_breaks[line_no - 1] - 1
            };
            let len = r - l;
            let node_type = match instruction {
                Instruction::SetBPM(_) => "SetBPM",
                Instruction::SetKey(_) => "SetKey",
                Instruction::SetScale(_) => "SetScale",
                Instruction::PlayNote { .. } => "PlayNote",
                Instruction::SkipToNote => "SkipToNote",
                Instruction::SetHarmony(_) => "SetHarmony",
            }
            .to_string();
            Syntax {
                line_no,
                col_no,
                len,
                node_type,
            }
        })
        .collect();
    Ok(ParseResult {
        spanned_instructions,
        syntaxes,
    })
}

#[wasm_bindgen]
pub fn syntax(s: &str) -> Vec<Syntax> {
    parse(s).unwrap().syntaxes
}

#[wasm_bindgen]
// pos is the position in the song_file to skip to...
pub fn playback_for_note_input(song_file: &str, pos: usize) -> Vec<f32> {
    let parse_result = parse(song_file);
    if parse_result.is_err() {
        return vec![];
    }
    let parse_result = parse_result.unwrap();
    let instructions = parse_result
        .spanned_instructions
        .iter()
        .map(|spanned_instruction| spanned_instruction.instruction)
        .collect();
    let mut ctx = SongContext::default(instructions);
    while !ctx.is_done() {
        let cur_instruction = ctx.current_instruction();
        match cur_instruction {
            Instruction::PlayNote { .. } => {
                let spanned_instruction = &parse_result.spanned_instructions[ctx.pc];
                if pos >= spanned_instruction.l && pos <= spanned_instruction.r {
                    return ctx.iterate();
                } else {
                    ctx.skip();
                }
            }
            Instruction::SetBPM(_)
            | Instruction::SetKey(_)
            | Instruction::SetScale(_)
            | Instruction::SkipToNote
            | Instruction::SetHarmony(_) => {
                ctx.iterate();
            }
        }
    }
    vec![]
}

pub mod songs {
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

            III: ~4 ~3 ~2 ~3 , 4 ~-5 ~-7

            i: ~2, ~1, 1~.

            iv: ~1 ~5 ~4 ~3 , VII: 4. ~~5 ~~6#

            v: ~7 ~5 , 5~.

            VI: ~5 ~6, ~5  ~4 , vi7: 3. ~4

            III: ~5 ~-5 , ~-6, ~-7 , i: ~2 ~1 , 1_ iv: , _~1 ~1

            ~2 ~3 , 4~. i: ~3

            ~4 ~5 VII7: 6~.
        }
    }
}

fn find_skip_to_index(instructions: &[Instruction]) -> Option<usize> {
    instructions
        .iter()
        .enumerate()
        .find_map(|(i, &inst)| match inst {
            Instruction::SkipToNote => Some(i),
            _ => None,
        })
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

            // // First harmonic
            // chord_sample += 0.5 * (2.0 * *p).sin();
            // // Second harmonic
            // chord_sample += 0.25 * (3.0 * *p).sin();

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

fn scale_degree_to_semitones(scale: Scale, degree: u8) -> i8 {
    if degree == 0 {
        panic!("scale degree cannot be 0, it doesn't make sense")
    }
    if degree == 1 {
        return 0;
    }
    scale_ascending(scale)
        .iter()
        .cycle()
        .take((degree - 1) as usize)
        .sum()
}

pub struct SongContext {
    bpm: u16,
    key: Key,
    scale: Scale,
    phase: f32,
    harmony: Option<Harmony>,

    skip_to_note_index: Option<usize>,

    pc: usize,
    instructions: Vec<Instruction>,
    on_harmony: Option<usize>,
    off_on_next_tick: Option<usize>,
    on_instructions: HashSet<usize>,
}

impl SongContext {
    pub fn current_instruction(&self) -> &Instruction {
        &self.instructions[self.pc]
    }

    pub fn is_done(&self) -> bool {
        self.pc >= self.instructions.len()
    }

    pub fn skip(&mut self) {
        self.pc += 1;
    }

    pub fn iterate(&mut self) -> Vec<f32> {
        if self.is_done() {
            panic!("iteration called on done song context")
        }

        if let Some(i) = self.off_on_next_tick {
            self.on_instructions.remove(&i);
        }

        let cur_instruction = self.instructions[self.pc];
        let samples = self.eval(cur_instruction).into_iter().flatten().collect();
        match cur_instruction {
            Instruction::SetBPM(_)
            | Instruction::SetKey(_)
            | Instruction::SetScale(_)
            | Instruction::PlayNote(_)
            | Instruction::SkipToNote => {
                self.off_on_next_tick = Some(self.pc);
                self.on_instructions.insert(self.pc);
            }
            Instruction::SetHarmony(_) => {
                if let Some(prev_harmony) = self.on_harmony {
                    self.on_instructions.remove(&prev_harmony);
                }
                self.on_instructions.insert(self.pc);
                self.on_harmony = Some(self.pc);
            }
        };
        self.pc += 1;
        samples
    }

    pub fn default(instructions: Vec<Instruction>) -> Self {
        let skip_to_index = find_skip_to_index(&instructions);
        Self::new(
            instructions,
            120,
            Key {
                abc: ABC::C,
                accidental: Accidental::Natural,
            },
            Scale::Major,
            skip_to_index,
        )
    }

    fn new(
        instructions: Vec<Instruction>,
        bpm: u16,
        key: Key,
        scale: Scale,
        skip_to_index: Option<usize>,
    ) -> Self {
        Self {
            bpm,
            key,
            scale,
            phase: 0.,
            harmony: None,
            pc: 0,
            instructions,
            on_harmony: None,
            off_on_next_tick: None,
            on_instructions: HashSet::new(),
            skip_to_note_index: skip_to_index,
        }
    }

    fn chord_freqs(&self) -> Vec<f32> {
        let mut freqs = vec![];
        if let Some(mut harmony) = self.harmony {
            let num_semitones_to_chord_base = scale_degree_to_semitones(self.scale, harmony.degree);
            let freq_of_base_of_chord =
                shift_up_by_interval(self.freq_of_tonic(), num_semitones_to_chord_base);
            freqs.push(freq_of_base_of_chord);

            let semitones_to_3 = scale_degree_to_semitones(harmony.scale, 3);
            let freq_of_3 = shift_up_by_interval(freq_of_base_of_chord, semitones_to_3);
            freqs.push(freq_of_3);

            let semitones_to_5 = scale_degree_to_semitones(harmony.scale, 5);
            let freq_of_5 = shift_up_by_interval(freq_of_base_of_chord, semitones_to_5);
            freqs.push(freq_of_5);

            if harmony.add_7 {
                let semitones_to_7 = scale_degree_to_semitones(harmony.scale, 7);
                let freq_of_7 = shift_up_by_interval(freq_of_base_of_chord, semitones_to_7);
                freqs.push(freq_of_7);
            }

            while harmony.shift < 0 {
                let last = freqs.pop().unwrap();
                freqs.insert(0, last / 2.);
                harmony.shift += 1;
            }
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
                let mut freq = shift_up_by_interval(self.freq_of_tonic(), offset);

                if pitch.octave > 0 {
                    freq *= 2.0f32.powf(pitch.octave as f32);
                } else if pitch.octave < 0 {
                    freq /= 2.0f32.powf(-pitch.octave as f32);
                }
                freq
            }
        }
    }

    fn freq_of_tonic(&self) -> f32 {
        to_freq(self.key.abc, self.key.accidental)
    }

    pub fn play<'a>(&'a mut self, instrs: &'a [Instruction]) -> impl Iterator<Item = f32> + 'a {
        instrs
            .iter()
            .enumerate()
            .flat_map(move |(_i, &inst)| self.eval(inst))
            .flatten()
    }

    fn eval(&mut self, inst: Instruction) -> Option<impl Iterator<Item = f32>> {
        match inst {
            Instruction::SetBPM(bpm) => {
                self.bpm = bpm;
                None
            }
            Instruction::SetKey(key) => {
                self.key = key;
                None
            }
            Instruction::SetScale(scale) => {
                self.scale = scale;
                None
            }
            Instruction::PlayNote(note) => {
                if let Some(skip_to_note_index) = self.skip_to_note_index {
                    // this is a little bit strange because eval didn't know about pc and now it does...
                    if self.pc < skip_to_note_index {
                        return None;
                    }
                }
                Some(self.render_note(note))
            }
            Instruction::SkipToNote => None,
            Instruction::SetHarmony(harmony) => {
                self.harmony = Some(harmony);
                None
            }
        }
    }
}

fn freq_to_abc(freq: f32) -> String {
    let a = 440.0;
    let n = (12.0 * (freq / a).log2()).round() as i32;
    let abc = match n % 12 {
        0 => "A",
        1 => "A# / Bb",
        2 => "B",
        3 => "C",
        4 => "C# / Db",
        5 => "D",
        6 => "D# / Eb",
        7 => "E",
        8 => "F",
        9 => "F# / Gb",
        10 => "G",
        11 => "G# / Ab",
        _ => panic!("impossible"),
    };
    let octave = n / 12 - 1;
    format!("{}{}", abc, octave)
}
