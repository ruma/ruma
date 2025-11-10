//! Code to convert the Rust-styled field/variant (e.g. `my_field`, `MyType`) to the
//! case of the source (e.g. `my-field`, `MY_FIELD`).
//!
//! This is a minimally modified version of the same code [in serde].
//!
//! [serde]: https://github.com/serde-rs/serde/blame/a9f8ea0a1e8ba1206f8c28d96b924606847b85a9/serde_derive/src/internals/case.rs

use syn::{LitStr, parse::Parse};

use self::RenameRule::*;

/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, Default, PartialEq)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    #[default]
    None,
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    Uppercase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
    /// Rename direct children to "M_MATRIX_ERROR_CASE" style, as used for responses with error in
    /// Matrix spec.
    MatrixErrorCase,
    /// Rename the direct children to "m.lowercase" style.
    MatrixLowerCase,
    /// Rename the direct children to "m.snake_case" style.
    MatrixSnakeCase,
    /// Rename the direct children to "m.rule.snake_case" style.
    MatrixRuleSnakeCase,
    /// Rename the direct children to "m.role.snake_case" style.
    MatrixRoleSnakeCase,
}

impl RenameRule {
    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply_to_variant(&self, variant: &str) -> String {
        match *self {
            None => variant.to_owned(),
            LowerCase => variant.to_ascii_lowercase(),
            Uppercase => variant.to_ascii_uppercase(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
            MatrixErrorCase => String::from("M_") + &ScreamingSnakeCase.apply_to_variant(variant),
            MatrixLowerCase => String::from("m.") + &LowerCase.apply_to_variant(variant),
            MatrixSnakeCase => String::from("m.") + &SnakeCase.apply_to_variant(variant),
            MatrixRuleSnakeCase => String::from(".m.rule.") + &SnakeCase.apply_to_variant(variant),
            MatrixRoleSnakeCase => String::from("m.role.") + &SnakeCase.apply_to_variant(variant),
        }
    }
}

impl Parse for RenameRule {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let str: LitStr = input.parse()?;

        match str.value().as_str() {
            "lowercase" => Ok(LowerCase),
            "UPPERCASE" => Ok(Uppercase),
            "camelCase" => Ok(CamelCase),
            "snake_case" => Ok(SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(ScreamingSnakeCase),
            "kebab-case" => Ok(KebabCase),
            "M_MATRIX_ERROR_CASE" => Ok(MatrixErrorCase),
            "m.snake_case" => Ok(MatrixSnakeCase),
            "m.lowercase" => Ok(MatrixLowerCase),
            ".m.rule.snake_case" => Ok(MatrixRuleSnakeCase),
            "m.role.snake_case" => Ok(MatrixRoleSnakeCase),
            _ => Err(syn::Error::new_spanned(
                str,
                "unsupported value for `#[ruma_enum(rename_all)]`'s rule",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RenameRule::*;

    #[test]
    fn rename_variants() {
        for &(
            original,
            lower,
            upper,
            camel,
            snake,
            screaming,
            kebab,
            matrix_error,
            m_lower,
            m_snake,
            m_rule_snake,
            m_role_snake,
        ) in &[
            (
                "Outcome",
                "outcome",
                "OUTCOME",
                "outcome",
                "outcome",
                "OUTCOME",
                "outcome",
                "M_OUTCOME",
                "m.outcome",
                "m.outcome",
                ".m.rule.outcome",
                "m.role.outcome",
            ),
            (
                "VeryTasty",
                "verytasty",
                "VERYTASTY",
                "veryTasty",
                "very_tasty",
                "VERY_TASTY",
                "very-tasty",
                "M_VERY_TASTY",
                "m.verytasty",
                "m.very_tasty",
                ".m.rule.very_tasty",
                "m.role.very_tasty",
            ),
            ("A", "a", "A", "a", "a", "A", "a", "M_A", "m.a", "m.a", ".m.rule.a", "m.role.a"),
            (
                "Z42",
                "z42",
                "Z42",
                "z42",
                "z42",
                "Z42",
                "z42",
                "M_Z42",
                "m.z42",
                "m.z42",
                ".m.rule.z42",
                "m.role.z42",
            ),
        ] {
            assert_eq!(None.apply_to_variant(original), original);
            assert_eq!(LowerCase.apply_to_variant(original), lower);
            assert_eq!(Uppercase.apply_to_variant(original), upper);
            assert_eq!(CamelCase.apply_to_variant(original), camel);
            assert_eq!(SnakeCase.apply_to_variant(original), snake);
            assert_eq!(ScreamingSnakeCase.apply_to_variant(original), screaming);
            assert_eq!(KebabCase.apply_to_variant(original), kebab);
            assert_eq!(MatrixErrorCase.apply_to_variant(original), matrix_error);
            assert_eq!(MatrixLowerCase.apply_to_variant(original), m_lower);
            assert_eq!(MatrixSnakeCase.apply_to_variant(original), m_snake);
            assert_eq!(MatrixRuleSnakeCase.apply_to_variant(original), m_rule_snake);
            assert_eq!(MatrixRoleSnakeCase.apply_to_variant(original), m_role_snake);
        }
    }
}
