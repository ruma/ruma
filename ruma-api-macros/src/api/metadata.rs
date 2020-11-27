//! Details of the `metadata` section of the procedural macro.

use syn::{
    braced,
    parse::{Parse, ParseStream},
    Expr, ExprLit, ExprPath, FieldValue, Ident, Lit, LitBool, LitStr, Member, Token,
};

use crate::util;

mod kw {
    syn::custom_keyword!(metadata);
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

impl Parse for Metadata {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let metadata_kw = input.parse::<kw::metadata>()?;
        input.parse::<Token![:]>()?;
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
            let identifier = match field_value.member.clone() {
                Member::Named(identifier) => identifier,
                _ => panic!("expected Member::Named"),
            };
            let expr = field_value.expr.clone();

            match &identifier.to_string()[..] {
                "description" => match expr {
                    Expr::Lit(ExprLit { lit: Lit::Str(literal), .. }) => {
                        description = Some(literal);
                    }
                    _ => return Err(syn::Error::new_spanned(expr, "expected a string literal")),
                },
                "method" => match expr {
                    Expr::Path(ExprPath { ref path, .. }) if path.segments.len() == 1 => {
                        method = Some(path.segments[0].ident.clone());
                    }
                    _ => return Err(syn::Error::new_spanned(expr, "expected an identifier")),
                },
                "name" => match expr {
                    Expr::Lit(ExprLit { lit: Lit::Str(literal), .. }) => {
                        name = Some(literal);
                    }
                    _ => return Err(syn::Error::new_spanned(expr, "expected a string literal")),
                },
                "path" => match expr {
                    Expr::Lit(ExprLit { lit: Lit::Str(literal), .. }) => {
                        let path_str = literal.value();
                        if !util::is_valid_endpoint_path(&path_str) {
                            return Err(syn::Error::new_spanned(
                                literal,
                                "path may only contain printable ASCII characters with no spaces",
                            ));
                        }
                        path = Some(literal);
                    }
                    _ => return Err(syn::Error::new_spanned(expr, "expected a string literal")),
                },
                "rate_limited" => match expr {
                    Expr::Lit(ExprLit { lit: Lit::Bool(literal), .. }) => {
                        rate_limited = Some(literal);
                    }
                    _ => return Err(syn::Error::new_spanned(expr, "expected a bool literal")),
                },
                "authentication" => match expr {
                    Expr::Path(ExprPath { ref path, .. }) if path.segments.len() == 1 => {
                        authentication = Some(path.segments[0].ident.clone());
                    }
                    _ => return Err(syn::Error::new_spanned(expr, "expected an identifier")),
                },
                _ => return Err(syn::Error::new_spanned(field_value, "unexpected field")),
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
