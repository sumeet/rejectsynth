#[derive(Debug, Clone, Copy)]
pub enum ABC {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
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
