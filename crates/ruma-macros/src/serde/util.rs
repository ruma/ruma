use proc_macro2::TokenStream;
use quote::quote;

mod parse;

/// Parsed `ruma_enum` attributes on a container.
#[derive(Default)]
pub(super) struct RumaEnumAttrs {
    /// The global renaming rule for the variants.
    pub(super) rename_all: RenameAll,
}

/// A parsed unit variant of an enum with `ruma_enum` attributes.
pub(super) struct UnitVariant {
    /// The name of the variant.
    pub(super) ident: syn::Ident,

    /// The custom string representation for the variant.
    rename: Option<syn::LitStr>,

    /// Alternative string representations for the variant.
    pub(super) aliases: Vec<syn::LitStr>,
}

impl UnitVariant {
    /// The string representation of this variant.
    pub(super) fn string_representation(&self, rename_all: &RenameAll) -> String {
        if let Some(rename) = &self.rename {
            rename.value()
        } else {
            rename_all.apply(&self.ident.to_string())
        }
    }
}

/// A parsed tuple or struct variant with a single field.
pub(super) struct VariantWithSingleField {
    /// The name of the variant.
    pub(super) ident: syn::Ident,

    /// The data field of the variant.
    pub(super) field: syn::Field,
}

impl VariantWithSingleField {
    /// Generate the code to extract or set the inner value of this variant into or from a variable
    /// called `inner`.
    ///
    /// The generated code looks like:
    ///
    /// ```ignore
    /// Ident(inner)
    /// ```
    ///
    /// or
    ///
    /// ```ignore
    /// Ident { field: inner }
    /// ```
    pub(super) fn expand_variable(&self) -> TokenStream {
        let ident = &self.ident;

        match &self.field.ident {
            Some(field) => quote! { #ident { #field: inner } },
            None => quote! { #ident (inner) },
        }
    }
}

/// The transformations to apply to all unit variants without a custom string representation in an
/// enum.
#[derive(Default)]
pub(super) struct RenameAll {
    /// The prefix to add to the variants.
    prefix: Option<syn::LitStr>,

    /// The transformation to apply to the variants.
    rule: RenameRule,
}

impl RenameAll {
    /// Apply this transformation to the given enum variant.
    pub(super) fn apply(&self, variant: &str) -> String {
        let mut renamed = self.rule.apply(variant);

        if let Some(prefix) = &self.prefix {
            renamed = format!("{}{renamed}", prefix.value());
        }

        renamed
    }
}

/// The different ways to change the string representation of unit variants in an enum.
#[derive(Copy, Clone, Default, PartialEq)]
pub(super) enum RenameRule {
    /// Don't apply a default rename rule.
    #[default]
    None,

    /// Convert to "lowercase" style.
    LowerCase,

    /// Convert to "UPPERCASE" style.
    Uppercase,

    /// Convert to "camelCase" style.
    CamelCase,

    /// Convert to "snake_case" style, as commonly used for fields.
    SnakeCase,

    /// Convert to "SCREAMING_SNAKE_CASE" style, as commonly used for constants.
    ScreamingSnakeCase,

    /// Convert to "kebab-case" style.
    KebabCase,
}

impl RenameRule {
    /// Split the given variant name at the uppercase letters by adding the given separator.
    ///
    /// The uppercase letters are transformed to lowercase.
    fn split_variant_name(variant: &str, separator: char) -> String {
        let mut s = String::with_capacity(variant.len());

        for (i, ch) in variant.char_indices() {
            if i > 0 && ch.is_uppercase() {
                s.push(separator);
            }

            s.push(ch.to_ascii_lowercase());
        }

        s
    }

    /// Apply this rule to the given variant.
    pub(super) fn apply(&self, variant: &str) -> String {
        match *self {
            Self::None => variant.to_owned(),
            Self::LowerCase => variant.to_ascii_lowercase(),
            Self::Uppercase => variant.to_ascii_uppercase(),
            Self::CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            Self::SnakeCase => Self::split_variant_name(variant, '_'),
            Self::ScreamingSnakeCase => Self::SnakeCase.apply(variant).to_ascii_uppercase(),
            Self::KebabCase => Self::split_variant_name(variant, '-'),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{RenameAll, RenameRule};

    #[test]
    fn rename_all_apply() {
        let variant = "VeryTasty";

        // Default, no change.
        let rename_all = RenameAll::default();
        assert_eq!(rename_all.apply(variant), variant);

        // Only rule.
        let rename_all = RenameAll { rule: RenameRule::ScreamingSnakeCase, ..Default::default() };
        assert_eq!(rename_all.apply(variant), "VERY_TASTY");

        let rename_all = RenameAll { rule: RenameRule::SnakeCase, ..Default::default() };
        assert_eq!(rename_all.apply(variant), "very_tasty");

        // Only prefix.
        let rename_all = RenameAll { prefix: Some(parse_quote!("m.rule.")), ..Default::default() };
        assert_eq!(rename_all.apply(variant), "m.rule.VeryTasty");

        let rename_all = RenameAll { prefix: Some(parse_quote!("M_")), ..Default::default() };
        assert_eq!(rename_all.apply(variant), "M_VeryTasty");

        // Rule and prefix.
        let rename_all = RenameAll {
            prefix: Some(parse_quote!("m.rule.")),
            rule: RenameRule::ScreamingSnakeCase,
        };
        assert_eq!(rename_all.apply(variant), "m.rule.VERY_TASTY");

        let rename_all =
            RenameAll { prefix: Some(parse_quote!("m.rule.")), rule: RenameRule::SnakeCase };
        assert_eq!(rename_all.apply(variant), "m.rule.very_tasty");

        let rename_all =
            RenameAll { prefix: Some(parse_quote!("M_")), rule: RenameRule::ScreamingSnakeCase };
        assert_eq!(rename_all.apply(variant), "M_VERY_TASTY");

        let rename_all =
            RenameAll { prefix: Some(parse_quote!("M_")), rule: RenameRule::SnakeCase };
        assert_eq!(rename_all.apply(variant), "M_very_tasty");
    }

    #[test]
    fn rename_rule_apply() {
        for &(original, lower, upper, camel, snake, screaming, kebab) in &[
            ("Outcome", "outcome", "OUTCOME", "outcome", "outcome", "OUTCOME", "outcome"),
            (
                "VeryTasty",
                "verytasty",
                "VERYTASTY",
                "veryTasty",
                "very_tasty",
                "VERY_TASTY",
                "very-tasty",
            ),
            ("A", "a", "A", "a", "a", "A", "a"),
            ("Z42", "z42", "Z42", "z42", "z42", "Z42", "z42"),
        ] {
            assert_eq!(RenameRule::None.apply(original), original);
            assert_eq!(RenameRule::LowerCase.apply(original), lower);
            assert_eq!(RenameRule::Uppercase.apply(original), upper);
            assert_eq!(RenameRule::CamelCase.apply(original), camel);
            assert_eq!(RenameRule::SnakeCase.apply(original), snake);
            assert_eq!(RenameRule::ScreamingSnakeCase.apply(original), screaming);
            assert_eq!(RenameRule::KebabCase.apply(original), kebab);
        }
    }
}
