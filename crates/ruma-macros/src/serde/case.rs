//! Code to convert the Rust-styled field/variant (e.g. `my_field`, `MyType`) to the
//! case of the source (e.g. `my-field`, `MY_FIELD`).
//!
//! This is a minimally modified version of the same code [in serde].
//!
//! [serde]: https://github.com/serde-rs/serde/blame/a9f8ea0a1e8ba1206f8c28d96b924606847b85a9/serde_derive/src/internals/case.rs

use std::str::FromStr;

use self::RenameRule::*;

/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, PartialEq)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    None,
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    Uppercase,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
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
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebabCase,
    /// Rename direct children to "M_MATRIX_ERROR_CASE" style, as used for responses with error in
    /// Matrix spec.
    MatrixErrorCase,
    /// Rename the direct children to "m.lowercase" style.
    MatrixLowerCase,
    /// Rename the direct children to "m.snake_case" style.
    MatrixSnakeCase,
    /// Rename the direct children to "m.dotted.case" style.
    MatrixDottedCase,
    /// Rename the direct children to "m.rule.snake_case" style.
    MatrixRuleSnakeCase,
    /// Rename the direct children to "m.role.snake_case" style.
    MatrixRoleSnakeCase,
}

impl RenameRule {
    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply_to_variant(&self, variant: &str) -> String {
        match *self {
            None | PascalCase => variant.to_owned(),
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
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_variant(variant).replace('_', "-"),
            MatrixErrorCase => String::from("M_") + &ScreamingSnakeCase.apply_to_variant(variant),
            MatrixLowerCase => String::from("m.") + &LowerCase.apply_to_variant(variant),
            MatrixSnakeCase => String::from("m.") + &SnakeCase.apply_to_variant(variant),
            MatrixDottedCase => {
                String::from("m.") + &SnakeCase.apply_to_variant(variant).replace('_', ".")
            }
            MatrixRuleSnakeCase => String::from(".m.rule.") + &SnakeCase.apply_to_variant(variant),
            MatrixRoleSnakeCase => String::from("m.role.") + &SnakeCase.apply_to_variant(variant),
        }
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    #[allow(dead_code)]
    pub fn apply_to_field(&self, field: &str) -> String {
        match *self {
            None | LowerCase | SnakeCase => field.to_owned(),
            Uppercase => field.to_ascii_uppercase(),
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
            KebabCase => field.replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
            MatrixErrorCase => String::from("M_") + &ScreamingSnakeCase.apply_to_field(field),
            MatrixLowerCase => String::from("m.") + field,
            MatrixSnakeCase => String::from("m.") + field,
            MatrixDottedCase => String::from("m.") + &field.replace('_', "."),
            MatrixRuleSnakeCase => String::from(".m.rule.") + field,
            MatrixRoleSnakeCase => String::from("m.role.") + field,
        }
    }
}

impl FromStr for RenameRule {
    type Err = ();

