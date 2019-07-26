//! Details of the `metadata` section of the procedural macro.

use proc_macro2::Ident;
use syn::{Expr, ExprLit, FieldValue, Lit, LitBool, LitStr, Member};

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

impl From<Vec<FieldValue>> for Metadata {
    fn from(field_values: Vec<FieldValue>) -> Self {
        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut path = None;
        let mut rate_limited = None;
        let mut requires_authentication = None;

        for field_value in field_values {
            let identifier = match field_value.member {
                Member::Named(identifier) => identifier,
                _ => panic!("expected Member::Named"),
            };

            match &identifier.to_string()[..] {
                "description" => {
                    let literal = match field_value.expr {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) => s,
                        _ => panic!("expected string literal"),
                    };
                    description = Some(literal);
                }
                "method" => {
                    let expr_path = match field_value.expr {
                        Expr::Path(expr_path) => expr_path,
                        _ => panic!("expected Expr::Path"),
                    };
                    let path = expr_path.path;
                    let mut segments = path.segments.iter();
                    let method_name = segments.next().expect("expected non-empty path");
                    assert!(
                        segments.next().is_none(),
                        "ruma_api! expects a one-component path for `metadata` `method`"
                    );
                    method = Some(method_name.ident.clone());
                }
                "name" => {
                    let literal = match field_value.expr {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) => s,
                        _ => panic!("expected string literal"),
                    };
                    name = Some(literal);
                }
                "path" => {
                    let literal = match field_value.expr {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) => s,
                        _ => panic!("expected string literal"),
                    };
                    path = Some(literal);
                }
                "rate_limited" => {
                    let literal = match field_value.expr {
                        Expr::Lit(ExprLit {
                            lit: Lit::Bool(b), ..
                        }) => b,
                        _ => panic!("expected Expr::Lit"),
                    };
                    rate_limited = Some(literal)
                }
                "requires_authentication" => {
                    let literal = match field_value.expr {
                        Expr::Lit(ExprLit {
                            lit: Lit::Bool(b), ..
                        }) => b,
                        _ => panic!("expected Expr::Lit"),
                    };
                    requires_authentication = Some(literal)
                }
                _ => panic!("ruma_api! metadata included unexpected field"),
            }
        }

        Self {
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
