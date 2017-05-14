use quote::{ToTokens, Tokens};
use syn::{Field, Lit, MetaItem};

#[derive(Debug)]
pub struct Request {
    fields: Vec<RequestField>,
}

impl From<Vec<Field>> for Request {
    fn from(fields: Vec<Field>) -> Self {
        let request_fields = fields.into_iter().map(|field| {
            for attr in field.attrs.clone().iter() {
                match attr.value {
                    MetaItem::Word(ref ident) => {
                        if ident == "query" {
                            return RequestField::Query(field);
                        }
                    }
                    MetaItem::List(_, _) => {}
                    MetaItem::NameValue(ref ident, ref lit) => {
                        if ident == "header" {
                            if let Lit::Str(ref name, _) = *lit {
                                return RequestField::Header(name.clone(), field);
                            } else {
                                panic!("ruma_api! header attribute expects a string value");
                            }
                        } else if ident == "path" {
                            if let Lit::Str(ref name, _) = *lit {
                                return RequestField::Path(name.clone(), field);
                            } else {
                                panic!("ruma_api! path attribute expects a string value");
                            }
                        }
                    }
                }
            }

            return RequestField::Body(field);
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
            #[derive(Debug)]
            pub struct Request
        });

        if self.fields.len() == 0 {
            tokens.append(";");
        } else {
            tokens.append("{");

            for request_field in self.fields.iter() {
                match *request_field {
                    RequestField::Body(ref field) => field.to_tokens(&mut tokens),
                    RequestField::Header(_, ref field) => field.to_tokens(&mut tokens),
                    RequestField::Path(_, ref field) => field.to_tokens(&mut tokens),
                    RequestField::Query(ref field) => field.to_tokens(&mut tokens),
                }
            }

            tokens.append("}");
        }
    }
}

#[derive(Debug)]
pub enum RequestField {
    Body(Field),
    Header(String, Field),
    Path(String, Field),
    Query(Field),
}
