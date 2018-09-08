use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, ToTokens};
use syn::spanned::Spanned;
use syn::{Field, Ident, Lit, Meta, NestedMeta};

use api::strip_serde_attrs;

pub struct Request {
    fields: Vec<RequestField>,
}

impl Request {
    pub fn add_headers_to_request(&self) -> TokenStream {
        self.header_fields().fold(TokenStream::new(), |mut header_tokens, request_field| {
            let (field, header_name_string) = match request_field {
                RequestField::Header(field, header_name_string) => (field, header_name_string),
                _ => panic!("expected request field to be header variant"),
            };

            let field_name = &field.ident;
            let header_name = Ident::new(header_name_string.as_ref(), Span::call_site());

            header_tokens.append_all(quote! {
                headers.append(
                    ::http::header::#header_name,
                    ::http::header::HeaderValue::from_str(request.#field_name.as_ref())
                        .expect("failed to convert value into HeaderValue"),
                );
            });

            header_tokens
        })
    }

    pub fn parse_headers_from_request(&self) -> TokenStream {
        self.header_fields().fold(TokenStream::new(), |mut header_tokens, request_field| {
            let (field, header_name_string) = match request_field {
                RequestField::Header(field, header_name_string) => (field, header_name_string),
                _ => panic!("expected request field to be header variant"),
            };

            let field_name = &field.ident;
            let header_name = Ident::new(header_name_string.as_ref(), Span::call_site());

            header_tokens.append_all(quote! {
                #field_name: headers.get(::http::header::#header_name)
                    .and_then(|v| v.to_str().ok())
                    .ok_or(::serde_json::Error::missing_field(#header_name_string))?
                    .to_owned(),
            });

            header_tokens
        })
    }

    pub fn has_body_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_body())
    }

    pub fn has_header_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_header())
    }
    pub fn has_path_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_path())
    }

    pub fn has_query_fields(&self) -> bool {
        self.fields.iter().any(|field| field.is_query())
    }

    pub fn header_fields(&self) -> impl Iterator<Item = &RequestField> {
        self.fields.iter().filter(|field| field.is_header())
    }

    pub fn path_field_count(&self) -> usize {
        self.fields.iter().filter(|field| field.is_path()).count()
    }

    pub fn path_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter()
            .flat_map(|f| f.field_(RequestFieldKind::Path))
            .find(|field| {
                field.ident.as_ref()
                    .expect("expected field to have an identifier")
                    .to_string()
                    == name
            })
    }

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

    pub fn request_body_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Body, quote!(request))
    }

    pub fn request_path_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Path, quote!(request))
    }

    pub fn request_query_init_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Query, quote!(request))
    }

    pub fn request_init_body_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Body, quote!(request_body))
    }

    pub fn request_init_query_fields(&self) -> TokenStream {
        self.struct_init_fields(RequestFieldKind::Query, quote!(request_query))
    }

    fn struct_init_fields(&self, request_field_kind: RequestFieldKind, src: TokenStream) -> TokenStream {
        let mut tokens = TokenStream::new();

        for field in self.fields.iter().flat_map(|f| f.field_(request_field_kind)) {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let span = field.span();

            tokens.append_all(quote_spanned! {span=>
                #field_name: #src.#field_name,
            });
        }

        tokens
    }
}

