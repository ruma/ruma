use quote::{ToTokens, Tokens};
use syn::{Field, MetaItem, NestedMetaItem};

#[derive(Debug)]
pub struct Request {
    fields: Vec<RequestField>,
}

impl Request {
    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    pub fn has_query_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_query())
    }

    pub fn newtype_body_field(&self) -> Option<&Field> {
        for request_field in self.fields.iter() {
            match *request_field {
                RequestField::NewtypeBody(ref field) => {

                    return Some(field);
                }
                _ => continue,
            }
        }

        None
    }

    pub fn request_body_init_fields(&self) -> Tokens {
        let mut tokens = Tokens::new();

        for request_field in self.body_fields() {
            let field =  match *request_field {
                RequestField::Body(ref field) => field,
                _ => panic!("expected body field"),
            };

            let field_name = field.ident.as_ref().expect("expected body field to have a name");

            tokens.append(quote! {
                #field_name: request.#field_name,
            });
        }

        tokens
    }

    pub fn request_query_init_fields(&self) -> Tokens {
        let mut tokens = Tokens::new();

        for query_field in self.query_fields() {
            let field = match *query_field {
                RequestField::Query(ref field) => field,
                _ => panic!("expected query field"),
            };

            let field_name = field.ident.as_ref().expect("expected query field to have a name");

            tokens.append(quote! {
                #field_name: request.#field_name,
            });
        }

        tokens
    }

    fn body_fields(&self) -> RequestBodyFields {
        RequestBodyFields::new(&self.fields)
    }

    fn query_fields(&self) -> RequestQueryFields {
        RequestQueryFields::new(&self.fields)
    }
}

impl From<Vec<Field>> for Request {
    fn from(fields: Vec<Field>) -> Self {
        let mut has_newtype_body = false;

        let request_fields = fields.into_iter().map(|mut field| {
            let mut request_field_kind = RequestFieldKind::Body;

            field.attrs = field.attrs.into_iter().filter(|attr| {
                let (attr_ident, nested_meta_items) = match attr.value {
                    MetaItem::List(ref attr_ident, ref nested_meta_items) => (attr_ident, nested_meta_items),
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
                                        request_field_kind = RequestFieldKind::NewtypeBody;
                                    } else if ident == "header" {
                                        request_field_kind = RequestFieldKind::Header;
                                    } else if ident == "path" {
                                        request_field_kind = RequestFieldKind::Path;
                                    } else if ident == "query" {
                                        request_field_kind = RequestFieldKind::Query;
                                    } else {
                                        panic!(
                                            "ruma_api! attribute meta item on requests must be: body, header, path, or query"
                                        );
                                    }
                                }
                                _ => panic!(
                                    "ruma_api! attribute meta item on requests cannot be a list or name/value pair"
                                ),
                            }
                        }
                        NestedMetaItem::Literal(_) => panic!(
                            "ruma_api! attribute meta item on requests must be: body, header, path, or query"
                        ),
                    }
                }

                false
            }).collect();

            match request_field_kind {
                RequestFieldKind::Body => {
                    if has_newtype_body {
                        panic!("ruma_api! requests cannot have both normal body fields and a newtype body field");
                    } else {
                        return RequestField::Body(field);
                    }
                }
                RequestFieldKind::Header => RequestField::Header(field),
                RequestFieldKind::NewtypeBody => RequestField::NewtypeBody(field),
                RequestFieldKind::Path => RequestField::Path(field),
                RequestFieldKind::Query => RequestField::Query(field),
            }
        }).collect();

        Request {
            fields: request_fields,
        }
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, mut tokens: &mut Tokens) {
        tokens.append(quote! {
            /// Data for a request to this API endpoint.
            #[derive(Debug, Serialize)]
            pub struct Request
        });

        if self.fields.len() == 0 {
            tokens.append(";");
        } else {
            tokens.append("{");

            for request_field in self.fields.iter() {
                match *request_field {
                    RequestField::Body(ref field) => field.to_tokens(&mut tokens),
                    RequestField::Header(ref field) => field.to_tokens(&mut tokens),
                    RequestField::NewtypeBody(ref field) => field.to_tokens(&mut tokens),
                    RequestField::Path(ref field) => field.to_tokens(&mut tokens),
                    RequestField::Query(ref field) => field.to_tokens(&mut tokens),
                }

                tokens.append(",");
            }

            tokens.append("}");
        }

        if let Some(newtype_body_field) = self.newtype_body_field() {
            let mut field = newtype_body_field.clone();

            field.ident = None;

            tokens.append(quote! {
                /// Data in the request body.
                #[derive(Debug, Serialize)]
                struct RequestBody
            });

            tokens.append("(");

            field.to_tokens(&mut tokens);

            tokens.append(");");
        } else if self.has_body_fields() {
            tokens.append(quote! {
                /// Data in the request body.
                #[derive(Debug, Serialize)]
                struct RequestBody
            });

            tokens.append("{");

            for request_field in self.fields.iter() {
                match *request_field {
                    RequestField::Body(ref field) => {
                        field.to_tokens(&mut tokens);

                        tokens.append(",");
                    }
                    _ => {}
                }
            }

            tokens.append("}");
        }

        if self.has_query_fields() {
            tokens.append(quote! {
                /// Data in the request url's query parameters
                #[derive(Debug, Serialize)]
                struct RequestQuery
            });

            tokens.append("{");

            for request_field in self.fields.iter() {
                match *request_field {
                    RequestField::Query(ref field) => {
                        field.to_tokens(&mut tokens);

                        tokens.append(",");
                    }
                    _ => {}
                }
            }

            tokens.append("}");
        }
    }
}

#[derive(Debug)]
pub enum RequestField {
    Body(Field),
    Header(Field),
    NewtypeBody(Field),
    Path(Field),
    Query(Field),
}

impl RequestField {
    fn is_body(&self) -> bool {
        match *self {
            RequestField::Body(_) => true,
            _ => false,
        }
    }

    fn is_query(&self) -> bool {
        match *self {
            RequestField::Query(_) => true,
            _ => false,
        }
    }
}

enum RequestFieldKind {
    Body,
    Header,
    NewtypeBody,
    Path,
    Query,
}

#[derive(Debug)]
pub struct RequestBodyFields<'a> {
    fields: &'a [RequestField],
    index: usize,
}

impl<'a> RequestBodyFields<'a> {
    pub fn new(fields: &'a [RequestField]) -> Self {
        RequestBodyFields {
            fields,
            index: 0,
        }
    }
}

impl<'a> Iterator for RequestBodyFields<'a> {
    type Item = &'a RequestField;

    fn next(&mut self) -> Option<&'a RequestField> {
        while let Some(value) = self.fields.get(self.index) {
            self.index += 1;

            if value.is_body() {
                return Some(value);
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct RequestQueryFields<'a> {
    fields: &'a [RequestField],
    index: usize,
}

impl<'a> RequestQueryFields<'a> {
    pub fn new(fields: &'a [RequestField]) -> Self {
        RequestQueryFields {
            fields,
            index: 0,
        }
    }
}

impl<'a> Iterator for RequestQueryFields<'a> {
    type Item = &'a RequestField;

    fn next(&mut self) -> Option<&'a RequestField> {
        while let Some(value) = self.fields.get(self.index) {
            self.index += 1;

            if value.is_query() {
                return Some(value);
            }
        }

        None
    }
}
