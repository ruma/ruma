//! Details of the `request` section of the procedural macro.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Field, Ident};

use crate::api::{
    attribute::{Meta, MetaNameValue},
    strip_serde_attrs,
};

/// The result of processing the `request` section of the macro.
pub struct Request {
    /// The fields of the request.
    fields: Vec<RequestField>,
}

impl Request {
    /// Produces code to add necessary HTTP headers to an `http::Request`.
    pub fn add_headers_to_request(&self) -> TokenStream {
        let append_stmts = self.header_fields().map(|request_field| {
            let (field, header_name) = match request_field {
                RequestField::Header(field, header_name) => (field, header_name),
                _ => panic!("expected request field to be header variant"),
            };

            let field_name = &field.ident;

            quote! {
                headers.append(
                    ruma_api::exports::http::header::#header_name,
                    ruma_api::exports::http::header::HeaderValue::from_str(request.#field_name.as_ref())
                        .expect("failed to convert value into HeaderValue"),
                );
            }
        });

        quote! {
            #(#append_stmts)*
        }
    }

    /// Whether or not this request has any data in the HTTP body.
    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    /// Whether or not this request has any data in HTTP headers.
    pub fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }

    /// Whether or not this request has any data in the URL path.
    pub fn has_path_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_path())
    }

    /// Whether or not this request has any data in the query string.
    pub fn has_query_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_query())
    }

    /// Produces an iterator over all the header fields.
    pub fn header_fields(&self) -> impl Iterator<Item = &RequestField> {
        self.fields.iter().filter(|field| field.is_header())
    }

    /// Gets the number of path fields.
    pub fn path_field_count(&self) -> usize {
        self.fields.iter().filter(|field| field.is_path()).count()
    }

    /// Returns the body field.
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

    /// Produces code for a struct initializer for body fields on a variable named `request`.
    pub fn request_body_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Body, quote!(request))
    }

    /// Produces code for a struct initializer for path fields on a variable named `request`.
    pub fn request_path_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Path, quote!(request))
    }

    /// Produces code for a struct initializer for query string fields on a variable named `request`.
    pub fn request_query_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Query, quote!(request))
    }

    /// Produces code for a struct initializer for the given field kind to be accessed through the
    /// given variable name.
    fn struct_init_fields(
        &self,
        request_field_kind: RequestFieldKind,
        src: TokenStream,
    ) -> TokenStream {
        let fields = self.fields.iter().filter_map(|f| {
            f.field_of_kind(request_field_kind).map(|field| {
                let field_name = field
                    .ident
                    .as_ref()
                    .expect("expected field to have an identifier");
                let span = field.span();

                quote_spanned! {span=>
                    #field_name: #src.#field_name
                }
            })
        });

        quote! {
            #(#fields,)*
        }
    }
}

impl From<Vec<Field>> for Request {
    fn from(fields: Vec<Field>) -> Self {
        let fields: Vec<_> = fields.into_iter().map(|mut field| {
            let mut field_kind = None;
            let mut header = None;

            field.attrs.retain(|attr| {
                let meta = match Meta::from_attribute(attr) {
                    Some(m) => m,
                    None => return true,
                };

                match meta {
                    Meta::Word(ident) => {
                        assert!(
                            field_kind.is_none(),
                            "ruma_api! field kind can only be set once per field"
                        );

                        field_kind = Some(match &ident.to_string()[..] {
                            "body" => RequestFieldKind::NewtypeBody,
                            "path" => RequestFieldKind::Path,
                            "query" => RequestFieldKind::Query,
                            _ => panic!("ruma_api! single-word attribute on requests must be: body, path, or query"),
                        });
                    }
                    Meta::NameValue(MetaNameValue { name, value }) => {
                        assert!(
                            name == "header",
                            "ruma_api! name/value pair attribute on requests must be: header"
                        );
                        assert!(
                            field_kind.is_none(),
                            "ruma_api! field kind can only be set once per field"
                        );

                        header = Some(value);
                        field_kind = Some(RequestFieldKind::Header);
                    }
                }

                false
            });

            RequestField::new(field_kind.unwrap_or(RequestFieldKind::Body), field, header)
        }).collect();

        let num_body_fields = fields.iter().filter(|f| f.is_body()).count();
        let num_newtype_body_fields = fields.iter().filter(|f| f.is_newtype_body()).count();
        assert!(
            num_newtype_body_fields <= 1,
            "ruma_api! request can only have one newtype body field"
        );
        if num_newtype_body_fields == 1 {
            assert!(
                num_body_fields == 0,
                "ruma_api! request can't have both regular body fields and a newtype body field"
            );
        }

        Self { fields }
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let request_struct_header = quote! {
            #[derive(Debug, Clone)]
            pub struct Request
        };

        let request_struct_body = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields = self
                .fields
                .iter()
                .map(|request_field| strip_serde_attrs(request_field.field()));

            quote! {
                {
                    #(#fields),*
                }
            }
        };

