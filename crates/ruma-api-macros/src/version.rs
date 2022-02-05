use std::{convert::TryInto, num::NonZeroU8, str::FromStr};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, Error, LitFloat};

#[derive(Clone)]
pub struct MatrixVersionLiteral {
    maj: NonZeroU8,
    min: u8,
}

impl Parse for MatrixVersionLiteral {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let fl: LitFloat = input.parse()?;

        if !fl.suffix().is_empty() {
            return Err(Error::new_spanned(
                fl,
                "matrix version variable contained invalid float suffix",
            ));
        }

        let ver_vec: Vec<&str> = fl.base10_digits().split('.').collect();

        let ver: [&str; 2] = ver_vec.try_into().map_err(|_| {
            Error::new_spanned(&fl, "did not contain only both an X and Y value like X.Y")
        })?;

        let maj: NonZeroU8 = ver[0].parse().map_err(|e| {
            Error::new_spanned(&fl, format!("major number failed to parse as >0 number: {}", e))
        })?;
        let min: u8 = ver[1]
            .parse()
            .map_err(|e| Error::new_spanned(&fl, format!("minor number failed to parse: {}", e)))?;

        Ok(Self { maj, min })
    }
}

impl ToTokens for MatrixVersionLiteral {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(
            TokenStream::from_str(&format!(
                "::ruma_api::MatrixVersion::V{}_{}",
                self.maj, self.min
            ))
            .expect("is statically defined"),
        );
    }
}