impl From<Vec<Field>> for Request {
    fn from(fields: Vec<Field>) -> Self {
        let mut has_newtype_body = false;

        let fields = fields.into_iter().map(|mut field| {
            let mut field_kind = RequestFieldKind::Body;
            let mut header = None;

            field.attrs = field.attrs.into_iter().filter(|attr| {
                let meta = attr.interpret_meta()
                    .expect("ruma_api! could not parse request field attributes");

                let meta_list = match meta {
                    Meta::List(meta_list) => meta_list,
                    _ => return true,
                };

                if meta_list.ident != "ruma_api" {
                    return true;
                }

                for nested_meta_item in meta_list.nested {
                    match nested_meta_item {
                        NestedMeta::Meta(meta_item) => {
                            match meta_item {
                                Meta::Word(ident) => {
                                    match ident.to_string().as_ref() {
                                        "body" => {
                                            has_newtype_body = true;
                                            field_kind = RequestFieldKind::NewtypeBody;
                                        }
                                        "path" => field_kind = RequestFieldKind::Path,
                                        "query" => field_kind = RequestFieldKind::Query,
                                        _ => panic!("ruma_api! single-word attribute on requests must be: body, path, or query"),
                                    }
                                }
                                Meta::NameValue(name_value) => {
                                    match name_value.ident.to_string().as_ref() {
                                        "header" => {
                                            match name_value.lit {
                                                Lit::Str(lit_str) => header = Some(lit_str.value()),
                                                _ => panic!("ruma_api! header attribute's value must be a string literal"),
                                            }

                                            field_kind = RequestFieldKind::Header;
                                        }
                                        _ => panic!("ruma_api! name/value pair attribute on requests must be: header"),
                                    }
                                }
                                _ => panic!("ruma_api! attributes on requests must be a single word or a name/value pair"),
                            }
                        }
                        NestedMeta::Literal(_) => panic!(
                            "ruma_api! attributes on requests must be: body, header, path, or query"
                        ),
                    }
                }

                false
            }).collect();

            if field_kind == RequestFieldKind::Body {
                assert!(
                    !has_newtype_body,
                    "ruma_api! requests cannot have both normal body fields and a newtype body field"
                );
            }

            RequestField::new(field_kind, field, header)
        }).collect();

        Request {
            fields,
        }
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let request_struct_header = quote! {
            /// Data for a request to this API endpoint.
            #[derive(Debug)]
            pub struct Request
        };

        let request_struct_body = if self.fields.is_empty() {
            quote!(;)
        } else {
            let fields = self.fields.iter().fold(TokenStream::new(), |mut field_tokens, request_field| {
                let field = request_field.field();
                let span = field.span();

                let stripped_field = strip_serde_attrs(field);

                field_tokens.append_all(quote_spanned!(span=> #stripped_field,));

                field_tokens
            });

            quote! {
                {
                    #fields
                }
            }
        };

        let request_body_struct;

        if let Some(newtype_body_field) = self.newtype_body_field() {
            let mut field = newtype_body_field.clone();
            let ty = &field.ty;
            let span = field.span();

            request_body_struct = quote_spanned! {span=>
                /// Data in the request body.
                #[derive(Debug, Deserialize, Serialize)]
                struct RequestBody(#ty);
            };
        } else if self.has_body_fields() {
            let fields = self.fields.iter().fold(TokenStream::new(), |mut field_tokens, request_field| {
                match *request_field {
                    RequestField::Body(ref field) => {
                        let span = field.span();

                        field_tokens.append_all(quote_spanned!(span=> #field,));

                        field_tokens
                    }
                    _ => field_tokens,
                }
            });

            request_body_struct = quote! {
                /// Data in the request body.
                #[derive(Debug, Deserialize, Serialize)]
                struct RequestBody {
                    #fields
                }
            };
        } else {
            request_body_struct = TokenStream::new();
        }

        let request_path_struct;

        if self.has_path_fields() {
            let fields = self.fields.iter().fold(TokenStream::new(), |mut field_tokens, request_field| {
                match *request_field {
                    RequestField::Path(ref field) => {
                        let span = field.span();

                        field_tokens.append_all(quote_spanned!(span=> #field,));

                        field_tokens
                    }
                    _ => field_tokens,
                }
            });

            request_path_struct = quote! {
                /// Data in the request path.
                #[derive(Debug, Deserialize, Serialize)]
                struct RequestPath {
                    #fields
                }
            };
        } else {
            request_path_struct = TokenStream::new();
        }

        let request_query_struct;

        if self.has_query_fields() {
            let fields = self.fields.iter().fold(TokenStream::new(), |mut field_tokens, request_field| {
                match *request_field {
                    RequestField::Query(ref field) => {
                        let span = field.span();

                        field_tokens.append_all(quote_spanned!(span=> #field,));

                        field_tokens
                    }
                    _ => field_tokens,
                }
            });

            request_query_struct = quote! {
                /// Data in the request's query string.
                #[derive(Debug, Deserialize, Serialize)]
                struct RequestQuery {
                    #fields
                }
            };
        } else {
            request_query_struct = TokenStream::new();
        }

        tokens.append_all(quote! {
            #request_struct_header
            #request_struct_body
            #request_body_struct
            #request_path_struct
            #request_query_struct
        });
    }
}

pub enum RequestField {
    Body(Field),
    Header(Field, String),
    NewtypeBody(Field),
    Path(Field),
    Query(Field),
}

impl RequestField {
    fn new(kind: RequestFieldKind, field: Field, header: Option<String>) -> RequestField {
        match kind {
            RequestFieldKind::Body => RequestField::Body(field),
            RequestFieldKind::Header => RequestField::Header(field, header.expect("missing header name")),
            RequestFieldKind::NewtypeBody => RequestField::NewtypeBody(field),
            RequestFieldKind::Path => RequestField::Path(field),
            RequestFieldKind::Query => RequestField::Query(field),
        }
    }

    fn kind(&self) -> RequestFieldKind {
        match *self {
            RequestField::Body(..) => RequestFieldKind::Body,
            RequestField::Header(..) => RequestFieldKind::Header,
            RequestField::NewtypeBody(..) => RequestFieldKind::NewtypeBody,
            RequestField::Path(..) => RequestFieldKind::Path,
            RequestField::Query(..) => RequestFieldKind::Query,
        }
    }

    fn is_body(&self) -> bool {
        self.kind() == RequestFieldKind::Body
    }

    fn is_header(&self) -> bool {
        self.kind() == RequestFieldKind::Header
    }

    fn is_path(&self) -> bool {
        self.kind() == RequestFieldKind::Path
    }

    fn is_query(&self) -> bool {
        self.kind() == RequestFieldKind::Query
    }

    fn field(&self) -> &Field {
        match *self {
            RequestField::Body(ref field) => field,
            RequestField::Header(ref field, _) => field,
            RequestField::NewtypeBody(ref field) => field,
            RequestField::Path(ref field) => field,
            RequestField::Query(ref field) => field,
        }
    }

    fn field_(&self, kind: RequestFieldKind) -> Option<&Field> {
        if self.kind() == kind {
            Some(self.field())
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestFieldKind {
    Body,
    Header,
    NewtypeBody,
    Path,
    Query,
}
