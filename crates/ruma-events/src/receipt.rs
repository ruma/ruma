//! Types for the [`m.receipt`] event.
//!
//! [`m.receipt`]: https://spec.matrix.org/latest/client-server-api/#mreceipt

mod receipt_thread_serde;

use std::{
    collections::{btree_map, BTreeMap},
    ops::{Deref, DerefMut},
};

use ruma_common::{
    EventId, IdParseError, MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedUserId, UserId,
};
use ruma_macros::{EventContent, OrdAsRefStr, PartialEqAsRefStr, PartialOrdAsRefStr, StringEnum};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The content of an `m.receipt` event.
///
/// A mapping of event ID to a collection of receipts for this event ID. The event ID is the ID of
/// the event being acknowledged and *not* an ID for the receipt itself.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[allow(clippy::exhaustive_structs)]
#[ruma_event(type = "m.receipt", kind = EphemeralRoom)]
pub struct ReceiptEventContent(pub BTreeMap<OwnedEventId, Receipts>);

impl ReceiptEventContent {
    /// Get the receipt for the given user ID with the given receipt type, if it exists.
    pub fn user_receipt(
        &self,
        user_id: &UserId,
        receipt_type: ReceiptType,
    ) -> Option<(&EventId, &Receipt)> {
        self.iter().find_map(|(event_id, receipts)| {
            let receipt = receipts.get(&receipt_type)?.get(user_id)?;
            Some((event_id.as_ref(), receipt))
        })
    }
}

