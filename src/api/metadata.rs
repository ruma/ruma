use std::convert::TryFrom;

use quote::{ToTokens, Tokens};
use syn::synom::Synom;
use syn::{Expr, ExprStruct, Ident, Member};

pub struct Metadata {
    pub description: Expr,
    pub method: Expr,
    pub name: Expr,
    pub path: Expr,
    pub rate_limited: Expr,
    pub requires_authentication: Expr,
}

impl TryFrom<ExprStruct> for Metadata {
    type Error = &'static str;

    fn try_from(expr: ExprStruct) -> Result<Self, Self::Error> {
        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut path = None;
        let mut rate_limited = None;
        let mut requires_authentication = None;

        for field in expr.fields {
            let Member::Named(identifier) = field.member;

            match identifier.as_ref() {
                "description" => description = Some(field.expr),
                "method" => method = Some(field.expr),
                "name" => name = Some(field.expr),
                "path" => path = Some(field.expr),
                "rate_limited" => rate_limited = Some(field.expr),
                "requires_authentication" => requires_authentication = Some(field.expr),
                _ => return Err("ruma_api! metadata included unexpected field"),
            }
        }

        if description.is_none() {
            return Err("ruma_api! metadata is missing description");
        }

        if method.is_none() {
            return Err("ruma_api! metadata is missing method");
        }

        if name.is_none() {
            return Err("ruma_api! metadata is missing name");
        }

        if path.is_none() {
            return Err("ruma_api! metadata is missing path");
        }

        if rate_limited.is_none() {
            return Err("ruma_api! metadata is missing rate_limited");
        }

        if requires_authentication.is_none() {
            return Err("ruma_api! metadata is missing requires_authentication");
        }

        Ok(Metadata {
            description: description.unwrap(),
            method: method.unwrap(),
            name: name.unwrap(),
            path: path.unwrap(),
            rate_limited: rate_limited.unwrap(),
            requires_authentication: requires_authentication.unwrap(),
        })
    }
}
