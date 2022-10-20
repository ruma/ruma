use std::num::NonZeroU8;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parse, Error, LitFloat};

#[derive(Clone, Debug, PartialEq)]
pub struct MatrixVersionLiteral {
    pub(crate) major: NonZeroU8,
    pub(crate) minor: u8,
}

const ONE: NonZeroU8 = unsafe { NonZeroU8::new_unchecked(1) };

impl MatrixVersionLiteral {
    pub const V1_0: Self = Self { major: ONE, minor: 0 };
    pub const V1_1: Self = Self { major: ONE, minor: 1 };
}

impl Parse for MatrixVersionLiteral {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let fl: LitFloat = input.parse()?;

        if !fl.suffix().is_empty() {
            return Err(Error::new_spanned(
                fl,
                "matrix version has to be only two positive numbers separated by a `.`",
            ));
        }

        let ver_vec: Vec<String> = fl.to_string().split('.').map(&str::to_owned).collect();

        let ver: [String; 2] = ver_vec.try_into().map_err(|_| {
            Error::new_spanned(&fl, "did not contain only both an X and Y value like X.Y")
        })?;

        let major: NonZeroU8 = ver[0].parse().map_err(|e| {
            Error::new_spanned(&fl, format!("major number failed to parse as >0 number: {e}"))
        })?;
        let minor: u8 = ver[1]
            .parse()
            .map_err(|e| Error::new_spanned(&fl, format!("minor number failed to parse: {e}")))?;

        Ok(Self { major, minor })
    }
}

impl ToTokens for MatrixVersionLiteral {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let variant = format_ident!("V{}_{}", u8::from(self.major), self.minor);
        tokens.extend(quote! { ::ruma_common::api::MatrixVersion::#variant });
    }
}
