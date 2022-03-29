//! Methods and types for generating identifiers.

use syn::{parse::Parse, LitStr, Path, Token};

pub struct IdentifierInput {
    pub dollar_crate: Path,
    pub id: LitStr,
}

impl Parse for IdentifierInput {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let dollar_crate = input.parse()?;
        let _: Token![,] = input.parse()?;
        let id = input.parse()?;

        Ok(Self { dollar_crate, id })
    }
}
