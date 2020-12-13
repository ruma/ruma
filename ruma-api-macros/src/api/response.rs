//! Details of the `response` section of the procedural macro.

use std::mem;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Field, Ident, Token,
};

use crate::{
    api::attribute::{Meta, MetaNameValue},
    util,
};

mod kw {
    syn::custom_keyword!(response);
}

/// The result of processing the `response` section of the macro.
pub struct Response {
    /// The attributes that will be applied to the struct definition.
    attributes: Vec<Attribute>,

    /// The fields of the response.
    fields: Vec<ResponseField>,

    // Guarantee `ruma_api` is available and named something we can refer to.
    ruma_api_import: TokenStream,
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

    /// Produces code for a response struct initializer.
    pub fn init_fields(&self) -> TokenStream {
        let ruma_api = &self.ruma_api_import;
        let http = quote! { #ruma_api::exports::http };

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
                    let optional_header = match &field.ty {
                        syn::Type::Path(syn::TypePath {
                            path: syn::Path { segments, .. }, ..
                        }) if segments.last().unwrap().ident == "Option" => {
                            quote! {
                                #field_name: #ruma_api::try_deserialize!(
                                    response,
                                    headers.remove(#http::header::#header_name)
                                        .map(|h| h.to_str().map(|s| s.to_owned()))
                                        .transpose()
                                )
                            }
                        }
                        _ => quote! {
                            #field_name: #ruma_api::try_deserialize!(
                                response,
                                headers.remove(#http::header::#header_name)
                                    .expect("response missing expected header")
                                    .to_str()
                            )
                            .to_owned()
                        },
                    };
                    quote_spanned! {span=> #optional_header }
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
        let ruma_api = &self.ruma_api_import;
        let http = quote! { #ruma_api::exports::http };

        let header_calls = self.fields.iter().filter_map(|response_field| {
            if let ResponseField::Header(ref field, ref header_name) = *response_field {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let span = field.span();

                let optional_header = match &field.ty {
                    syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                        if segments.last().unwrap().ident == "Option" =>
                    {
                        quote! {
                            if let Some(header) = response.#field_name {
                                headers
                                    .insert(
                                        #http::header::#header_name,
                                        header.parse()?,
                                    );
                            }
                        }
                    }
                    _ => quote! {
                        headers
                            .insert(
                                #http::header::#header_name,
                                response.#field_name.parse()?,
                            );
                    },
                };

                Some(quote_spanned! {span=>
                    #optional_header
                })
            } else {
                None
            }
        });

        quote! { #(#header_calls)* }
    }

    /// Produces code to initialize the struct that will be used to create the response body.
    pub fn to_body(&self) -> TokenStream {
        let ruma_api = &self.ruma_api_import;
        let serde_json = quote! { #ruma_api::exports::serde_json };

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

        quote!(#serde_json::to_vec(&#body)?)
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

impl Parse for Response {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let response_kw = input.parse::<kw::response>()?;
        input.parse::<Token![:]>()?;
        let fields;
        braced!(fields in input);

        let fields = fields
            .parse_terminated::<Field, Token![,]>(Field::parse_named)?
            .into_iter()
            .map(|f| {
                if util::has_lifetime(&f.ty) {
                    Err(syn::Error::new(
                        f.ident.span(),
                        "Lifetimes on Response fields cannot be supported until GAT are stable",
                    ))
                } else {
                    Ok(f)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut newtype_body_field = None;

        let fields = fields
            .into_iter()
            .map(|mut field| {
                let mut field_kind = None;
                let mut header = None;

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
                response_kw,
                "Can't have both a newtype body field and regular body fields",
            ));
        }

        Ok(Self { attributes, fields, ruma_api_import: util::import_ruma_api() })
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ruma_api = &self.ruma_api_import;
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde = quote! { #ruma_api::exports::serde };

        let struct_attributes = &self.attributes;

        let response_def = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields = self.fields.iter().map(|response_field| response_field.field());
            quote! { { #(#fields),* } }
        };

        let def = if let Some(body_field) = self.fields.iter().find(|f| f.is_newtype_body()) {
            let field = Field { ident: None, colon_token: None, ..body_field.field().clone() };

            quote! { (#field); }
        } else if self.has_body_fields() {
            let fields = self.fields.iter().filter(|f| f.is_body());

            let fields = fields.map(ResponseField::field);

            quote! { { #(#fields),* } }
        } else {
            quote! { {} }
        };

        let response_body_struct = quote! {
            /// Data in the response body.
            #[derive(Debug, #ruma_serde::Outgoing, #serde::Deserialize, #serde::Serialize)]
            struct ResponseBody #def
        };

        let response = quote! {
            #[derive(Debug, Clone, #ruma_serde::Outgoing, #ruma_serde::_FakeDeriveSerde)]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            #[incoming_derive(!Deserialize)]
            #( #struct_attributes )*
            pub struct Response #response_def

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
