use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use super::{Request, RequestBody, RequestHeaders, RequestQuery};
use crate::util::{StructFieldExt, TypeExt};

impl Request {
    /// Generate the `ruma_common::api::OutgoingRequest` implementation for this request struct.
    pub fn expand_outgoing(&self, ruma_common: &TokenStream) -> TokenStream {
        let bytes = quote! { #ruma_common::exports::bytes };
        let http = quote! { #ruma_common::exports::http };

        let path_fields = self.path.expand_fields();
        let path_idents = self.path.0.iter().map(|field| field.ident());

        let query_serialize = self.query.expand_serialize(ruma_common);
        let query_fields = self.query.expand_fields();

        let headers_serialize = self.headers.expand_serialize(&self.body, ruma_common, &http);
        let headers_fields = self.headers.expand_fields();

        let body_serialize = self.body.expand_serialize(ruma_common);
        let body_fields = self.body.expand_fields();

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let error_ty = &self.error_ty;

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            impl #impl_generics #ruma_common::api::OutgoingRequest for #ident #ty_generics #where_clause {
                type EndpointError = #error_ty;
                type IncomingResponse = Response;

                fn try_into_http_request<T: ::std::default::Default + #bytes::BufMut + ::std::convert::AsRef<[::std::primitive::u8]>>(
                    self,
                    base_url: &::std::primitive::str,
                    authentication_input: <<Self as #ruma_common::api::Metadata>::Authentication as #ruma_common::api::auth_scheme::AuthScheme>::Input<'_>,
                    path_builder_input: <<Self as #ruma_common::api::Metadata>::PathBuilder as #ruma_common::api::path_builder::PathBuilder>::Input<'_>,
                ) -> ::std::result::Result<#http::Request<T>, #ruma_common::api::error::IntoHttpError> {
                    let Self {
                        #path_fields
                        #query_fields
                        #headers_fields
                        #body_fields
                    } = self;

                    let request_query_string = #query_serialize;

                    let mut http_request = #http::Request::builder()
                        .method(<Self as #ruma_common::api::Metadata>::METHOD)
                        .uri(<Self as #ruma_common::api::Metadata>::make_endpoint_url(
                            path_builder_input,
                            base_url,
                            &[ #( &#path_idents ),* ],
                            &request_query_string,
                        )?)
                        .body(#body_serialize)?;

                    #headers_serialize

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

impl RequestQuery {
    /// Generate code to serialize the query string.
    fn expand_serialize(&self, ruma_common: &TokenStream) -> TokenStream {
        if matches!(self, Self::None) {
            return quote! { "" };
        }

        let serde_html_form = quote! { #ruma_common::exports::serde_html_form };
        let fields = self.expand_fields();

        quote! {{
            let request_query = RequestQuery {
                #fields
            };

            &#serde_html_form::to_string(request_query)?
        }}
    }
}

impl RequestHeaders {
    /// Generate code to serialize the headers for a `http::request::Request`.
    fn expand_serialize(
        &self,
        body: &RequestBody,
        ruma_common: &TokenStream,
        http: &TokenStream,
    ) -> Option<TokenStream> {
        let mut serialize = TokenStream::new();

        // If there is no `CONTENT_TYPE` header, add one if necessary.
        let content_type: syn::Ident = parse_quote!(CONTENT_TYPE);
        if !self.0.contains_key(&content_type)
            && let Some(content_type) = body.content_type()
        {
            serialize.extend(quote! {
                headers.insert(
                    #http::header::CONTENT_TYPE,
                    #ruma_common::http_headers::#content_type,
                );
            });
        }

        if serialize.is_empty() && self.0.is_empty() {
            return None;
        }

        for (header_name, field) in &self.0 {
            let ident = field.ident();
            let cfg_attrs = field.cfg_attrs();

            let header = if field.ty.option_inner_type().is_some() {
                quote! {
                    #( #cfg_attrs )*
                    if let Some(header_val) = #ident.as_ref() {
                        headers.insert(
                            #header_name,
                            #http::header::HeaderValue::from_str(&header_val.to_string())?,
                        );
                    }
                }
            } else {
                quote! {
                    #( #cfg_attrs )*
                    headers.insert(
                        #header_name,
                        #http::header::HeaderValue::from_str(&#ident.to_string())?,
                    );
                }
            };

            serialize.extend(header);
        }

        Some(quote! {{
            let headers = http_request.headers_mut();
            #serialize
        }})
    }
}

impl RequestBody {
    /// The content type of the body, if it can be determined.
    ///
    /// Returns a `const` from `ruma_common::http_headers`.
    fn content_type(&self) -> Option<syn::Ident> {
        match &self {
            Self::Empty => {
                // If there are no body fields, the request body might be empty (not `{}`), so the
                // `application/json` content-type would be wrong. It may also cause problems with
                // CORS policies that don't allow the `Content-Type` header (for things such as
                // `.well-known` that are commonly handled by something else than a
                // homeserver). However, a server should always return a JSON body.
                None
            }
            Self::JsonFields(_) | Self::JsonAll(_) => Some(parse_quote! { APPLICATION_JSON }),
            // This might not be the actual content type, but this is a better default than
            // `application/json` when sending raw data.
            Self::Raw(_) => Some(parse_quote! { APPLICATION_OCTET_STREAM }),
        }
    }

    /// Generate code to serialize the body.
    fn expand_serialize(&self, ruma_common: &TokenStream) -> TokenStream {
        match self {
            Self::Empty => {
                quote! { <Self as #ruma_common::api::Metadata>::empty_request_body::<T>() }
            }
            Self::JsonFields(_) => self.expand_serialize_json(ruma_common),
            Self::JsonAll(_) => self.expand_serialize_json(ruma_common),
            Self::Raw(field) => {
                let ident = field.ident();
                quote! { #ruma_common::serde::slice_to_buf(&#ident) }
            }
        }
    }

    /// Generate code to serialize the JSON body with the given fields.
    fn expand_serialize_json(&self, ruma_common: &TokenStream) -> TokenStream {
        let fields = self.expand_fields();

        quote! {
            #ruma_common::serde::json_to_buf(&RequestBody { #fields })?
        }
    }
}
