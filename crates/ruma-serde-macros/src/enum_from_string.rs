use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Fields, FieldsNamed, FieldsUnnamed, ItemEnum};

use crate::util::{get_rename, get_rename_rule};

pub fn expand_enum_from_string(input: &ItemEnum) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let rename_rule = get_rename_rule(&input)?;
    let mut fallback = None;
    let mut fallback_ty = None;
    let branches: Vec<_> = input
        .variants
        .iter()
        .map(|v| {
            let variant_name = &v.ident;
            let variant_str = match (get_rename(v)?, &v.fields) {
                (None, Fields::Unit) => Some(
                    rename_rule.apply_to_variant(&variant_name.to_string()).into_token_stream(),
                ),
                (Some(rename), Fields::Unit) => Some(rename.into_token_stream()),
                (None, Fields::Named(FieldsNamed { named: fields, .. }))
                | (None, Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. })) => {
                    if fields.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            v,
                            "multiple data fields are not supported",
                        ));
                    }

                    if fallback.is_some() {
                        return Err(syn::Error::new_spanned(
                            v,
                            "multiple data-carrying variants are not supported",
                        ));
                    }

                    let member = match &fields[0].ident {
                        Some(name) => name.into_token_stream(),
                        None => quote! { 0 },
                    };

                    fallback = Some(quote! {
                        _ => #enum_name :: #variant_name { #member: s.into() }
                    });

                    fallback_ty = Some(&fields[0].ty);

                    None
                }
                (Some(_), _) => {
                    return Err(syn::Error::new_spanned(
                        v,
                        "ruma_enum(rename) is only allowed on unit variants",
                    ));
                }
            };

            Ok(variant_str.map(|s| quote! { #s => #enum_name :: #variant_name }))
        })
        .collect::<syn::Result<_>>()?;

    // Remove `None` from the iterator to avoid emitting consecutive commas in repetition
    let branches = branches.iter().flatten();

    if fallback.is_none() {
        return Err(syn::Error::new(Span::call_site(), "required fallback variant not found"));
    }

    Ok(quote! {
        impl<T> ::std::convert::From<T> for #enum_name
        where
            T: ::std::convert::AsRef<::std::primitive::str>
                + ::std::convert::Into<#fallback_ty>
        {
            fn from(s: T) -> Self {
                match s.as_ref() {
                    #( #branches, )*
                    #fallback
                }
            }
        }
    })
}
