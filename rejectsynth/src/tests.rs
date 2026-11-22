use super::*;

#[test]
fn repeated_tildes_double_or_halve_note_lengths() {
    for (text, expected) in [
        ("1", (1, 1)),
        ("~1", (1, 2)),
        ("~~1", (1, 4)),
        ("~~~1", (1, 8)),
        ("~~~~1", (1, 16)),
        ("1~", (2, 1)),
        ("1~~", (4, 1)),
        ("1~~~", (8, 1)),
        ("1~~~~", (16, 1)),
        ("1.", (3, 2)),
        ("~1.", (3, 4)),
        ("1~.", (6, 2)),
        ("~1~", (2, 2)),
    ] {
        let Instruction::PlayNote(note) = grammar::instruction(text).unwrap() else {
            panic!("expected a note: {text}");
        };
        assert_eq!(
            (note.duration.numerator, note.duration.denominator),
            expected,
            "{text}"
        );
    }
    for text in ["~~~~~~~~1", "1~~~~~~~~", "~~~~~~~1.", "1~~~~~~~."] {
        assert!(
            grammar::instruction(text).is_err(),
            "overflow should be a parse error: {text}"
        );
    }
}

#[test]
fn inversions_produce_the_expected_chord_voicings() {
    for (text, semitones) in [
        ("I:", vec![0, 4, 7]),
        ("I>:", vec![4, 7, 12]),
        ("I>>:", vec![7, 12, 16]),
        ("I>>>:", vec![12, 16, 19]),
        (">I:", vec![4, 7, 12]),
        (">>I:", vec![7, 12, 16]),
        (">>>I:", vec![12, 16, 19]),
        ("<I:", vec![-5, 0, 4]),
        ("<<I:", vec![-8, -5, 0]),
        ("<<<I:", vec![-12, -8, -5]),
        (">i:", vec![3, 7, 12]),
        ("i>:", vec![3, 7, 12]),
        ("I7:", vec![0, 4, 7, 10]),
        ("i7:", vec![0, 3, 7, 10]),
        ("V7:", vec![7, 11, 14, 17]),
        ("v7:", vec![7, 10, 14, 17]),
        ("I9:", vec![0, 4, 7, 10, 14]),
        ("i9:", vec![0, 3, 7, 10, 14]),
        ("V9:", vec![7, 11, 14, 17, 21]),
        ("v9:", vec![7, 10, 14, 17, 21]),
        ("I9>:", vec![4, 7, 10, 14, 12]),
        ("I7>:", vec![4, 7, 10, 12]),
        (">I7:", vec![4, 7, 10, 12]),
        ("<I7:", vec![-2, 0, 4, 7]),
        ("VIII:", vec![12, 16, 19]),
        ("viii:", vec![12, 15, 19]),
        (">VIII:", vec![16, 19, 24]),
        ("VIII>:", vec![16, 19, 24]),
    ] {
        let Instruction::SetHarmony(harmony) = grammar::instruction(text).unwrap() else {
            panic!("expected a chord: {text}");
        };
        let mut context = SongContext::default(vec![]);
        context.harmony = Some(harmony);
        let frequencies = context.chord_freqs();
        assert_eq!(frequencies.len(), semitones.len(), "{text}");
        for (frequency, semitone) in frequencies.iter().zip(semitones) {
            let actual = 12. * (frequency / context.freq_of_tonic()).log2();
            assert!(
                (actual - semitone as f32).abs() < 0.001,
                "{text}: expected {semitone}, got {actual}"
            );
        }
    }
}

#[test]
fn macro_and_text_notation_produce_the_same_ninth_chord() {
    let instructions = m!(key C V9: 1);
    let Instruction::SetHarmony(from_macro) = instructions[1] else {
        panic!("expected a chord");
    };
    let Instruction::SetHarmony(from_text) = grammar::instruction("V9:").unwrap() else {
        panic!("expected a chord");
    };
    let mut context = SongContext::default(vec![]);
    context.harmony = Some(from_macro);
    let frequencies = context.chord_freqs();
    assert_eq!(frequencies.len(), 5);
    context.harmony = Some(from_text);
    assert_eq!(frequencies, context.chord_freqs());
}

#[test]
fn standalone_playback_marker_remains_distinct_from_upward_inversions() {
    let instructions: Vec<_> = grammar::song("1 > V>: 2 v7>>: 3")
        .unwrap()
        .into_iter()
        .map(|node| node.instruction)
        .collect();
    assert_eq!(find_skip_to_index(&instructions), Some(1));
    assert!(matches!(
        instructions[2],
        Instruction::SetHarmony(Harmony { shift: 1, .. })
    ));
    assert!(matches!(
        instructions[4],
        Instruction::SetHarmony(Harmony { shift: 2, .. })
    ));

    let mut context = SongContext::default(instructions);
    let mut audible_notes = 0;
    while !context.is_done() {
        if !context.iterate().is_empty() {
            audible_notes += 1;
        }
    }
    assert_eq!(audible_notes, 2);
}
