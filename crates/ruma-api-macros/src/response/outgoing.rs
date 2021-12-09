use proc_macro2::TokenStream;
use quote::quote;

use super::{Response, ResponseField};

impl Response {
    pub fn expand_outgoing(&self, ruma_api: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_api::exports::http };

        let outgoing_body_type = if self.has_raw_body() {
            quote! { #ruma_api::IncomingRawHttpBody }
        } else if self.has_body_fields() {
            quote! { ResponseBody }
        } else {
            quote! { #ruma_api::RawHttpBody<'static> }
        };

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
            })
        });

        let body = if let Some(field) =
            self.fields.iter().find_map(ResponseField::as_raw_body_field)
        {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { #ruma_api::IncomingRawHttpBody(self.#field_name) }
        } else if self.has_body_fields() {
            let fields = self.fields.iter().filter_map(|response_field| {
                response_field.as_body_field().map(|field| {
                    let field_name =
                        field.ident.as_ref().expect("expected field to have an identifier");
                    let cfg_attrs = field.attrs.iter().filter(|a| a.path.is_ident("cfg"));

                    quote! {
                        #( #cfg_attrs )*
                        #field_name: self.#field_name,
                    }
                })
            });

            quote! {
                ResponseBody { #(#fields)* }
            }
        } else {
            quote! {
                #ruma_api::RawHttpBody(b"{}")
            }
        };

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_api::OutgoingResponse for Response {
                type OutgoingBody = #outgoing_body_type; // impl #ruma_api::IntoHttpBody;

                fn try_into_http_response(
                    self,
                ) -> ::std::result::Result<
                    #http::Response<Self::OutgoingBody>,
                    #ruma_api::error::IntoHttpError,
                > {
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
