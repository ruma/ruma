use proc_macro2::TokenStream;
use quote::quote;

use super::{Response, ResponseField};

impl Response {
    pub fn expand_incoming(&self, error_ty: &TokenStream, ruma_api: &TokenStream) -> TokenStream {
        let bytes = quote! { #ruma_api::exports::bytes };
        let http = quote! { #ruma_api::exports::http };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde_json = quote! { #ruma_api::exports::serde_json };

        let extract_response_headers = if self.has_header_fields() {
            quote! {
                let mut headers = response.headers().clone();
            }
        } else {
            TokenStream::new()
        };

        let typed_response_body_decl =
            if self.has_body_fields() || self.newtype_body_field().is_some() {
                quote! {
                    let response_body: <
                        ResponseBody
                        as #ruma_serde::Outgoing
                    >::Incoming = {
                        let body = response.into_body();
                        if #bytes::Buf::has_remaining(&body) {
                            #serde_json::from_reader(#bytes::Buf::reader(body))?
                        } else {
                            // If the reponse body is completely empty, pretend it is an empty JSON
                            // object instead. This allows responses with only optional body
                            // parameters to be deserialized in that case.
                            #serde_json::from_str("{}")?
                        }
                    };
                }
            } else {
                TokenStream::new()
            };

        let response_init_fields = {
            let mut fields = vec![];
            let mut new_type_raw_body = None;

            for response_field in &self.fields {
                let field = response_field.field();
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                fields.push(match response_field {
                    ResponseField::Body(_) => {
                        quote! {
                            #( #cfg_attrs )*
                            #field_name: response_body.#field_name
                        }
                    }
                    ResponseField::Header(_, header_name) => {
                        let optional_header = match &field.ty {
                            syn::Type::Path(syn::TypePath {
                                path: syn::Path { segments, .. },
                                ..
                            }) if segments.last().unwrap().ident == "Option" => {
                                quote! {
                                    #field_name: {
                                        headers.remove(#http::header::#header_name)
                                            .map(|h| h.to_str().map(|s| s.to_owned()))
                                            .transpose()?
                                    }
                                }
                            }
                            _ => quote! {
                                #field_name: {
                                    headers.remove(#http::header::#header_name)
                                        .expect("response missing expected header")
                                        .to_str()?
                                        .to_owned()
                                }
                            },
                        };
                        quote! { #optional_header }
                    }
                    ResponseField::NewtypeBody(_) => {
                        quote! {
                            #field_name: response_body.0
                        }
                    }
                    // This field must be instantiated last to avoid `use of move value` error.
                    // We are guaranteed only one new body field because of a check in `try_from`.
                    ResponseField::NewtypeRawBody(_) => {
                        new_type_raw_body = Some(quote! {
                            #field_name: {
                                let mut reader = #bytes::Buf::reader(response.into_body());
                                let mut vec = ::std::vec::Vec::new();
                                ::std::io::Read::read_to_end(&mut reader, &mut vec)
                                    .expect("reading from a bytes::Buf never fails");
                                vec
                            }
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
        };

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #ruma_api::IncomingResponse for Response {
                type EndpointError = #error_ty;

                fn try_from_http_response<T: #bytes::Buf>(
                    response: #http::Response<T>,
                ) -> ::std::result::Result<
                    Self,
                    #ruma_api::error::FromHttpResponseError<#error_ty>,
                > {
                    if response.status().as_u16() < 400 {
                        #extract_response_headers
                        #typed_response_body_decl

                        Ok(Self {
                            #response_init_fields
                        })
                    } else {
                        match <#error_ty as #ruma_api::EndpointError>::try_from_response(response) {
                            Ok(err) => Err(#ruma_api::error::ServerError::Known(err).into()),
                            Err(response_err) => {
                                Err(#ruma_api::error::ServerError::Unknown(response_err).into())
                            }
                        }
                    }
                }
            }
        }
    }
}
