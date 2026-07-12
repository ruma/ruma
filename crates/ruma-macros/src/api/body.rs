use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::util::{RumaCommon, RumaCommonReexport};

pub(crate) fn expand_derive_outgoing_body_json(input: DeriveInput) -> TokenStream {
    let ruma_common = RumaCommon::new();
    let bytes = ruma_common.reexported(RumaCommonReexport::Bytes);
    let serde_json = ruma_common.reexported(RumaCommonReexport::SerdeJson);
    let ident = input.ident;

    quote! {
        #[automatically_derived]
        impl #ruma_common::api::OutgoingBody for #ident {
            type Error = #serde_json::Error;

            fn try_into_buf<T>(self) -> #serde_json::Result<T>
            where
                T: ::std::default::Default
                    + #bytes::BufMut
                    + ::std::convert::AsRef<[::std::primitive::u8]>,
            {
                #ruma_common::serde::json_to_buf(&self)
            }
        }
    }
}
