use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Field;

use super::{Request, RequestField};
use crate::auth_scheme::AuthScheme;

impl Request {
    pub fn expand_incoming(&self, ruma_api: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_api::exports::http };
        let percent_encoding = quote! { #ruma_api::exports::percent_encoding };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };

        let method = &self.method;
        let error_ty = &self.error_ty;

        let incoming_request_type = if self.has_lifetimes() {
            quote! { IncomingRequest }
        } else {
            quote! { Request }
        };

        let incoming_body_type = if !self.has_body_fields() || self.has_raw_body() {
            quote! { #ruma_api::IncomingRawHttpBody }
        } else if self.lifetimes.body.is_empty() {
            quote! { RequestBody }
        } else {
            quote! { IncomingRequestBody }
        };

        // FIXME: the rest of the field initializer expansions are gated `cfg(...)`
        // except this one. If we get errors about missing fields in IncomingRequest for
        // a path field look here.
        let (parse_request_path, path_vars) = if self.has_path_fields() {
            let path_string = self.path.value();

            assert!(path_string.starts_with('/'), "path needs to start with '/'");
            assert!(
                path_string.chars().filter(|c| *c == ':').count() == self.path_field_count(),
                "number of declared path parameters needs to match amount of placeholders in path"
            );

            let path_var_decls = path_string[1..]
                .split('/')
                .enumerate()
                .filter(|(_, seg)| seg.starts_with(':'))
                .map(|(i, seg)| {
                    let path_var = Ident::new(&seg[1..], Span::call_site());
                    quote! {
                        let #path_var = {
                            let segment = path_segments[#i].as_bytes();
                            let decoded =
                                #percent_encoding::percent_decode(segment).decode_utf8()?;

                            ::std::convert::TryFrom::try_from(&*decoded)?
                        };
                    }
                });

            let parse_request_path = quote! {
                let path_segments: ::std::vec::Vec<&::std::primitive::str> =
                    request.uri().path()[1..].split('/').collect();

                #(#path_var_decls)*
            };

            let path_vars = path_string[1..]
                .split('/')
                .filter(|seg| seg.starts_with(':'))
                .map(|seg| Ident::new(&seg[1..], Span::call_site()));

            (parse_request_path, quote! { #(#path_vars,)* })
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        let (parse_query, query_vars) = if let Some(field) = self.query_map_field() {
            let cfg_attrs =
                field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                #( #cfg_attrs )*
                let #field_name = #ruma_serde::urlencoded::from_str(
                    &request.uri().query().unwrap_or(""),
                )?;
            };

            (
                parse,
                quote! {
                    #( #cfg_attrs )*
                    #field_name,
                },
            )
        } else if self.has_query_fields() {
            let (decls, names) = vars(
                self.fields.iter().filter_map(RequestField::as_query_field),
                quote! { request_query },
            );

            let parse = quote! {
                let request_query: <RequestQuery as #ruma_serde::Outgoing>::Incoming =
                    #ruma_serde::urlencoded::from_str(
                        &request.uri().query().unwrap_or("")
                    )?;

                #decls
            };

            (parse, names)
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        let (parse_headers, header_vars) = if self.has_header_fields() {
            let (decls, names): (TokenStream, Vec<_>) = self
                .header_fields()
                .map(|request_field| {
                    let (field, header_name) = match request_field {
                        RequestField::Header(field, header_name) => (field, header_name),
                        _ => panic!("expected request field to be header variant"),
                    };

                    let cfg_attrs =
                        field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                    let field_name = &field.ident;
                    let header_name_string = header_name.to_string();

                    let (some_case, none_case) = match &field.ty {
                        syn::Type::Path(syn::TypePath {
                            path: syn::Path { segments, .. }, ..
                        }) if segments.last().unwrap().ident == "Option" => {
                            (quote! { Some(str_value.to_owned()) }, quote! { None })
                        }
                        _ => (
                            quote! { str_value.to_owned() },
                            quote! {
                                return Err(
                                    #ruma_api::error::HeaderDeserializationError::MissingHeader(
                                        #header_name_string.into()
                                    ).into(),
                                )
                            },
                        ),
                    };

                    let decl = quote! {
                        #( #cfg_attrs )*
                        let #field_name = match headers.get(#http::header::#header_name) {
                            Some(header_value) => {
                                let str_value = header_value.to_str()?;
                                #some_case
                            }
                            None => #none_case,
                        };
                    };

                    (
                        decl,
                        quote! {
                            #( #cfg_attrs )*
                            #field_name
                        },
                    )
                })
                .unzip();

            let parse = quote! {
                let headers = request.headers();

                #decls
            };

            (parse, quote! { #(#names,)* })
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        let extract_body = self.has_body_fields().then(|| {
            quote! {
                let request_body: #incoming_body_type = request.into_body();
            }
        });

        let (parse_body, body_vars) = if let Some(field) = self.raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name = request.into_body().0;
            };

            (parse, quote! { #field_name, })
        } else if self.has_body_fields() {
            vars(self.body_fields(), quote! { request_body })
        } else {
            // FIXME: Add body type assertion once TAIT is used
            (quote! {}, quote! {})
        };

        let non_auth_impl = matches!(self.authentication, AuthScheme::None(_)).then(|| {
            quote! {
                #[automatically_derived]
                #[cfg(feature = "server")]
                impl #ruma_api::IncomingNonAuthRequest for #incoming_request_type {}
            }
        });

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_api::IncomingRequest for #incoming_request_type {
                type IncomingBody = #incoming_body_type;
                    //impl #ruma_api::FromHttpBody<#ruma_api::error::FromHttpRequestError>;
                type EndpointError = #error_ty;
                type OutgoingResponse = Response;

                const METADATA: #ruma_api::Metadata = METADATA;

                fn try_from_http_request(
                    request: #http::Request<Self::IncomingBody>,
                ) -> ::std::result::Result<Self, #ruma_api::error::FromHttpRequestError> {
                    if request.method() != #http::Method::#method {
                        return Err(#ruma_api::error::FromHttpRequestError::MethodMismatch {
                            expected: #http::Method::#method,
                            received: request.method().clone(),
                        });
                    }

                    #parse_request_path
                    #parse_query
                    #parse_headers

                    #extract_body
                    #parse_body

                    ::std::result::Result::Ok(Self {
                        #path_vars
                        #query_vars
                        #header_vars
                        #body_vars
                    })
                }
            }

            #non_auth_impl
        }
    }
}

fn vars<'a>(
    fields: impl IntoIterator<Item = &'a Field>,
    src: TokenStream,
) -> (TokenStream, TokenStream) {
    fields
        .into_iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let cfg_attrs =
                field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

            let decl = quote! {
                #( #cfg_attrs )*
                let #field_name = #src.#field_name;
            };

            (
                decl,
                quote! {
                    #( #cfg_attrs )*
                    #field_name,
                },
            )
        })
        .unzip()
}
