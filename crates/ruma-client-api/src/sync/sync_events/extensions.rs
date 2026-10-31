//! Extensions shared by the sliding sync endpoints.

pub(super) mod request {
    use js_int::UInt;
    use serde::{Deserialize, Serialize};

    /// To-device messages extension.
    ///
    /// According to [MSC3885](https://github.com/matrix-org/matrix-spec-proposals/pull/3885).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ToDevice {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// Maximum number of to-device messages per response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Give messages since this token only.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub since: Option<String>,
    }

    impl ToDevice {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none() && self.limit.is_none() && self.since.is_none()
        }
    }

    /// E2EE extension configuration.
    ///
    /// According to [MSC3884](https://github.com/matrix-org/matrix-spec-proposals/pull/3884).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct E2EE {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,
    }

    impl E2EE {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Thread subscriptions extension.
    ///
    /// Specified as part of [MSC4308](https://github.com/matrix-org/matrix-spec-proposals/pull/4308).
    #[cfg(feature = "unstable-msc4308")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadSubscriptions {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// Maximum number of thread subscription changes to receive in the response.
        ///
        /// Defaults to 100.
        /// Servers may impose a smaller limit than what is requested here.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    #[cfg(feature = "unstable-msc4308")]
    impl ThreadSubscriptions {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none() && self.limit.is_none()
        }
    }

    /// Threads extension.
    ///
    /// Specified as part of [MSC4360](https://github.com/matrix-org/matrix-spec-proposals/pull/4360).
    #[cfg(feature = "unstable-msc4360")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Threads {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// Whether to include the thread root events in the response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub include_roots: Option<bool>,

        /// Maximum number of thread updates to receive in the response.
        ///
        /// Defaults to 100. Servers may impose a smaller limit than what is requested here.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    #[cfg(feature = "unstable-msc4360")]
    impl Threads {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none() && self.include_roots.is_none() && self.limit.is_none()
        }
    }
}

pub(super) mod response {
    use std::collections::BTreeMap;

