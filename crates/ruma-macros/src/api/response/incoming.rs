use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

use super::{Response, ResponseFieldKind};

impl Response {
    pub fn expand_incoming(&self, error_ty: &Type, ruma_common: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_common::exports::http };
        let serde_json = quote! { #ruma_common::exports::serde_json };

        let extract_response_headers = self.has_header_fields().then(|| {
            quote! {
                let mut headers = response.headers().clone();
            }
        });

        let typed_response_body_decl = self.has_body_fields().then(|| {
            quote! {
                let response_body: ResponseBody = {
                    let body = ::std::convert::AsRef::<[::std::primitive::u8]>::as_ref(
                        response.body(),
                    );

                    #serde_json::from_slice(match body {
                        // If the response body is completely empty, pretend it is an empty
                        // JSON object instead. This allows responses with only optional body
                        // parameters to be deserialized in that case.
                        [] => b"{}",
                        b => b,
                    })?
                };
            }
        });

        let response_init_fields = {
            let mut fields = vec![];
            let mut raw_body = None;

            for response_field in &self.fields {
                let field = &response_field.inner;
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path().is_ident("cfg")).collect::<Vec<_>>();

                fields.push(match &response_field.kind {
                    ResponseFieldKind::Body | ResponseFieldKind::NewtypeBody => {
                        quote! {
                            #( #cfg_attrs )*
                            #field_name: response_body.#field_name
                        }
                    }
                    ResponseFieldKind::Header(header_name) => {
                        let optional_header = match &field.ty {
                            Type::Path(syn::TypePath {
                                path: syn::Path { segments, .. }, ..
                            }) if segments.last().unwrap().ident == "Option" => {
                                quote! {
                                    #( #cfg_attrs )*
                                    #field_name: {
                                        headers.remove(#header_name)
                                            .map(|h| h.to_str().map(|s| s.to_owned()))
                                            .transpose()?
                                    }
                                }
                            }
                            _ => quote! {
                                #( #cfg_attrs )*
                                #field_name: {
                                    headers.remove(#header_name)
                                        .expect("response missing expected header")
                                        .to_str()?
                                        .to_owned()
                                }
                            },
                        };
                        quote! { #optional_header }
                    }
                    // This field must be instantiated last to avoid `use of move value` error.
                    // We are guaranteed only one new body field because of a check in
                    // `parse_response`.
                    ResponseFieldKind::RawBody => {
                        raw_body = Some(quote! {
                            #( #cfg_attrs )*
                            #field_name: {
                                ::std::convert::AsRef::<[::std::primitive::u8]>::as_ref(
                                    response.body(),
                                )
                                .to_vec()
                            }
                        });
                        // skip adding to the vec
                        continue;
                    }
                });
            }

            fields.extend(raw_body);

            quote! {
                #(#fields,)*
            }
        };

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #ruma_common::api::IncomingResponse for Response {
                type EndpointError = #error_ty;

                fn try_from_http_response<T: ::std::convert::AsRef<[::std::primitive::u8]>>(
                    response: #http::Response<T>,
                ) -> ::std::result::Result<
                    Self,
                    #ruma_common::api::error::FromHttpResponseError<#error_ty>,
                > {
                    if response.status().as_u16() < 400 {
                        #extract_response_headers
                        #typed_response_body_decl

                        ::std::result::Result::Ok(Self {
                            #response_init_fields
                        })
                    } else {
                        Err(#ruma_common::api::error::FromHttpResponseError::Server(
                            <#error_ty as #ruma_common::api::EndpointError>::from_http_response(
                                response,
                            )
                        ))
                    }
                }
            }
        }
    }
}
