use crate::{
    display::{enum_display, struct_display}, parse_stdfmt::{Argument, FormatString},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

mod display;
/// Module parsing the rust format strings.  
mod parse_stdfmt;

pub fn type_destructure(tpe_ident: TokenStream, fmt: &FormatString, span: Span) -> TokenStream {
    // check if this is a struct OR tuple type.
    if fmt.named_fields().all(Argument::is_identifier) {
        let fields = fmt
            .named_fields()
            .into_iter()
            .map(|name| Ident::new_raw(name.as_identifier().unwrap(), span));
        quote! {
            #tpe_ident { #(#fields,)* }
        }
    } else if fmt.named_fields().all(Argument::is_intiger) {
        let fields = fmt
            .named_fields()
            .into_iter()
            .map(|name| Ident::new_raw(&format!("f{}", name.as_intiger().unwrap()), span));
        quote! {
            #tpe_ident ( #(#fields,)* )
        }
    } else {
        quote! {compile_error!("Nonsense mix of named / unnamed fields in", stringift!(#tpe_ident))}
    }
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
        quote! {Bar :: Barely (r#f0 , r#f1 ,)}
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
                    let Bar::Barely(r#f0, r#f1,) = self;
                    qparse_fmt_ident.write_str("foo");
                    qparse_fmt_ident.write_str(" uwu");
                    write!(qparse_fmt_ident, "{}", f0)?;
                    write!(qparse_fmt_ident, "{}", f1)?;
                    Ok(())
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
                (Ident::new("Bar", Span::call_site()), FormatString::parse("foo{0} uwu{1:x}").unwrap().1),
                (Ident::new("Baz", Span::call_site()), FormatString::parse("bar{billy}").unwrap().1),
            ],
        ),
        quote!{
            impl std::fmt::Display for Foo {
                fn fmt(&self, qparse_fmt_ident: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use std::fmt::Write;
                    match self {
                        Foo::Bar(r#f0, r#f1,) => {
                            qparse_fmt_ident.write_str("foo");
                            qparse_fmt_ident.write_str(" uwu");
                            write!(qparse_fmt_ident, "{}", f0)?;
                            write!(qparse_fmt_ident, "{}", f1)?;
                        }
                        Foo::Baz { r#billy ,} => {
                            qparse_fmt_ident.write_str("bar");
                            write!(qparse_fmt_ident, "{}", billy)?;
                        }
                    }
                    Ok(())
                }
            }
        }
    );
}