    #[cfg(any(feature = "unstable-msc4308", feature = "unstable-msc4360"))]
    use ruma_common::OwnedEventId;
    #[cfg(any(feature = "unstable-msc4262", feature = "unstable-msc4559"))]
    use ruma_common::OwnedUserId;
    #[cfg(feature = "unstable-msc4559")]
    use ruma_common::presence::{PresenceState, PresenceStatus};
    #[cfg(feature = "unstable-msc4262")]
    use ruma_common::profile::UserProfileUpdate;
    use ruma_common::{OwnedRoomId, serde::Raw};
    #[cfg(feature = "unstable-msc4480")]
    use ruma_events::AnySyncTimelineEvent;
    use ruma_events::{AnyGlobalAccountDataEvent, AnyRoomAccountDataEvent};
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "unstable-msc4308")]
    use crate::threads::get_thread_subscriptions_changes::unstable::{
        ThreadSubscription, ThreadUnsubscription,
    };
    #[cfg(feature = "unstable-msc4360")]
    use crate::threads::get_thread_updates::unstable::ThreadUpdate;

    /// Account-data extension response .
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/account_data.go>
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AccountData {
        /// The global private data created by this user.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub global: Vec<Raw<AnyGlobalAccountDataEvent>>,

        /// The private data that this user has attached to each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, Vec<Raw<AnyRoomAccountDataEvent>>>,
    }

    impl AccountData {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.global.is_empty() && self.rooms.is_empty()
        }
    }

    /// Thread subscriptions extension response.
    ///
    /// Specified as part of [MSC4308](https://github.com/matrix-org/matrix-spec-proposals/pull/4308).
    #[cfg(feature = "unstable-msc4308")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadSubscriptions {
        /// New thread subscriptions.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub subscribed: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadSubscription>>,

        /// New thread unsubscriptions.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub unsubscribed: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadUnsubscription>>,

        /// A token that can be used to backpaginate (via the companion endpoint) other thread
        /// subscription changes that occurred since the last sync, but that were not included in
        /// this response.
        ///
        /// Only set when there are more changes to fetch.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
    }

    #[cfg(feature = "unstable-msc4308")]
    impl ThreadSubscriptions {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.subscribed.is_empty() && self.unsubscribed.is_empty() && self.prev_batch.is_none()
        }
    }

    /// Threads extension response.
    ///
    /// Specified as part of [MSC4360](https://github.com/matrix-org/matrix-spec-proposals/pull/4360).
    #[cfg(feature = "unstable-msc4360")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Threads {
        /// The updated threads, by room ID and thread root event ID.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub updates: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadUpdate>>,

        /// A token to paginate the thread updates that were not included in this response, with
        /// the `/thread_updates` endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
    }

    #[cfg(feature = "unstable-msc4360")]
    impl Threads {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.updates.is_empty() && self.prev_batch.is_none()
        }
    }

    /// Profiles extension response.
    ///
    /// Specified as part of [MSC4262](https://github.com/matrix-org/matrix-spec-proposals/pull/4262).
    #[cfg(feature = "unstable-msc4262")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Profiles {
        /// Profile updates keyed by user ID.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub users: BTreeMap<OwnedUserId, UserProfileUpdate>,
    }

    #[cfg(feature = "unstable-msc4262")]
    impl Profiles {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.users.is_empty()
        }
    }

    /// Sticky events extension response.
    ///
    /// According to [MSC4480](https://github.com/matrix-org/matrix-spec-proposals/pull/4480).
    #[cfg(feature = "unstable-msc4480")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StickyEvents {
        /// The sticky events, grouped by room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, StickyEventsRoom>,
    }

    #[cfg(feature = "unstable-msc4480")]
    impl StickyEvents {
        /// Whether the extension data is empty.
        pub fn is_empty(&self) -> bool {
            self.rooms.is_empty()
        }
    }

    /// Sticky events for a single room in the sticky events extension response.
    #[cfg(feature = "unstable-msc4480")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StickyEventsRoom {
        /// The sticky events for this room.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events: Vec<Raw<AnySyncTimelineEvent>>,
    }

    /// An update to a user's presence sent down sync.
    ///
    /// As defined in [MSC4559](https://github.com/matrix-org/matrix-spec-proposals/pull/4559).
    #[cfg(feature = "unstable-msc4559")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct PresenceUpdate {
        /// The user's presence state.
        ///
        /// Defaults to `PresenceState::Offline`.
        #[serde(default = "presence_offline", skip_serializing_if = "is_presence_offline")]
        pub presence: PresenceState,

        /// The status information for this user's presence, using the format from [MSC4532].
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub status: PresenceStatus,
    }

    #[cfg(feature = "unstable-msc4559")]
    fn presence_offline() -> PresenceState {
        PresenceState::Offline
    }

    #[cfg(feature = "unstable-msc4559")]
    fn is_presence_offline(presence: &PresenceState) -> bool {
        *presence == PresenceState::Offline
    }

    /// Presence extension response.
    ///
    /// Specified as part of [MSC4559](https://github.com/matrix-org/matrix-spec-proposals/pull/4559).
    #[cfg(feature = "unstable-msc4559")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Presence {
        /// Presence updates keyed by user ID.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub user_ids: BTreeMap<OwnedUserId, PresenceUpdate>,
    }

    #[cfg(feature = "unstable-msc4559")]
    impl Presence {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.user_ids.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "unstable-msc4480")]
    #[test]
    fn deserialize_sticky_events() {
        use super::response::StickyEvents;

        let sticky_events: StickyEvents = serde_json::from_value(serde_json::json!({
            "rooms": {
                "!room:example.com": {
                    "events": [{
                        "content": { "body": "sticky", "msgtype": "m.text" },
                        "event_id": "$1:example.com",
                        "origin_server_ts": 1,
                        "sender": "@alice:example.com",
                        "type": "m.room.message",
                        "sticky": { "duration_ms": 300_000 },
                    }]
                }
            }
        }))
        .unwrap();

        assert_eq!(sticky_events.rooms.len(), 1);
        let room = sticky_events.rooms.values().next().unwrap();
        assert_eq!(room.events.len(), 1);
    }

    #[cfg(feature = "unstable-msc4360")]
    #[test]
    fn threads_serde() {
        use ruma_common::{assert_to_canonical_json_eq, owned_event_id, owned_room_id};

        use super::{request, response};

        let threads = request::Threads {
            enabled: Some(true),
            include_roots: Some(true),
            ..Default::default()
        };
        assert_to_canonical_json_eq!(
            threads,
            serde_json::json!({ "enabled": true, "include_roots": true })
        );

        let threads: response::Threads = serde_json::from_value(serde_json::json!({
            "updates": {
                "!room:example.com": {
                    "$root:example.com": { "prev_batch": "t1" },
                },
            },
            "prev_batch": "t2",
        }))
        .unwrap();

        let update = &threads.updates[&owned_room_id!("!room:example.com")]
            [&owned_event_id!("$root:example.com")];
        assert!(update.thread_root.is_none());
        assert_eq!(update.prev_batch.as_deref(), Some("t1"));
        assert_eq!(threads.prev_batch.as_deref(), Some("t2"));
    }

    #[cfg(feature = "unstable-msc4559")]
    #[test]
    fn presence_update_defaults() {
        use ruma_common::{assert_to_canonical_json_eq, presence::PresenceState};

        use super::response::PresenceUpdate;

        let update: PresenceUpdate = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(update.presence, PresenceState::Offline);
        assert_to_canonical_json_eq!(update, serde_json::json!({}));
    }
}
