use proc_macro2::TokenStream;
use quote::quote;

use super::{KIND, Request, RequestQuery};
use crate::{
    api::StructSuffix,
    util::{RumaCommon, RumaCommonReexport, StructFieldExt},
};

impl Request {
    /// Generate the `ruma_common::api::OutgoingRequest` implementation for this request struct.
    pub fn expand_outgoing(&self, ruma_common: &RumaCommon) -> TokenStream {
        let bytes = ruma_common.reexported(RumaCommonReexport::Bytes);
        let http = ruma_common.reexported(RumaCommonReexport::Http);

        let path_fields = self.path.expand_fields();
        let path_idents = self.path.0.iter().map(|field| field.ident());

        let query_serialize = self.query.expand_serialize(ruma_common);
        let query_fields = self.query.expand_fields();

        let headers_serialize = self.headers.expand_serialize(KIND, &self.body, ruma_common, &http);
        let headers_fields = self.headers.expand_fields();

        let body_serialize = self.body.expand_serialize(KIND, ruma_common);
        let body_fields = self.body.expand_fields();

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let ident = &self.ident;
        let error_ty = &self.error_ty;
        let request = KIND.as_variable_ident();

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

                    let mut #request = #http::Request::builder()
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
                        &mut #request,
                        authentication_input
                    )
                        .map_err(|error| #ruma_common::api::error::IntoHttpError::Authentication(error.into()))?;

                    Ok(#request)
                }
            }
        }
    }
}

impl RequestQuery {
    /// Generate code to serialize the query string.
    fn expand_serialize(&self, ruma_common: &RumaCommon) -> TokenStream {
        if matches!(self, Self::None) {
            return quote! { "" };
        }

        let serde_html_form = ruma_common.reexported(RumaCommonReexport::SerdeHtmlForm);
        let fields = self.expand_fields();
        let serde_struct = KIND.as_struct_ident(StructSuffix::Query);

        quote! {{
            let request_query = #serde_struct {
                #fields
            };

            &#serde_html_form::to_string(request_query)?
        }}
    }
}
