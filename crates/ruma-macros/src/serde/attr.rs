use proc_macro2::Span;
use syn::{
    LitStr, Token,
    meta::ParseNestedMeta,
    parse::{Parse, ParseStream},
    token,
};

use super::case::RenameRule;

mod kw {
    syn::custom_keyword!(alias);
    syn::custom_keyword!(rename);
    syn::custom_keyword!(rename_all);
}

#[derive(Default)]
pub struct EnumAttrs {
    pub rename: Option<LitStr>,
    pub aliases: Vec<LitStr>,
}

pub enum Attr {
    Alias(LitStr),
    Rename(LitStr),
}

impl Parse for Attr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::alias) {
            let _: kw::alias = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self::Alias(input.parse()?))
        } else if lookahead.peek(kw::rename) {
            let _: kw::rename = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self::Rename(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

/// The attribute for the transformations to apply to all variants in an enum.
#[derive(Default)]
pub(super) struct RenameAllAttr {
    /// The prefix to add to the variants.
    prefix: Option<LitStr>,

    /// The case transformation to apply to the variants.
    rule: RenameRule,
}

impl RenameAllAttr {
    /// Set the prefix of this attribute.
    ///
    /// Returns an error if the prefix was already set.
    fn set_prefix(&mut self, prefix: LitStr) -> Result<(), syn::Error> {
        if self.prefix.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                "found multiple values for `#[ruma_enum(rename_all)]`'s prefix",
            ));
        }

        self.prefix = Some(prefix);
        Ok(())
    }

    /// Set the rule of this attribute.
    ///
    /// Returns an error if the rule was already set.
    fn set_rule(&mut self, rule: RenameRule) -> Result<(), syn::Error> {
        if self.rule != RenameRule::None {
            return Err(syn::Error::new(
                Span::call_site(),
                "found multiple values for `#[ruma_enum(rename_all)]`'s rule",
            ));
        }

        self.rule = rule;
        Ok(())
    }

    /// Try to merge the values of the attributes in the given meta in this one.
    ///
    /// Returns an error if parsing the meta fails or a value is set twice.
    pub(super) fn try_merge(&mut self, meta: ParseNestedMeta<'_>) -> Result<(), syn::Error> {
        if meta.input.peek(Token![=]) {
            self.set_rule(meta.value()?.parse()?)?;

            return Ok(());
        }

        if meta.input.peek(token::Paren) {
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

    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub(super) fn apply_to_variant(&self, variant: &str) -> String {
        let mut renamed = self.rule.apply_to_variant(variant);

        if let Some(prefix) = &self.prefix {
            renamed = format!("{}{renamed}", prefix.value());
        }

        renamed
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::RenameAllAttr;
    use crate::serde::case::RenameRule;

    #[test]
    fn rename_all_attr_apply_to_variant() {
        let variant = "VeryTasty";

        // Default, no change.
        let rename_all = RenameAllAttr::default();
        assert_eq!(rename_all.apply_to_variant(variant), variant);

        // Only rule.
        let rename_all =
            RenameAllAttr { rule: RenameRule::ScreamingSnakeCase, ..Default::default() };
        assert_eq!(rename_all.apply_to_variant(variant), "VERY_TASTY");

        let rename_all = RenameAllAttr { rule: RenameRule::SnakeCase, ..Default::default() };
        assert_eq!(rename_all.apply_to_variant(variant), "very_tasty");

        // Only prefix.
        let rename_all =
            RenameAllAttr { prefix: Some(parse_quote!("m.rule.")), ..Default::default() };
        assert_eq!(rename_all.apply_to_variant(variant), "m.rule.VeryTasty");

        let rename_all = RenameAllAttr { prefix: Some(parse_quote!("M_")), ..Default::default() };
        assert_eq!(rename_all.apply_to_variant(variant), "M_VeryTasty");

        // Rule and prefix.
        let rename_all = RenameAllAttr {
            prefix: Some(parse_quote!("m.rule.")),
            rule: RenameRule::ScreamingSnakeCase,
        };
        assert_eq!(rename_all.apply_to_variant(variant), "m.rule.VERY_TASTY");

        let rename_all =
            RenameAllAttr { prefix: Some(parse_quote!("m.rule.")), rule: RenameRule::SnakeCase };
        assert_eq!(rename_all.apply_to_variant(variant), "m.rule.very_tasty");

        let rename_all = RenameAllAttr {
            prefix: Some(parse_quote!("M_")),
            rule: RenameRule::ScreamingSnakeCase,
        };
        assert_eq!(rename_all.apply_to_variant(variant), "M_VERY_TASTY");

        let rename_all =
            RenameAllAttr { prefix: Some(parse_quote!("M_")), rule: RenameRule::SnakeCase };
        assert_eq!(rename_all.apply_to_variant(variant), "M_very_tasty");
    }
}
