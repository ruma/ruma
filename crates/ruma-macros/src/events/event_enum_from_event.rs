//! Implementation of the `EventEnumFromEvent` derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput};

/// `EventEnumFromEvent` derive macro code generation.
pub fn expand_event_enum_from_event(input: DeriveInput) -> TokenStream {
    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        panic!("this derive macro only works with enums");
    };

    let from_impls = variants.iter().map(|variant| match &variant.fields {
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let inner_struct = &fields.unnamed.first().unwrap().ty;
            let var_ident = &variant.ident;
            let id = &input.ident;
            quote! {
                #[automatically_derived]
                impl ::std::convert::From<#inner_struct> for #id {
                    fn from(c: #inner_struct) -> Self {
                        Self::#var_ident(c)
                    }
                }
            }
        }
        _ => {
            panic!("this derive macro only works with enum variants with a single unnamed field")
        }
    });

    quote! {
        #( #from_impls )*
    }
}