    fn from_str(rename_all_str: &str) -> Result<Self, Self::Err> {
        match rename_all_str {
            "lowercase" => Ok(LowerCase),
            "UPPERCASE" => Ok(Uppercase),
            "PascalCase" => Ok(PascalCase),
            "camelCase" => Ok(CamelCase),
            "snake_case" => Ok(SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(ScreamingSnakeCase),
            "kebab-case" => Ok(KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(ScreamingKebabCase),
            "M_MATRIX_ERROR_CASE" => Ok(MatrixErrorCase),
            "m.snake_case" => Ok(MatrixSnakeCase),
            "m.lowercase" => Ok(MatrixLowerCase),
            "m.dotted.case" => Ok(MatrixDottedCase),
            ".m.rule.snake_case" => Ok(MatrixRuleSnakeCase),
            "m.role.snake_case" => Ok(MatrixRoleSnakeCase),
            _ => Err(()),
        }
    }
}

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
        screaming_kebab,
        matrix_error,
        m_lower,
        m_snake,
        m_dotted,
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
            "OUTCOME",
            "M_OUTCOME",
            "m.outcome",
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
            "VERY-TASTY",
            "M_VERY_TASTY",
            "m.verytasty",
            "m.very_tasty",
            "m.very.tasty",
            ".m.rule.very_tasty",
            "m.role.very_tasty",
        ),
        (
            "A",
            "a",
            "A",
            "a",
            "a",
            "A",
            "a",
            "A",
            "M_A",
            "m.a",
            "m.a",
            "m.a",
            ".m.rule.a",
            "m.role.a",
        ),
        (
            "Z42",
            "z42",
            "Z42",
            "z42",
            "z42",
            "Z42",
            "z42",
            "Z42",
            "M_Z42",
            "m.z42",
            "m.z42",
            "m.z42",
            ".m.rule.z42",
            "m.role.z42",
        ),
    ] {
        assert_eq!(None.apply_to_variant(original), original);
        assert_eq!(LowerCase.apply_to_variant(original), lower);
        assert_eq!(Uppercase.apply_to_variant(original), upper);
        assert_eq!(PascalCase.apply_to_variant(original), original);
        assert_eq!(CamelCase.apply_to_variant(original), camel);
        assert_eq!(SnakeCase.apply_to_variant(original), snake);
        assert_eq!(ScreamingSnakeCase.apply_to_variant(original), screaming);
        assert_eq!(KebabCase.apply_to_variant(original), kebab);
        assert_eq!(ScreamingKebabCase.apply_to_variant(original), screaming_kebab);
        assert_eq!(MatrixErrorCase.apply_to_variant(original), matrix_error);
        assert_eq!(MatrixLowerCase.apply_to_variant(original), m_lower);
        assert_eq!(MatrixSnakeCase.apply_to_variant(original), m_snake);
        assert_eq!(MatrixDottedCase.apply_to_variant(original), m_dotted);
        assert_eq!(MatrixRuleSnakeCase.apply_to_variant(original), m_rule_snake);
        assert_eq!(MatrixRoleSnakeCase.apply_to_variant(original), m_role_snake);
    }
}

#[test]
fn rename_fields() {
    for &(
        original,
        upper,
        pascal,
        camel,
        screaming,
        kebab,
        screaming_kebab,
        matrix_error,
        m_lower,
        m_snake,
        m_dotted,
        m_rule_snake,
        m_role_snake,
    ) in &[
        (
            "outcome",
            "OUTCOME",
            "Outcome",
            "outcome",
            "OUTCOME",
            "outcome",
            "OUTCOME",
            "M_OUTCOME",
            "m.outcome",
            "m.outcome",
            "m.outcome",
            ".m.rule.outcome",
            "m.role.outcome",
        ),
        (
            "very_tasty",
            "VERY_TASTY",
            "VeryTasty",
            "veryTasty",
            "VERY_TASTY",
            "very-tasty",
            "VERY-TASTY",
            "M_VERY_TASTY",
            "m.very_tasty",
            "m.very_tasty",
            "m.very.tasty",
            ".m.rule.very_tasty",
            "m.role.very_tasty",
        ),
        ("a", "A", "A", "a", "A", "a", "A", "M_A", "m.a", "m.a", "m.a", ".m.rule.a", "m.role.a"),
        (
            "z42",
            "Z42",
            "Z42",
            "z42",
            "Z42",
            "z42",
            "Z42",
            "M_Z42",
            "m.z42",
            "m.z42",
            "m.z42",
            ".m.rule.z42",
            "m.role.z42",
        ),
    ] {
        assert_eq!(None.apply_to_field(original), original);
        assert_eq!(Uppercase.apply_to_field(original), upper);
        assert_eq!(PascalCase.apply_to_field(original), pascal);
        assert_eq!(CamelCase.apply_to_field(original), camel);
        assert_eq!(SnakeCase.apply_to_field(original), original);
        assert_eq!(ScreamingSnakeCase.apply_to_field(original), screaming);
        assert_eq!(KebabCase.apply_to_field(original), kebab);
        assert_eq!(ScreamingKebabCase.apply_to_field(original), screaming_kebab);
        assert_eq!(MatrixErrorCase.apply_to_field(original), matrix_error);
        assert_eq!(MatrixLowerCase.apply_to_field(original), m_lower);
        assert_eq!(MatrixSnakeCase.apply_to_field(original), m_snake);
        assert_eq!(MatrixDottedCase.apply_to_field(original), m_dotted);
        assert_eq!(MatrixRuleSnakeCase.apply_to_field(original), m_rule_snake);
        assert_eq!(MatrixRoleSnakeCase.apply_to_field(original), m_role_snake);
    }
}
