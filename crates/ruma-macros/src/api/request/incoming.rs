use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

use super::{KIND, Request, RequestPath, RequestQuery};
use crate::{api::StructSuffix, util::expand_fields_as_variable_declarations};

impl Request {
    /// Generate the `ruma_common::api::IncomingRequest` implementation for this request struct.
    pub fn expand_incoming(&self, ruma_common: &TokenStream) -> TokenStream {
        let http = quote! { #ruma_common::exports::http };

        let path_parse = self.path.expand_parse(ruma_common);
        let path_fields = self.path.expand_fields();

        let query_parse = self.query.expand_parse(ruma_common);
        let query_fields = self.query.expand_fields();

        let headers_parse = self.headers.expand_parse(KIND, ruma_common);
        let headers_fields = self.headers.expand_fields();

        let body_parse = self.body.expand_parse(KIND, ruma_common);
        let body_fields = self.body.expand_fields();

        let request = KIND.as_variable_ident();
        let error_ty = &self.error_ty;
        let ident = &self.ident;

        quote! {
            #[automatically_derived]
            #[cfg(feature = "server")]
            impl #ruma_common::api::IncomingRequest for #ident {
                type EndpointError = #error_ty;
                type OutgoingResponse = Response;

                fn try_from_http_request<B, S>(
                    #request: #http::Request<B>,
                    path_args: &[S],
                ) -> ::std::result::Result<Self, #ruma_common::api::error::FromHttpRequestError>
                where
                    B: ::std::convert::AsRef<[::std::primitive::u8]>,
                    S: ::std::convert::AsRef<::std::primitive::str>,
                {
                    <Self as #ruma_common::api::IncomingRequest>::check_request_method(#request.method())?;

                    #path_parse
                    #query_parse
                    #headers_parse
                    #body_parse

                    ::std::result::Result::Ok(Self {
                        #path_fields
                        #query_fields
                        #headers_fields
                        #body_fields
                    })
                }
            }
        }
    }
}

impl RequestPath {
    /// Generate code to parse the path arguments of a `&[dyn AsRef<[u8]>]` named `path_args`.
    fn expand_parse(&self, ruma_common: &TokenStream) -> Option<TokenStream> {
        if self.0.is_empty() {
            return None;
        }

        let serde = quote! { #ruma_common::exports::serde };
        let fields = self.expand_fields();

        Some(quote! {
            let (#fields) = #serde::Deserialize::deserialize(
                #serde::de::value::SeqDeserializer::<_, #serde::de::value::Error>::new(
                    path_args.iter().map(::std::convert::AsRef::as_ref)
                )
            )?;
        })
    }
}

impl RequestQuery {
    /// Generate code to parse the query from an `http::request::Request`.
    fn expand_parse(&self, ruma_common: &TokenStream) -> Option<TokenStream> {
        let fields = match self {
            Self::None => return None,
            Self::Fields(fields) => fields.as_slice(),
            Self::All(field) => std::slice::from_ref(field),
        };

        let serde_html_form = quote! { #ruma_common::exports::serde_html_form };
        let src = parse_quote! { request_query };
        let request = KIND.as_variable_ident();
        let serde_struct = KIND.as_struct_ident(StructSuffix::Query);

        let decls = expand_fields_as_variable_declarations(fields, &src);

        Some(quote! {
            let #src: #serde_struct =
                #serde_html_form::from_str(#request.uri().query().unwrap_or(""))?;

            #decls
        })
    }
}
