use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn expand_partial_eq_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::std::cmp::PartialEq for #ident {
            fn eq(&self, other: &Self) -> bool {
                let other = ::std::convert::AsRef::<::std::primitive::str>::as_ref(other);
                ::std::convert::AsRef::<::std::primitive::str>::as_ref(self) == other
            }
        }
    })
}
