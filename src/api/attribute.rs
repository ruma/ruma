//! Details of the `#[ruma_api(...)]` attributes.

use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Ident, Token,
};

/// Like syn::Meta, but only parses ruma_api attributes
pub enum Meta {
    /// A single word, like `query` in `#[ruma_api(query)]`
    Word(Ident),
    /// A name-value pair, like `header = CONTENT_TYPE` in `#[ruma_api(header = CONTENT_TYPE)]`
    NameValue(MetaNameValue),
}

impl Meta {
    /// Check if the given attribute is a ruma_api attribute. If it is, parse it, if not, return
    /// it unchanged. Panics if the argument is an invalid ruma_api attribute.
    pub fn from_attribute(attr: syn::Attribute) -> Result<Self, syn::Attribute> {
        match &attr.path {
            syn::Path {
                leading_colon: None,
                segments,
            } => {
                if segments.len() == 1 && segments[0].ident == "ruma_api" {
                    Ok(syn::parse2(attr.tts)
                        .expect("ruma_api! could not parse request field attributes"))
                } else {
                    Err(attr)
                }
            }
            _ => Err(attr),
        }
    }
}

/// Like syn::MetaNameValue, but expects an identifier as the value. Also, we don't care about the
/// the span of the equals sign, so we don't have the `eq_token` field from syn::MetaNameValue.
pub struct MetaNameValue {
    /// The part left of the equals sign
    pub name: Ident,
    /// The part right of the equals sign
    pub value: Ident,
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let ident = content.parse()?;

        if content.peek(Token![=]) {
            let _ = content.parse::<Token![=]>();
            Ok(Meta::NameValue(MetaNameValue {
                name: ident,
                value: content.parse()?,
            }))
        } else {
            Ok(Meta::Word(ident))
        }
    }
}
