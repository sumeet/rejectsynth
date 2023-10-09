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
    pub accidental: Accidental,
}

#[derive(Debug, Clone, Copy)]
pub enum NotePitchEnum {
    ScaleDegree(i8),
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
    pub degree: i8,
}

impl Harmony {
    pub fn parse(s: &str) -> Self {
        let scale = if s.chars().next().unwrap().is_uppercase() {
            Scale::Major
        } else {
            Scale::Minor
        };
        match s.to_lowercase().as_str() {
            "i" => Self { scale, degree: 1 },
            "ii" => Self { scale, degree: 2 },
            "iii" => Self { scale, degree: 3 },
            "iv" => Self { scale, degree: 4 },
            "v" => Self { scale, degree: 5 },
            "vi" => Self { scale, degree: 6 },
            "vii" => Self { scale, degree: 7 },
            _ => panic!("unknown harmony: {}", s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub duration: Duration,
    pub pitch: NotePitch,
}
