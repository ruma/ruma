//! Endpoints to look up Matrix IDs bound to 3PIDs.

use ruma_common::serde::StringEnum;

use crate::PrivOwnedStr;

pub mod get_hash_parameters;
pub mod lookup_3pid;

/// The algorithms that can be used to hash the identifiers used for lookup, as defined in the
/// Matrix Spec.
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from a
/// string with `::from() / .into()`. To check for formats that are not available as a documented
/// variant here, use its string representation, obtained through `.as_str()`.
#[derive(Debug, PartialEq, Eq, Clone, StringEnum)]
#[non_exhaustive]
#[ruma_enum(rename_all = "snake_case")]
pub enum IdentifierHashingAlgorithm {
    /// The SHA-256 hashing algorithm.
    Sha256,

    /// No algorithm is used, and identifier strings are directly used for lookup.
    None,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod test {
    use super::IdentifierHashingAlgorithm;

    #[test]
    fn parse_identifier_hashing_algorithm() {
        assert_eq!(IdentifierHashingAlgorithm::from("sha256"), IdentifierHashingAlgorithm::Sha256);
        assert_eq!(IdentifierHashingAlgorithm::from("none"), IdentifierHashingAlgorithm::None);
    }
}
