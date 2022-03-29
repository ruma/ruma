use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::util::import_ruma_common;

pub fn expand_serialize_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    let ruma_common = import_ruma_common();

    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl #ruma_common::exports::serde::ser::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: #ruma_common::exports::serde::ser::Serializer,
            {
                ::std::convert::AsRef::<::std::primitive::str>::as_ref(self).serialize(serializer)
            }
        }
    })
}
