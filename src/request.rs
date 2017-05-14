use syn::{Field, Lit, MetaItem};

#[derive(Debug)]
pub struct Request {
    pub fields: Vec<RequestField>,
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

#[derive(Debug)]
pub enum RequestField {
    Body(Field),
    Header(String, Field),
    Path(String, Field),
    Query(Field),
}
