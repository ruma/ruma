use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::crate_name;
use quote::quote;

pub fn import_ruma_common() -> TokenStream {
    if let Ok(possibly_renamed) = crate_name("ruma-common") {
        let import = Ident::new(&possibly_renamed, Span::call_site());
        quote! { ::#import }
    } else if let Ok(possibly_renamed) = crate_name("ruma") {
        let import = Ident::new(&possibly_renamed, Span::call_site());
        quote! { ::#import }
    } else {
        quote! { ::ruma_common }
    }
}
