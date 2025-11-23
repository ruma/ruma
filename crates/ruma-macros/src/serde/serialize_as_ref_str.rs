use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::util::{RumaCommon, RumaCommonReexport};

/// Generate the `serde::ser::Serialize` implementation for the type with the given ident, using its
/// `AsRef<str>` implementation.
pub fn expand_serialize_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    let ruma_common = RumaCommon::new();
    let serde = ruma_common.reexported(RumaCommonReexport::Serde);

    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl #serde::ser::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: #serde::ser::Serializer,
            {
                ::std::convert::AsRef::<::std::primitive::str>::as_ref(self).serialize(serializer)
            }
        }
    })
}
