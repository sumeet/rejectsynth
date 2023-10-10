use dsl::Instruction;

#[derive(Debug)]
pub struct SpannedInstruction {
    pub instruction: Instruction,
    pub l: usize,
    pub r: usize,
}

peg::parser! {
    pub grammar grammar() for str {
        pub rule song() -> Vec<SpannedInstruction>
            = instr:spanned_instruction() ** _ { instr }

        pub rule spanned_instruction() -> SpannedInstruction
            = l:position!() instruction:instruction() r:position!() {
                SpannedInstruction {
                    instruction,
                    l,
                    r,
                }
            }

        pub rule instruction() -> Instruction
            = set_bpm() / set_key() / set_scale()

        rule set_bpm() -> Instruction
            = "bpm" _ bpm:int() { Instruction::SetBPM(bpm as _) }

        rule set_key() -> Instruction
            = "key" _ key:key_name() { Instruction::SetKey(key) }

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

        rule int() -> i128
            = int:$("0" / "-"? ['1' ..= '9']+ ['0' ..= '9']*) {? int.parse().or(Err("not a number")) }
        rule onespace() = [' ' | '\t']
        rule nbspace() = onespace()+
        rule newline() = "\n" / "\r\n"
        rule whitespace() = (nbspace() / newline())+
        rule _() = quiet!{ whitespace() }
    }
}
