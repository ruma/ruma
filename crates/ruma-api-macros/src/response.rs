use std::{
    convert::{TryFrom, TryInto},
    mem,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    visit::Visit,
    DeriveInput, Field, Generics, Ident, Lifetime, Token, Type,
};

use crate::{
    attribute::{Meta, MetaNameValue, MetaValue},
    util,
};

mod incoming;
mod outgoing;

pub fn expand_derive_response(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        syn::Data::Struct(s) => s.fields,
        _ => panic!("This derive macro only works on structs"),
    };

    let fields = fields.into_iter().map(ResponseField::try_from).collect::<syn::Result<_>>()?;
    let mut error_ty = None;
    for attr in input.attrs {
        if !attr.path.is_ident("ruma_api") {
            continue;
        }

        let meta = attr.parse_args_with(Punctuated::<_, Token![,]>::parse_terminated)?;
        for MetaNameValue { name, value } in meta {
            match value {
                MetaValue::Type(t) if name == "error_ty" => {
                    error_ty = Some(t);
                }
                _ => unreachable!("invalid ruma_api({}) attribute", name),
            }
        }
    }

    let response = Response {
        ident: input.ident,
        generics: input.generics,
        fields,
        error_ty: error_ty.unwrap(),
    };

    response.check()?;
    Ok(response.expand_all())
}

struct Response {
    ident: Ident,
    generics: Generics,
    fields: Vec<ResponseField>,
    error_ty: Type,
}

impl Response {
    /// Whether or not this request has any data in the HTTP body.
    fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, ResponseField::Body(_)))
    }

    /// Returns the body field.
    fn newtype_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(ResponseField::as_newtype_body_field)
    }

    /// Returns the body field.
    fn newtype_raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(ResponseField::as_newtype_raw_body_field)
    }

    /// Whether or not this request has any data in the URL path.
    fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, &ResponseField::Header(..)))
    }

    fn expand_all(&self) -> TokenStream {
        let ruma_api = util::import_ruma_api();
        let ruma_api_macros = quote! { #ruma_api::exports::ruma_api_macros };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde = quote! { #ruma_api::exports::serde };

        let response_body_struct =
            self.fields.iter().all(|f| !matches!(f, ResponseField::NewtypeRawBody(_))).then(|| {
                let newtype_body_field =
                    self.fields.iter().find(|f| matches!(f, ResponseField::NewtypeBody(_)));
                let def = if let Some(body_field) = newtype_body_field {
                    let field =
                        Field { ident: None, colon_token: None, ..body_field.field().clone() };
                    quote! { (#field); }
                } else {
                    let fields = self.fields.iter().filter_map(|f| f.as_body_field());
                    quote! { { #(#fields),* } }
                };

                quote! {
                    /// Data in the response body.
                    #[derive(
                        Debug,
                        #ruma_api_macros::_FakeDeriveRumaApi,
                        #ruma_serde::Outgoing,
                        #serde::Deserialize,
                        #serde::Serialize,
                    )]
                    struct ResponseBody #def
                }
            });

        let outgoing_response_impl = self.expand_outgoing(&ruma_api);
        let incoming_response_impl = self.expand_incoming(&self.error_ty, &ruma_api);

        quote! {
            #response_body_struct

            #outgoing_response_impl
            #incoming_response_impl
        }
    }

    pub fn check(&self) -> syn::Result<()> {
        // TODO: highlight problematic fields

        if !self.generics.params.is_empty() || self.generics.where_clause.is_some() {
            panic!("This macro doesn't support generic types");
        }

        let newtype_body_fields = self.fields.iter().filter(|f| {
            matches!(f, ResponseField::NewtypeBody(_) | ResponseField::NewtypeRawBody(_))
        });

        let has_newtype_body_field = match newtype_body_fields.count() {
            0 => false,
            1 => true,
            _ => {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "Can't have more than one newtype body field",
                ))
            }
        };

        let has_body_fields = self.fields.iter().any(|f| matches!(f, ResponseField::Body(_)));
        if has_newtype_body_field && has_body_fields {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        Ok(())
    }
}

/// The types of fields that a response can have.
enum ResponseField {
    /// JSON data in the body of the response.
    Body(Field),

    /// Data in an HTTP header.
    Header(Field, Ident),

    /// A specific data type in the body of the response.
    NewtypeBody(Field),

    /// Arbitrary bytes in the body of the response.
    NewtypeRawBody(Field),
}

impl ResponseField {
    /// Gets the inner `Field` value.
    fn field(&self) -> &Field {
        match self {
            ResponseField::Body(field)
            | ResponseField::Header(field, _)
            | ResponseField::NewtypeBody(field)
            | ResponseField::NewtypeRawBody(field) => field,
        }
    }

    /// Return the contained field if this response field is a body kind.
    fn as_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::Body(field) => Some(field),
            _ => None,
        }
    }

    /// Return the contained field if this response field is a newtype body kind.
    fn as_newtype_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::NewtypeBody(field) => Some(field),
            _ => None,
        }
    }

    /// Return the contained field if this response field is a newtype raw body kind.
    fn as_newtype_raw_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::NewtypeRawBody(field) => Some(field),
            _ => None,
        }
    }
}

