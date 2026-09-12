//! Implementations and types to parse the `IdDst` macro input.

use as_variant::as_variant;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse_quote;

use super::{IdDst, OwnedId};
use crate::{
    identifiers::{RumaIdAttrs, SMALLVEC_INLINE_BYTES_DEFAULT, Types},
    util::RumaCommon,
};

impl IdDst {
    /// Parse the given `IdDst` macro input.
    pub(super) fn parse(input: syn::ItemStruct) -> syn::Result<Self> {
        let mut id_dst_attrs = RumaIdAttrs::default();

        for attr in &input.attrs {
            if !attr.path().is_ident("ruma_id") {
                continue;
            }

            attr.parse_nested_meta(|meta| id_dst_attrs.try_merge(meta))?;
        }

        let RumaIdAttrs { validate, smallvec_inline_bytes } = id_dst_attrs;

        if validate.is_none() && !input.generics.params.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "IDs without validation and with generics are not supported",
            ));
        }

        if input.generics.where_clause.is_some() {
            // So we don't have to insert #where_clause everywhere when it is always None in
            // practice.
            return Err(syn::Error::new(
                Span::call_site(),
                "where clauses on IDs are not supported",
            ));
        }

        let str_field_index = as_variant!(
            &input.fields,
            syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => unnamed
        )
        .and_then(|unnamed| unnamed.len().checked_sub(1))
        .ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "Only tuple structs with a `str` as the last field are supported",
            )
        })?
        .into();

        let smallvec_inline_bytes = smallvec_inline_bytes.unwrap_or(SMALLVEC_INLINE_BYTES_DEFAULT);

        let generics = input.generics;
        let (impl_generics, type_generics, _where_clause) = generics.split_for_impl();
        let impl_generics = quote! { #impl_generics };

        let ident = input.ident;
        let id_type = parse_quote! { #ident #type_generics };
        let owned_ident = format_ident!("Owned{ident}");
        let owned_id_type = parse_quote! { #owned_ident #type_generics };

        let owned_id = OwnedId::new(owned_ident, owned_id_type, smallvec_inline_bytes);
        let ruma_common = RumaCommon::new();
        let types = Types::new(&ruma_common, smallvec_inline_bytes);

        Ok(Self {
            ident,
            id_type,
            generics,
            impl_generics,
            validate,
            str_field_index,
            owned_id,
            types,
            ruma_common,
        })
    }
}
