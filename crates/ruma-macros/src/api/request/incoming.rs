use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use super::{Request, RequestBody, RequestHeaders, RequestPath, RequestQuery};
use crate::util::{StructFieldExt, TypeExt, expand_fields_as_variable_declarations};

impl Request {
    /// Generate the `ruma_common::api::IncomingRequest` implementation for this request struct.
    pub fn expand_incoming(&self, ruma_common: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_common::exports::http };

        let path_parse = self.path.expand_parse(ruma_common);
        let path_fields = self.path.expand_fields();

        let query_parse = self.query.expand_parse(ruma_common);
        let query_fields = self.query.expand_fields();

        let headers_parse = self.headers.expand_parse(ruma_common);
        let headers_fields = self.headers.expand_fields();

        let body_parse = self.body.expand_parse(ruma_common);
        let body_fields = self.body.expand_fields();

        let error_ty = &self.error_ty;
        let ident = &self.ident;

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_common::api::IncomingRequest for #ident {
                type EndpointError = #error_ty;
                type OutgoingResponse = Response;

                fn try_from_http_request<B, S>(
                    request: #http::Request<B>,
                    path_args: &[S],
                ) -> ::std::result::Result<Self, #ruma_common::api::error::FromHttpRequestError>
                where
                    B: ::std::convert::AsRef<[::std::primitive::u8]>,
                    S: ::std::convert::AsRef<::std::primitive::str>,
                {
                    <Self as #ruma_common::api::IncomingRequest>::check_request_method(request.method())?;

                    #path_parse
                    #query_parse
                    #headers_parse
                    #body_parse

                    ::std::result::Result::Ok(Self {
                        #path_fields
                        #query_fields
                        #headers_fields
                        #body_fields
                    })
                }
            }
        }
    }
}

impl RequestPath {
    /// Generate code to parse the path arguments of a `&[dyn AsRef<[u8]>]` named `path_args`.
    fn expand_parse(&self, ruma_common: &TokenStream) -> Option<TokenStream> {
        if self.0.is_empty() {
            return None;
        }

        let serde = quote! { #ruma_common::exports::serde };
        let fields = self.expand_fields();

        Some(quote! {
            let (#fields) = #serde::Deserialize::deserialize(
                #serde::de::value::SeqDeserializer::<_, #serde::de::value::Error>::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref)
                )
            )?;
        })
    }
}

impl RequestQuery {
    /// Generate code to parse the query from an `http::request::Request`.
    fn expand_parse(&self, ruma_common: &TokenStream) -> Option<TokenStream> {
        let fields = match self {
            Self::None => return None,
            Self::Fields(fields) => fields.as_slice(),
            Self::All(field) => std::slice::from_ref(field),
        };

        let serde_html_form = quote! { #ruma_common::exports::serde_html_form };
        let src = parse_quote! { request_query };
        let decls = expand_fields_as_variable_declarations(fields, &src);

        Some(quote! {
            let #src: RequestQuery =
                #serde_html_form::from_str(request.uri().query().unwrap_or(""))?;

            #decls
        })
    }
}

impl RequestHeaders {
    /// Generate code to parse the headers from an `http::request::Request`.
    fn expand_parse(&self, ruma_common: &TokenStream) -> Option<TokenStream> {
        if self.0.is_empty() {
            return None;
        }

        let decls = self
            .0
            .iter()
            .map(|(header_name, field)| Self::expand_parse_header(header_name, field, ruma_common));

        Some(quote! {
            let headers = request.headers();

            #( #decls )*
        })
    }

    /// Generate code to parse the header with the given name, to assign it to a variable for the
    /// given field, by extracting it from a `http::header::HeaderMap` named `headers`.
    fn expand_parse_header(
        header_name: &syn::Ident,
        field: &syn::Field,
        ruma_common: &TokenStream,
    ) -> TokenStream {
        let ident = field.ident();
        let cfg_attrs = field.cfg_attrs();
        let header_name_string = header_name.to_string();
        let field_type = &field.ty;

        // We need to handle optional fields manually, because we need to parse the inner type.
        let option_inner_type = field_type.option_inner_type();

        let some_case = if let Some(field_type) = option_inner_type {
            quote! {
                str_value.parse::<#field_type>().ok()
            }
        } else {
            quote! {
                str_value
                    .parse::<#field_type>()
                    .map_err(|e| #ruma_common::api::error::HeaderDeserializationError::InvalidHeader(e.into()))?
            }
        };

        let none_case = if option_inner_type.is_some() {
            quote! { None }
        } else {
            quote! {
                return Err(
                    #ruma_common::api::error::HeaderDeserializationError::MissingHeader(
                        #header_name_string.into()
                    ).into(),
                )
            }
        };

        quote! {
            #( #cfg_attrs )*
            let #ident = match headers.get(#header_name) {
                Some(header_value) => {
                    let str_value = header_value.to_str()?;
                    #some_case
                }
                None => #none_case,
            };
        }
    }
}

impl RequestBody {
    /// Generate code to parse the body from an `http::request::Request` named `request`.
    fn expand_parse(&self, ruma_common: &TokenStream) -> Option<TokenStream> {
        match self {
            Self::Empty => None,
            Self::JsonFields(fields) => Some(Self::expand_parse_json_body(fields, ruma_common)),
            Self::JsonAll(field) => {
                Some(Self::expand_parse_json_body(std::slice::from_ref(field), ruma_common))
            }
            Self::Raw(field) => {
                let ident = field.ident();
                let cfg_attrs = field.cfg_attrs();

                Some(quote! {
                    #( #cfg_attrs )*
                    let #ident =
                        ::std::convert::AsRef::<[u8]>::as_ref(request.body()).to_vec();
                })
            }
        }
    }

    /// Generate code to parse a JSON request body with the given fields, to assign it to a variable
    /// for the given field.
    fn expand_parse_json_body(fields: &[syn::Field], ruma_common: &TokenStream) -> TokenStream {
        let serde_json = quote! { #ruma_common::exports::serde_json };
        let src = parse_quote! { request_body };

        let assignments = expand_fields_as_variable_declarations(fields, &src);

        quote! {
            let #src: RequestBody = {
                let body = ::std::convert::AsRef::<[::std::primitive::u8]>::as_ref(
                    request.body(),
                );

                #serde_json::from_slice(match body {
                    // If the request body is completely empty, pretend it is an empty JSON
                    // object instead. This allows requests with only optional body parameters
                    // to be deserialized in that case.
                    [] => b"{}",
                    b => b,
                })?
            };

            #assignments
        }
    }
}
