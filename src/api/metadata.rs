use syn::punctuated::Pair;
use syn::{Expr, FieldValue, Lit, Member};

pub struct Metadata {
    pub description: String,
    pub method: String,
    pub name: String,
    pub path: String,
    pub rate_limited: bool,
    pub requires_authentication: bool,
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

            match identifier.as_ref() {
                "description" => {
                    let expr_lit = match field_value.expr {
                        Expr::Lit(expr_lit) => expr_lit,
                        _ => panic!("expected Expr::Lit"),
                    };
                    let lit_str = match expr_lit.lit {
                        Lit::Str(lit_str) => lit_str,
                        _ => panic!("expected Lit::Str"),
                    };
                    description = Some(lit_str.value());
                }
                "method" => {
                    let expr_path = match field_value.expr {
                        Expr::Path(expr_path) => expr_path,
                        _ => panic!("expected Expr::Path"),
                    };
                    let path = expr_path.path;
                    let segments = path.segments;
                    if segments.len() != 1 {
                        panic!("ruma_api! expects a one component path for `metadata` `method`");
                    }
                    let pair = segments.first().unwrap(); // safe because we just checked
                    let method_name = match pair {
                        Pair::End(method_name) => method_name,
                        _ => panic!("expected Pair::End"),
                    };
                    method = Some(method_name.ident.to_string());
                }
                "name" => {
                    let expr_lit = match field_value.expr {
                        Expr::Lit(expr_lit) => expr_lit,
                        _ => panic!("expected Expr::Lit"),
                    };
                    let lit_str = match expr_lit.lit {
                        Lit::Str(lit_str) => lit_str,
                        _ => panic!("expected Lit::Str"),
                    };
                    name = Some(lit_str.value());
                }
                "path" => {
                    let expr_lit = match field_value.expr {
                        Expr::Lit(expr_lit) => expr_lit,
                        _ => panic!("expected Expr::Lit"),
                    };
                    let lit_str = match expr_lit.lit {
                        Lit::Str(lit_str) => lit_str,
                        _ => panic!("expected Lit::Str"),
                    };
                    path = Some(lit_str.value());
                }
                "rate_limited" => {
                    let expr_lit = match field_value.expr {
                        Expr::Lit(expr_lit) => expr_lit,
                        _ => panic!("expected Expr::Lit"),
                    };
                    let lit_bool = match expr_lit.lit {
                        Lit::Bool(lit_bool) => lit_bool,
                        _ => panic!("expected Lit::Bool"),
                    };
                    rate_limited = Some(lit_bool.value)
                }
                "requires_authentication" => {
                    let expr_lit = match field_value.expr {
                        Expr::Lit(expr_lit) => expr_lit,
                        _ => panic!("expected Expr::Lit"),
                    };
                    let lit_bool = match expr_lit.lit {
                        Lit::Bool(lit_bool) => lit_bool,
                        _ => panic!("expected Lit::Bool"),
                    };
                    requires_authentication = Some(lit_bool.value)
                }
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
