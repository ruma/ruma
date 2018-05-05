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

impl From<ExprStruct> for Metadata {
    fn from(expr: ExprStruct) -> Self {
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
                _ => panic!("ruma_api! metadata included unexpected field"),
            }
        }

        Metadata {
            description: description.expect("ruma_api! `metadata` is missing `description`"),
            method: method.expect("ruma_api! `metadata` is missing `method`"),
            name: name.expect("ruma_api! `metadata` is missing `name`"),
            path: path.expect("ruma_api! `metadata` is missing `path`"),
            rate_limited: rate_limited.expect("ruma_api! `metadata` is missing `rate_limited`"),
            requires_authentication: requires_authentication
                .expect("ruma_api! `metadata` is missing `requires_authentication`"),
        }
    }
}
