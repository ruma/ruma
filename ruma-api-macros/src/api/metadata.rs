//! Details of the `metadata` section of the procedural macro.

use quote::ToTokens;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, LitBool, LitStr, Token,
};

use crate::util;

mod kw {
    syn::custom_keyword!(metadata);
    syn::custom_keyword!(description);
    syn::custom_keyword!(method);
    syn::custom_keyword!(name);
    syn::custom_keyword!(path);
    syn::custom_keyword!(rate_limited);
    syn::custom_keyword!(authentication);
}

/// The result of processing the `metadata` section of the macro.
pub struct Metadata {
    /// The description field.
    pub description: LitStr,

    /// The method field.
    pub method: Ident,

    /// The name field.
    pub name: LitStr,

    /// The path field.
    pub path: LitStr,

    /// The rate_limited field.
    pub rate_limited: LitBool,

    /// The authentication field.
    pub authentication: Ident,
}

fn set_field<T: ToTokens>(field: &mut Option<T>, value: T) -> syn::Result<()> {
    match field {
        Some(existing_value) => {
            let mut error = syn::Error::new_spanned(value, "duplicate field assignment");
            error.combine(syn::Error::new_spanned(existing_value, "first one here"));
            Err(error)
        }
        None => {
            *field = Some(value);
            Ok(())
        }
    }
}

impl Parse for Metadata {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metadata_kw: kw::metadata = input.parse()?;
        let _: Token![:] = input.parse()?;

        let field_values;
        braced!(field_values in input);

        let field_values =
            field_values.parse_terminated::<FieldValue, Token![,]>(FieldValue::parse)?;

        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut path = None;
        let mut rate_limited = None;
        let mut authentication = None;

        for field_value in field_values {
            match field_value {
                FieldValue::Description(d) => set_field(&mut description, d)?,
                FieldValue::Method(m) => set_field(&mut method, m)?,
                FieldValue::Name(n) => set_field(&mut name, n)?,
                FieldValue::Path(p) => set_field(&mut path, p)?,
                FieldValue::RateLimited(rl) => set_field(&mut rate_limited, rl)?,
                FieldValue::Authentication(a) => set_field(&mut authentication, a)?,
            }
        }

        let missing_field =
            |name| syn::Error::new_spanned(metadata_kw, format!("missing field `{}`", name));

        Ok(Self {
            description: description.ok_or_else(|| missing_field("description"))?,
            method: method.ok_or_else(|| missing_field("method"))?,
            name: name.ok_or_else(|| missing_field("name"))?,
            path: path.ok_or_else(|| missing_field("path"))?,
            rate_limited: rate_limited.ok_or_else(|| missing_field("rate_limited"))?,
            authentication: authentication.ok_or_else(|| missing_field("authentication"))?,
        })
    }
}

enum Field {
    Description,
    Method,
    Name,
    Path,
    RateLimited,
    Authentication,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::description) {
            let _: kw::description = input.parse()?;
            Ok(Self::Description)
        } else if lookahead.peek(kw::method) {
            let _: kw::method = input.parse()?;
            Ok(Self::Method)
        } else if lookahead.peek(kw::name) {
            let _: kw::name = input.parse()?;
            Ok(Self::Name)
        } else if lookahead.peek(kw::path) {
            let _: kw::path = input.parse()?;
            Ok(Self::Path)
        } else if lookahead.peek(kw::rate_limited) {
            let _: kw::rate_limited = input.parse()?;
            Ok(Self::RateLimited)
        } else if lookahead.peek(kw::authentication) {
            let _: kw::authentication = input.parse()?;
            Ok(Self::Authentication)
        } else {
            Err(lookahead.error())
        }
    }
}

enum FieldValue {
    Description(LitStr),
    Method(Ident),
    Name(LitStr),
    Path(LitStr),
    RateLimited(LitBool),
    Authentication(Ident),
}

impl Parse for FieldValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: Field = input.parse()?;
        let _: Token![:] = input.parse()?;

        Ok(match field {
            Field::Description => Self::Description(input.parse()?),
            Field::Method => Self::Method(input.parse()?),
            Field::Name => Self::Name(input.parse()?),
            Field::Path => {
                let path: LitStr = input.parse()?;

                if !util::is_valid_endpoint_path(&path.value()) {
                    return Err(syn::Error::new_spanned(
                        &path,
                        "path may only contain printable ASCII characters with no spaces",
                    ));
                }

                Self::Path(path)
            }
            Field::RateLimited => Self::RateLimited(input.parse()?),
            Field::Authentication => Self::Authentication(input.parse()?),
        })
    }
}
