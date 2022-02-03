//! Details of the `metadata` section of the procedural macro.

use quote::ToTokens;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, Ident, LitBool, LitFloat, LitStr, Token,
};

use crate::{auth_scheme::AuthScheme, util};

mod kw {
    syn::custom_keyword!(metadata);
    syn::custom_keyword!(description);
    syn::custom_keyword!(method);
    syn::custom_keyword!(name);
    syn::custom_keyword!(path);
    syn::custom_keyword!(rate_limited);
    syn::custom_keyword!(authentication);
    syn::custom_keyword!(added);
    syn::custom_keyword!(deprecated);
    syn::custom_keyword!(removed);
}

/// A field of Metadata that contains attribute macros
pub struct MetadataField<T> {
    /// attributes over the field
    pub attrs: Vec<Attribute>,

    /// the field itself
    pub value: T,
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
    pub rate_limited: Vec<MetadataField<LitBool>>,

    /// The authentication field.
    pub authentication: Vec<MetadataField<AuthScheme>>,

    /// The added field.
    pub added: Option<LitFloat>,

    /// The deprecated field.
    pub deprecated: Option<LitFloat>,

    /// The removed field.
    pub removed: Option<LitFloat>,
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
        let mut rate_limited = vec![];
        let mut authentication = vec![];
        let mut added = None;
        let mut deprecated = None;
        let mut removed = None;

        for field_value in field_values {
            match field_value {
                FieldValue::Description(d) => set_field(&mut description, d)?,
                FieldValue::Method(m) => set_field(&mut method, m)?,
                FieldValue::Name(n) => set_field(&mut name, n)?,
                FieldValue::Path(p) => set_field(&mut path, p)?,
                FieldValue::RateLimited(value, attrs) => {
                    rate_limited.push(MetadataField { attrs, value });
                }
                FieldValue::Authentication(value, attrs) => {
                    authentication.push(MetadataField { attrs, value });
                }
                FieldValue::Added(v) => set_field(&mut added, v)?,
                FieldValue::Deprecated(v) => set_field(&mut deprecated, v)?,
                FieldValue::Removed(v) => set_field(&mut removed, v)?,
            }
        }

        let missing_field =
            |name| syn::Error::new_spanned(metadata_kw, format!("missing field `{}`", name));

        if let Some(deprecated) = &deprecated {
            if added.is_none() {
                return Err(syn::Error::new_spanned(
                    deprecated,
                    "deprecated version is defined while added version is not defined",
                ));
            }
        }

        // note: It is possible that matrix will remove endpoints in a single version, while not
        // having a deprecation version inbetween, but that would not be allowed by their own
        // deprecation policy, so lets just assume there's always a deprecation version before a
        // removal one.
        //
        // If matrix does so anyways, we can just alter this.
        if let Some(removed) = &removed {
            if deprecated.is_none() {
                return Err(syn::Error::new_spanned(
                    removed,
                    "removed version is defined while deprecated version is not defined",
                ));
            }
        }

        Ok(Self {
            description: description.ok_or_else(|| missing_field("description"))?,
            method: method.ok_or_else(|| missing_field("method"))?,
            name: name.ok_or_else(|| missing_field("name"))?,
            path: path.ok_or_else(|| missing_field("path"))?,
            rate_limited: if rate_limited.is_empty() {
                return Err(missing_field("rate_limited"));
            } else {
                rate_limited
            },
            authentication: if authentication.is_empty() {
                return Err(missing_field("authentication"));
            } else {
                authentication
            },
            added,
            deprecated,
            removed,
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
    Added,
    Deprecated,
    Removed,
}

macro_rules! map_kw {
    ($input:ident, $kw:path => $el:expr, $($next_kw:path => $next_el:expr,)*) => {
        let lookahead = $input.lookahead1();

        if lookahead.peek($kw) {
            let _: $kw = $input.parse()?;
            Ok($el)
        }
        $(
            else if lookahead.peek($next_kw) {
                let _: $next_kw = $input.parse()?;
                Ok($next_el)
            }
        )*
        else {
            Err(lookahead.error())
        }
    };
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        map_kw! { input,
            kw::description => Self::Description,
            kw::method => Self::Method,
            kw::name => Self::Name,
            kw::path => Self::Path,
            kw::rate_limited => Self::RateLimited,
            kw::authentication => Self::Authentication,
            kw::added => Self::Added,
            kw::deprecated => Self::Deprecated,
            kw::removed => Self::Removed,
        }
    }
}

enum FieldValue {
    Description(LitStr),
    Method(Ident),
    Name(LitStr),
    Path(LitStr),
    RateLimited(LitBool, Vec<Attribute>),
    Authentication(AuthScheme, Vec<Attribute>),
    Added(LitFloat),
    Deprecated(LitFloat),
    Removed(LitFloat),
}

impl Parse for FieldValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        for attr in attrs.iter() {
            if !util::is_cfg_attribute(attr) {
                return Err(syn::Error::new_spanned(
                    &attr,
                    "only `cfg` attributes may appear here",
                ));
            }
        }
        let field: Field = input.parse()?;
        let _: Token![:] = input.parse()?;

        fn valid_version(input: ParseStream<'_>) -> syn::Result<LitFloat> {
            let ver: LitFloat = input.parse()?;

            let _ = util::parse_matrix_version_from_literal_float(&ver)?;

            Ok(ver)
        }

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
            Field::RateLimited => Self::RateLimited(input.parse()?, attrs),
            Field::Authentication => Self::Authentication(input.parse()?, attrs),
            Field::Added => Self::Added(valid_version(input)?),
            Field::Deprecated => Self::Deprecated(valid_version(input)?),
            Field::Removed => Self::Removed(valid_version(input)?),
        })
    }
}
