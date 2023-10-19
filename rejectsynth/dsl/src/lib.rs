#![feature(const_fn_floating_point_arithmetic)]

#[derive(Debug, Clone, Copy)]
pub enum ABC {
    A = 0,
    B = 2,
    C = 3,
    D = 5,
    E = 7,
    F = 8,
    G = 10,
}

#[derive(Debug, Clone, Copy)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
}

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub abc: ABC,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, Copy)]
pub enum Scale {
    Major,
    Minor,
}

#[derive(Debug, Clone, Copy)]
pub struct Duration {
    pub numerator: u8,
    pub denominator: u8,
}

impl Duration {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NotePitch {
    pub enum_: NotePitchEnum,
    pub octave: i8,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, Copy)]
pub enum NotePitchEnum {
    ScaleDegree(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    SetBPM(u16),
    SetKey(Key),
    SetScale(Scale),
    PlayNote(Note),
    SkipToNote,
    SetHarmony(Harmony),
}

#[derive(Debug, Clone, Copy)]
pub struct Harmony {
    pub scale: Scale,
    pub degree: u8,
    pub add_7: bool,
    pub shift: i8,
}

impl Harmony {
    pub fn parse(mut s: &str) -> Self {
        let add_7 = s.ends_with('7');
        if add_7 {
            s = &s[..s.len() - 1];
        }

        let scale = if s.chars().next().unwrap().is_uppercase() {
            Scale::Major
        } else {
            Scale::Minor
        };
        let degree = match s.to_lowercase().as_str() {
            "i" => 1,
            "ii" => 2,
            "iii" => 3,
            "iv" => 4,
            "v" => 5,
            "vi" => 6,
            "vii" => 7,
            _ => panic!("unknown harmony: {}", s),
        };
        Self {
            scale,
            degree,
            add_7,
            shift: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub duration: Duration,
    pub pitch: NotePitch,
    pub ties_to_next: bool,
    pub ties_to_prev: bool,
}
