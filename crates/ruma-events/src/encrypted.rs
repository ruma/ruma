//! Types for extensible encrypted events ([MSC3956]).
//!
//! [MSC3956]: https://github.com/matrix-org/matrix-spec-proposals/pull/3956

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::room::encrypted::{EncryptedEventScheme, Relation};

/// The payload for an extensible encrypted message.
///
/// This is the new primary type introduced in [MSC3956] and should only be sent in rooms with a
/// version that supports it. See the documentation of the [`message`] module for more information.
///
/// [MSC3956]: https://github.com/matrix-org/matrix-spec-proposals/pull/3956
/// [`message`]: super::message
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc1767.encrypted", kind = MessageLike)]
pub struct EncryptedEventContent {
    /// The encrypted content.
    #[serde(rename = "org.matrix.msc1767.encrypted")]
    pub encrypted: EncryptedContentBlock,

    /// Information about related events.
    #[serde(rename = "m.relates_to", skip_serializing_if = "Option::is_none")]
    pub relates_to: Option<Relation>,
}

impl EncryptedEventContent {
    /// Creates a new `EncryptedEventContent` with the given scheme and relation.
    pub fn new(scheme: EncryptedEventScheme, relates_to: Option<Relation>) -> Self {
        Self { encrypted: scheme.into(), relates_to }
    }
}

impl From<EncryptedEventScheme> for EncryptedEventContent {
    fn from(scheme: EncryptedEventScheme) -> Self {
        Self { encrypted: scheme.into(), relates_to: None }
    }
}

/// A block for encrypted content.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct EncryptedContentBlock {
    /// Algorithm-specific fields.
    #[serde(flatten)]
    pub scheme: EncryptedEventScheme,
}

impl From<EncryptedEventScheme> for EncryptedContentBlock {
    fn from(scheme: EncryptedEventScheme) -> Self {
        Self { scheme }
    }
}
