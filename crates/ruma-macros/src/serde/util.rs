use proc_macro2::Span;
use syn::{ItemEnum, LitStr, Variant};

use super::{
    attr::{RenameAllAttr, RenameAttr},
    case::RenameRule,
};

pub fn get_rename_rule(input: &ItemEnum) -> syn::Result<RenameRule> {
    let rules: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("ruma_enum"))
        .map(|attr| attr.parse_args::<RenameAllAttr>().map(RenameAllAttr::into_inner))
        .collect::<syn::Result<_>>()?;

    match rules.len() {
        0 => Ok(RenameRule::None),
        1 => Ok(rules[0]),
        _ => Err(syn::Error::new(
            Span::call_site(),
            "found multiple ruma_enum(rename_all) attributes",
        )),
    }
}

pub fn get_rename(input: &Variant) -> syn::Result<Option<LitStr>> {
    let renames: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("ruma_enum"))
        .map(|attr| attr.parse_args::<RenameAttr>().map(RenameAttr::into_inner))
        .collect::<syn::Result<_>>()?;

    match renames.len() {
        0 | 1 => Ok(renames.into_iter().next()),
        _ => Err(syn::Error::new(Span::call_site(), "found multiple ruma_enum(rename) attributes")),
    }
}
