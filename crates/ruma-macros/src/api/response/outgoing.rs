use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::{Response, ResponseField};

impl Response {
    pub fn expand_outgoing(&self, status_ident: &Ident, ruma_common: &TokenStream) -> TokenStream {
        let bytes = quote! { #ruma_common::exports::bytes };
        let http = quote! { #ruma_common::exports::http };

        let serialize_response_headers = self.fields.iter().filter_map(|response_field| {
            response_field.as_header_field().map(|(field, header_name)| {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");

                match &field.ty {
                    syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                        if segments.last().unwrap().ident == "Option" =>
                    {
                        quote! {
                            if let Some(header) = self.#field_name {
                                headers.insert(
                                    #header_name,
                                    header.parse()?,
                                );
                            }
                        }
                    }
                    _ => quote! {
                        headers.insert(
                            #header_name,
                            self.#field_name.parse()?,
                        );
                    },
                }
            })
        });

        let body = if let Some(field) =
            self.fields.iter().find_map(ResponseField::as_raw_body_field)
        {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { #ruma_common::serde::slice_to_buf(&self.#field_name) }
        } else {
            let fields = self.fields.iter().filter_map(|response_field| {
                response_field.as_body_field().map(|field| {
                    let field_name =
                        field.ident.as_ref().expect("expected field to have an identifier");
                    let cfg_attrs = field.attrs.iter().filter(|a| a.path().is_ident("cfg"));

                    quote! {
                        #( #cfg_attrs )*
                        #field_name: self.#field_name,
                    }
                })
            });

            quote! {
                #ruma_common::serde::json_to_buf(&ResponseBody { #(#fields)* })?
            }
        };

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_common::api::OutgoingResponse for Response {
                fn try_into_http_response<T: ::std::default::Default + #bytes::BufMut>(
                    self,
                ) -> ::std::result::Result<#http::Response<T>, #ruma_common::api::error::IntoHttpError> {
                    let mut resp_builder = #http::Response::builder()
                        .status(#http::StatusCode::#status_ident)
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
