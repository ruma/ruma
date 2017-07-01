use quote::{ToTokens, Tokens};
use syn::{Field, MetaItem, NestedMetaItem};

#[derive(Debug)]
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
                    let field_name = field.ident.as_ref()
                        .expect("expected body field to have a name");

                    tokens.append(quote! {
                        #field_name: response_body.#field_name,
                    });
                }
                ResponseField::Header(ref field) => {
                    let field_name = field.ident.as_ref()
                        .expect("expected body field to have a name");
                    let field_type = &field.ty;

                    tokens.append(quote! {
                        #field_name: headers.remove::<#field_type>()
                            .expect("missing expected request header"),
                    });
                }
                ResponseField::NewtypeBody(ref field) => {
                    let field_name = field.ident.as_ref()
                        .expect("expected body field to have a name");

                    tokens.append(quote! {
                        #field_name: response_body,
                    });
                }
            }
        }

        tokens
    }

    pub fn newtype_body_field(&self) -> Option<&Field> {
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

impl From<Vec<Field>> for Response {
    fn from(fields: Vec<Field>) -> Self {
        let mut has_newtype_body = false;

        let response_fields = fields.into_iter().map(|mut field| {
            let mut response_field_kind = ResponseFieldKind::Body;

            field.attrs = field.attrs.into_iter().filter(|attr| {
                let (attr_ident, nested_meta_items) = match attr.value {
                    MetaItem::List(ref attr_ident, ref nested_meta_items) => {
                        (attr_ident, nested_meta_items)
                    }
                    _ => return true,
                };

                if attr_ident != "ruma_api" {
                    return true;
                }

                for nested_meta_item in nested_meta_items {
                    match *nested_meta_item {
                        NestedMetaItem::MetaItem(ref meta_item) => {
                            match *meta_item {
                                MetaItem::Word(ref ident) => {
                                    if ident == "body" {
                                        has_newtype_body = true;
                                        response_field_kind = ResponseFieldKind::NewtypeBody;
                                    } else if ident == "header" {
                                        response_field_kind = ResponseFieldKind::Header;
                                    } else {
                                        panic!(
                                            "ruma_api! attribute meta item on responses must be: header"
                                        );
                                    }
                                }
                                _ => panic!(
                                    "ruma_api! attribute meta item on requests cannot be a list or name/value pair"
                                ),
                            }
                        }
                        NestedMetaItem::Literal(_) => panic!(
                            "ruma_api! attribute meta item on responses must be: header"
                        ),
                    }
                }

                false
            }).collect();

            match response_field_kind {
                ResponseFieldKind::Body => {
                    if has_newtype_body {
                        panic!("ruma_api! responses cannot have both normal body fields and a newtype body field");
                    } else {
                        return ResponseField::Body(field);
                    }
                }
                ResponseFieldKind::Header => ResponseField::Header(field),
                ResponseFieldKind::NewtypeBody => ResponseField::NewtypeBody(field),
            }
        }).collect();

        Response {
            fields: response_fields,
        }
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, mut tokens: &mut Tokens) {
        tokens.append(quote! {
            /// Data in the response from this API endpoint.
            #[derive(Debug)]
            pub struct Response
        });

        if self.fields.len() == 0 {
            tokens.append(";");
        } else {
            tokens.append("{");

            for response in self.fields.iter() {
                match *response {
                    ResponseField::Body(ref field) => field.to_tokens(&mut tokens),
                    ResponseField::Header(ref field) => field.to_tokens(&mut tokens),
                    ResponseField::NewtypeBody(ref field) => field.to_tokens(&mut tokens),
                }

                tokens.append(",");
            }

            tokens.append("}");
        }

        if self.has_body_fields() {
            tokens.append(quote! {
                /// Data in the response body.
                #[derive(Debug, Deserialize)]
                struct ResponseBody
            });

            tokens.append("{");

            for response_field in self.fields.iter() {
                match *response_field {
                    ResponseField::Body(ref field) => field.to_tokens(&mut tokens),
                    _ => {}
                }

                tokens.append(",");
            }

            tokens.append("}");
        }
    }
}

#[derive(Debug)]
pub enum ResponseField {
    Body(Field),
    Header(Field),
    NewtypeBody(Field),
}

impl ResponseField {
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
