use syn::{
    parse::{Parse, ParseStream},
    Ident, Path, Type,
};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(with);
}

/// The inside of a `#[wrap_incoming]` attribute
#[derive(Default)]
pub struct Meta {
    pub type_to_wrap: Option<Type>,
    pub wrapper_type: Option<Path>,
}

impl Meta {
    /// Check if the given attribute is a wrap_incoming attribute. If it is, parse it.
    pub fn from_attribute(attr: &syn::Attribute) -> syn::Result<Option<Self>> {
        if attr.path.is_ident("wrap_incoming") {
            if attr.tokens.is_empty() {
                Ok(Some(Self::default()))
            } else {
                attr.parse_args().map(Some)
            }
        } else {
            Ok(None)
        }
    }
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut type_to_wrap = None;
        let mut wrapper_type = try_parse_wrapper_type(input)?;

        if wrapper_type.is_none() && input.peek(Ident) {
            type_to_wrap = Some(input.parse()?);
            wrapper_type = try_parse_wrapper_type(input)?;
        }

        if input.is_empty() {
            Ok(Self { type_to_wrap, wrapper_type })
        } else {
            Err(input.error("expected end of attribute args"))
        }
    }
}

fn try_parse_wrapper_type(input: ParseStream) -> syn::Result<Option<Path>> {
    if input.peek(kw::with) {
        input.parse::<kw::with>()?;
        Ok(Some(input.parse()?))
    } else {
        Ok(None)
    }
}
