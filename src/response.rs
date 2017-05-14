use quote::{ToTokens, Tokens};
use syn::{Field, Lit, MetaItem};

#[derive(Debug)]
pub struct Response {
    fields: Vec<ResponseField>,
}

impl Response {
    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }
}

impl From<Vec<Field>> for Response {
    fn from(fields: Vec<Field>) -> Self {
        let response_fields = fields.into_iter().map(|field| {
            for attr in field.attrs.clone().iter() {
                match attr.value {
                    MetaItem::Word(_) | MetaItem::List(_, _) => {}
                    MetaItem::NameValue(ref ident, ref lit) => {
                        if ident == "header" {
                            if let Lit::Str(ref name, _) = *lit {
                                return ResponseField::Header(name.clone(), field);
                            } else {
                                panic!("ruma_api! header attribute expects a string value");
                            }
                        }
                    }
                }
            }

            return ResponseField::Body(field);
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
                    ResponseField::Header(_, ref field) => field.to_tokens(&mut tokens),
                }
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
            }

            tokens.append("}");
        }
    }
}

#[derive(Debug)]
pub enum ResponseField {
    Body(Field),
    Header(String, Field),
}

impl ResponseField {
    fn is_body(&self) -> bool {
        match *self {
            ResponseField::Body(_) => true,
            _ => false,
        }
    }
}
