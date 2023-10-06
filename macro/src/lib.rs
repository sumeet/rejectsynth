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
                            v.push(dsl::Inst::SetBPM(#bpm));
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
                            v.push(dsl::Inst::SetKey(dsl::Key {
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
                            v.push(dsl::Inst::SetScale(dsl::Scale::#scale));
                        });
                    }
                    _ => panic!("unknown ident: {ident:?}"),
                }
            }
            TokenTree::Punct(punct) => {
                let punct = punct.to_string();
                match punct.as_str() {
                    "-" => {
                        ts.next(); // consume the minus
                        code.extend(note_literal(&mut ts, true));
                    }
                    _ => {
                        println!("skipping punctuation for now: {t:?}");
                        ts.next();
                    }
                }
            }
            TokenTree::Literal(_) => {
                code.extend(note_literal(&mut ts, false));
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
    println!("{}", code.clone().to_string());
    code.into()
}

fn note_literal(ts: &mut Peekable<IntoIter>, is_negative: bool) -> TokenStream2 {
    let mut accidental = quote! { dsl::Accidental::Natural };
    let mut lit = ts.next().unwrap().to_string();
    if lit.ends_with('b') {
        accidental = quote! { dsl::Accidental::Flat };
        lit.replace_range(lit.len() - 1.., "");
    }
    let n = match lit.parse::<i8>() {
        Ok(n) => n * if is_negative { -1 } else { 1 },
        Err(_) => panic!("unknown literal: {lit:?}"),
    };
    if let Some(TokenTree::Punct(punct)) = ts.peek() {
        if punct.to_string() == "#" {
            ts.next();
            accidental = quote! { dsl::Accidental::Sharp };
        }
    }

    quote! {
        v.push(dsl::Inst::PlayNote (dsl::Note{
            duration: dsl::Duration::Quarter,
            pitch: dsl::NotePitch {
                enum_: dsl::NotePitchEnum::ScaleDegree(#n),
                accidental: #accidental,
            },
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
