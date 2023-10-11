#![feature(proc_macro_span)]

use proc_macro::token_stream::IntoIter;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::iter::Peekable;
use syn::__private::TokenStream2;

fn titlecase(mut s: String) -> String {
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        s.replace_range(..1, &c.to_uppercase().to_string());
    }
    s
}

fn parse(ts: TokenStream) -> TokenStream {
    let mut code = quote! { let mut v = vec![]; };
    let mut ts = ts.into_iter().peekable();
    while let Some(t) = ts.peek().cloned() {
        match &t {
            TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                match ident.as_str() {
                    "bpm" => {
                        ts.next();
                        let bpm = match ts.next() {
                            Some(TokenTree::Literal(lit)) => lit.to_string(),
                            _ => panic!("expected literal"),
                        };
                        let bpm = syn::LitInt::new(&bpm, proc_macro2::Span::call_site());
                        code.extend(quote! {
                            v.push(dsl::Instruction::SetBPM(#bpm));
                        });
                    }
                    "key" => {
                        ts.next();
                        let key = match ts.next() {
                            Some(TokenTree::Ident(ident)) => ident.to_string(),
                            _ => panic!("expected ident"),
                        };
                        let key = titlecase(key);
                        let key = syn::Ident::new(&key, proc_macro2::Span::call_site());
                        code.extend(quote! {
                            v.push(dsl::Instruction::SetKey(dsl::Key {
                                abc: dsl::ABC::#key,
                                accidental: dsl::Accidental::Natural,
                            }));
                        });
                    }
                    "scale" => {
                        ts.next();
                        let scale = match ts.next() {
                            Some(TokenTree::Ident(ident)) => titlecase(ident.to_string()),
                            _ => panic!("expected ident"),
                        };
                        let scale = syn::Ident::new(&scale, proc_macro2::Span::call_site());
                        code.extend(quote! {
                            v.push(dsl::Instruction::SetScale(dsl::Scale::#scale));
                        });
                    }
                    // underscore means it's a note with preceding tie
                    "_" => code.extend(note_literal(&mut ts)),
                    harmony if harmony.starts_with(['i', 'v', 'I', 'V']) => {
                        ts.next();
                        let colon = ts.next().unwrap();
                        code.extend(quote! {
                            v.push(dsl::Instruction::SetHarmony(dsl::Harmony::parse(#harmony)));
                        });
                        assert_eq!(colon.to_string(), ":", "expected colon");
                    }
                    _ => panic!("unknown ident: {ident:?}"),
                }
            }
            TokenTree::Punct(t) => {
                let punct = t.to_string();
                match punct.as_str() {
                    "~" | "-" => {
                        code.extend(note_literal(&mut ts));
                    }
                    "," => {
                        // println!("skipping punctuation for now: {t:?}");
                        ts.next();
                    }
                    ">" => {
                        code.extend(quote! {
                            v.push(dsl::Instruction::SkipToNote);
                        });
                        ts.next();
                    }
                    _ => panic!("unknown punct: {punct:?}"),
                }
            }
            TokenTree::Literal(_) => {
                code.extend(note_literal(&mut ts));
            }
            TokenTree::Group(_) => unreachable!(),
        }
    }
    let code = quote! {
        {
            #code
            v
        }
    };
    code.into()
}

fn note_literal(ts: &mut Peekable<IntoIter>) -> TokenStream2 {
    let mut numerator = 1u8;
    let mut denominator = 1u8;
    let mut num_octaves_shift = 0i8;
    let mut ties_to_next = false;
    let mut ties_to_prev = false;

    while let Some(first) = ts.peek() {
        let first = first.to_string();
        if first == "~" {
            ts.next();
            denominator *= 2;
        } else if first == "-" {
            ts.next();
            num_octaves_shift -= 1;
        } else if first == "_" {
            ts.next();
            ties_to_prev = true;
        } else {
            break;
        }
    }

    let mut accidental = quote! { dsl::Accidental::Natural };
    let lit = ts.next().unwrap();
    let mut lit_text = lit.to_string();
    if lit_text.ends_with('b') {
        accidental = quote! { dsl::Accidental::Flat };
        lit_text.replace_range(lit_text.len() - 1.., "");
    }
    if lit_text.ends_with('.') {
        numerator *= 3;
        denominator *= 2;
        lit_text.replace_range(lit_text.len() - 1.., "");
    }
    if lit_text.ends_with('_') {
        ties_to_next = true;
        lit_text.replace_range(lit_text.len() - 1.., "");
    }

    let n = match lit_text.parse::<u8>() {
        Ok(n) => n,
        Err(_) => panic!("unknown literal: {lit:?}"),
    };
    if let Some(TokenTree::Punct(punct)) = ts.peek() {
        if punct.to_string() == "#" {
            ts.next();
            accidental = quote! { dsl::Accidental::Sharp };
        }
    }

    while let Some(TokenTree::Punct(punct)) = ts.peek() {
        let no_ws = lit.span().end().column() == punct.span().start().column();
        if punct.to_string() == "~" && no_ws {
            if numerator == 1 {
                numerator = 2
            } else {
                numerator += 2
            };
            ts.next();
        } else if punct.to_string() == "." {
            ts.next();
            numerator *= 3;
            denominator *= 2;
        } else {
            break;
        }
    }

    quote! {
        v.push(dsl::Instruction::PlayNote (dsl::Note{
            duration: dsl::Duration::new(#numerator, #denominator),
            pitch: dsl::NotePitch {
                enum_: dsl::NotePitchEnum::ScaleDegree(#n),
                accidental: #accidental,
                octave: #num_octaves_shift,
            },
            ties_to_next: #ties_to_next,
            ties_to_prev: #ties_to_prev,
        }));
    }
}

#[proc_macro]
pub fn m(ts: TokenStream) -> TokenStream {
    // for t in ts.clone() {
    //     println!("t: {:?}", t);
    // }
    parse(ts)
}
