use crate::parse_stdfmt::{Argument, FormatString};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
pub(crate) fn type_destructure_inner<'a>(
    tpe_ident: TokenStream,
    named_fields: impl Iterator<Item = &'a Argument> + Clone,
    span: Span,
) -> TokenStream {
    let mut named_fields: Vec<Argument> = named_fields.cloned().collect();
    named_fields.sort();
    named_fields.dedup();
    // check if this is a struct OR tuple type.
    if named_fields.iter().all(Argument::is_identifier) {
        let fields = named_fields
            .iter()
            .map(|name| Ident::new_raw(name.as_identifier().unwrap(), span));
        quote! {
            #tpe_ident { #(#fields,)* }
        }
    } else if named_fields.clone().iter().all(Argument::is_intiger) {
        let mut named_fields: Vec<_> = named_fields
            .iter()
            .map(|n| *n.as_intiger().unwrap())
            .collect();
        named_fields.sort();
        let fields = named_fields
            .into_iter()
            .map(|name| Ident::new(&format!("f{}", name), span));
        quote! {
            #tpe_ident ( #(#fields,)* )
        }
    } else {
        let msg = format!("Nonsense mix of named / unnamed fields in {tpe_ident}");
        quote! {compile_error!(#msg)}
    }
}
pub(crate) fn type_destructure(
    tpe_ident: TokenStream,
    fmt: &FormatString,
    span: Span,
) -> TokenStream {
    type_destructure_inner(tpe_ident, fmt.named_fields(), span)
}
macro_rules! macro_assert {
    ($expect:expr, $fmt:expr) => {
        if !$expect {
            let fmt = $fmt;
            return quote! { compile_error!(#fmt);};
        }
    };
}
pub(crate) use macro_assert;
