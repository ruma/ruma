use quote::{ToTokens, Tokens};
use syn::{Expr, ExprStruct, Field, FieldValue, FieldsNamed, Member, Meta, NestedMeta};

use api::strip_serde_attrs;

pub struct Response {
    fields: Vec<ResponseField>,
}

impl Response {
    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    pub fn has_fields(&self) -> bool {
        self.fields.len() != 0
    }

    pub fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }

    pub fn init_fields(&self) -> Tokens {
        let mut tokens = Tokens::new();

        for response_field in self.fields.iter() {
            match *response_field {
                ResponseField::Body(ref field) => {
                    let field_name = match field.member {
                        Member::Named(field_name) => field_name,
                        _ => panic!("expected Member::Named"),
                    };

                    tokens.append_all(quote! {
                        #field_name: response_body.#field_name,
                    });
                }
                ResponseField::Header(ref field) => {
                    let field_name = match field.member {
                        Member::Named(field_name) => field_name,
                        _ => panic!("expected Member::Named"),
                    };

                    let field_type = match field.expr {
                        Expr::Path(ref field_type) => field_type,
                        _ => panic!("expected Expr::Path"),
                    };

                    tokens.append_all(quote! {
                        #field_name: headers.remove::<#field_type>()
                            .expect("missing expected request header"),
                    });
                }
                ResponseField::NewtypeBody(ref field) => {
                    let field_name = match field.member {
                        Member::Named(field_name) => field_name,
                        _ => panic!("expected Member::Named"),
                    };

                    tokens.append_all(quote! {
                        #field_name: response_body,
                    });
                }
            }
        }

        tokens
    }

    pub fn newtype_body_field(&self) -> Option<&FieldValue> {
        for response_field in self.fields.iter() {
            match *response_field {
                ResponseField::NewtypeBody(ref field) => {

                    return Some(field);
                }
                _ => continue,
            }
        }

        None
    }

}

impl From<ExprStruct> for Response {
    fn from(expr: ExprStruct) -> Self {
        let mut has_newtype_body = false;

        let fields = expr.fields.into_iter().map(|mut field_value| {
            let mut field_kind = ResponseFieldKind::Body;

            field_value.attrs = field_value.attrs.into_iter().filter(|attr| {
                let meta = attr.interpret_meta()
                    .expect("ruma_api! could not parse response field attributes");

                let meta_list = match meta {
                    Meta::List(meta_list) => meta_list,
                    _ => panic!("expected Meta::List"),
                };

                if meta_list.ident.as_ref() != "ruma_api" {
                    return true;
                }

                for nested_meta_item in meta_list.nested {
                    match nested_meta_item {
                        NestedMeta::Meta(meta_item) => {
                            match meta_item {
                                Meta::Word(ident) => {
                                    match ident.as_ref() {
                                        "body" => {
                                        has_newtype_body = true;
                                        field_kind = ResponseFieldKind::NewtypeBody;
                                    }
                                    "header" => field_kind = ResponseFieldKind::Header,
                                    _ => panic!(
                                            "ruma_api! attribute meta item on responses must be: header"
                                        ),
                                    }
                                }
                                _ => panic!(
                                    "ruma_api! attribute meta item on responses cannot be a list or name/value pair"
                                ),
                            }
                        }
                        NestedMeta::Literal(_) => panic!(
                            "ruma_api! attribute meta item on responses must be: header"
                        ),
                    }
                }

                false
            }).collect();

            match field_kind {
                ResponseFieldKind::Body => {
                    if has_newtype_body {
                        panic!("ruma_api! responses cannot have both normal body fields and a newtype body field");
                    } else {
                        return ResponseField::Body(field_value);
                    }
                }
                ResponseFieldKind::Header => ResponseField::Header(field_value),
                ResponseFieldKind::NewtypeBody => ResponseField::NewtypeBody(field_value),
            }
        }).collect();

        Response {
            fields,
        }
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, mut tokens: &mut Tokens) {
        tokens.append_all(quote! {
            /// Data in the response from this API endpoint.
            #[derive(Debug)]
            pub struct Response
        });

        if self.fields.len() == 0 {
            tokens.append_all(";".into_tokens());
        } else {
            tokens.append_all("{".into_tokens());

            for response_field in self.fields.iter() {
                strip_serde_attrs(response_field.field()).to_tokens(&mut tokens);

                tokens.append_all(",".into_tokens());
            }

            tokens.append_all("}".into_tokens());
        }

        if let Some(newtype_body_field) = self.newtype_body_field() {
            let mut field = newtype_body_field.clone();
            let expr = field.expr;

            tokens.append_all(quote! {
                /// Data in the response body.
                #[derive(Debug, Deserialize)]
                struct ResponseBody(#expr);
            });
        } else if self.has_body_fields() {
            tokens.append_all(quote! {
                /// Data in the response body.
                #[derive(Debug, Deserialize)]
                struct ResponseBody
            });

            tokens.append_all("{".into_tokens());

            for response_field in self.fields.iter() {
                match *response_field {
                    ResponseField::Body(ref field) => {
                        field.to_tokens(&mut tokens);

                        tokens.append_all(",".into_tokens());
                    }
                    _ => {}
                }
            }

            tokens.append_all("}".into_tokens());
        }
    }
}

pub enum ResponseField {
    Body(FieldValue),
    Header(FieldValue),
    NewtypeBody(FieldValue),
}

impl ResponseField {
    fn field(&self) -> &FieldValue {
        match *self {
            ResponseField::Body(ref field) => field,
            ResponseField::Header(ref field) => field,
            ResponseField::NewtypeBody(ref field) => field,
        }
    }

    fn is_body(&self) -> bool {
        match *self {
            ResponseField::Body(_) => true,
            _ => false,
        }
    }

    fn is_header(&self) -> bool {
        match *self {
            ResponseField::Header(_) => true,
            _ => false,
        }
    }
}

enum ResponseFieldKind {
    Body,
    Header,
    NewtypeBody,
}
