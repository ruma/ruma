use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::util::{RumaCommon, RumaCommonReexport};

pub fn expand_deserialize_from_cow_str(ident: &Ident) -> syn::Result<TokenStream> {
    let ruma_common = RumaCommon::new();
    let serde = ruma_common.reexported(RumaCommonReexport::Serde);

    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl<'de> #serde::de::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: #serde::de::Deserializer<'de>,
            {
                type CowStr<'a> = ::std::borrow::Cow<'a, ::std::primitive::str>;

                let cow = #ruma_common::serde::deserialize_cow_str(deserializer)?;
                Ok(::std::convert::From::<CowStr<'_>>::from(cow))
            }
        }
    })
}
