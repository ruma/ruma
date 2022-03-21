use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, LitStr};

use super::{Request, RequestField};
use crate::api::{auth_scheme::AuthScheme, util};

impl Request {
    pub fn expand_outgoing(&self, ruma_common: &TokenStream) -> TokenStream {
        let bytes = quote! { #ruma_common::exports::bytes };
        let http = quote! { #ruma_common::exports::http };
        let percent_encoding = quote! { #ruma_common::exports::percent_encoding };

        let method = &self.method;
        let error_ty = &self.error_ty;

        let (unstable_path, r0_path, stable_path) = if self.has_path_fields() {
            let path_format_args_call_with_percent_encoding = |s: &LitStr| -> TokenStream {
                util::path_format_args_call(s.value(), &percent_encoding)
            };

            (
                self.unstable_path.as_ref().map(path_format_args_call_with_percent_encoding),
                self.r0_path.as_ref().map(path_format_args_call_with_percent_encoding),
                self.stable_path.as_ref().map(path_format_args_call_with_percent_encoding),
            )
        } else {
            (
                self.unstable_path.as_ref().map(|path| quote! { format_args!(#path) }),
                self.r0_path.as_ref().map(|path| quote! { format_args!(#path) }),
                self.stable_path.as_ref().map(|path| quote! { format_args!(#path) }),
            )
        };

        let unstable_path = util::map_option_literal(&unstable_path);
        let r0_path = util::map_option_literal(&r0_path);
        let stable_path = util::map_option_literal(&stable_path);

        let request_query_string = if let Some(field) = self.query_map_field() {
            let field_name = field.ident.as_ref().expect("expected field to have identifier");

            quote! {{
                // This function exists so that the compiler will throw an error when the type of
                // the field with the query_map attribute doesn't implement
                // `IntoIterator<Item = (String, String)>`.
                //
                // This is necessary because the `ruma_common::serde::urlencoded::to_string` call will
                // result in a runtime error when the type cannot be encoded as a list key-value
                // pairs (?key1=value1&key2=value2).
                //
                // By asserting that it implements the iterator trait, we can ensure that it won't
                // fail.
                fn assert_trait_impl<T>(_: &T)
                where
                    T: ::std::iter::IntoIterator<
                        Item = (::std::string::String, ::std::string::String),
                    >,
                {}

                let request_query = RequestQuery(self.#field_name);
                assert_trait_impl(&request_query.0);

                format_args!(
                    "?{}",
                    #ruma_common::serde::urlencoded::to_string(request_query)?
                )
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

                format_args!(
                    "?{}",
                    #ruma_common::serde::urlencoded::to_string(request_query)?
                )
            }}
        } else {
            quote! { "" }
        };

        // If there are no body fields, the request body will be empty (not `{}`), so the
        // `application/json` content-type would be wrong. It may also cause problems with CORS
        // policies that don't allow the `Content-Type` header (for things such as `.well-known`
        // that are commonly handled by something else than a homeserver).
        let mut header_kvs = if self.raw_body_field().is_some() || self.has_body_fields() {
            quote! {
                req_headers.insert(
                    #http::header::CONTENT_TYPE,
                    #http::header::HeaderValue::from_static("application/json"),
                );
            }
        } else {
            TokenStream::new()
        };

        header_kvs.extend(self.header_fields().map(|request_field| {
            let (field, header_name) = match request_field {
                RequestField::Header(field, header_name) => (field, header_name),
                _ => unreachable!("expected request field to be header variant"),
            };

            let field_name = &field.ident;

            match &field.ty {
                syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. })
                    if segments.last().unwrap().ident == "Option" =>
                {
                    quote! {
                        if let Some(header_val) = self.#field_name.as_ref() {
                            req_headers.insert(
                                #http::header::#header_name,
                                #http::header::HeaderValue::from_str(header_val)?,
                            );
                        }
                    }
                }
                _ => quote! {
                    req_headers.insert(
                        #http::header::#header_name,
                        #http::header::HeaderValue::from_str(self.#field_name.as_ref())?,
                    );
                },
            }
        }));

        header_kvs.extend(match self.authentication {
            AuthScheme::AccessToken(_) => quote! {
                req_headers.insert(
                    #http::header::AUTHORIZATION,
                    ::std::convert::TryFrom::<_>::try_from(::std::format!(
                        "Bearer {}",
                        access_token
                            .get_required_for_endpoint()
                            .ok_or(#ruma_common::api::error::IntoHttpError::NeedsAuthentication)?,
                    ))?,
                );
            },
            AuthScheme::None(_) => quote! {
                if let Some(access_token) = access_token.get_not_required_for_endpoint() {
                    req_headers.insert(
                        #http::header::AUTHORIZATION,
                        ::std::convert::TryFrom::<_>::try_from(
                            ::std::format!("Bearer {}", access_token),
                        )?
                    );
                }
            },
            AuthScheme::QueryOnlyAccessToken(_) | AuthScheme::ServerSignatures(_) => quote! {},
        });

        let request_body = if let Some(field) = self.raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { #ruma_common::serde::slice_to_buf(&self.#field_name) }
        } else if self.has_body_fields() {
            let initializers = struct_init_fields(self.body_fields(), quote! { self });

            quote! {
                #ruma_common::serde::json_to_buf(&RequestBody { #initializers })?
            }
        } else if method == "GET" {
            quote! { <T as ::std::default::Default>::default() }
        } else {
            quote! { #ruma_common::serde::slice_to_buf(b"{}") }
        };

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let non_auth_impl = matches!(self.authentication, AuthScheme::None(_)).then(|| {
            quote! {
                #[automatically_derived]
                #[cfg(feature = "client")]
                impl #impl_generics #ruma_common::api::OutgoingNonAuthRequest
                    for Request #ty_generics #where_clause {}
            }
        });

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #impl_generics #ruma_common::api::OutgoingRequest for Request #ty_generics #where_clause {
                type EndpointError = #error_ty;
                type IncomingResponse = Response;

                const METADATA: #ruma_common::api::Metadata = self::METADATA;

                fn try_into_http_request<T: ::std::default::Default + #bytes::BufMut>(
                    self,
                    base_url: &::std::primitive::str,
                    access_token: #ruma_common::api::SendAccessToken<'_>,
                    considering_versions: &'_ [#ruma_common::api::MatrixVersion],
                ) -> ::std::result::Result<#http::Request<T>, #ruma_common::api::error::IntoHttpError> {
                    let metadata = self::METADATA;

                    let mut req_builder = #http::Request::builder()
                        .method(#http::Method::#method)
                        .uri(::std::format!(
                            "{}{}{}",
                            base_url.strip_suffix('/').unwrap_or(base_url),
                            #ruma_common::api::select_path(considering_versions, &metadata, #unstable_path, #r0_path, #stable_path)?,
                            #request_query_string,
                        ));

                    if let Some(mut req_headers) = req_builder.headers_mut() {
                        #header_kvs
                    }

                    let http_request = req_builder.body(#request_body)?;

                    Ok(http_request)
                }
            }

            #non_auth_impl
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
                field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

            quote! {
                #( #cfg_attrs )*
                #field_name: #src.#field_name,
            }
        })
        .collect()
}
