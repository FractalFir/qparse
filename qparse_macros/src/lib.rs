use crate::{
    display::{enum_display, struct_display},
    ir::{ParserIR, parse_for_enum, parse_for_struct},
    parse_stdfmt::{Argument, FormatString},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
mod display;
mod ir;
/// Module parsing the rust format strings.  
mod parse_stdfmt;
pub(crate) fn type_destructure_inner<'a>(
    tpe_ident: TokenStream,
    named_fields: impl Iterator<Item = &'a Argument> + Clone,
    span: Span,
) -> TokenStream {
    // check if this is a struct OR tuple type.
    if named_fields.clone().all(Argument::is_identifier) {
        let fields = named_fields.map(|name| Ident::new_raw(name.as_identifier().unwrap(), span));
        quote! {
            #tpe_ident { #(#fields,)* }
        }
    } else if named_fields.clone().all(Argument::is_intiger) {
        let mut named_fields: Vec<_> = named_fields.map(|n| *n.as_intiger().unwrap()).collect();
        named_fields.sort();
        let fields = named_fields
            .into_iter()
            .map(|name| Ident::new(&format!("f{}", name), span));
        quote! {
            #tpe_ident ( #(#fields,)* )
        }
    } else {
        quote! {compile_error!("Nonsense mix of named / unnamed fields in", stringify!(#tpe_ident))}
    }
}
pub fn type_destructure(tpe_ident: TokenStream, fmt: &FormatString, span: Span) -> TokenStream {
    type_destructure_inner(tpe_ident, fmt.named_fields(), span)
}
#[macro_export]
macro_rules! macro_assert {
    ($expect:expr, $fmt:expr) => {
        if !$expect {
            let fmt = $fmt;
            return quote! { compile_error!(#fmt);};
        }
    };
}
macro_rules! assert_tokens_eq {
    ($actual:expr, $expected:expr $(,)?) => {
        assert_eq!($actual.to_string(), $expected.to_string())
    };
}
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
