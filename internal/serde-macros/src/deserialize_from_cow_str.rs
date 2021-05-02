use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::util::import_ruma_serde;

pub fn expand_deserialize_from_cow_str(ident: &Ident) -> syn::Result<TokenStream> {
    let ruma_serde = import_ruma_serde();

    Ok(quote! {
        impl<'de> #ruma_serde::exports::serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: #ruma_serde::exports::serde::de::Deserializer<'de>,
            {
                type CowStr<'a> = ::std::borrow::Cow<'a, ::std::primitive::str>;

                let cow = #ruma_serde::deserialize_cow_str(deserializer)?;
                Ok(::std::convert::From::<CowStr<'_>>::from(cow))
            }
        }
    })
}
