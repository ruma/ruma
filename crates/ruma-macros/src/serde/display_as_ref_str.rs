use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Generate the `std::fmt::Display` implementation for the type with the given ident, using its
/// `AsRef<str>` implementation.
pub fn expand_display_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::std::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(::std::convert::AsRef::<::std::primitive::str>::as_ref(self))
            }
        }
    })
}
