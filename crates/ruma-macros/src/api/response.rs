use std::ops::Not;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    visit::Visit,
    DeriveInput, Field, Generics, Ident, Lifetime, Token, Type,
};

use super::attribute::{DeriveResponseMeta, ResponseMeta};
use crate::util::import_ruma_common;

mod incoming;
mod outgoing;

pub fn expand_derive_response(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        syn::Data::Struct(s) => s.fields,
        _ => panic!("This derive macro only works on structs"),
    };

    let fields = fields.into_iter().map(ResponseField::try_from).collect::<syn::Result<_>>()?;
    let mut manual_body_serde = false;
    let mut error_ty = None;
    for attr in input.attrs {
        if !attr.path.is_ident("ruma_api") {
            continue;
        }

        let metas =
            attr.parse_args_with(Punctuated::<DeriveResponseMeta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                DeriveResponseMeta::ManualBodySerde => manual_body_serde = true,
                DeriveResponseMeta::ErrorTy(t) => error_ty = Some(t),
            }
        }
    }

    let response = Response {
        ident: input.ident,
        generics: input.generics,
        fields,
        manual_body_serde,
        error_ty: error_ty.unwrap(),
    };

    response.check()?;
    Ok(response.expand_all())
}

struct Response {
    ident: Ident,
    generics: Generics,
    fields: Vec<ResponseField>,
    manual_body_serde: bool,
    error_ty: Type,
}

impl Response {
    /// Whether or not this request has any data in the HTTP body.
    fn has_body_fields(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f, ResponseField::Body(_) | &ResponseField::NewtypeBody(_)))
    }

    /// Whether or not this request has a single newtype body field.
    fn has_newtype_body(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, ResponseField::NewtypeBody(_)))
    }

    /// Whether or not this request has a single raw body field.
    fn has_raw_body(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, ResponseField::RawBody(_)))
    }

    /// Whether or not this request has any data in the URL path.
    fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|f| matches!(f, &ResponseField::Header(..)))
    }

    fn expand_all(&self) -> TokenStream {
        let ruma_common = import_ruma_common();
        let ruma_macros = quote! { #ruma_common::exports::ruma_macros };
        let serde = quote! { #ruma_common::exports::serde };

        let response_body_struct = (!self.has_raw_body()).then(|| {
            let serde_derives = self.manual_body_serde.not().then(|| {
                quote! {
                    #[cfg_attr(feature = "client", derive(#serde::Deserialize))]
                    #[cfg_attr(feature = "server", derive(#serde::Serialize))]
                }
            });

            let serde_attr = self.has_newtype_body().then(|| quote! { #[serde(transparent)] });
            let fields = self.fields.iter().filter_map(ResponseField::as_body_field);

            quote! {
                /// Data in the response body.
                #[derive(Debug, #ruma_macros::_FakeDeriveRumaApi, #ruma_macros::_FakeDeriveSerde)]
                #serde_derives
                #serde_attr
                struct ResponseBody { #(#fields),* }
            }
        });

        let outgoing_response_impl = self.expand_outgoing(&ruma_common);
        let incoming_response_impl = self.expand_incoming(&self.error_ty, &ruma_common);

        quote! {
            #response_body_struct

            #outgoing_response_impl
            #incoming_response_impl
        }
    }

    pub fn check(&self) -> syn::Result<()> {
        // TODO: highlight problematic fields

        assert!(
            self.generics.params.is_empty() && self.generics.where_clause.is_none(),
            "This macro doesn't support generic types"
        );

        let newtype_body_fields = self
            .fields
            .iter()
            .filter(|f| matches!(f, ResponseField::NewtypeBody(_) | ResponseField::RawBody(_)));

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
    RawBody(Field),
}

impl ResponseField {
    /// Creates a new `ResponseField`.
    fn new(field: Field, kind_attr: Option<ResponseMeta>) -> Self {
        if let Some(attr) = kind_attr {
            match attr {
                ResponseMeta::NewtypeBody => ResponseField::NewtypeBody(field),
                ResponseMeta::RawBody => ResponseField::RawBody(field),
                ResponseMeta::Header(header) => ResponseField::Header(field, header),
            }
        } else {
            ResponseField::Body(field)
        }
    }

    /// Gets the inner `Field` value.
    fn field(&self) -> &Field {
        match self {
            ResponseField::Body(field)
            | ResponseField::Header(field, _)
            | ResponseField::NewtypeBody(field)
            | ResponseField::RawBody(field) => field,
        }
    }

    /// Return the contained field if this response field is a body kind.
    fn as_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::Body(field) | ResponseField::NewtypeBody(field) => Some(field),
            _ => None,
        }
    }

    /// Return the contained field if this response field is a raw body kind.
    fn as_raw_body_field(&self) -> Option<&Field> {
        match self {
            ResponseField::RawBody(field) => Some(field),
            _ => None,
        }
    }

    /// Return the contained field and HTTP header ident if this response field is a header kind.
    fn as_header_field(&self) -> Option<(&Field, &Ident)> {
        match self {
            ResponseField::Header(field, ident) => Some((field, ident)),
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

        let (mut api_attrs, attrs) =
            field.attrs.into_iter().partition::<Vec<_>, _>(|attr| attr.path.is_ident("ruma_api"));
        field.attrs = attrs;

        let kind_attr = match api_attrs.as_slice() {
            [] => None,
            [_] => Some(api_attrs.pop().unwrap().parse_args::<ResponseMeta>()?),
            _ => {
                return Err(syn::Error::new_spanned(
                    &api_attrs[1],
                    "multiple field kind attribute found, there can only be one",
                ));
            }
        };

        Ok(ResponseField::new(field, kind_attr))
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