        let request_body_struct = if let Some(newtype_body_field) = self.newtype_body_field() {
            let field = newtype_body_field.clone();
            let ty = &field.ty;
            let span = field.span();

            quote_spanned! {span=>
                /// Data in the request body.
                #[derive(Debug, ruma_api::exports::serde::Serialize)]
                struct RequestBody(#ty);
            }
        } else if self.has_body_fields() {
            let fields = self.fields.iter().filter_map(RequestField::as_body_field);

            quote! {
                /// Data in the request body.
                #[derive(Debug, ruma_api::exports::serde::Serialize)]
                struct RequestBody {
                    #(#fields),*
                }
            }
        } else {
            TokenStream::new()
        };

        let request_path_struct = if self.has_path_fields() {
            let fields = self.fields.iter().filter_map(RequestField::as_path_field);

            quote! {
                /// Data in the request path.
                #[derive(
                    Debug,
                    ruma_api::exports::serde::Deserialize,
                    ruma_api::exports::serde::Serialize,
                )]
                struct RequestPath {
                    #(#fields),*
                }
            }
        } else {
            TokenStream::new()
        };

        let request_query_struct = if self.has_query_fields() {
            let fields = self.fields.iter().filter_map(RequestField::as_query_field);

            quote! {
                /// Data in the request's query string.
                #[derive(
                    Debug,
                    ruma_api::exports::serde::Deserialize,
                    ruma_api::exports::serde::Serialize,
                )]
                struct RequestQuery {
                    #(#fields),*
                }
            }
        } else {
            TokenStream::new()
        };

        let request = quote! {
            #request_struct_header
            #request_struct_body
            #request_body_struct
            #request_path_struct
            #request_query_struct
        };

        request.to_tokens(tokens);
    }
}

/// The types of fields that a request can have.
pub enum RequestField {
    /// JSON data in the body of the request.
    Body(Field),
    /// Data in an HTTP header.
    Header(Field, Ident),
    /// A specific data type in the body of the request.
    NewtypeBody(Field),
    /// Data that appears in the URL path.
    Path(Field),
    /// Data that appears in the query string.
    Query(Field),
}

impl RequestField {
    /// Creates a new `RequestField`.
    fn new(kind: RequestFieldKind, field: Field, header: Option<Ident>) -> Self {
        match kind {
            RequestFieldKind::Body => RequestField::Body(field),
            RequestFieldKind::Header => {
                RequestField::Header(field, header.expect("missing header name"))
            }
            RequestFieldKind::NewtypeBody => RequestField::NewtypeBody(field),
            RequestFieldKind::Path => RequestField::Path(field),
            RequestFieldKind::Query => RequestField::Query(field),
        }
    }

    /// Gets the kind of the request field.
    fn kind(&self) -> RequestFieldKind {
        match *self {
            RequestField::Body(..) => RequestFieldKind::Body,
            RequestField::Header(..) => RequestFieldKind::Header,
            RequestField::NewtypeBody(..) => RequestFieldKind::NewtypeBody,
            RequestField::Path(..) => RequestFieldKind::Path,
            RequestField::Query(..) => RequestFieldKind::Query,
        }
    }

    /// Whether or not this request field is a body kind.
    fn is_body(&self) -> bool {
        self.kind() == RequestFieldKind::Body
    }

    /// Whether or not this request field is a header kind.
    fn is_header(&self) -> bool {
        self.kind() == RequestFieldKind::Header
    }

    /// Whether or not this request field is a newtype body kind.
    fn is_newtype_body(&self) -> bool {
        self.kind() == RequestFieldKind::NewtypeBody
    }

    /// Whether or not this request field is a path kind.
    fn is_path(&self) -> bool {
        self.kind() == RequestFieldKind::Path
    }

    /// Whether or not this request field is a query string kind.
    fn is_query(&self) -> bool {
        self.kind() == RequestFieldKind::Query
    }

    /// Return the contained field if this request field is a body kind.
    fn as_body_field(&self) -> Option<&Field> {
        if let RequestField::Body(field) = self {
            Some(field)
        } else {
            None
        }
    }

    /// Return the contained field if this request field is a path kind.
    fn as_path_field(&self) -> Option<&Field> {
        if let RequestField::Path(field) = self {
            Some(field)
        } else {
            None
        }
    }

    /// Return the contained field if this request field is a query kind.
    fn as_query_field(&self) -> Option<&Field> {
        if let RequestField::Query(field) = self {
            Some(field)
        } else {
            None
        }
    }

    /// Gets the inner `Field` value.
    fn field(&self) -> &Field {
        match *self {
            RequestField::Body(ref field)
            | RequestField::Header(ref field, _)
            | RequestField::NewtypeBody(ref field)
            | RequestField::Path(ref field)
            | RequestField::Query(ref field) => field,
        }
    }

    /// Gets the inner `Field` value if it's of the provided kind.
    fn field_of_kind(&self, kind: RequestFieldKind) -> Option<&Field> {
        if self.kind() == kind {
            Some(self.field())
        } else {
            None
        }
    }
}

/// The types of fields that a request can have, without their values.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestFieldKind {
    /// See the similarly named variant of `RequestField`.
    Body,
    /// See the similarly named variant of `RequestField`.
    Header,
    /// See the similarly named variant of `RequestField`.
    NewtypeBody,
    /// See the similarly named variant of `RequestField`.
    Path,
    /// See the similarly named variant of `RequestField`.
    Query,
}
