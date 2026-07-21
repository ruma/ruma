use proc_macro2::TokenStream;
use quote::quote;

use super::{KIND, Response};
use crate::util::{RumaCommon, RumaCommonReexport};

impl Response {
    /// Generate the `ruma_common::api::IncomingResponse` implementation for this response struct.
    pub fn expand_incoming(&self, ruma_common: &RumaCommon) -> TokenStream {
        let http = ruma_common.reexported(RumaCommonReexport::Http);

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

                fn try_from_http_response_inner(
                    #src: #http::Response<&[::std::primitive::u8]>,
                ) -> ::std::result::Result<Self, #ruma_common::api::error::DeserializationError> {
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
