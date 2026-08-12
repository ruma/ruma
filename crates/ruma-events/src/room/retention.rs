//! Types for the `m.room.retention` state event.
//!
//! This event uses the unstable prefix defined in [MSC1763].
//!
//! [MSC1763]: https://github.com/matrix-org/matrix-spec-proposals/pull/1763

use std::{ops::RangeBounds, time::Duration};

use js_int::UInt;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{EmptyStateKey, PossiblyRedactedStateEventContent, StateEventType};

/// The content of an `m.room.retention` state event.
///
/// The `m.room.retention` state event lets room admins or moderators set or modify the history
/// retention behaviour for a given room.
///
/// This event uses the unstable prefix defined in [MSC1763].
///
/// [MSC1763]: https://github.com/matrix-org/matrix-spec-proposals/pull/1763
#[derive(Clone, Debug, Default, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc1763.retention", kind = State, state_key_type = EmptyStateKey, custom_possibly_redacted)]
pub struct RoomRetentionEventContent {
    /// The minimum amount of time messages should be kept on the homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_lifetime: Option<UInt>,

    /// The maximum amount of time messages should be kept on the homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_lifetime: Option<UInt>,
}

impl RoomRetentionEventContent {
    /// Create a new [`RoomRetentionEventContent`] with no retention limits set.
    ///
    /// This method can be combined with the [`RoomRetentionEventContent::at_least`] and
    /// [`RoomRetentionEventContent::at_most`] methods to configure the individual limits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use ruma_events::room::retention::RoomRetentionEventContent;
    /// # fn doctest() -> Option<()> {
    /// let content = RoomRetentionEventContent::new()
    ///     .at_least(Duration::from_hours(24))?
    ///     .at_most(Duration::from_hours(24 * 10))?;
    /// # None
    /// # }
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`RoomRetentionEventContent`] with the given maximum and minimum limits.
    ///
    /// This will return `None` if the duration of one of the limits, expressed as milliseconds,
    /// doesn't fall into the [0, (2^53)-1] range, or if `max_lifetime` < `min_lifetime`.
    fn new_impl(min_lifetime: Option<Duration>, max_lifetime: Option<Duration>) -> Option<Self> {
        // The lifetimes are defined as a duration in milliseconds represented as an integer in the
        // range [0, (2^53)-1], this range is the same as what our UInt type enforces.

        // First convert the duration into milliseconds, then attempt to convert the number of
        // milliseconds into an UInt.
        let max_lifetime = max_lifetime.map(|l| UInt::try_from(l.as_millis())).transpose().ok()?;
        let min_lifetime = min_lifetime.map(|l| UInt::try_from(l.as_millis())).transpose().ok()?;

        if is_valid_lifetime_combination(min_lifetime, max_lifetime) {
            Some(Self { max_lifetime, min_lifetime })
        } else {
            None
        }
    }

    /// Create a new [`RoomRetentionEventContent`] from a range.
    ///
    /// Returns `None` if the duration of one of the limits, expressed as milliseconds, doesn't
    /// fall into the [0, (2^53)-1] range, or if the lower bound of the range is bigger than the
    /// upper bound, i.e. `10..0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use ruma_events::room::retention::RoomRetentionEventContent;
    /// # fn doctest() -> Option<()> {
    /// let content = RoomRetentionEventContent::from_range(
    ///     Duration::from_hours(24)..Duration::from_hours(24 * 10),
    /// )?;
    /// # None
    /// # }
    /// ```
    pub fn from_range(lifetime_range: impl RangeBounds<Duration>) -> Option<Self> {
        let min_lifetime = match lifetime_range.start_bound() {
            std::ops::Bound::Included(v) => Some(*v),
            std::ops::Bound::Excluded(v) => Some(v.saturating_add(Duration::from_millis(1))),
            std::ops::Bound::Unbounded => None,
        };

        let max_lifetime = match lifetime_range.end_bound() {
            std::ops::Bound::Included(v) => Some(*v),
            std::ops::Bound::Excluded(v) => Some(v.saturating_sub(Duration::from_millis(1))),
            std::ops::Bound::Unbounded => None,
        };

        Self::new_impl(min_lifetime, max_lifetime)
    }

    /// Set the maximum amount of time a message should be kept on the homeserver.
    ///
    /// Returns `None` if the given limit, expressed as milliseconds, doesn't fall into the [0,
    /// (2^53)-1] range, or if the limits don't adhere to the `max` < `min` constraint.
    pub fn at_most(self, max: Duration) -> Option<Self> {
        let min = self.min_lifetime();
        Self::new_impl(min, Some(max))
    }

    /// Set the minimum amount of time a message should be kept on the homeserver.
    ///
    /// Returns `None` if the given limit, expressed as milliseconds, doesn't fall into the [0,
    /// (2^53)-1] range, or if the limits don't adhere to the `max` < `min` constraint.
    pub fn at_least(self, min: Duration) -> Option<Self> {
        let max = self.max_lifetime();
        Self::new_impl(Some(min), max)
    }

    /// Get the maximum event lifetime defined by this state event, if any.
    pub fn max_lifetime(&self) -> Option<Duration> {
        self.max_lifetime.map(|l| Duration::from_millis(l.into()))
    }

    /// Get the minimum event lifetime defined by this state event, if any.
    pub fn min_lifetime(&self) -> Option<Duration> {
        self.min_lifetime.map(|l| Duration::from_millis(l.into()))
    }
}

