use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Generate the `std::cmp::Ord` and `std::cmp::PartialOrd` implementations for the type with the
/// given ident, using its `AsRef<str>` implementation.
pub fn expand_ord_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::std::cmp::Ord for #ident {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let other = ::std::convert::AsRef::<::std::primitive::str>::as_ref(other);
                ::std::convert::AsRef::<::std::primitive::str>::as_ref(self).cmp(other)
            }
        }

        #[automatically_derived]
        #[allow(deprecated)]
        impl ::std::cmp::PartialOrd for #ident {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                ::std::option::Option::Some(::std::cmp::Ord::cmp(self, other))
            }
        }
    })
}
