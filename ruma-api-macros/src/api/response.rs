//! Details of the `response` section of the procedural macro.

use std::{collections::BTreeSet, convert::TryFrom, mem};

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Field, Ident, Lifetime};

use crate::{
    api::{
        attribute::{Meta, MetaNameValue},
        strip_serde_attrs, RawResponse,
    },
    util,
};

/// The result of processing the `response` section of the macro.
pub struct Response {
    /// The fields of the response.
    fields: Vec<ResponseField>,

    /// The collected lifetime identifiers from the declared fields.
    lifetimes: Vec<Lifetime>,
}

impl Response {
    /// Whether or not this response has any data in the HTTP body.
    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    /// Whether or not this response has any data in HTTP headers.
    pub fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }

    /// Whether any field has a lifetime.
    pub fn contains_lifetimes(&self) -> bool {
        self.fields.iter().any(|f| util::has_lifetime(&f.field().ty))
    }

    pub fn lifetimes(&self) -> impl Iterator<Item = &Lifetime> {
        self.lifetimes.iter()
    }

    /// Produces code for a response struct initializer.
    pub fn init_fields(&self) -> TokenStream {
        let mut fields = vec![];
        let mut new_type_raw_body = None;
        for response_field in &self.fields {
            let field = response_field.field();
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let span = field.span();
            let cfg_attrs =
                field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

            fields.push(match response_field {
                ResponseField::Body(_) => {
                    quote_spanned! {span=>
                        #( #cfg_attrs )*
                        #field_name: response_body.#field_name
                    }
                }
                ResponseField::Header(_, header_name) => {
                    quote_spanned! {span=>
                        #field_name: ::ruma_api::try_deserialize!(
                            response,
                            headers.remove(::ruma_api::exports::http::header::#header_name)
                                .expect("response missing expected header")
                                .to_str()
                            )
                            .to_owned()
                    }
                }
                ResponseField::NewtypeBody(_) => {
                    quote_spanned! {span=>
                        #field_name: response_body.0
                    }
                }
                // This field must be instantiated last to avoid `use of move value` error.
                // We are guaranteed only one new body field because of a check in `try_from`.
                ResponseField::NewtypeRawBody(_) => {
                    new_type_raw_body = Some(quote_spanned! {span=>
                        #field_name: response.into_body()
                    });
                    // skip adding to the vec
                    continue;
                }
            });
        }

        fields.extend(new_type_raw_body);

        quote! {
            #(#fields,)*
        }
    }

    /// Produces code to add necessary HTTP headers to an `http::Response`.
    pub fn apply_header_fields(&self) -> TokenStream {
        let header_calls = self.fields.iter().filter_map(|response_field| {
            if let ResponseField::Header(ref field, ref header_name) = *response_field {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let span = field.span();

                Some(quote_spanned! {span=>
                    .header(::ruma_api::exports::http::header::#header_name, response.#field_name)
                })
            } else {
                None
            }
        });

        quote! { #(#header_calls)* }
    }

    /// Produces code to initialize the struct that will be used to create the response body.
    pub fn to_body(&self) -> TokenStream {
        if let Some(field) = self.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let span = field.span();
            return quote_spanned!(span=> response.#field_name);
        }

        let body = if let Some(field) = self.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let span = field.span();
            quote_spanned!(span=> response.#field_name)
        } else {
            let fields = self.fields.iter().filter_map(|response_field| {
                if let ResponseField::Body(ref field) = *response_field {
                    let field_name =
                        field.ident.as_ref().expect("expected field to have an identifier");
                    let span = field.span();
                    let cfg_attrs =
                        field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                    Some(quote_spanned! {span=>
                        #( #cfg_attrs )*
                        #field_name: response.#field_name
                    })
                } else {
                    None
                }
            });

            quote! {
                ResponseBody { #(#fields),* }
            }
        };

        quote!(::ruma_api::exports::serde_json::to_vec(&#body)?)
    }

    /// Gets the newtype body field, if this response has one.
    pub fn newtype_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(ResponseField::as_newtype_body_field)
    }

    /// Gets the newtype raw body field, if this response has one.
    pub fn newtype_raw_body_field(&self) -> Option<&Field> {
        self.fields.iter().find_map(ResponseField::as_newtype_raw_body_field)
    }
}

impl TryFrom<RawResponse> for Response {
    type Error = syn::Error;

    fn try_from(raw: RawResponse) -> syn::Result<Self> {
        let mut newtype_body_field = None;
        let mut lifetimes = BTreeSet::new();

        let fields = raw
            .fields
            .into_iter()
            .map(|mut field| {
                let mut field_kind = None;
                let mut header = None;

                util::copy_lifetime_ident(&mut lifetimes, &field.ty);

                for attr in mem::replace(&mut field.attrs, Vec::new()) {
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
                            s @ "body" | s @ "raw_body" => util::req_res_meta_word(
                                s,
                                &field,
                                &mut newtype_body_field,
                                ResponseFieldKind::NewtypeBody,
                                ResponseFieldKind::NewtypeRawBody,
                            )?,
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "Invalid #[ruma_api] argument with value, expected `body`",
                                ));
                            }
                        },
                        Meta::NameValue(MetaNameValue { name, value }) => util::req_res_name_value(
                            name,
                            value,
                            &mut header,
                            ResponseFieldKind::Header,
                        )?,
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
            })
            .collect::<syn::Result<Vec<_>>>()?;

        if newtype_body_field.is_some() && fields.iter().any(|f| f.is_body()) {
            // TODO: highlight conflicting fields,
            return Err(syn::Error::new_spanned(
                raw.response_kw,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        Ok(Self { fields, lifetimes: lifetimes.into_iter().collect() })
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let response_def = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields =
                self.fields.iter().map(|response_field| strip_serde_attrs(response_field.field()));

            quote! { { #(#fields),* } }
        };

        let (derive_deserialize, def) = if let Some(body_field) =
            self.fields.iter().find(|f| f.is_newtype_body())
        {
            let field = Field { ident: None, colon_token: None, ..body_field.field().clone() };

            let (derive_deserialize, lifetimes) = if util::has_lifetime(&body_field.field().ty) {
                (
                    TokenStream::new(),
                    util::collect_generic_idents(Some(&body_field.field().ty).into_iter()),
                )
            } else {
                (quote!(::ruma_api::exports::serde::Deserialize), TokenStream::new())
            };

            (derive_deserialize, quote! { #lifetimes (#field); })
        } else if self.has_body_fields() {
            let fields = self.fields.iter().filter(|f| f.is_body());
            let (derive_deserialize, lifetimes) =
                if fields.clone().any(|f| util::has_lifetime(&f.field().ty)) {
                    (
                        TokenStream::new(),
                        util::collect_generic_idents(fields.clone().map(|f| &f.field().ty)),
                    )
                } else {
                    (quote!(::ruma_api::exports::serde::Deserialize), TokenStream::new())
                };

            let fields = fields.map(ResponseField::field);

            (derive_deserialize, quote!( #lifetimes { #(#fields),* }))
        } else {
            (TokenStream::new(), quote!({}))
        };

        let response_body_struct = quote! {
            /// Data in the response body.
            #[derive(
                Debug,
                ::ruma_api::Outgoing,
                ::ruma_api::exports::serde::Serialize,
                #derive_deserialize
            )]
            struct ResponseBody #def
        };

        let response_generics = util::generics_to_tokens(self.lifetimes.iter());
        let response = quote! {
            #[derive(Debug, Clone, ::ruma_api::Outgoing)]
            #[incoming_no_deserialize]
            pub struct Response #response_generics #response_def

            #response_body_struct
        };

        response.to_tokens(tokens);
    }
}

/// The types of fields that a response can have.
pub enum ResponseField {
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

    /// Whether or not this response field is a body kind.
    fn is_body(&self) -> bool {
        self.as_body_field().is_some()
    }

    /// Whether or not this response field is a header kind.
    fn is_header(&self) -> bool {
        matches!(self, ResponseField::Header(..))
    }

    /// Whether or not this response field is a newtype body kind.
    fn is_newtype_body(&self) -> bool {
        self.as_newtype_body_field().is_some()
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

/// The types of fields that a response can have, without their values.
enum ResponseFieldKind {
    /// See the similarly named variant of `ResponseField`.
    Body,
    /// See the similarly named variant of `ResponseField`.
    Header,
    /// See the similarly named variant of `ResponseField`.
    NewtypeBody,
    /// See the similarly named variant of `ResponseField`.
    NewtypeRawBody,
}
