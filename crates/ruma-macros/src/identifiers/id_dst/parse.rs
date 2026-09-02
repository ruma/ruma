//! Implementations and types to parse the `IdDst` macro input.

use as_variant::as_variant;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{meta::ParseNestedMeta, parse_quote};

use super::{IdDst, OwnedId, Types};
use crate::util::RumaCommon;

/// The default size of the inline array for the `SmallVec` inner representation.
const SMALLVEC_INLINE_BYTES_DEFAULT: usize = 32;

impl IdDst {
    /// Parse the given `IdDst` macro input.
    pub(super) fn parse(input: syn::ItemStruct) -> syn::Result<Self> {
        let mut id_dst_attrs = IdDstAttrs::default();

        for attr in &input.attrs {
            if !attr.path().is_ident("ruma_id") {
                continue;
            }

            attr.parse_nested_meta(|meta| id_dst_attrs.try_merge(meta, attr))?;
        }

        let IdDstAttrs { validate, smallvec_inline_bytes } = id_dst_attrs;

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
        let types = Types::new(&ruma_common, &owned_id);

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

/// The parsed attributes of the [`IdDst`].
#[derive(Default)]
struct IdDstAttrs {
    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,

    /// The size of the inline array for the `SmallVec` inner representation.
    smallvec_inline_bytes: Option<usize>,
}

impl IdDstAttrs {
    /// Set the path to the function to use to validate the identifier.
    ///
    /// Returns an error if it is already set.
    fn set_validate(&mut self, validate: syn::Path, attr: &syn::Attribute) -> syn::Result<()> {
        if self.validate.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `validate` attribute",
            ));
        }

        self.validate = Some(validate);
        Ok(())
    }

    /// Set the size of the inline array for the `SmallVec` inner representation.
    ///
    /// Returns an error if it is already set or if the value doesn't fit into a `usize`.
    fn set_smallvec_inline_bytes(
        &mut self,
        inline_bytes: syn::LitInt,
        attr: &syn::Attribute,
    ) -> syn::Result<()> {
        if self.smallvec_inline_bytes.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `smallvec_inline_bytes` attribute",
            ));
        }

        self.smallvec_inline_bytes = Some(inline_bytes.base10_parse()?);
        Ok(())
    }

    /// Try to parse the given meta item and merge it into this `IdDstAttrs`.
    ///
    /// Returns an error if an unknown `ruma_id` attribute is encountered, or if an attribute
    /// that accepts a single value appears several times.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>, attr: &syn::Attribute) -> syn::Result<()> {
        if meta.path.is_ident("validate") {
            return self.set_validate(meta.value()?.parse()?, attr);
        }

        if meta.path.is_ident("smallvec_inline_bytes") {
            return self.set_smallvec_inline_bytes(meta.value()?.parse()?, attr);
        }

        Err(meta.error("unsupported `ruma_id` attribute"))
    }
}
