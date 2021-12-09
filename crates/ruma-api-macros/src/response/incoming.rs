use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

use super::{Response, ResponseField};

impl Response {
    pub fn expand_incoming(&self, error_ty: &Type, ruma_api: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_api::exports::http };

        let incoming_body_type = if !self.has_body_fields() || self.has_raw_body() {
            quote! { #ruma_api::IncomingRawHttpBody }
        } else {
            quote! { ResponseBody }
        };

        let extract_response_headers = self.has_header_fields().then(|| {
            quote! {
                let mut headers = response.headers().clone();
            }
        });

        let extract_body = self.has_body_fields().then(|| {
            quote! {
                let response_body: ResponseBody = response.into_body();
            }
        });

        let response_init_fields = {
            let mut fields = vec![];
            let mut raw_body = None;

            for response_field in &self.fields {
                let field = response_field.field();
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                fields.push(match response_field {
                    ResponseField::Body(_) | ResponseField::NewtypeBody(_) => {
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
                                    #( #cfg_attrs )*
                                    #field_name: {
                                        headers.remove(#http::header::#header_name)
                                            .map(|h| h.to_str().map(|s| s.to_owned()))
                                            .transpose()?
                                    }
                                }
                            }
                            _ => quote! {
                                #( #cfg_attrs )*
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
                    // This field must be instantiated last to avoid `use of move value` error.
                    // We are guaranteed only one new body field because of a check in
                    // `parse_response`.
                    ResponseField::RawBody(_) => {
                        raw_body = Some(quote! {
                            #( #cfg_attrs )*
                            #field_name: response.into_body().0
                        });
                        // skip adding to the vec
                        continue;
                    }
                });
            }

            fields.extend(raw_body);

            quote! {
                #(#fields),*
            }
        };

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #ruma_api::IncomingResponse for Response {
                type IncomingBody = #incoming_body_type;
                // impl #ruma_api::FromHttpBody<
                //     #ruma_api::error::FromHttpResponseError<Self::EndpointError>,
                // >;
                type EndpointError = #error_ty;

                fn try_from_http_response(
                    response: #http::Response<Self::IncomingBody>,
                ) -> ::std::result::Result<
                    Self,
                    #ruma_api::error::FromHttpResponseError<#error_ty>,
                > {
                    #extract_response_headers
                    #extract_body

                    ::std::result::Result::Ok(Self { #response_init_fields })
                }
            }
        }
    }
}
