use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn expand_debug_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                <::std::primitive::str as ::std::fmt::Debug>::fmt(
                    ::std::convert::AsRef::<::std::primitive::str>::as_ref(self),
                    f,
                )
            }
        }
    })
}
