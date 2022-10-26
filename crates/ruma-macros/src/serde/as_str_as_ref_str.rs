use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn expand_as_str_as_ref_str(ident: &Ident) -> syn::Result<TokenStream> {
    let as_str_doc = format!("Creates a string slice from this `{ident}`.");
    Ok(quote! {
        #[automatically_derived]
        #[allow(deprecated)]
        impl #ident {
            #[doc = #as_str_doc]
            pub fn as_str(&self) -> &str {
                self.as_ref()
            }
        }
    })
}
