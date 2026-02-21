//! Implementations and types to parse the `IdDst` macro input.

use as_variant::as_variant;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::meta::ParseNestedMeta;

use super::{IdDst, StorageCfg, Types};
use crate::util::RumaCommon;

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

        let IdDstAttrs { validate } = id_dst_attrs;

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

        let generics = input.generics;
        let (impl_generics, type_generics, _where_clause) = generics.split_for_impl();
        let impl_generics = quote! { #impl_generics };

        let ident = input.ident;
        let owned_ident = format_ident!("Owned{ident}");
        let types = Types::new(&ident, &owned_ident, type_generics);

        Ok(Self {
            ident,
            owned_ident,
            generics,
            impl_generics,
            validate,
            str_field_index,
            types,
            storage_cfg: StorageCfg::new(),
            ruma_common: RumaCommon::new(),
        })
    }
}

/// The parsed attributes of the [`IdDst`].
#[derive(Default)]
struct IdDstAttrs {
    /// The path to the function to use to validate the identifier.
    validate: Option<syn::Path>,
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

    /// Try to parse the given meta item and merge it into this `IdDstAttrs`.
    ///
    /// Returns an error if an unknown `ruma_id` attribute is encountered, or if an attribute
    /// that accepts a single value appears several times.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>, attr: &syn::Attribute) -> syn::Result<()> {
        if meta.path.is_ident("validate") {
            return self.set_validate(meta.value()?.parse()?, attr);
        }

        Err(meta.error("unsupported `ruma_id` attribute"))
    }
}
