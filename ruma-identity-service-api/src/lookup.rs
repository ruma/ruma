//! Endpoints to look up Matrix IDs bound to 3PIDs.

use ruma_serde_macros::{AsRefStr, DisplayAsRefStr, FromString};
use serde::{Deserialize, Serialize};

pub mod lookup_3pid;
pub mod get_hash_parameters;

/// The algorithms that can be used to hash the identifiers used for lookup, as defined in the
/// Matrix Spec.
///
/// This type can hold an arbitrary string. To check for algorithms that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    AsRefStr,
    DisplayAsRefStr,
    FromString,
    Serialize,
    Deserialize,
)]
#[non_exhaustive]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(feature = "serde", derive(DeserializeFromCowStr, SerializeAsRefStr))]
pub enum IdentifierHashingAlgorithm {
    /// The SHA-256 hashing algorithm.
    Sha256,

    /// No algorithm is used, and identifier strings are directly used for lookup.
    None,

    #[doc(hidden)]
    _Custom(String),
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
