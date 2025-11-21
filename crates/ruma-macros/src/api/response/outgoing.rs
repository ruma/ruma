use proc_macro2::TokenStream;
use quote::quote;

use super::{KIND, Response};

impl Response {
    /// Generate the `ruma_common::api::OutgoingResponse` implementation for this response struct.
    pub fn expand_outgoing(&self, ruma_common: &TokenStream) -> TokenStream {
        let bytes = quote! { #ruma_common::exports::bytes };
        let http = quote! { #ruma_common::exports::http };

        let headers_serialize = self.headers.expand_serialize(KIND, &self.body, ruma_common, &http);
        let headers_fields = self.headers.expand_fields();

        let body_serialize = self.body.expand_serialize(KIND, ruma_common);
        let body_fields = self.body.expand_fields();

        let ident = &self.ident;
        let status = &self.status;
        let src = KIND.as_variable_ident();

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            #[allow(deprecated)]
            impl #ruma_common::api::OutgoingResponse for #ident {
                fn try_into_http_response<T: ::std::default::Default + #bytes::BufMut>(
                    self,
                ) -> ::std::result::Result<#http::Response<T>, #ruma_common::api::error::IntoHttpError> {
                    let Self {
                        #headers_fields
                        #body_fields
                    } = self;

                    let mut #src = #http::Response::builder()
                        .status(#http::StatusCode::#status)
                        .body(#body_serialize)?;

                    #headers_serialize

                    ::std::result::Result::Ok(#src)
                }
            }
        }
    }
}
