//! `Serialize` and `Deserialize` helpers for unstable poll kind (MSC3381).

use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{events::poll::start::PollKind, PrivOwnedStr};

/// Serializes a PollKind using the unstable prefixes.
pub(super) fn serialize<S>(kind: &PollKind, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = match kind {
        PollKind::Undisclosed => "org.matrix.msc3381.poll.undisclosed",
        PollKind::Disclosed => "org.matrix.msc3381.poll.disclosed",
        PollKind::_Custom(s) => &s.0,
    };

    s.serialize(serializer)
}

/// Deserializes a PollKind using the unstable prefixes.
pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<PollKind, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Cow::<'_, str>::deserialize(deserializer)?;

    let kind = match &*s {
        "org.matrix.msc3381.poll.undisclosed" => PollKind::Undisclosed,
        "org.matrix.msc3381.poll.disclosed" => PollKind::Disclosed,
        _ => PollKind::_Custom(PrivOwnedStr(s.into())),
    };

    Ok(kind)
}