/// Validate a retention lifetime pair.
///
/// Returns false if both lifetimes are defined and the max lifetime is smaller than the min
/// lifetime.
pub fn is_valid_lifetime_combination(
    min_lifetime: Option<UInt>,
    max_lifetime: Option<UInt>,
) -> bool {
    match (min_lifetime, max_lifetime) {
        (Some(min), Some(max)) if max < min => false,
        (Some(_), Some(_)) | (None, None) | (None, Some(_)) | (Some(_), None) => true,
    }
}

impl<'de> Deserialize<'de> for RoomRetentionEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            max_lifetime: Option<UInt>,
            min_lifetime: Option<UInt>,
        }

        let Helper { max_lifetime, min_lifetime } = Helper::deserialize(deserializer)?;

        if is_valid_lifetime_combination(min_lifetime, max_lifetime) {
            Ok(Self { max_lifetime, min_lifetime })
        } else {
            Err(serde::de::Error::custom(
                "Invalid lifetimes, max_lifetime must always be higher or equal to min_lifetime."
                    .to_owned(),
            ))
        }
    }
}

/// The PossiblyRedacted version of [`RoomRetentionEventContent`].
///
/// Since the event has only optional fields it's already compatible with the redacted version of
/// the state event content.
pub type PossiblyRedactedRoomRetentionEventContent = RoomRetentionEventContent;

impl PossiblyRedactedStateEventContent for PossiblyRedactedRoomRetentionEventContent {
    type StateKey = EmptyStateKey;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomRetention
    }
}

impl From<RedactedRoomRetentionEventContent> for PossiblyRedactedRoomRetentionEventContent {
    fn from(_value: RedactedRoomRetentionEventContent) -> Self {
        Self { min_lifetime: None, max_lifetime: None }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{Value as JsonValue, from_value as from_json_value, json};

    use super::*;
    use crate::OriginalStateEvent;

    fn raw_json(
        min_lifetime: impl Into<Option<UInt>>,
        max_lifetime: impl Into<Option<UInt>>,
    ) -> JsonValue {
        json!({
            "content": {
                "max_lifetime": max_lifetime.into(),
                "min_lifetime": min_lifetime.into(),
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "org.matrix.msc1763.retention"
        })
    }

    #[test]
    fn deserialization() {
        let json_data = raw_json(None, None);
        let RoomRetentionEventContent { max_lifetime, min_lifetime, .. } =
            from_json_value::<OriginalStateEvent<RoomRetentionEventContent>>(json_data)
                .expect("No lifetimes should deserliaze")
                .content;

        assert_eq!(max_lifetime, None);
        assert_eq!(min_lifetime, None);

        let json_data = raw_json(uint!(10), None);
        let RoomRetentionEventContent { max_lifetime, min_lifetime, .. } =
            from_json_value::<OriginalStateEvent<RoomRetentionEventContent>>(json_data)
                .expect("A min lifetime and no max lifetime should deserialize")
                .content;

        assert_eq!(min_lifetime, Some(uint!(10)));
        assert_eq!(max_lifetime, None);

        let json_data = raw_json(uint!(10), uint!(10));
        let RoomRetentionEventContent { max_lifetime, min_lifetime, .. } =
            from_json_value::<OriginalStateEvent<RoomRetentionEventContent>>(json_data)
                .expect("Setting both lifetimes, should still deserialize")
                .content;

        assert_eq!(min_lifetime, Some(uint!(10)));
        assert_eq!(max_lifetime, Some(uint!(10)));

        let json_data = raw_json(uint!(20), uint!(10));
        from_json_value::<OriginalStateEvent<RoomRetentionEventContent>>(json_data).expect_err(
            "If the max lifetime is smaller than the min lifetime, we should fail to deserialize",
        );
    }

    #[test]
    fn serialization() {
        assert!(
            RoomRetentionEventContent::from_range(
                Duration::from_millis(10)..Duration::from_millis(0)
            )
            .is_none(),
            "Giving a max lifetime that's smaller than the min lifetime should give you a None"
        );

        let content = RoomRetentionEventContent::new().at_least(Duration::from_millis(10)).unwrap();

        assert_to_canonical_json_eq!(
            content,
            json!({
                "min_lifetime": uint!(10),
            }),
        );
    }
}