impl Deref for ReceiptEventContent {
    type Target = BTreeMap<OwnedEventId, Receipts>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReceiptEventContent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for ReceiptEventContent {
    type Item = (OwnedEventId, Receipts);
    type IntoIter = btree_map::IntoIter<OwnedEventId, Receipts>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(OwnedEventId, Receipts)> for ReceiptEventContent {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (OwnedEventId, Receipts)>,
    {
        Self(BTreeMap::from_iter(iter))
    }
}

/// A collection of receipts.
pub type Receipts = BTreeMap<ReceiptType, UserReceipts>;

/// The type of receipt.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialOrdAsRefStr, OrdAsRefStr, PartialEqAsRefStr, Eq, StringEnum, Hash)]
#[non_exhaustive]
pub enum ReceiptType {
    /// A [public read receipt].
    ///
    /// Indicates that the given event has been presented to the user. It is
    /// also the point from where the unread notifications count is computed.
    ///
    /// This receipt is federated to other users.
    ///
    /// If both `Read` and `ReadPrivate` are present, the one that references
    /// the most recent event is used to get the latest read receipt.
    ///
    /// [public read receipt]: https://spec.matrix.org/latest/client-server-api/#receipts
    #[ruma_enum(rename = "m.read")]
    Read,

    /// A [private read receipt].
    ///
    /// Indicates that the given event has been presented to the user. It is
    /// also the point from where the unread notifications count is computed.
    ///
    /// This read receipt is not federated so only the user and their homeserver
    /// are aware of it.
    ///
    /// If both `Read` and `ReadPrivate` are present, the one that references
    /// the most recent event is used to get the latest read receipt.
    ///
    /// [private read receipt]: https://spec.matrix.org/latest/client-server-api/#private-read-receipts
    #[ruma_enum(rename = "m.read.private")]
    ReadPrivate,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// A mapping of user ID to receipt.
///
/// The user ID is the entity who sent this receipt.
pub type UserReceipts = BTreeMap<OwnedUserId, Receipt>;

/// An acknowledgement of an event.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Receipt {
    /// The time when the receipt was sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<MilliSecondsSinceUnixEpoch>,

    /// The thread this receipt applies to.
    #[serde(rename = "thread_id", default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub thread: ReceiptThread,
}

impl Receipt {
    /// Creates a new `Receipt` with the given timestamp.
    ///
    /// To create an empty receipt instead, use [`Receipt::default`].
    pub fn new(ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self { ts: Some(ts), thread: ReceiptThread::Unthreaded }
    }
}

/// The [thread a receipt applies to].
///
/// This type can hold an arbitrary string. To build this with a custom value, convert it from an
/// `Option<String>` with `::from()` / `.into()`. [`ReceiptThread::Unthreaded`] can be constructed
/// from `None`.
///
/// To check for values that are not available as a documented variant here, use its string
/// representation, obtained through [`.as_str()`](Self::as_str()).
///
/// [thread a receipt applies to]: https://spec.matrix.org/latest/client-server-api/#threaded-read-receipts
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReceiptThread {
    /// The receipt applies to the timeline, regardless of threads.
    ///
    /// Used by clients that are not aware of threads.
    ///
    /// This is the default.
    #[default]
    Unthreaded,

    /// The receipt applies to the main timeline.
    ///
    /// Used for events that don't belong to a thread.
    Main,

    /// The receipt applies to a thread.
    ///
    /// Used for events that belong to a thread with the given thread root.
    Thread(OwnedEventId),

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl ReceiptThread {
    /// Get the string representation of this `ReceiptThread`.
    ///
    /// [`ReceiptThread::Unthreaded`] returns `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Unthreaded => None,
            Self::Main => Some("main"),
            Self::Thread(event_id) => Some(event_id.as_str()),
            Self::_Custom(s) => Some(&s.0),
        }
    }
}

impl<T> TryFrom<Option<T>> for ReceiptThread
where
    T: AsRef<str> + Into<Box<str>>,
{
    type Error = IdParseError;

    fn try_from(s: Option<T>) -> Result<Self, Self::Error> {
        let res = match s {
            None => Self::Unthreaded,
            Some(s) => match s.as_ref() {
                "main" => Self::Main,
                s_ref if s_ref.starts_with('$') => Self::Thread(EventId::parse(s_ref)?),
                _ => Self::_Custom(PrivOwnedStr(s.into())),
            },
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{owned_event_id, MilliSecondsSinceUnixEpoch};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{Receipt, ReceiptThread};

    #[test]
    fn serialize_receipt() {
        let mut receipt = Receipt::default();
        assert_eq!(to_json_value(receipt.clone()).unwrap(), json!({}));

        receipt.thread = ReceiptThread::Main;
        assert_eq!(to_json_value(receipt.clone()).unwrap(), json!({ "thread_id": "main" }));

        receipt.thread = ReceiptThread::Thread(owned_event_id!("$abcdef76543"));
        assert_eq!(to_json_value(receipt).unwrap(), json!({ "thread_id": "$abcdef76543" }));

        let mut receipt =
            Receipt::new(MilliSecondsSinceUnixEpoch(1_664_702_144_365_u64.try_into().unwrap()));
        assert_eq!(to_json_value(receipt.clone()).unwrap(), json!({ "ts": 1_664_702_144_365_u64 }));

        receipt.thread = ReceiptThread::try_from(Some("io.ruma.unknown")).unwrap();
        assert_eq!(
            to_json_value(receipt).unwrap(),
            json!({ "ts": 1_664_702_144_365_u64, "thread_id": "io.ruma.unknown" })
        );
    }

    #[test]
    fn deserialize_receipt() {
        let receipt = from_json_value::<Receipt>(json!({})).unwrap();
        assert_eq!(receipt.ts, None);
        assert_eq!(receipt.thread, ReceiptThread::Unthreaded);

        let receipt = from_json_value::<Receipt>(json!({ "thread_id": "main" })).unwrap();
        assert_eq!(receipt.ts, None);
        assert_eq!(receipt.thread, ReceiptThread::Main);

        let receipt = from_json_value::<Receipt>(json!({ "thread_id": "$abcdef76543" })).unwrap();
        assert_eq!(receipt.ts, None);
        assert_matches!(receipt.thread, ReceiptThread::Thread(event_id));
        assert_eq!(event_id, "$abcdef76543");

        let receipt = from_json_value::<Receipt>(json!({ "ts": 1_664_702_144_365_u64 })).unwrap();
        assert_eq!(
            receipt.ts.unwrap(),
            MilliSecondsSinceUnixEpoch(1_664_702_144_365_u64.try_into().unwrap())
        );
        assert_eq!(receipt.thread, ReceiptThread::Unthreaded);

        let receipt = from_json_value::<Receipt>(
            json!({ "ts": 1_664_702_144_365_u64, "thread_id": "io.ruma.unknown" }),
        )
        .unwrap();
        assert_eq!(
            receipt.ts.unwrap(),
            MilliSecondsSinceUnixEpoch(1_664_702_144_365_u64.try_into().unwrap())
        );
        assert_matches!(&receipt.thread, ReceiptThread::_Custom(_));
        assert_eq!(receipt.thread.as_str().unwrap(), "io.ruma.unknown");
    }
}
