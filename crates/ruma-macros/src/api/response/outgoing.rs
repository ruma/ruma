use proc_macro2::TokenStream;
use quote::quote;

use super::{KIND, Response};
use crate::util::{RumaCommon, RumaCommonReexport};

impl Response {
    /// Generate the `ruma_common::api::OutgoingResponse` implementation for this response struct.
    pub fn expand_outgoing(&self, ruma_common: &RumaCommon) -> TokenStream {
        let http = ruma_common.reexported(RumaCommonReexport::Http);

        let headers_serialize = self.headers.expand_serialize(KIND, &http);
        let headers_fields = self.headers.expand_fields();

        let body_type = self.body.type_name(KIND, ruma_common, &self.ident);
        let body_expr = self.body.body_expr(KIND, ruma_common);
        let body_fields = self.body.expand_fields();

        let ident = &self.ident;
        let status = &self.status;
        let src = KIND.as_variable_ident();

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            #[allow(deprecated)]
            impl #ruma_common::api::OutgoingResponse for #ident {
                type Body = #body_type;

                fn try_into_http_response_inner(
                    self,
                ) -> ::std::result::Result<#http::Response<Self::Body>, #ruma_common::api::error::IntoHttpError> {
                    let Self {
                        #headers_fields
                        #body_fields
                    } = self;

                    let mut #src = #http::Response::builder()
                        .status(#http::StatusCode::#status)
                        .body(#body_expr)?;

                    #headers_serialize

                    ::std::result::Result::Ok(#src)
                }
            }
        }
    }
}
