//! Details of the `#[ruma_api(...)]` attributes.

use std::vec;

use syn::{
    parse::{Parse, ParseStream},
    punctuated::{Pair, Punctuated},
    Ident, Token,
};

/// Like syn::MetaNameValue, but expects an identifier as the value. Also, we don't care about the
/// the span of the equals sign, so we don't have the `eq_token` field from syn::MetaNameValue.
pub struct MetaNameValue {
    /// The part left of the equals sign
    pub name: Ident,
    /// The part right of the equals sign
    pub value: Ident,
}

/// Like syn::Meta, but only parses ruma_api attributes
pub enum Meta {
    /// A single word, like `query` in `#[ruma_api(query)]`
    Word(Ident),
    /// A name-value pair, like `header = CONTENT_TYPE` in `#[ruma_api(header = CONTENT_TYPE)]`
    NameValue(MetaNameValue),
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;

        if input.peek(Token![=]) {
            let _ = input.parse::<Token![=]>();
            Ok(Meta::NameValue(MetaNameValue {
                name: ident,
                value: input.parse()?,
            }))
        } else {
            Ok(Meta::Word(ident))
        }
    }
}

/// List of `Meta`s
pub struct MetaList(Vec<Meta>);

impl MetaList {
    /// Check if the given attribute is a ruma_api attribute. If it is, parse it.
    ///
    /// # Panics
    ///
    /// Panics if the given attribute is a ruma_api attribute, but fails to parse.
    pub fn from_attribute(attr: &syn::Attribute) -> Option<Self> {
        match &attr.path {
            syn::Path {
                leading_colon: None,
                segments,
            } => {
                if segments.len() == 1 && segments[0].ident == "ruma_api" {
                    Some(
                        attr.parse_args()
                            .expect("ruma_api! could not parse request field attributes"),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Parse for MetaList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MetaList(
            Punctuated::<Meta, Token![,]>::parse_terminated(input)?
                .into_pairs()
                .map(Pair::into_value)
                .collect(),
        ))
    }
}

impl IntoIterator for MetaList {
    type Item = Meta;
    type IntoIter = vec::IntoIter<Meta>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
