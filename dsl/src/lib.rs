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
pub enum Duration {
    Quarter,
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
pub enum Inst {
    BPM(u16),
    Key(Key),
    Scale(Scale),
    Note(Note),
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub duration: Duration,
    pub pitch: NotePitch,
}
