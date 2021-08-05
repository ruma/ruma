use proc_macro2::TokenStream;
use quote::quote;

use super::{Response, ResponseField};

impl Response {
    pub fn expand_outgoing(&self, ruma_api: &TokenStream) -> TokenStream {
        let bytes = quote! { #ruma_api::exports::bytes };
        let http = quote! { #ruma_api::exports::http };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };

        let serialize_response_headers = self.fields.iter().map(|response_field| {
            if let ResponseField::Header(field, header_name) = response_field {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");

                match &field.ty {
                    syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                        if segments.last().unwrap().ident == "Option" =>
                    {
                        quote! {
                            if let Some(header) = self.#field_name {
                                headers.insert(
                                    #http::header::#header_name,
                                    header.parse()?,
                                );
                            }
                        }
                    }
                    _ => quote! {
                        headers.insert(
                            #http::header::#header_name,
                            self.#field_name.parse()?,
                        );
                    },
                }
            } else {
                TokenStream::new()
            }
        });

        let body = if let Some(field) = self.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { #ruma_serde::slice_to_buf(&self.#field_name) }
        } else if let Some(field) = self.newtype_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! {
                #ruma_serde::json_to_buf(&self.#field_name)?
            }
        } else {
            let fields = self.fields.iter().map(|response_field| {
                if let ResponseField::Body(field) = response_field {
                    let field_name =
                        field.ident.as_ref().expect("expected field to have an identifier");
                    let cfg_attrs = field.attrs.iter().filter(|a| a.path.is_ident("cfg"));

                    quote! {
                        #( #cfg_attrs )*
                        #field_name: self.#field_name,
                    }
                } else {
                    TokenStream::new()
                }
            });

            quote! {
                #ruma_serde::json_to_buf(&ResponseBody { #(#fields)* })?
            }
        };

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_api::OutgoingResponse for Response {
                fn try_into_http_response<T: ::std::default::Default + #bytes::BufMut>(
                    self,
                ) -> ::std::result::Result<#http::Response<T>, #ruma_api::error::IntoHttpError> {
                    let mut resp_builder = #http::Response::builder()
                        .header(#http::header::CONTENT_TYPE, "application/json");

                    if let Some(mut headers) = resp_builder.headers_mut() {
                        #(#serialize_response_headers)*
                    }

                    ::std::result::Result::Ok(resp_builder.body(#body)?)
                }
            }
        }
    }
}
