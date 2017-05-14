use syn::{Field, Lit, MetaItem};

#[derive(Debug)]
pub struct Response {
    pub fields: Vec<ResponseField>,
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

#[derive(Debug)]
pub enum ResponseField {
    Body(Field),
    Header(String, Field),
}
