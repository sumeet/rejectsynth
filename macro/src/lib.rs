#![feature(proc_macro_quote)]
use proc_macro::{quote, TokenStream};

#[proc_macro]
pub fn m(_: TokenStream) -> TokenStream {
    let guys = quote! {
        vec![
            dsl::Inst::BPM(90),
            dsl::Inst::Key(dsl::Key {
                abc: dsl::ABC::G,
                accidental: dsl::Accidental::Natural,
            }),
            dsl::Inst::Scale(dsl::Scale::Minor),
            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(2),
                    accidental: dsl::Accidental::Natural,
                },
            },
            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(1),
                    accidental: dsl::Accidental::Natural,
                },
            },
            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(0),
                    accidental: dsl::Accidental::Sharp,
                },
            },
            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(1),
                    accidental: dsl::Accidental::Natural,
                },
            },

            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(1),
                    accidental: dsl::Accidental::Natural,
                },
            },
            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(0),
                    accidental: dsl::Accidental::Natural,
                },
            },
            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(-1),
                    accidental: dsl::Accidental::Natural,
                },
            },
            dsl::Inst::Note {
                duration: dsl::Duration::Quarter,
                pitch: dsl::NotePitch {
                    enum_: dsl::NotePitchEnum::ScaleDegree(0),
                    accidental: dsl::Accidental::Natural,
                },
            },
        ]
    };
    guys.into()
}
