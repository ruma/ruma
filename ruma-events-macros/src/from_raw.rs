//! Implementation of the `FromRaw` derive macro

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, DeriveInput, Fields};

/// Create a `FromRaw` implementation for a struct
pub fn expand_from_raw(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fs) => fs.named,
            _ => panic!("#[derive(FromRaw)] only supports structs with named fields!"),
        },
        _ => panic!("#[derive(FromRaw)] only supports structs!"),
    };
    let ident = &input.ident;

    let raw_content = {
        let fields = fields.iter();
        quote! {
            #[derive(Clone, Debug, serde::Deserialize)]
            pub struct #ident {
                #(#fields),*
            }
        }
    };

    let init_list = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_span = field.span();

        if field_ident == "content" {
            quote_spanned! {field_span=>
                content: ::ruma_events::FromRaw::from_raw(raw.content),
            }
        } else if field_ident == "prev_content" {
            quote_spanned! {field_span=>
                prev_content: raw.prev_content.map(::ruma_events::FromRaw::from_raw),
            }
        } else {
            quote_spanned! {field_span=>
                #field_ident: raw.#field_ident,
            }
        }
    });

    Ok(quote! {
        impl ::ruma_events::FromRaw for #ident {
            type Raw = raw::#ident;

            fn from_raw(raw: raw::#ident) -> Self {
                Self {
                    #(#init_list)*
                }
            }
        }

        pub(crate) mod raw {
            use super::*;

            #raw_content
        }
    })
}
