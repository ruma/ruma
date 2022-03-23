use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;

use super::{Request, RequestField};
use crate::api::auth_scheme::AuthScheme;

impl Request {
    pub fn expand_incoming(&self, ruma_common: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_common::exports::http };
        let serde = quote! { #ruma_common::exports::serde };
        let serde_json = quote! { #ruma_common::exports::serde_json };

        let method = &self.method;
        let error_ty = &self.error_ty;

        let incoming_request_type = if self.has_lifetimes() {
            quote! { IncomingRequest }
        } else {
            quote! { Request }
        };

        // FIXME: the rest of the field initializer expansions are gated `cfg(...)`
        // except this one. If we get errors about missing fields in IncomingRequest for
        // a path field look here.
        let (parse_request_path, path_vars) = if self.has_path_fields() {
            let path_vars: Vec<_> =
                self.path_fields_ordered().filter_map(|f| f.ident.as_ref()).collect();

            let parse_request_path = quote! {
                let (#(#path_vars,)*) = #serde::Deserialize::deserialize(
                    #serde::de::value::SeqDeserializer::<_, #serde::de::value::Error>::new(
                        path_args.iter().map(::std::convert::AsRef::as_ref)
                    )
                )?;
            };

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
                let #field_name = #ruma_common::serde::urlencoded::from_str(
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

            let request_query_ty = if self.lifetimes.query.is_empty() {
                quote! { RequestQuery }
            } else {
                quote! { IncomingRequestQuery }
            };

            let parse = quote! {
                let request_query: #request_query_ty = #ruma_common::serde::urlencoded::from_str(
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
                                    #ruma_common::api::error::HeaderDeserializationError::MissingHeader(
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
            let request_body_ty = if self.lifetimes.body.is_empty() {
                quote! { RequestBody }
            } else {
                quote! { IncomingRequestBody }
            };

            quote! {
                let request_body: #request_body_ty = {
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
            }
        });

        let (parse_body, body_vars) = if let Some(field) = self.raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name =
                    ::std::convert::AsRef::<[u8]>::as_ref(request.body()).to_vec();
            };

            (parse, quote! { #field_name, })
        } else {
            vars(self.body_fields(), quote! { request_body })
        };

        let non_auth_impl = matches!(self.authentication, AuthScheme::None(_)).then(|| {
            quote! {
                #[automatically_derived]
                #[cfg(feature = "server")]
                impl #ruma_common::api::IncomingNonAuthRequest for #incoming_request_type {}
            }
        });

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_common::api::IncomingRequest for #incoming_request_type {
                type EndpointError = #error_ty;
                type OutgoingResponse = Response;

                const METADATA: #ruma_common::api::Metadata = self::METADATA;

                fn try_from_http_request<B, S>(
                    request: #http::Request<B>,
                    path_args: &[S],
                ) -> ::std::result::Result<Self, #ruma_common::api::error::FromHttpRequestError>
                where
                    B: ::std::convert::AsRef<[::std::primitive::u8]>,
                    S: ::std::convert::AsRef<::std::primitive::str>,
                {
                    if request.method() != #http::Method::#method {
                        return Err(#ruma_common::api::error::FromHttpRequestError::MethodMismatch {
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
