use dsl::Instruction;

#[derive(Debug, Clone)]
pub struct SpannedInstruction {
    pub instruction: Instruction,
    pub l: usize,
    pub r: usize,
}

impl SpannedInstruction {
    pub fn range(&self) -> std::ops::Range<usize> {
        self.l..self.r
    }
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
            = ties_to_prev:tie() divisor:duration_multiplier() pitch:pitch() multiplier:duration_multiplier() is_dotted:dot() ties_to_next:tie() {?
                let mut numerator = multiplier;
                let mut denominator = divisor;
                if is_dotted {
                    numerator = numerator.checked_mul(3).ok_or("note duration is too large")?;
                    denominator = denominator.checked_mul(2).ok_or("note duration is too small")?;
                }
                Ok(dsl::Note {
                    duration: dsl::Duration::new(numerator, denominator),
                    pitch,
                    ties_to_next,
                    ties_to_prev,
                })
            }

        rule tie() -> bool
            = "_" { true }
            / "" { false }

        rule dot() -> bool
            = "." { true }
            / "" { false }

        rule duration_multiplier() -> u8
            = tildes:$("~"*) {?
                1u8.checked_shl(tildes.len() as u32).ok_or("too many duration modifiers")
            }

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
            = chord_base:chord_base() extension:chord_extension() shift:upward_inversion() {
                dsl::Harmony {degree: chord_base.0, scale: chord_base.1, extension, shift}
            }
            // Keep the original prefix notation readable in existing songs.
            / shift:inversion_shift() chord_base:chord_base() extension:chord_extension() {
                dsl::Harmony {degree: chord_base.0, scale: chord_base.1, extension, shift}
            }

        rule upward_inversion() -> i8
            = gts:$(">"+) {? i8::try_from(gts.len()).map_err(|_| "too many inversions") }

        rule inversion_shift() -> i8
            = lts:$("<"+) {? i8::try_from(lts.len()).map(|n| -n).map_err(|_| "too many inversions") }
            / shift:upward_inversion() { shift }
            / "" { 0 }

        rule chord_base() -> (u8, dsl::Scale)
            = "iii" { (3, dsl::Scale::Minor) }
            / "ii" { (2, dsl::Scale::Minor) }
            / "iv" { (4, dsl::Scale::Minor) }
            / "i" { (1, dsl::Scale::Minor) }
            / "viii" { (8, dsl::Scale::Minor) }
            / "vii" { (7, dsl::Scale::Minor) }
            / "vi" { (6, dsl::Scale::Minor) }
            / "v" { (5, dsl::Scale::Minor) }
            / "III" { (3, dsl::Scale::Major) }
            / "II" { (2, dsl::Scale::Major) }
            / "IV" { (4, dsl::Scale::Major) }
            / "I" { (1, dsl::Scale::Major) }
            / "VIII" { (8, dsl::Scale::Major) }
            / "VII" { (7, dsl::Scale::Major) }
            / "VI" { (6, dsl::Scale::Major) }
            / "V" { (5, dsl::Scale::Major) }

        rule chord_extension() -> Option<dsl::ChordExtension>
            = "7" { Some(dsl::ChordExtension::Seventh) }
            / "9" { Some(dsl::ChordExtension::Ninth) }
            / "" { None }

        rule uint() -> u128
            = int:$("0" / ['1' ..= '9']+ ['0' ..= '9']*) {? int.parse().or(Err("not a number")) }
        rule onespace() = [' ' | '\t']
        rule nbspace() = onespace()+
        rule newline() = "\n" / "\r\n"
        rule whitespace() = (nbspace() / newline())+
        rule _() = quiet!{ whitespace() }
    }
}
