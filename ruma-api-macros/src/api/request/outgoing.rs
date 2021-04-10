use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::{Metadata, Request, RequestField, RequestFieldKind};

impl Request {
    pub fn expand_outgoing(
        &self,
        metadata: &Metadata,
        error_ty: &TokenStream,
        lifetimes: &TokenStream,
        ruma_api: &TokenStream,
    ) -> TokenStream {
        let http = quote! { #ruma_api::exports::http };
        let percent_encoding = quote! { #ruma_api::exports::percent_encoding };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };
        let serde_json = quote! { #ruma_api::exports::serde_json };

        let method = &metadata.method;
        let request_path_string = if self.has_path_fields() {
            let mut format_string = metadata.path.value();
            let mut format_args = Vec::new();

            while let Some(start_of_segment) = format_string.find(':') {
                // ':' should only ever appear at the start of a segment
                assert_eq!(&format_string[start_of_segment - 1..start_of_segment], "/");

                let end_of_segment = match format_string[start_of_segment..].find('/') {
                    Some(rel_pos) => start_of_segment + rel_pos,
                    None => format_string.len(),
                };

                let path_var = Ident::new(
                    &format_string[start_of_segment + 1..end_of_segment],
                    Span::call_site(),
                );
                format_args.push(quote! {
                    #percent_encoding::utf8_percent_encode(
                        &self.#path_var.to_string(),
                        #percent_encoding::NON_ALPHANUMERIC,
                    )
                });
                format_string.replace_range(start_of_segment..end_of_segment, "{}");
            }

            quote! {
                format_args!(#format_string, #(#format_args),*)
            }
        } else {
            quote! { metadata.path.to_owned() }
        };

        let request_query_string = if let Some(field) = self.query_map_field() {
            let field_name = field.ident.as_ref().expect("expected field to have identifier");

            quote!({
                // This function exists so that the compiler will throw an error when the type of
                // the field with the query_map attribute doesn't implement
                // `IntoIterator<Item = (String, String)>`.
                //
                // This is necessary because the `ruma_serde::urlencoded::to_string` call will
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
                    #ruma_serde::urlencoded::to_string(request_query)?
                )
            })
        } else if self.has_query_fields() {
            let request_query_init_fields =
                self.struct_init_fields(RequestFieldKind::Query, quote!(self));

            quote!({
                let request_query = RequestQuery {
                    #request_query_init_fields
                };

                format_args!(
                    "?{}",
                    #ruma_serde::urlencoded::to_string(request_query)?
                )
            })
        } else {
            quote! { "" }
        };

        let mut header_kvs: TokenStream = self
            .header_fields()
            .map(|request_field| {
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
            })
            .collect();

        for auth in &metadata.authentication {
            if auth.value == "AccessToken" {
                let attrs = &auth.attrs;
                header_kvs.extend(quote! {
                    #( #attrs )*
                    req_headers.insert(
                        #http::header::AUTHORIZATION,
                        #http::header::HeaderValue::from_str(
                            &::std::format!(
                                "Bearer {}",
                                access_token.ok_or(
                                    #ruma_api::error::IntoHttpError::NeedsAuthentication
                                )?
                            )
                        )?
                    );
                });
            }
        }

        let request_body = if let Some(field) = self.newtype_raw_body_field() {
            let field_name = field.ident.as_ref().expect("expected field to have an identifier");
            quote! { self.#field_name }
        } else if self.has_body_fields() || self.newtype_body_field().is_some() {
            let request_body_initializers = if let Some(field) = self.newtype_body_field() {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                quote! { (self.#field_name) }
            } else {
                let initializers = self.struct_init_fields(RequestFieldKind::Body, quote!(self));
                quote! { { #initializers } }
            };

            quote! {
                {
                    let request_body = RequestBody #request_body_initializers;
                    #serde_json::to_vec(&request_body)?
                }
            }
        } else {
            quote! { Vec::new() }
        };

        let non_auth_impls = metadata.authentication.iter().map(|auth| {
            if auth.value == "None" {
                let attrs = &auth.attrs;
                quote! {
                    #( #attrs )*
                    #[automatically_derived]
                    #[cfg(feature = "client")]
                    impl #lifetimes #ruma_api::OutgoingNonAuthRequest for Request #lifetimes {}
                }
            } else {
                TokenStream::new()
            }
        });

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #lifetimes #ruma_api::OutgoingRequest for Request #lifetimes {
                type EndpointError = #error_ty;
                type IncomingResponse = <Response as #ruma_serde::Outgoing>::Incoming;

                const METADATA: #ruma_api::Metadata = self::METADATA;

                fn try_into_http_request(
                    self,
                    base_url: &::std::primitive::str,
                    access_token: ::std::option::Option<&str>,
                ) -> ::std::result::Result<
                    #http::Request<Vec<u8>>,
                    #ruma_api::error::IntoHttpError,
                > {
                    let metadata = self::METADATA;

                    let mut req_builder = #http::Request::builder()
                        .method(#http::Method::#method)
                        .uri(::std::format!(
                            "{}{}{}",
                            base_url.strip_suffix('/').unwrap_or(base_url),
                            #request_path_string,
                            #request_query_string,
                        ))
                        .header(
                            #ruma_api::exports::http::header::CONTENT_TYPE,
                            "application/json",
                        );

                    let mut req_headers = req_builder
                        .headers_mut()
                        .expect("`http::RequestBuilder` is in unusable state");

                    #header_kvs

                    let http_request = req_builder.body(#request_body)?;

                    Ok(http_request)
                }
            }

            #(#non_auth_impls)*
        }
    }

    /// Produces code for a struct initializer for the given field kind to be accessed through the
    /// given variable name.
    fn struct_init_fields(
        &self,
        request_field_kind: RequestFieldKind,
        src: TokenStream,
    ) -> TokenStream {
        self.fields
            .iter()
            .filter_map(|f| f.field_of_kind(request_field_kind))
            .map(|field| {
                let field_name =
                    field.ident.as_ref().expect("expected field to have an identifier");
                let cfg_attrs =
                    field.attrs.iter().filter(|a| a.path.is_ident("cfg")).collect::<Vec<_>>();

                quote! {
                    #( #cfg_attrs )*
                    #field_name: #src.#field_name,
                }
            })
            .collect()
    }
}
