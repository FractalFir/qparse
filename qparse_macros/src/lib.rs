use crate::{
    display::{enum_display, struct_display},
    ir::{parse_for_enum, parse_for_struct},
    parse_stdfmt::FormatString,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemEnum, LitStr, spanned::Spanned};
mod display;

mod ir;
mod utilis;
pub(crate) use utilis::*;
/// Module parsing the rust format strings.  
mod parse_stdfmt;

#[cfg(test)]
macro_rules! assert_tokens_eq {
    ($actual:expr, $expected:expr $(,)?) => {
        assert_eq!($actual.to_string(), $expected.to_string())
    };
}
#[cfg(test)]
use proc_macro2::{Ident, Span};
#[test]
fn dest() {
    assert_tokens_eq!(
        type_destructure(
            quote! {Bar::Barely},
            &FormatString::parse("foo{0} uwu{1:x}").unwrap().1,
            Span::call_site()
        ),
        quote! {Bar :: Barely (f0 , f1 ,)}
    );
    assert_tokens_eq!(
        type_destructure(
            quote! {Foo},
            &FormatString::parse("foo{bar} uwu{baz:x}").unwrap().1,
            Span::call_site()
        ),
        quote! {Foo { r#bar , r#baz , }}
    );
}
#[test]
fn disp() {
    assert_tokens_eq!(
        struct_display(
            quote! {Bar::Barely},
            &FormatString::parse("foo{0} uwu{1:x}").unwrap().1,
            Span::call_site()
        ),
        quote! {
            impl std::fmt::Display for Bar::Barely {
                fn fmt(&self, qparse_fmt_ident: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use std::fmt::Write;
                    let Bar::Barely(f0, f1,) = self;
                    qparse_fmt_ident.write_str("foo")?;
                    write!(qparse_fmt_ident, "{}", f0)?;
                    qparse_fmt_ident.write_str(" uwu")?;
                    write!(qparse_fmt_ident, "{:x}", f1)?;
                    Ok(())
                }
            }
        }
    );
}
#[test]
fn simple_parse() {
    assert_tokens_eq!(
        parse_for_struct(
            Ident::new("Barely", Span::call_site()),
            &FormatString::parse("foo{0} owo uwu{1:x}").unwrap().1
        ),
        quote! {
            impl qparse::Parseable<qparse::Display> for Barely {
                fn parse(qparse_input_ident: &str) -> nom::IResult<&str, Self> {
                    use nom::Parser;
                    let (qparse_input_ident, _) = nom::bytes::complete::tag("foo").parse(qparse_input_ident)?;
                    let (qparse_input_ident, f0) =
                        qparse::Parseable::<qparse::Display>::parse(qparse_input_ident)?;
                    let (qparse_input_ident, _) =
                        nom::character::complete::multispace1.parse(qparse_input_ident)?;
                    let (qparse_input_ident, _) = nom::bytes::complete::tag("owo").parse(qparse_input_ident)?;
                    let (qparse_input_ident, _) =
                        nom::character::complete::multispace1.parse(qparse_input_ident)?;
                    let (qparse_input_ident, _) = nom::bytes::complete::tag("uwu").parse(qparse_input_ident)?;
                    let (qparse_input_ident, f1) =
                        qparse::Parseable::<qparse::LowerHex>::parse(qparse_input_ident)?;
                    Ok((qparse_input_ident, Barely(f0, f1,)))
                }
            }
        }
    );
}
#[test]
fn disp_enum() {
    assert_tokens_eq!(
        enum_display(
            Ident::new("Foo", Span::call_site()),
            &[
                (
                    Ident::new("Bar", Span::call_site()),
                    FormatString::parse("foo{0} uwu{1:x}").unwrap().1
                ),
                (
                    Ident::new("Baz", Span::call_site()),
                    FormatString::parse("bar{billy}").unwrap().1
                ),
            ],
        ),
        quote! {
            impl std::fmt::Display for Foo {
                fn fmt(&self, qparse_fmt_ident: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use std::fmt::Write;
                    match self {
                        Foo::Bar(f0, f1,) => {
                            qparse_fmt_ident.write_str("foo")?;
                            write!(qparse_fmt_ident, "{}", f0)?;
                            qparse_fmt_ident.write_str(" uwu")?;
                            write!(qparse_fmt_ident, "{:x}", f1)?;
                        }
                        Foo::Baz { r#billy ,} => {
                            qparse_fmt_ident.write_str("bar")?;
                            write!(qparse_fmt_ident, "{}", billy)?;
                        }
                    }
                    Ok(())
                }
            }
        }
    );
}

#[test]
fn parse_enum() {
    assert_tokens_eq!(
        parse_for_enum(
            FormatString::parse("").unwrap().1,
            Ident::new("Foo", Span::call_site()),
            &[
                (
                    Ident::new("Bar", Span::call_site()),
                    FormatString::parse("foo{0} uwu{1:x}").unwrap().1
                ),
                (
                    Ident::new("Baz", Span::call_site()),
                    FormatString::parse("bar{billy}").unwrap().1
                ),
            ],
        ),
        quote! {
            impl qparse::Parseable<qparse::Display> for Foo {
                fn parse(qparse_input_ident: &str) -> nom::IResult<&str, Self> {
                    use nom::Parser;
                    nom::branch::alt((
                        |qparse_input_ident| {
                            let (qparse_input_ident, _) =
                                nom::bytes::complete::tag("foo").parse(qparse_input_ident)?;
                            let (qparse_input_ident, f0) =
                                qparse::Parseable::<qparse::Display>::parse(qparse_input_ident)?;
                            let (qparse_input_ident, _) =
                                nom::character::complete::multispace1.parse(qparse_input_ident)?;
                            let (qparse_input_ident, _) =
                                nom::bytes::complete::tag("uwu").parse(qparse_input_ident)?;
                            let (qparse_input_ident, f1) =
                                qparse::Parseable::<qparse::LowerHex>::parse(qparse_input_ident)?;
                            Ok((qparse_input_ident, Foo::Bar(f0, f1,)))
                        },
                        |qparse_input_ident| {
                            let (qparse_input_ident, _) =
                                nom::bytes::complete::tag("bar").parse(qparse_input_ident)?;
                            let (qparse_input_ident, billy) =
                                qparse::Parseable::<qparse::Display>::parse(qparse_input_ident)?;
                            Ok((qparse_input_ident, Foo::Baz { r#billy ,}))
                        }
                    ))
                    .parse(qparse_input_ident)
                }
            }
        }
    );
}

fn qparse_inner(attr: TokenStream, item_toks: TokenStream) -> syn::Result<TokenStream> {
    let item: syn::Item = syn::parse2(item_toks.clone())?;
    let fmt: LitStr = syn::parse2(attr.clone())?;
    let fmt = FormatString::parse(&fmt.value())
        .map_err(|e| syn::Error::new_spanned(&fmt, format!("invalid qparse fmt: {e}")))?
        .1;
    let res = match item {
        syn::Item::Enum(item_enum) => {
            let ItemEnum {
                attrs,
                vis,
                enum_token,
                ident,
                generics,
                brace_token,
                mut variants,
            } = item_enum;
            let mut variant_defs = vec![];
            for variant in &mut variants {
                let def = take_def_attr(&mut variant.attrs)?;
                let def = FormatString::parse(&def)
                    .map_err(|e| {
                        syn::Error::new_spanned(variant.clone(), format!("invalid qparse fmt: {e}"))
                    })?
                    .1;
                variant_defs.push((variant.ident.clone(), def))
            }
            let disp = enum_display(ident.clone(), &variant_defs);
            let parse = parse_for_enum(fmt, ident.clone(), &variant_defs);
            let enm = ItemEnum {
                attrs,
                vis,
                enum_token,
                ident,
                generics,
                brace_token,
                variants,
            };
            quote! {#enm #disp #parse}
        }
        syn::Item::Struct(item_struct) => {
            let ident = item_struct.ident.clone();
            let display_impl = struct_display(quote! {#ident}, &fmt, attr.span());
            let parser_impl = parse_for_struct(ident, &fmt);
            quote! {
                #item_struct
                #parser_impl
                #display_impl
            }
        }
        _ => quote! {compile_error!("qparse only supports structs / enums");},
    };
    Ok(res)
}

#[proc_macro_attribute]
pub fn qparse(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    qparse_err(attr.into(), item.into()).into()
}
fn qparse_err(attr: TokenStream, item: TokenStream) -> TokenStream {
    qparse_inner(attr, item).unwrap_or_else(syn::Error::into_compile_error)
}
fn is_qparse(attr: &syn::Attribute) -> bool {
    let segs = &attr.path().segments;
    match segs.len() {
        1 => segs[0].ident == "qparse",
        2 => segs[0].ident == "qparse" && segs[1].ident == "qparse",
        _ => false,
    }
}
fn take_def_attr(attrs: &mut Vec<syn::Attribute>) -> syn::Result<String> {
    let pos = attrs.iter().position(is_qparse).ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "variant is missing a parser def",
        )
    })?;
    let attr = attrs.remove(pos);
    let lit: syn::LitStr = attr.parse_args()?;
    Ok(lit.value())
}
#[test]
fn tqparse_inner() {
    assert_tokens_eq!(
        qparse_err(
            quote! {"HIA{foo:x} {bar}"},
            quote! {
                struct Billy{
                    foo:u64,
                    bar:u128,
                }
            },
        ),
        quote! {
            struct Billy {
                foo: u64,
                bar: u128,
            }
            impl qparse::Parseable<qparse::Display> for Billy {
                fn parse(qparse_input_ident: &str) -> nom::IResult<&str, Self> {
                    use nom::Parser;
                    let (qparse_input_ident, _) = nom::bytes::complete::tag("HIA").parse(qparse_input_ident)?;
                    let (qparse_input_ident, foo) =
                        qparse::Parseable::<qparse::LowerHex>::parse(qparse_input_ident)?;
                    let (qparse_input_ident, _) =
                        nom::character::complete::multispace1.parse(qparse_input_ident)?;
                    let (qparse_input_ident, bar) =
                        qparse::Parseable::<qparse::Display>::parse(qparse_input_ident)?;
                    Ok((qparse_input_ident, Billy { r#bar, r#foo, }))
                }
            }
            impl std::fmt::Display for Billy {
                fn fmt(&self, qparse_fmt_ident: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use std::fmt::Write;
                    let Billy { r#bar,r#foo, } = self;
                    qparse_fmt_ident.write_str("HIA")?;
                    write!(qparse_fmt_ident, "{:x}", foo)?;
                    qparse_fmt_ident.write_str(" ")?;
                    write!(qparse_fmt_ident, "{}", bar)?;
                    Ok(())
                }
            }
        }
    )
}
#[test]
fn tqparse_inner_enum() {
    assert_tokens_eq!(
        qparse_err(
            quote! {""},
            quote! {
                enum BobOrNot{
                    #[qparse("bob{s}")]
                    Bob{s:String},
                    #[qparse("{0}")]
                    String(String)
                }
            },
        ),
        quote! {
        enum BobOrNot {
            Bob { s: String },
            String(String)
        }
        impl std::fmt::Display for BobOrNot {
            fn fmt(&self, qparse_fmt_ident: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use std::fmt::Write;
                match self {
                    BobOrNot::Bob { r#s ,} => {
                        qparse_fmt_ident.write_str("bob")?;
                        write!(qparse_fmt_ident, "{}", s)?;
                    }
                    BobOrNot::String(f0,) => {
                        write!(qparse_fmt_ident, "{}", f0)?;
                    }
                }
                Ok(())
            }
        }
        impl qparse::Parseable<qparse::Display> for BobOrNot {
            fn parse(qparse_input_ident: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::branch::alt((
                    |qparse_input_ident| {
                        let (qparse_input_ident, _) =
                            nom::bytes::complete::tag("bob").parse(qparse_input_ident)?;
                        let (qparse_input_ident, s) =
                            qparse::Parseable::<qparse::Display>::parse(qparse_input_ident)?;
                        Ok((qparse_input_ident, BobOrNot::Bob { r#s, }))
                    },
                    |qparse_input_ident| {
                        let (qparse_input_ident, f0) =
                            qparse::Parseable::<qparse::Display>::parse(qparse_input_ident)?;
                        Ok((qparse_input_ident, BobOrNot::String(f0,)))
                    }
                ))
                .parse(qparse_input_ident)
                }
            }
        }
    );
}
