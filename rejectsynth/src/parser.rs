use dsl::Instruction;

#[derive(Debug, Clone)]
pub struct SpannedInstruction {
    pub instruction: Instruction,
    pub l: usize,
    pub r: usize,
}

peg::parser! {
    pub grammar grammar() for str {
        pub rule song() -> Vec<SpannedInstruction>
            // TODO: will handle commas for reals at some point ithink
            // = instr:spanned_instruction() ** _ _? { instr }
            = instr:spanned_instruction() ** comma_or_space() _? { instr }

        // TODO: get rid of this, commas are gonna have real meaning
        rule comma_or_space()
            = _? "," _?
            / _

        pub rule spanned_instruction() -> SpannedInstruction
            = l:position!() instruction:instruction() r:position!() {
                SpannedInstruction {
                    instruction,
                    l,
                    r,
                }
            }

        pub rule instruction() -> Instruction
            = set_bpm() / set_key() / set_scale() / set_harmony() / play_note()
            / skip_to_note()

        rule skip_to_note() -> Instruction
            = ">" { Instruction::SkipToNote }

        rule set_bpm() -> Instruction
            = "bpm" _ bpm:uint() { Instruction::SetBPM(bpm as _) }

        rule set_key() -> Instruction
            = "key" _ key:key_name() { Instruction::SetKey(key) }

        rule play_note() -> Instruction
            = note:note() { Instruction::PlayNote(note) }

        rule note() -> dsl::Note
            = ties_to_prev:tie() num_half:note_mul_2()  pitch:pitch() num_twice:note_mul_2() is_dotted:dot() ties_to_next:tie() {
                let mut numerator = if num_twice == 0 { 1 } else { num_twice * 2 };
                let mut denominator = if num_half == 0 { 1 } else { num_half * 2 };
                if is_dotted {
                    numerator *= 3;
                    denominator *= 2;
                }
                dsl::Note {
                    duration: dsl::Duration::new(numerator, denominator),
                    pitch,
                    ties_to_next,
                    ties_to_prev,
                }
            }

        rule tie() -> bool
            = "_" { true }
            / "" { false }

        rule dot() -> bool
            = "." { true }
            / "" { false }

        rule note_mul_2() -> u8
            = s:"~"+ { (s.len() as u8) }
            / "" { 0 }

        rule pitch() -> dsl::NotePitch
            = octave:octave() num:uint() accidental:accidental() {
                dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(num as _),
                    accidental,
                    octave,
                }
            }

        rule octave() -> i8
            = minuses:"-"+ { (minuses.len() as i8) * -1 }
            / pluses:"+"+ { (pluses.len() as i8) * 1 }
            / "" { 0 }

        rule accidental() -> dsl::Accidental
            = "#" { dsl::Accidental::Sharp }
            / "b" { dsl::Accidental::Flat }
            / "" { dsl::Accidental::Natural }

        // add support for accidentals later
        rule key_name() -> dsl::Key
            = abc:abc() { dsl::Key { abc, accidental: dsl::Accidental::Natural } }

        rule abc() -> dsl::ABC
            = "A" { dsl::ABC::A }
            / "B" { dsl::ABC::B }
            / "C" { dsl::ABC::C }
            / "D" { dsl::ABC::D }
            / "E" { dsl::ABC::E }
            / "F" { dsl::ABC::F }
            / "G" { dsl::ABC::G }

        rule set_scale() -> dsl::Instruction
            = "scale" _ scale:scale_name() { dsl::Instruction::SetScale(scale) }

        rule scale_name() -> dsl::Scale
            = "major" { dsl::Scale::Major }
            / "minor" { dsl::Scale::Minor }

        rule set_harmony() -> dsl::Instruction
            = harmony:harmony() ":" { dsl::Instruction::SetHarmony(harmony) }

        rule harmony() -> dsl::Harmony
            = downshift:downshift() chord_base:chord_base() add_7:add_7() {
                let shift = (downshift as i8 * -1);
                dsl::Harmony {degree: chord_base.0, scale: chord_base.1, add_7, shift}
            }

        rule downshift() -> usize
            = lts:"<"* { lts.len() }

        rule chord_base() -> (u8, dsl::Scale)
            = "iii" { (3, dsl::Scale::Minor) }
            / "ii" { (2, dsl::Scale::Minor) }
            / "iv" { (4, dsl::Scale::Minor) }
            / "i" { (1, dsl::Scale::Minor) }
            / "vii" { (7, dsl::Scale::Minor) }
            / "vi" { (6, dsl::Scale::Minor) }
            / "v" { (5, dsl::Scale::Minor) }
            / "III" { (3, dsl::Scale::Major) }
            / "II" { (2, dsl::Scale::Major) }
            / "IV" { (4, dsl::Scale::Major) }
            / "I" { (1, dsl::Scale::Major) }
            / "VII" { (7, dsl::Scale::Major) }
            / "VI" { (6, dsl::Scale::Major) }
            / "V" { (5, dsl::Scale::Major) }

        rule add_7() -> bool
            = "7" { true }
            / "" { false }

        rule uint() -> u128
            = int:$("0" / ['1' ..= '9']+ ['0' ..= '9']*) {? int.parse().or(Err("not a number")) }
        rule onespace() = [' ' | '\t']
        rule nbspace() = onespace()+
        rule newline() = "\n" / "\r\n"
        rule whitespace() = (nbspace() / newline())+
        rule _() = quiet!{ whitespace() }
    }
}
