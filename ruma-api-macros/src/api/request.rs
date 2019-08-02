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
                    ::http::header::#header_name,
                    ::http::header::HeaderValue::from_str(request.#field_name.as_ref())
                        .expect("failed to convert value into HeaderValue"),
                );
            }
        });

        quote! {
            #(#append_stmts)*
        }
    }

    /// Produces code to extract fields from the HTTP headers in an `http::Request`.
    pub fn parse_headers_from_request(&self) -> TokenStream {
        let fields = self.header_fields().map(|request_field| {
            let (field, header_name) = match request_field {
                RequestField::Header(field, header_name) => (field, header_name),
                _ => panic!("expected request field to be header variant"),
            };

            let field_name = &field.ident;
            let header_name_string = header_name.to_string();

            quote! {
                #field_name: headers.get(::http::header::#header_name)
                    .and_then(|v| v.to_str().ok())
                    .ok_or(::serde_json::Error::missing_field(#header_name_string))?
                    .to_owned()
            }
        });

        quote! {
            #(#fields,)*
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

    /// Gets the path field with the given name.
    pub fn path_field(&self, name: &str) -> Option<&Field> {
        self.fields
            .iter()
            .flat_map(|f| f.field_of_kind(RequestFieldKind::Path))
            .find(|field| {
                field
                    .ident
                    .as_ref()
                    .expect("expected field to have an identifier")
                    == name
            })
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

    /// Produces code for a struct initializer for body fields on a variable named `request_body`.
    pub fn request_init_body_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Body, quote!(request_body))
    }

    /// Produces code for a struct initializer for query string fields on a variable named
    /// `request_query`.
    pub fn request_init_query_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Query, quote!(request_query))
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
        let mut has_newtype_body = false;

        let fields = fields.into_iter().map(|mut field| {
            let mut field_kind = RequestFieldKind::Body;
            let mut header = None;

            field.attrs = field.attrs.into_iter().filter_map(|attr| {
                let meta = match Meta::from_attribute(attr) {
                    Ok(meta) => meta,
                    Err(attr) => return Some(attr),
                };

                match meta {
                    Meta::Word(ident) => {
                        match &ident.to_string()[..] {
                            "body" => {
                                has_newtype_body = true;
                                field_kind = RequestFieldKind::NewtypeBody;
                            }
                            "path" => field_kind = RequestFieldKind::Path,
                            "query" => field_kind = RequestFieldKind::Query,
                            _ => panic!("ruma_api! single-word attribute on requests must be: body, path, or query"),
                        }
                    }
                    Meta::NameValue(MetaNameValue { name, value }) => {
                        assert!(
                            name == "header",
                            "ruma_api! name/value pair attribute on requests must be: header"
                        );

                        header = Some(value);
                        field_kind = RequestFieldKind::Header;
                    }
                }

                None
            }).collect();

            if field_kind == RequestFieldKind::Body {
                assert!(
                    !has_newtype_body,
                    "ruma_api! requests cannot have both normal body fields and a newtype body field"
                );
            }

            RequestField::new(field_kind, field, header)
        }).collect();

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
            let fields = self.fields.iter().map(|request_field| {
                let field = request_field.field();
                let span = field.span();

                let stripped_field = strip_serde_attrs(field);

                quote_spanned!(span=> #stripped_field)
            });

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
                #[derive(Debug, Deserialize, Serialize)]
                struct RequestBody(#ty);
            }
        } else if self.has_body_fields() {
            let fields = self
                .fields
                .iter()
                .filter_map(|request_field| match *request_field {
                    RequestField::Body(ref field) => {
                        let span = field.span();
                        Some(quote_spanned!(span=> #field))
                    }
                    _ => None,
                });

            quote! {
                /// Data in the request body.
                #[derive(Debug, Deserialize, Serialize)]
                struct RequestBody {
                    #(#fields),*
                }
            }
        } else {
            TokenStream::new()
        };

        let request_path_struct = if self.has_path_fields() {
            let fields = self
                .fields
                .iter()
                .filter_map(|request_field| match *request_field {
                    RequestField::Path(ref field) => {
                        let span = field.span();

                        Some(quote_spanned!(span=> #field))
                    }
                    _ => None,
                });

            quote! {
                /// Data in the request path.
                #[derive(Debug, Deserialize, Serialize)]
                struct RequestPath {
                    #(#fields),*
                }
            }
        } else {
            TokenStream::new()
        };

        let request_query_struct = if self.has_query_fields() {
            let fields = self
                .fields
                .iter()
                .filter_map(|request_field| match *request_field {
                    RequestField::Query(ref field) => {
                        let span = field.span();
                        Some(quote_spanned!(span=> #field))
                    }
                    _ => None,
                });

            quote! {
                /// Data in the request's query string.
                #[derive(Debug, Deserialize, Serialize)]
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

    /// Whether or not this request field is a path kind.
    fn is_path(&self) -> bool {
        self.kind() == RequestFieldKind::Path
    }

    /// Whether or not this request field is a query string kind.
    fn is_query(&self) -> bool {
        self.kind() == RequestFieldKind::Query
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
