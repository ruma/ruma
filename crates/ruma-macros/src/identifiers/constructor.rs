use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};

/// A parsed identifier constructor input.
///
/// This validates the identifier at compile time and constructs it at runtime.
///
/// This is meant to be used in a `macro_rules!` macro like this:
///
/// ```
/// macro_rules! identifier {
///     ($str:literal) => {
///         $crate::proc_macros::identifier!($crate, $str)
///     };
/// }
/// ```
pub(crate) struct IdentifierConstructor {
    /// The crate where the identifier is located.
    src_crate: syn::Path,

    /// The string value of the identifier.
    str: syn::LitStr,
}

impl IdentifierConstructor {
    /// Validate the input string and generate its conversion from the string value.
    ///
    /// The conversion uses the `TryFrom<&str>` implementation of the given type at
    /// runtime.
    ///
    /// ## Parameters
    ///
    /// * `id_type`: The type of the identifier to convert to. The type must be located at the root
    ///   of the `src_crate` and may be preceded with a `&` if the output is a borrowed type.
    /// * `error_message`: The message to present if the compile-time validation fails.
    /// * `validate_fn`: The function to use to validate the string value.
    ///
    /// Panics with the given error message if the validation fails.
    pub(crate) fn validate_and_expand_str_conversion<F, T, E>(
        &self,
        id_type: &str,
        validate_fn: F,
    ) -> TokenStream
    where
        F: FnOnce(&str) -> Result<T, E>,
    {
        let (id_type, is_ref) = if let Some(id_type) = id_type.strip_prefix('&') {
            (id_type, true)
        } else {
            (id_type, false)
        };

        assert!(validate_fn(&self.str.value()).is_ok(), "Invalid {id_type}");

        let src_crate = &self.src_crate;
        let str = &self.str;
        let ampersand = is_ref.then(|| quote! { & });
        let ident = syn::Ident::new(id_type, Span::call_site());

        quote! {
            <#ampersand #src_crate::#ident as ::std::convert::TryFrom<&str>>::try_from(#str).unwrap()
        }
    }
}

impl Parse for IdentifierConstructor {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let src_crate = input.parse()?;
        let _: syn::Token![,] = input.parse()?;
        let str = input.parse()?;

        Ok(Self { src_crate, str })
    }
}