impl TryFrom<Field> for ResponseField {
    type Error = syn::Error;

    fn try_from(mut field: Field) -> syn::Result<Self> {
        if has_lifetime(&field.ty) {
            return Err(syn::Error::new_spanned(
                field.ident,
                "Lifetimes on Response fields cannot be supported until GAT are stable",
            ));
        }

        let mut field_kind = None;
        let mut header = None;

        for attr in mem::take(&mut field.attrs) {
            let meta = match Meta::from_attribute(&attr)? {
                Some(m) => m,
                None => {
                    field.attrs.push(attr);
                    continue;
                }
            };

            if field_kind.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "There can only be one field kind attribute",
                ));
            }

            field_kind = Some(match meta {
                Meta::Word(ident) => match &ident.to_string()[..] {
                    "body" => ResponseFieldKind::NewtypeBody,
                    "raw_body" => ResponseFieldKind::NewtypeRawBody,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "Invalid #[ruma_api] argument with value, expected `body`",
                        ));
                    }
                },
                Meta::NameValue(MetaNameValue { name, value }) => {
                    if name != "header" {
                        return Err(syn::Error::new_spanned(
                            name,
                            "Invalid #[ruma_api] argument with value, expected `header`",
                        ));
                    }

                    header = Some(value);
                    ResponseFieldKind::Header
                }
            });
        }

        Ok(match field_kind.unwrap_or(ResponseFieldKind::Body) {
            ResponseFieldKind::Body => ResponseField::Body(field),
            ResponseFieldKind::Header => {
                ResponseField::Header(field, header.expect("missing header name"))
            }
            ResponseFieldKind::NewtypeBody => ResponseField::NewtypeBody(field),
            ResponseFieldKind::NewtypeRawBody => ResponseField::NewtypeRawBody(field),
        })
    }
}

impl Parse for ResponseField {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.call(Field::parse_named)?.try_into()
    }
}

impl ToTokens for ResponseField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.field().to_tokens(tokens)
    }
}

/// The types of fields that a response can have, without their values.
enum ResponseFieldKind {
    Body,
    Header,
    NewtypeBody,
    NewtypeRawBody,
}

fn has_lifetime(ty: &Type) -> bool {
    struct Visitor {
        found_lifetime: bool,
    }

    impl<'ast> Visit<'ast> for Visitor {
        fn visit_lifetime(&mut self, _lt: &'ast Lifetime) {
            self.found_lifetime = true;
        }
    }

    let mut vis = Visitor { found_lifetime: false };
    vis.visit_type(ty);
    vis.found_lifetime
}
