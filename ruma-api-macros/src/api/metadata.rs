//! Details of the `metadata` section of the procedural macro.

use std::convert::TryFrom;

use syn::{Expr, ExprLit, ExprPath, Ident, Lit, LitBool, LitStr, Member};

use crate::{api::RawMetadata, util};

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
    /// The description field.
    pub requires_authentication: LitBool,
}

impl TryFrom<RawMetadata> for Metadata {
    type Error = syn::Error;

    fn try_from(raw: RawMetadata) -> syn::Result<Self> {
        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut path = None;
        let mut rate_limited = None;
        let mut requires_authentication = None;

        for field_value in raw.field_values {
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
                        if !util::is_ascii_printable(&path_str) || path_str.contains(' ') {
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
                "requires_authentication" => match expr {
                    Expr::Lit(ExprLit { lit: Lit::Bool(literal), .. }) => {
                        requires_authentication = Some(literal);
                    }
                    _ => return Err(syn::Error::new_spanned(expr, "expected a bool literal")),
                },
                _ => return Err(syn::Error::new_spanned(field_value, "unexpected field")),
            }
        }

        let metadata_kw = raw.metadata_kw;
        let missing_field =
            |name| syn::Error::new_spanned(metadata_kw, format!("missing field `{}`", name));

        Ok(Self {
            description: description.ok_or_else(|| missing_field("description"))?,
            method: method.ok_or_else(|| missing_field("method"))?,
            name: name.ok_or_else(|| missing_field("name"))?,
            path: path.ok_or_else(|| missing_field("path"))?,
            rate_limited: rate_limited.ok_or_else(|| missing_field("rate_limited"))?,
            requires_authentication: requires_authentication
                .ok_or_else(|| missing_field("requires_authentication"))?,
        })
    }
}
