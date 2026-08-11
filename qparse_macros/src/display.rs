use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::{
    macro_assert,
    parse_stdfmt::{FormatSpec, FormatString, Type},
    type_destructure,
};
pub fn display_writes(fmt: &FormatString, f: &Ident) -> TokenStream {
    let mut writes = TokenStream::new();
    if !fmt.text.is_empty() {
        let text = &fmt.text;
        writes.extend(quote! {#f.write_str(#text)?;});
    }
    for (fmt, text) in &fmt.fragments {
        if fmt.fmt_spec.tpe.is_custom() {
            let FormatSpec {
                fill_align,
                sign,
                alt_form,
                sign_aware_zero_pad,
                width,
                precision,
                tpe,
            } = &fmt.fmt_spec;
            macro_assert!(
                fill_align.is_none(),
                "fill_align not supported with qparse-specific formats!"
            );
            macro_assert!(
                sign.is_none(),
                "sign not supported with qparse-specific formats!"
            );
            macro_assert!(
                !alt_form,
                "alt_form not supported with qparse-specific formats!"
            );
            macro_assert!(
                !sign_aware_zero_pad,
                "sign_aware_zero_pad not supported with qparse-specific formats!"
            );
            macro_assert!(
                width.is_none(),
                "width not supported with qparse-specific formats!"
            );
            macro_assert!(
                precision.is_none(),
                "precision not supported with qparse-specific formats!"
            );
            todo!("custom parser {tpe:?} not supported!")
        } else {
            let FormatSpec {
                fill_align,
                sign,
                alt_form,
                sign_aware_zero_pad,
                width,
                precision,
                tpe,
            } = &fmt.fmt_spec;
            macro_assert!(fill_align.is_none(), "fill_align not supported!");
            macro_assert!(sign.is_none(), "sign not supported!");
            macro_assert!(!alt_form, "alt_form not supported!");
            macro_assert!(!sign_aware_zero_pad, "sign_aware_zero_pad not supported!");
            macro_assert!(width.is_none(), "width not supported!");
            macro_assert!(precision.is_none(), "precision not supported!");
            let arg = match &fmt.argument {
                crate::parse_stdfmt::Argument::Intiger(i) => Ident::new(&format!("f{i}"), f.span()),
                crate::parse_stdfmt::Argument::Identifier(i) => Ident::new(i, f.span()),
            };
            match tpe {
                Type::Display => writes.extend(quote! {write!(#f,"{}",#arg)?;}),
                Type::LowerHex => writes.extend(quote! {write!(#f,"{:x}",#arg)?;}),
                _ => macro_assert!(false, format!("unsupported fmt type {tpe:?} for {arg}!")),
            }
        }
        if !text.is_empty() {
            writes.extend(quote! {#f.write_str(#text)?;});
        }
    }
    writes
}
pub fn struct_display(tpe_ident: TokenStream, fmt: &FormatString, span: Span) -> TokenStream {
    let f = Ident::new("qparse_fmt_ident", span);
    let desc = type_destructure(tpe_ident.clone(), fmt, span);
    let writes = display_writes(fmt, &f);
    quote! {
        impl std::fmt::Display for #tpe_ident{
            fn fmt(&self, #f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use std::fmt::Write;
                let #desc = self;
                #writes
                Ok(())
            }
        }
    }
}
pub fn enum_display(enum_name: Ident, variants: &[(Ident, FormatString)]) -> TokenStream {
    let f = Ident::new("qparse_fmt_ident", enum_name.span());
    let mut arms = TokenStream::new();
    for (variant, fmt) in variants {
        let desc = type_destructure(quote! {#enum_name::#variant}, fmt, enum_name.span());
        let writes = display_writes(fmt, &f);
        arms.extend(quote! {
            #desc => {
                #writes
            }
        });
    }
    quote! {
        impl std::fmt::Display for #enum_name{
            fn fmt(&self, #f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use std::fmt::Write;
                match self{
                    #arms
                }
                Ok(())
            }
        }
    }
}
