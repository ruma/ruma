use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::{Metadata, Request, RequestField, RequestFieldKind};

impl Request {
    pub fn expand_incoming(
        &self,
        metadata: &Metadata,
        error_ty: &TokenStream,
        ruma_api: &TokenStream,
    ) -> TokenStream {
        let bytes = quote! { #ruma_api::exports::bytes };
        let http = quote! { #ruma_api::exports::http };
        let percent_encoding = quote! { #ruma_api::exports::percent_encoding };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde_json = quote! { #ruma_api::exports::serde_json };

        let method = &metadata.method;

        let incoming_request_type =
            if self.contains_lifetimes() { quote!(IncomingRequest) } else { quote!(Request) };

        let (parse_request_path, path_vars) = if self.has_path_fields() {
            let path_string = metadata.path.value();

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
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name = #ruma_serde::urlencoded::from_str(
                    &request.uri().query().unwrap_or(""),
                )?;
            };

            (parse, quote! { #field_name, })
        } else if self.has_query_fields() {
            let (decls, names) = self.vars(RequestFieldKind::Query, quote!(request_query));

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
                        let #field_name = match headers.get(#http::header::#header_name) {
                            Some(header_value) => {
                                let str_value = header_value.to_str()?;
                                #some_case
                            }
                            None => #none_case,
                        };
                    };

                    (decl, field_name)
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

        let extract_body = if self.has_body_fields() || self.newtype_body_field().is_some() {
            let body_lifetimes = if self.has_body_lifetimes() {
                // duplicate the anonymous lifetime as many times as needed
                let lifetimes = std::iter::repeat(quote! { '_ }).take(self.lifetimes.body.len());
                quote! { < #( #lifetimes, )* >}
            } else {
                TokenStream::new()
            };

            quote! {
                let request_body: <
                    RequestBody #body_lifetimes
                    as #ruma_serde::Outgoing
                >::Incoming = {
                    let body = request.into_body();
                    if #bytes::Buf::has_remaining(&body) {
                        #serde_json::from_reader(#bytes::Buf::reader(body))?
                    } else {
                        // If the request body is completely empty, pretend it is an empty JSON
                        // object instead. This allows requests with only optional body parameters
                        // to be deserialized in that case.
                        #serde_json::from_str("{}")?
                    }
                };
            }
        } else {
            TokenStream::new()
        };

        let (parse_body, body_vars) = if let Some(field) = self.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name = request_body.0;
            };

            (parse, quote! { #field_name, })
        } else if let Some(field) = self.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let parse = quote! {
                let #field_name = {
                    let mut reader = #bytes::Buf::reader(request.into_body());
                    let mut vec = ::std::vec::Vec::new();
                    ::std::io::Read::read_to_end(&mut reader, &mut vec)
                        .expect("reading from a bytes::Buf never fails");
                    vec
                };
            };

            (parse, quote! { #field_name, })
        } else {
            self.vars(RequestFieldKind::Body, quote!(request_body))
        };

        let non_auth_impls = metadata.authentication.iter().map(|auth| {
            if auth.value == "None" {
                let attrs = &auth.attrs;
                quote! {
                    #( #attrs )*
                    #[automatically_derived]
                    #[cfg(feature = "server")]
                    impl #ruma_api::IncomingNonAuthRequest for #incoming_request_type {}
                }
            } else {
                TokenStream::new()
            }
        });

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_api::IncomingRequest for #incoming_request_type {
                type EndpointError = #error_ty;
                type OutgoingResponse = Response;

                const METADATA: #ruma_api::Metadata = self::METADATA;

                fn try_from_http_request<T: #bytes::Buf>(
                    request: #http::Request<T>
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

                    Ok(Self {
                        #path_vars
                        #query_vars
                        #header_vars
                        #body_vars
                    })
                }
            }

            #(#non_auth_impls)*
        }
    }

    fn vars(
        &self,
        request_field_kind: RequestFieldKind,
        src: TokenStream,
    ) -> (TokenStream, TokenStream) {
        self.fields
            .iter()
            .filter_map(|f| f.field_of_kind(request_field_kind))
            .map(|field| {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                let decl = quote! {
                    #( #cfg_attrs )*
                    let #field_name = #src.#field_name;
                };

                (decl, quote! { #field_name, })
            })
            .unzip()
    }
}
