use proc_macro2::Span;
use syn::{meta::ParseNestedMeta, parse::Parse};

use super::{RenameAll, RenameRule, RumaEnumAttrs, UnitVariant, VariantWithSingleField};

impl RumaEnumAttrs {
    /// Try to parse the given meta item and merge it into this `RumaEnumAttrs`.
    ///
    /// Returns an error if parsing the meta item fails, or if it sets a field that was already set.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if meta.path.is_ident("rename_all") {
            return self.rename_all.try_merge(meta);
        }

        Err(meta.error("unsupported `ruma_enum` attribute"))
    }
}

impl RumaEnumAttrs {
    /// Try to parse the `ruma_enum` attributes from the list.
    pub(crate) fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut enum_attrs = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("ruma_enum") {
                continue;
            }

            attr.parse_nested_meta(|meta| enum_attrs.try_merge(meta))?;
        }

        Ok(enum_attrs)
    }
}

impl RenameAll {
    /// Set the prefix of this attribute.
    ///
    /// Returns an error if the prefix was already set.
    fn set_prefix(&mut self, prefix: syn::LitStr) -> syn::Result<()> {
        if self.prefix.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "cannot have multiple values for `#[ruma_enum(rename_all)]`'s prefix",
            ));
        }

        self.prefix = Some(prefix);
        Ok(())
    }

    /// Set the rule of this attribute.
    ///
    /// Returns an error if the rule was already set.
    fn set_rule(&mut self, rule: RenameRule) -> syn::Result<()> {
        if self.rule != RenameRule::None {
            return Err(syn::Error::new(
                Span::call_site(),
                "cannot have multiple values for `#[ruma_enum(rename_all)]`'s rule",
            ));
        }

        self.rule = rule;
        Ok(())
    }

    /// Try to merge the values of the attributes in the given meta in this one.
    ///
    /// Returns an error if parsing the meta fails or a value is set twice.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> syn::Result<()> {
        if meta.input.peek(syn::Token![=]) {
            self.set_rule(meta.value()?.parse()?)?;

            return Ok(());
        }

        if meta.input.peek(syn::token::Paren) {
            meta.parse_nested_meta(|nested_meta| {
                if nested_meta.path.is_ident("prefix") {
                    self.set_prefix(nested_meta.value()?.parse()?)?;
                    return Ok(());
                }

                if nested_meta.path.is_ident("rule") {
                    self.set_rule(nested_meta.value()?.parse()?)?;
                    return Ok(());
                }

                Err(nested_meta.error(
                    "unsupported `rename_all` attribute, expected one of `prefix` or `rule`",
                ))
            })?;

            return Ok(());
        }

        Err(meta.error(
            "unexpected syntax for `rename_all` attribute, \
             expected `rename_all = \"rule\"` or \
             `rename_all(prefix = \"m.\", rule = \"rule\")`",
        ))
    }
}

impl Parse for RenameRule {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let str: syn::LitStr = input.parse()?;

        match str.value().as_str() {
            "lowercase" => Ok(Self::LowerCase),
            "UPPERCASE" => Ok(Self::Uppercase),
            "camelCase" => Ok(Self::CamelCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            "kebab-case" => Ok(Self::KebabCase),
            _ => Err(syn::Error::new_spanned(
                str,
                "unsupported value for `#[ruma_enum(rename_all)]`'s rule",
            )),
        }
    }
}

/// The parsed attributes of a [`RumaEnum`] unit variant.
#[derive(Default)]
struct UnitVariantAttrs {
    /// The custom string representation for the variant.
    rename: Option<syn::LitStr>,

    /// Alternative string representations for the variant.
    aliases: Vec<syn::LitStr>,
}

impl UnitVariantAttrs {
    /// Set custom string representation for the variant.
    ///
    /// Returns an error if the custom string representation was already set.
    fn set_rename(&mut self, rename: syn::LitStr, variant: &syn::Variant) -> syn::Result<()> {
        if self.rename.is_some() {
            return Err(syn::Error::new_spanned(
                variant,
                "cannot have multiple `#[ruma_enum(rename)]` attributes",
            ));
        }

        self.rename = Some(rename);
        Ok(())
    }

    /// Push the given alternative string representations for the variant.
    fn push_alias(&mut self, alias: syn::LitStr) {
        self.aliases.push(alias);
    }

    /// Try to merge the values of the attributes in the given meta in this one.
    ///
    /// Returns an error if parsing the meta fails or a value is set twice.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>, variant: &syn::Variant) -> syn::Result<()> {
        if meta.path.is_ident("rename") {
            return self.set_rename(meta.value()?.parse()?, variant);
        }

        if meta.path.is_ident("alias") {
            self.push_alias(meta.value()?.parse()?);
            return Ok(());
        }

        Err(meta.error("unsupported `ruma_enum` attribute"))
    }
}

impl TryFrom<&syn::Variant> for UnitVariant {
    type Error = syn::Error;

    fn try_from(variant: &syn::Variant) -> Result<Self, Self::Error> {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "UnitVariant can only be parsed from unit enum variants",
            ));
        }

        let mut variant_attrs = UnitVariantAttrs::default();

        for attr in &variant.attrs {
            if !attr.path().is_ident("ruma_enum") {
                continue;
            }

            attr.parse_nested_meta(|meta| variant_attrs.try_merge(meta, variant))?;
        }

        Ok(Self {
            ident: variant.ident.clone(),
            rename: variant_attrs.rename,
            aliases: variant_attrs.aliases,
        })
    }
}

impl TryFrom<&syn::Variant> for VariantWithSingleField {
    type Error = syn::Error;

    fn try_from(variant: &syn::Variant) -> Result<Self, Self::Error> {
        if variant.attrs.iter().any(|attr| attr.path().is_ident("ruma_enum")) {
            return Err(syn::Error::new_spanned(
                variant,
                "struct or tuple variant doesn't support any `#[ruma_enum]` attribute",
            ));
        }

        let fields = match &variant.fields {
            syn::Fields::Named(syn::FieldsNamed { named: fields, .. })
            | syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed: fields, .. }) => fields,
            syn::Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "VariantWithSingleField can only be parsed from tuple or struct enum variants",
                ));
            }
        };

        match fields.len() {
            0 => {
                return Err(syn::Error::new_spanned(
                    fields,
                    "struct or tuple variant must have one field",
                ));
            }
            1 => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    fields,
                    "struct or tuple variant cannot have multiple fields",
                ));
            }
        }

        Ok(Self {
            ident: variant.ident.clone(),
            field: fields
                .iter()
                .next()
                .cloned()
                .expect("struct or tuple variant should have at least one field"),
        })
    }
}
