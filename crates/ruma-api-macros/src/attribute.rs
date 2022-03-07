//! Details of the `#[ruma_api(...)]` attributes.

use syn::{
    parse::{Parse, ParseStream},
    Ident, Lit, Token, Type,
};

/// Value type used for request and response struct attributes
#[allow(clippy::large_enum_variant)]
pub enum MetaValue {
    Lit(Lit),
    Type(Type),
}

impl Parse for MetaValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Lit) {
            input.parse().map(Self::Lit)
        } else {
            input.parse().map(Self::Type)
        }
    }
}

/// Like syn::MetaNameValue, but expects an identifier as the value.
///
/// Also, we don't care about the the span of the equals sign, so we don't have the `eq_token` field
/// from syn::MetaNameValue.
pub struct MetaNameValue<V> {
    /// The part left of the equals sign
    pub name: Ident,

    /// The part right of the equals sign
    pub value: V,
}

impl<V: Parse> Parse for MetaNameValue<V> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        Ok(MetaNameValue { name: ident, value: input.parse()? })
    }
}

/// Like syn::Meta, but only parses ruma_api attributes
pub enum Meta<T> {
    /// A single word, like `query` in `#[ruma_api(query)]`
    Word(Ident),

    /// A name-value pair, like `header = CONTENT_TYPE` in `#[ruma_api(header = CONTENT_TYPE)]`
    NameValue(MetaNameValue<T>),
}

impl<T: Parse> Meta<T> {
    /// Check if the given attribute is a ruma_api attribute.
    ///
    /// If it is, parse it.
    pub fn from_attribute(attr: &syn::Attribute) -> syn::Result<Option<Self>> {
        if attr.path.is_ident("ruma_api") {
            attr.parse_args().map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<T: Parse> Parse for Meta<T> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;

        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            Ok(Meta::NameValue(MetaNameValue { name: ident, value: input.parse()? }))
        } else {
            Ok(Meta::Word(ident))
        }
    }
}
