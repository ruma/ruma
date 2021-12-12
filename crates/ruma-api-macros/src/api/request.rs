//! Details of the `request` section of the procedural macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, spanned::Spanned, Attribute, Field, Ident, Token};

use super::{kw, metadata::Metadata};
use crate::util::{all_cfgs_expr, all_lifetimes};

/// The result of processing the `request` section of the macro.
pub(crate) struct Request {
    /// The `request` keyword
    pub(super) request_kw: kw::request,

    /// The attributes that will be applied to the struct definition.
    pub(super) attributes: Vec<Attribute>,

    /// The fields of the request.
    pub(super) fields: Punctuated<Field, Token![,]>,
}

impl Request {
    pub(super) fn expand(
        &self,
        metadata: &Metadata,
        error_ty: &TokenStream,
        ruma_api: &TokenStream,
    ) -> TokenStream {
        let ruma_api_macros = quote! { #ruma_api::exports::ruma_api_macros };
        let ruma_serde = quote! { #ruma_api::exports::ruma_serde };

        let docs = format!(
            "Data for a request to the `{}` API endpoint.\n\n{}",
            metadata.name.value(),
            metadata.description.value(),
        );
        let struct_attributes = &self.attributes;

        let method = &metadata.method;
        let path = &metadata.path;
        let auth_attributes = metadata.authentication.iter().map(|field| {
            let cfg_expr = all_cfgs_expr(&field.attrs);
            let value = &field.value;

            match cfg_expr {
                Some(expr) => quote! { #[cfg_attr(#expr, ruma_api(authentication = #value))] },
                None => quote! { #[ruma_api(authentication = #value)] },
            }
        });

        let request_ident = Ident::new("Request", self.request_kw.span());
        let lifetimes =
            all_lifetimes(&self.fields).into_iter().map(|(lt, attr)| quote! { #attr #lt });
        let fields = &self.fields;

        quote! {
            #[doc = #docs]
            #[derive(
                Clone,
                Debug,
                #ruma_api_macros::Request,
                #ruma_serde::Outgoing,
                #ruma_serde::_FakeDeriveSerde,
            )]
            #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
            #[incoming_derive(!Deserialize, #ruma_api_macros::_FakeDeriveRumaApi)]
            #[ruma_api(
                method = #method,
                path = #path,
                error_ty = #error_ty,
            )]
            #( #auth_attributes )*
            #( #struct_attributes )*
            pub struct #request_ident < #(#lifetimes),* > {
                #fields
            }
        }
    }
}
