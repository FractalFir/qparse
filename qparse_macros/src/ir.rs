use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::{
    macro_assert,
    parse_stdfmt::{Argument, FormatSpec, FormatString, Type},
    type_destructure_inner,
};
#[derive(Clone)]
pub enum ParserIR {
    Whitespace {
        inner: Box<Self>,
    },
    Tag {
        tag: String,
        inner: Box<Self>,
    },
    Construct {
        ident: TokenStream,
        args: Vec<Argument>,
    },
    Parse {
        spec: FormatSpec,
        arg: Argument,
        inner: Box<Self>,
        verifier: bool,
    },
    Alt {
        variants: Vec<Self>,
    },
}
struct StackList<'a> {
    arg: &'a Argument,
    prev: Option<&'a Self>,
}
pub struct StackListIter<'a> {
    node: Option<&'a StackList<'a>>,
}

impl<'a> StackList<'a> {
    pub fn iter(&self) -> StackListIter<'_> {
        StackListIter { node: Some(self) }
    }
}

impl<'a> Iterator for StackListIter<'a> {
    type Item = &'a Argument;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node?;
        self.node = node.prev;
        Some(node.arg)
    }
}
impl ParserIR {
    pub fn tag_adjust(&mut self) {
        match self {
            ParserIR::Alt { variants } => variants.iter_mut().for_each(Self::tag_adjust),
            ParserIR::Parse { inner, .. } => {
                inner.tag_adjust();
            }
            ParserIR::Whitespace { inner } => inner.tag_adjust(),
            ParserIR::Tag { tag, inner } => {
                inner.tag_adjust();
                if tag.chars().any(char::is_whitespace) {
                    let tag = tag.clone();
                    let tags: Vec<_> = tag.split(char::is_whitespace).rev().collect();
                    let inner = (**inner).clone();
                    *self = inner;
                    for (i, tag) in tags.iter().enumerate() {
                        if i != 0 && !matches!(self, Self::Whitespace { .. }) {
                            *self = Self::Whitespace {
                                inner: Box::new(self.clone()),
                            };
                        }

                        if !tag.is_empty() {
                            *self = Self::Tag {
                                inner: Box::new(self.clone()),
                                tag: tag.to_string(),
                            };
                        }
                    }
                }
            }
            ParserIR::Construct { .. } => (),
        }
    }
    pub(crate) fn insert_verifiers(&mut self) {
        // usize::MAX as a dummy - impossible for somebody to have a struct with enough fields to fill the address space.
        self.insert_verifiers_inner(&StackList {
            arg: &Argument::Intiger(usize::MAX),
            prev: None,
        });
    }
    fn insert_verifiers_inner(&mut self, prev: &StackList) {
        match self {
            ParserIR::Whitespace { inner } | ParserIR::Tag { inner, .. } => {
                inner.insert_verifiers_inner(prev)
            }
            ParserIR::Construct { .. } => (),
            ParserIR::Parse {
                spec: _,
                arg,
                inner,
                verifier,
            } => {
                if prev.iter().any(|prev| prev == arg) {
                    *verifier = true;
                }
                let prev = StackList {
                    arg,
                    prev: Some(prev),
                };
                inner.insert_verifiers_inner(&prev);
            }
            ParserIR::Alt { variants } => variants
                .iter_mut()
                .for_each(|v| v.insert_verifiers_inner(prev)),
        }
    }
    pub fn from_fmt(fmt: &FormatString, ident: TokenStream) -> Self {
        // First: collect all the args
        let mut args: Vec<Argument> = fmt.named_fields().cloned().collect();
        args.sort();
        args.dedup();
        let mut root = Self::Construct { ident, args: args };
        for (fragment, text) in fmt.fragments.iter().rev() {
            if !text.is_empty() {
                root = Self::Tag {
                    tag: text.clone(),
                    inner: Box::new(root),
                };
            }
            root = Self::Parse {
                spec: fragment.fmt_spec.clone(),
                arg: fragment.argument.clone(),
                inner: Box::new(root),
                verifier: false,
            };
        }
        if !fmt.text.is_empty() {
            root = Self::Tag {
                tag: fmt.text.clone(),
                inner: Box::new(root),
            };
        }
        root
    }
    pub fn to_nom(&self, span: Span) -> TokenStream {
        let input = Ident::new("qparse_input_ident", span);
        match self {
            ParserIR::Construct { ident, args } => {
                let desc = type_destructure_inner(ident.clone(), args[..].iter(), span);
                quote! {
                    Ok((#input,#desc))
                }
            }
            ParserIR::Tag { tag, inner } => {
                let inner = inner.to_nom(span);
                quote! {
                    let (#input,_) = nom::bytes::complete::tag(#tag).parse(#input)?;
                    #inner
                }
            }
            ParserIR::Whitespace { inner } => {
                let inner = inner.to_nom(span);
                quote! {
                    let (#input,_) = nom::character::complete::multispace1.parse(#input)?;
                    #inner
                }
            }
            ParserIR::Parse {
                spec,
                arg,
                inner,
                verifier,
            } => {
                let FormatSpec {
                    fill_align,
                    sign,
                    alt_form,
                    sign_aware_zero_pad,
                    width,
                    precision,
                    tpe,
                } = &spec;

                macro_assert!(fill_align.is_none(), "fill_align not supported!");
                macro_assert!(sign.is_none(), "sign not supported!");
                macro_assert!(!alt_form, "alt_form not supported!");
                macro_assert!(!sign_aware_zero_pad, "sign_aware_zero_pad not supported!");
                macro_assert!(width.is_none(), "width not supported!");
                macro_assert!(precision.is_none(), "precision not supported!");
                let res = match tpe {
                    Type::LowerHex => {
                        quote! {qparse::Parseable::<qparse::LowerHex>::parse(#input)}
                    }
                    Type::UpperHex => {
                        quote! {qparse::Parseable::<qparse::UpperHex>::parse(#input)}
                    }
                    Type::Octal => {
                        quote! {qparse::Parseable::<qparse::Octal>::parse(#input)}
                    }
                    Type::Binary => {
                        quote! {qparse::Parseable::<qparse::Binary>::parse(#input)}
                    }
                    Type::Display => {
                        quote! {qparse::Parseable::<qparse::Display>::parse(#input)}
                    }
                    Type::LowerExp => {
                        quote! {qparse::Parseable::<qparse::LowerExp>::parse(#input)}
                    }
                    Type::UpperExp => {
                        quote! {qparse::Parseable::<qparse::UpperExp>::parse(#input)}
                    }
                    Type::Present(str) => {
                        quote! {
                            Ok::<_, nom::Err<E>>(
                                match nom::bytes::complete::tag::<_,_,E>(#str).parse(#input) {
                                    Ok((input, _discard)) => (input, true),
                                    Err(_) => (#input, false),
                                }
                            )
                        }
                    }
                    _ => {
                        macro_assert!(false, format!("{tpe:?} not supported in parsers!"));
                        todo!();
                    }
                };
                let arg = match &arg {
                    Argument::Intiger(i) => Ident::new(&format!("f{i}"), span),
                    Argument::Identifier(i) => Ident::new(i, span),
                };
                let inner = inner.to_nom(span);
                if *verifier {
                    let ver = Ident::new("qparse_verifier_ident", span);
                    let before = Ident::new("qparse_verify_start_ident", span);
                    quote! {
                        let #before = #input;
                        let (#input, #ver) = #res?;
                        infer_type(&#ver, &#arg);
                        if #ver != #arg{
                            return Err(nom::Err::Error(
                                <E as nom::error::ParseError<&str>>::from_error_kind(
                                    #before,
                                    nom::error::ErrorKind::Verify,
                                ),
                            ));
                        }
                        #inner
                    }
                } else {
                    quote! {
                        let (#input, #arg) = #res?;
                        #inner
                    }
                }
            }
            ParserIR::Alt { variants } => {
                let variant_arms = variants.iter().map(|v| {
                    let v = v.to_nom(span);
                    quote! {|#input| {#v}}
                });
                quote! {nom::branch::alt((#(#variant_arms),*)).parse(#input)}
            }
        }
    }
    pub fn opt(&mut self) {}
    pub fn normalize_alt(&mut self) {
        match self {
            ParserIR::Tag { inner, .. } | ParserIR::Whitespace { inner } => inner.normalize_alt(),
            ParserIR::Construct { .. } => (),
            ParserIR::Parse { inner, .. } => {
                inner.normalize_alt();
            }
            ParserIR::Alt { variants } => {
                while variants.len() > 20 {
                    *variants = variants
                        .chunks(20)
                        .map(|chunk| Self::Alt {
                            variants: chunk.to_vec(),
                        })
                        .collect();
                }
                variants.iter_mut().for_each(Self::normalize_alt);
                if variants.len() == 1 {
                    *self = variants[0].clone();
                }
            }
        }
    }
}
pub fn parse_for_struct(ident: Ident, fmt: &FormatString) -> TokenStream {
    let input = Ident::new("qparse_input_ident", ident.span());
    let mut ir = ParserIR::from_fmt(fmt, quote! {#ident});
    ir.tag_adjust();
    ir.insert_verifiers();
    ir.opt();
    ir.normalize_alt();
    let parser = ir.to_nom(ident.span());
    quote! {
        impl qparse::Parseable<qparse::Display> for #ident{
            fn parse<'a, E>(#input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: nom::error::ParseError<&'a str>+ nom::error::FromExternalError<&'a str, std::num::ParseIntError>{
                use nom::Parser;
                #[allow(dead_code)]
                fn infer_type<T:Sized>(a:&T,b:&T){}
                #parser
            }
        }
    }
}
pub fn parse_for_enum(
    prefix: FormatString,
    enum_name: Ident,
    variants: &[(Ident, FormatString)],
) -> TokenStream {
    macro_assert!(
        prefix.fragments.is_empty() && prefix.text.is_empty(),
        "Enums only support per-variant parsers"
    );
    let input = Ident::new("qparse_input_ident", enum_name.span());
    let variants = variants
        .iter()
        .map(|(variant, fmt)| ParserIR::from_fmt(fmt, quote! {#enum_name::#variant}));
    let mut root = ParserIR::Alt {
        variants: variants.collect(),
    };
    root.tag_adjust();
    root.insert_verifiers();
    root.opt();
    root.normalize_alt();
    let root = root.to_nom(enum_name.span());
    quote! {
        impl qparse::Parseable<qparse::Display> for #enum_name{
            fn parse<'a, E>(#input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: nom::error::ParseError<&'a str>+ nom::error::FromExternalError<&'a str, std::num::ParseIntError>{
                use nom::Parser;
                #[allow(dead_code)]
                fn infer_type<T:Sized>(a:&T,b:&T){}
                #root
            }
        }
    }
}
