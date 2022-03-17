use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Fields, FieldsNamed, FieldsUnnamed, ItemEnum};

use super::{
    attr::EnumAttrs,
    util::{get_enum_attributes, get_rename_rule},
};

pub fn expand_enum_as_ref_str(input: &ItemEnum) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;
    let rename_rule = get_rename_rule(input)?;
    let branches: Vec<_> = input
        .variants
        .iter()
        .map(|v| {
            let variant_name = &v.ident;
            let EnumAttrs { rename, .. } = get_enum_attributes(v)?;
            let (field_capture, variant_str) = match (rename, &v.fields) {
                (None, Fields::Unit) => (
                    None,
                    rename_rule.apply_to_variant(&variant_name.to_string()).into_token_stream(),
                ),
                (Some(rename), Fields::Unit) => (None, rename.into_token_stream()),
                (None, Fields::Named(FieldsNamed { named: fields, .. }))
                | (None, Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. })) => {
                    if fields.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            v,
                            "multiple data fields are not supported",
                        ));
                    }

                    let capture = match &fields[0].ident {
                        Some(name) => quote! { { #name: inner } },
                        None => quote! { (inner) },
                    };

                    (Some(capture), quote! { &inner.0 })
                }
                (Some(_), _) => {
                    return Err(syn::Error::new_spanned(
                        v,
                        "ruma_enum(rename) is only allowed on unit variants",
                    ));
                }
            };

            Ok(quote! {
                #enum_name :: #variant_name #field_capture => #variant_str
            })
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::std::convert::AsRef<::std::primitive::str> for #enum_name {
            fn as_ref(&self) -> &::std::primitive::str {
                match self { #(#branches),* }
            }
        }
    })
}
