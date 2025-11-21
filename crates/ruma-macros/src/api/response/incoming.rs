use proc_macro2::TokenStream;
use quote::quote;

use super::{KIND, Response};

impl Response {
    /// Generate the `ruma_common::api::IncomingResponse` implementation for this response struct.
    pub fn expand_incoming(&self, ruma_common: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_common::exports::http };

        let headers_parse = self.headers.expand_parse(KIND, ruma_common);
        let headers_fields = self.headers.expand_fields();

        let body_parse = self.body.expand_parse(KIND, ruma_common);
        let body_fields = self.body.expand_fields();

        let ident = &self.ident;
        let error_ty = &self.error_ty;
        let src = KIND.as_variable_ident();

        quote! {
            #[automatically_derived]
            #[cfg(feature = "client")]
            #[allow(deprecated)]
            impl #ruma_common::api::IncomingResponse for #ident {
                type EndpointError = #error_ty;

                fn try_from_http_response<T: ::std::convert::AsRef<[::std::primitive::u8]>>(
                    #src: #http::Response<T>,
                ) -> ::std::result::Result<
                    Self,
                    #ruma_common::api::error::FromHttpResponseError<#error_ty>,
                > {
                    if #src.status().as_u16() >= 400 {
                        return ::std::result::Result::Err(
                            #ruma_common::api::error::FromHttpResponseError::Server(
                                <#error_ty as #ruma_common::api::EndpointError>::from_http_response(
                                    #src,
                                )
                            )
                        );
                    }

                    #headers_parse
                    #body_parse

                    ::std::result::Result::Ok(Self {
                        #headers_fields
                        #body_fields
                    })
                }
            }
        }
    }
}
