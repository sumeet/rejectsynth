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
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub duration: Duration,
    pub pitch: NotePitch,
}
