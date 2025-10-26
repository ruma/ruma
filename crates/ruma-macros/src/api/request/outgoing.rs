use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;

use super::{Request, RequestField};

impl Request {
    pub fn expand_outgoing(&self, ruma_common: &TokenStream) -> TokenStream {
        let bytes = quote! { #ruma_common::exports::bytes };
        let http = quote! { #ruma_common::exports::http };
        let serde_html_form = quote! { #ruma_common::exports::serde_html_form };

        let error_ty = &self.error_ty;

        let path_fields =
            self.path_fields().map(|f| f.ident.as_ref().expect("path fields have a name"));

        let request_query_string = if let Some(field) = self.query_all_field() {
            let field_name = field.ident.as_ref().expect("expected field to have identifier");

            quote! {{
                let request_query = RequestQuery(self.#field_name);

                &#serde_html_form::to_string(request_query)?
            }}
        } else if self.has_query_fields() {
            let request_query_init_fields = struct_init_fields(
                self.fields.iter().filter_map(RequestField::as_query_field),
                quote! { self },
            );

            quote! {{
                let request_query = RequestQuery {
                    #request_query_init_fields
                };

                &#serde_html_form::to_string(request_query)?
            }}
        } else {
            quote! { "" }
        };

        // If there are no body fields, the request body will be empty (not `{}`), so the
        // `application/json` content-type would be wrong. It may also cause problems with CORS
        // policies that don't allow the `Content-Type` header (for things such as `.well-known`
        // that are commonly handled by something else than a homeserver).
        let mut header_kvs = if self.has_body_fields() {
            quote! {
                req_headers.insert(
                    #http::header::CONTENT_TYPE,
                    #ruma_common::http_headers::APPLICATION_JSON,
                );
            }
        } else if self.raw_body_field().is_some() {
            quote! {
                req_headers.insert(
                    #http::header::CONTENT_TYPE,
                    #ruma_common::http_headers::APPLICATION_OCTET_STREAM,
                );
            }
        } else {
            TokenStream::new()
        };

        header_kvs.extend(self.header_fields().map(|(field, header_name)| {
            let field_name = &field.ident;

            match &field.ty {
                syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                    if segments.last().unwrap().ident == "Option" =>
                {
                    quote! {
                        if let Some(header_val) = self.#field_name.as_ref() {
                            req_headers.insert(
                                #header_name,
                                #http::header::HeaderValue::from_str(&header_val.to_string())?,
                            );
                        }
                    }
                }
                _ => quote! {
                    req_headers.insert(
                        #header_name,
                        #http::header::HeaderValue::from_str(&self.#field_name.to_string())?,
                    );
                },
            }
        }));

        if !header_kvs.is_empty() {
            header_kvs = quote! {
                {
                    let req_headers = http_request.headers_mut();
                    #header_kvs
                }
            };
        }

        let request_body = if let Some(field) = self.raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { #ruma_common::serde::slice_to_buf(&self.#field_name) }
        } else if self.has_body_fields() {
            let initializers = struct_init_fields(self.body_fields(), quote! { self });

            quote! {
                #ruma_common::serde::json_to_buf(&RequestBody { #initializers })?
            }
        } else {
            quote! { <Self as #ruma_common::api::Metadata>::empty_request_body::<T>() }
        };

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #impl_generics #ruma_common::api::OutgoingRequest for Request #ty_generics #where_clause {
                type EndpointError = #error_ty;
                type IncomingResponse = Response;

                fn try_into_http_request<T: ::std::default::Default + #bytes::BufMut + ::std::convert::AsRef<[::std::primitive::u8]>>(
                    self,
                    base_url: &::std::primitive::str,
                    authentication_input: <<Self as #ruma_common::api::Metadata>::Authentication as #ruma_common::api::auth_scheme::AuthScheme>::Input<'_>,
                    path_builder_input: <<Self as #ruma_common::api::Metadata>::PathBuilder as #ruma_common::api::path_builder::PathBuilder>::Input<'_>,
                ) -> ::std::result::Result<#http::Request<T>, #ruma_common::api::error::IntoHttpError> {
                    let mut http_request = #http::Request::builder()
                        .method(<Self as #ruma_common::api::Metadata>::METHOD)
                        .uri(<Self as #ruma_common::api::Metadata>::make_endpoint_url(
                            path_builder_input,
                            base_url,
                            &[ #( &self.#path_fields ),* ],
                            #request_query_string,
                        )?)
                        .body(#request_body)?;

                    #header_kvs

                    <<Self as #ruma_common::api::Metadata>::Authentication as #ruma_common::api::auth_scheme::AuthScheme>::add_authentication(
                        &mut http_request,
                        authentication_input
                    )
                        .map_err(|error| #ruma_common::api::error::IntoHttpError::Authentication(error.into()))?;

                    Ok(http_request)
                }
            }
        }
    }
}

/// Produces code for a struct initializer for the given field kind to be accessed through the
/// given variable name.
fn struct_init_fields<'a>(
    fields: impl IntoIterator<Item = &'a Field>,
    src: TokenStream,
) -> TokenStream {
    fields
        .into_iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            let cfg_attrs =
                field.attrs.iter().filter(|a| a.path().is_ident("cfg")).collect::<Vec<_>>();

            quote! {
                #( #cfg_attrs )*
                #field_name: #src.#field_name,
            }
        })
        .collect()
}
