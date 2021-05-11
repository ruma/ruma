use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn expand_partial_ord_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    Ok(quote! {
        #[automatically_derived]
        impl ::std::cmp::PartialOrd for #ident {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                let other = ::std::convert::AsRef::<::std::primitive::str>::as_ref(other);
                ::std::convert::AsRef::<::std::primitive::str>::as_ref(self).partial_cmp(other)
            }
        }
    })
}

pub fn expand_ord_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    Ok(quote! {
        #[automatically_derived]
        impl ::std::cmp::Ord for #ident {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let other = ::std::convert::AsRef::<::std::primitive::str>::as_ref(other);
                ::std::convert::AsRef::<::std::primitive::str>::as_ref(self).cmp(other)
            }
        }
    })
}
