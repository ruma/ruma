use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::util::import_ruma_serde;

pub fn expand_serialize_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    let ruma_serde = import_ruma_serde();

    Ok(quote! {
        #[automatically_derived]
        impl #ruma_serde::exports::serde::ser::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: #ruma_serde::exports::serde::ser::Serializer,
            {
                <Self as ::std::convert::AsRef<::std::primitive::str>>::as_ref(self)
                    .serialize(serializer)
            }
        }
    })
}
