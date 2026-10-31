//! `/unstable/org.matrix.simplified_msc3575/` ([MSC])
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4186

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use js_option::JsOption;
use ruma_common::{
    OwnedMxcUri, OwnedRoomId, OwnedUserId, UserId,
    api::{auth_scheme::AccessToken, request, response},
    metadata,
    presence::PresenceState,
    serde::{Raw, duration::opt_ms},
};
use ruma_events::{AnySyncStateEvent, AnySyncTimelineEvent, StateEventType};
use serde::{Deserialize, Serialize};

use super::{UnreadNotificationsCount, v4};

metadata! {
    method: POST,
    rate_limited: false,
    authentication: AccessToken,
    history: {
        unstable("org.matrix.simplified_msc3575") => "/_matrix/client/unstable/org.matrix.simplified_msc3575/sync",
    }
}

/// Request type for the `/sync` endpoint.
#[request]
#[derive(Default)]
pub struct Request {
    /// A point in time to continue a sync from.
    ///
    /// This is an opaque value taken from the `pos` field of a previous `/sync`
    /// response. A `None` value asks the server to start a new _session_ (mind
    /// it can be costly)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ruma_api(query)]
    pub pos: Option<String>,

    /// A unique string identifier for this connection to the server.
    ///
    /// If this is missing, only one sliding sync connection can be made to
    /// the server at any one time. Clients need to set this to allow more
    /// than one connection concurrently, so the server can distinguish between
    /// connections. This must be provided with every request, if your client
    /// needs more than one concurrent connection.
    ///
    /// Limitation: it must not contain more than 16 chars, due to it being
    /// required with every request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn_id: Option<String>,

    /// Allows clients to know what request params reached the server,
    /// functionally similar to txn IDs on `/send` for events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_id: Option<String>,

    /// The maximum time to poll before responding to this request.
    ///
    /// `None` means no timeout, so virtually an infinite wait from the server.
    #[serde(with = "opt_ms", default, skip_serializing_if = "Option::is_none")]
    #[ruma_api(query)]
    pub timeout: Option<Duration>,

    /// Controls whether the client is automatically marked as online by polling this API.
    ///
    /// Defaults to `PresenceState::Online`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    #[ruma_api(query)]
    pub set_presence: PresenceState,

    /// Lists of rooms we are interested by, represented by ranges.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, request::List>,

    /// Specific rooms we are interested by.
    ///
    /// It is useful to receive updates from rooms that are possibly
    /// out-of-range of all the lists (see [`Self::lists`]).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub room_subscriptions: BTreeMap<OwnedRoomId, request::RoomSubscription>,

    /// Extensions.
    #[serde(default, skip_serializing_if = "request::Extensions::is_empty")]
    pub extensions: request::Extensions,
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Converts this request to a [`v4::Request`].
    ///
    /// `user_id` is the ID of the user making the request, used to replace the `$ME` state keys.
    ///
    /// The conversion is lossy:
    ///
    /// * `txn_id` is dropped, so it must be copied to the [`Response`] separately.
    /// * The `ranges` of each list are merged into a single range covering all of them.
    /// * `*` in the `rooms` of an extension becomes all the room subscriptions.
    pub fn into_v4(self, user_id: &UserId) -> v4::Request {
        let Self {
            pos,
            conn_id,
            txn_id: _,
            timeout,
            set_presence,
            lists,
            room_subscriptions,
            extensions,
        } = self;

        v4::Request {
            conn_id,
            pos,
            timeout,
            set_presence,
            lists: lists
                .into_iter()
                .map(|(key, list)| (key, list_into_v4(list, user_id)))
                .collect(),
            room_subscriptions: room_subscriptions
                .into_iter()
                .map(|(room_id, subscription)| {
                    let subscription = v4::request::RoomSubscription {
                        required_state: required_state_into_v4(
                            subscription.required_state,
                            user_id,
                        ),
                        timeline_limit: subscription.timeline_limit,
                    };
                    (room_id, subscription)
                })
                .collect(),
            extensions: extensions.into(),
        }
    }
}

fn list_into_v4(list: request::List, user_id: &UserId) -> v4::request::List {
    let request::List { ranges, room_details, filters } = list;

    v4::request::List {
        range: ranges.into_iter().reduce(|(start, end), (other_start, other_end)| {
            (start.min(other_start), end.max(other_end))
        }),
        room_details: v4::request::RoomDetails {
            required_state: required_state_into_v4(room_details.required_state, user_id),
            timeline_limit: room_details.timeline_limit,
        },
        filters: filters.map(|filters| v4::request::ListFilters {
            is_dm: filters.is_dm,
            is_encrypted: filters.is_encrypted,
            is_invited: filters.is_invite,
            room_types: filters.room_types,
            not_room_types: filters.not_room_types,
            ..Default::default()
        }),
    }
}

fn required_state_into_v4(
    required_state: Vec<(StateEventType, String)>,
    user_id: &UserId,
) -> v4::request::RequiredState {
    let mut v4_required_state = v4::request::RequiredState::default();

    for (event_type, state_key) in required_state {
        if event_type == StateEventType::RoomMember && state_key == "$LAZY" {
            v4_required_state.lazy_members = true;
            continue;
        }

        let event_type = (event_type.to_string() != "*").then_some(event_type);
        let state_key = match state_key.as_str() {
            "*" => None,
            "$ME" => Some(user_id.as_str().to_owned()),
            _ => Some(state_key),
        };

        v4_required_state.include.push(v4::request::RequiredStateElement { event_type, state_key });
    }

    v4_required_state
}

/// HTTP types related to a [`Request`].
pub mod request {
    use std::collections::BTreeMap;

    use as_variant::as_variant;
    use ruma_common::{OwnedRoomId, RoomId, directory::RoomTypeFilter, serde::deserialize_cow_str};
    use serde::de::Error as _;

    #[cfg(feature = "unstable-msc4308")]
    pub use super::super::extensions::request::ThreadSubscriptions;
    #[cfg(feature = "unstable-msc4360")]
    pub use super::super::extensions::request::Threads;
    pub use super::super::extensions::request::{E2EE, ToDevice};
    use super::{Deserialize, Serialize, StateEventType, UInt, v4};

    /// A sliding sync list request (see [`super::Request::lists`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The ranges of rooms we're interested in.
        pub ranges: Vec<(UInt, UInt)>,

        /// The details to be included per room.
        #[serde(flatten)]
        pub room_details: RoomDetails,

        /// Filters to apply to the list before sorting.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filters: Option<ListFilters>,
    }

    /// A sliding sync list request filters (see [`List::filters`]).
    ///
    /// All fields are applied with _AND_ operators. The absence of fields
    /// implies no filter on that criteria: it does NOT imply `false`.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ListFilters {
        /// Whether to return only DM rooms (as determined by the `m.direct` account data event),
        /// only non-DM rooms, or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_dm: Option<bool>,

        /// Whether to return only encrypted rooms (as determined by the existence of an
        /// `m.room.encryption` state event), only unencrypted rooms, or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_encrypted: Option<bool>,

        /// Whether to return only invited rooms, only joined rooms, or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_invite: Option<bool>,

        /// Only list rooms with these create-types, or all.
        ///
        /// If a room type is specified in both `room_types` and `not_room_types`,
        /// `not_room_types` wins and the corresponding rooms are not included.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub room_types: Vec<RoomTypeFilter>,

        /// Only list rooms that are not of these create-types, or all.
        ///
        /// If a room type is specified in both `room_types` and `not_room_types`,
        /// `not_room_types` wins and the corresponding rooms are not included.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub not_room_types: Vec<RoomTypeFilter>,
    }

    /// Sliding sync request room subscription (see [`super::Request::room_subscriptions`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomSubscription {
        /// Required state for each returned room. An array of event type and
        /// state key tuples.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub required_state: Vec<(StateEventType, String)>,

        /// The maximum number of timeline events to return per room.
        pub timeline_limit: UInt,
    }

    /// Sliding sync request room details (see [`List::room_details`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomDetails {
        /// Required state for each returned room. An array of event type and state key tuples.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub required_state: Vec<(StateEventType, String)>,

        /// The maximum number of timeline events to return per room.
        pub timeline_limit: UInt,
    }

    /// Sliding sync request extensions.
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Extensions {
        /// Configure the to-device extension.
        #[serde(default, skip_serializing_if = "ToDevice::is_empty")]
        pub to_device: ToDevice,

        /// Configure the E2EE extension.
        #[serde(default, skip_serializing_if = "E2EE::is_empty")]
        pub e2ee: E2EE,

        /// Configure the account data extension.
        #[serde(default, skip_serializing_if = "AccountData::is_empty")]
        pub account_data: AccountData,

        /// Configure the receipts extension.
        #[serde(default, skip_serializing_if = "Receipts::is_empty")]
        pub receipts: Receipts,

        /// Configure the typing extension.
        #[serde(default, skip_serializing_if = "Typing::is_empty")]
        pub typing: Typing,

        /// Configure the thread subscriptions extension.
        #[cfg(feature = "unstable-msc4308")]
        #[serde(
            default,
            skip_serializing_if = "ThreadSubscriptions::is_empty",
            rename = "io.element.msc4308.thread_subscriptions"
        )]
        pub thread_subscriptions: ThreadSubscriptions,

        /// Configure the threads extension.
        #[cfg(feature = "unstable-msc4360")]
        #[serde(
            default,
            skip_serializing_if = "Threads::is_empty",
            rename = "io.element.msc4360.threads"
        )]
        pub threads: Threads,

        /// Configure the profiles extension.
        #[cfg(feature = "unstable-msc4262")]
        #[serde(
            default,
            skip_serializing_if = "Profiles::is_empty",
            rename = "org.matrix.msc4262.profiles"
        )]
        pub profiles: Profiles,

        /// Configure the sticky events extension.
        #[cfg(feature = "unstable-msc4480")]
        #[serde(
            default,
            skip_serializing_if = "StickyEvents::is_empty",
            rename = "org.matrix.msc4354.sticky_events"
        )]
        pub sticky_events: StickyEvents,

        /// Configure the presence extension.
        #[cfg(feature = "unstable-msc4559")]
        #[serde(
            default,
            skip_serializing_if = "Presence::is_empty",
            rename = "org.continuwuity.presence_v2.msc4559.presence"
        )]
        pub presence: Presence,

        /// Extensions may add further fields to the list.
        #[serde(flatten)]
        other: BTreeMap<String, serde_json::Value>,
    }

    impl Extensions {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            #[allow(unused_mut)]
            let mut empty = self.to_device.is_empty()
                && self.e2ee.is_empty()
                && self.account_data.is_empty()
                && self.receipts.is_empty()
                && self.typing.is_empty()
                && self.other.is_empty();

            #[cfg(feature = "unstable-msc4308")]
            {
                empty = empty && self.thread_subscriptions.is_empty();
            }

            #[cfg(feature = "unstable-msc4360")]
            {
                empty = empty && self.threads.is_empty();
            }

            #[cfg(feature = "unstable-msc4262")]
            {
                empty = empty && self.profiles.is_empty();
            }

            #[cfg(feature = "unstable-msc4480")]
            {
                empty = empty && self.sticky_events.is_empty();
            }

            #[cfg(feature = "unstable-msc4559")]
            {
                empty = empty && self.presence.is_empty();
            }

            empty
        }
    }

    /// `*` in the `rooms` of an extension becomes all the room subscriptions.
    impl From<Extensions> for v4::request::Extensions {
        fn from(value: Extensions) -> Self {
            let Extensions {
                to_device,
                e2ee,
                account_data,
                receipts,
                typing,
                #[cfg(feature = "unstable-msc4308")]
                thread_subscriptions,
                #[cfg(feature = "unstable-msc4360")]
                threads,
                #[cfg(feature = "unstable-msc4262")]
                profiles,
                #[cfg(feature = "unstable-msc4480")]
                sticky_events,
                #[cfg(feature = "unstable-msc4559")]
                presence,
                other,
            } = value;

            Self {
                to_device,
                e2ee,
                account_data: v4::request::AccountData {
                    enabled: account_data.enabled,
                    lists: account_data.lists,
                    rooms: rooms_into_v4(account_data.rooms),
                },
                receipts: v4::request::Receipts {
                    enabled: receipts.enabled,
                    lists: receipts.lists,
                    rooms: rooms_into_v4(receipts.rooms),
                },
                typing: v4::request::Typing {
                    enabled: typing.enabled,
                    lists: typing.lists,
                    rooms: rooms_into_v4(typing.rooms),
                },
                #[cfg(feature = "unstable-msc4308")]
                thread_subscriptions,
                #[cfg(feature = "unstable-msc4360")]
                threads,
                #[cfg(feature = "unstable-msc4262")]
                profiles: v4::request::Profiles {
                    enabled: profiles.enabled,
                    fields: profiles.fields,
                    lists: profiles.lists,
                    rooms: rooms_into_v4(profiles.rooms),
                },
                #[cfg(feature = "unstable-msc4480")]
                sticky_events: v4::request::StickyEvents {
                    enabled: sticky_events.enabled,
                    limit: sticky_events.limit,
                    lists: sticky_events.lists,
                    rooms: rooms_into_v4(sticky_events.rooms),
                },
                #[cfg(feature = "unstable-msc4559")]
                presence: v4::request::Presence {
                    enabled: presence.enabled,
                    lists: presence.lists,
                    rooms: rooms_into_v4(presence.rooms),
                },
                other,
            }
        }
    }

    fn rooms_into_v4(rooms: Option<Vec<ExtensionRoomConfig>>) -> Option<Vec<OwnedRoomId>> {
        rooms?.into_iter().map(|room| as_variant!(room, ExtensionRoomConfig::Room)).collect()
    }

    /// Single entry for a room subscription configuration in an extension request.
    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum ExtensionRoomConfig {
        /// Apply extension to all global room subscriptions.
        AllSubscribed,

        /// Additionally apply extension to this specific room.
        Room(OwnedRoomId),
    }

    impl Serialize for ExtensionRoomConfig {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::AllSubscribed => serializer.serialize_str("*"),
                Self::Room(r) => r.serialize(serializer),
            }
        }
    }

    impl<'de> Deserialize<'de> for ExtensionRoomConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            match deserialize_cow_str(deserializer)?.as_ref() {
                "*" => Ok(Self::AllSubscribed),
                other => Ok(Self::Room(RoomId::parse(other).map_err(D::Error::custom)?)),
            }
        }
    }

    /// Sticky events extension configuration.
    ///
    /// According to [MSC4480](https://github.com/matrix-org/matrix-spec-proposals/pull/4480).
    #[cfg(feature = "unstable-msc4480")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StickyEvents {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// Maximum number of sticky events to return per response.
        ///
        /// Defaults to 100 (server-side) and the server may override it to a lower value.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// List of list names for which sticky events should be enabled.
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which sticky events should be enabled.
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    #[cfg(feature = "unstable-msc4480")]
    impl StickyEvents {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none() && self.limit.is_none()
        }
    }

    /// Account-data extension.
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/account_data.go>
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AccountData {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which account data should be enabled.
        ///
        /// This is specific to room account data (e.g. user-defined room tags).
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which account data should be enabled.
        ///
        /// This is specific to room account data (e.g. user-defined room tags).
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    impl AccountData {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Receipt extension.
    ///
    /// According to [MSC3960](https://github.com/matrix-org/matrix-spec-proposals/pull/3960)
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Receipts {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which receipts should be enabled.
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which receipts should be enabled.
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    impl Receipts {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Typing extension configuration.
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/typing.go>
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Typing {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which typing notifications should be enabled.
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which typing notifications should be enabled.
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    impl Typing {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// User profiles extension.
    ///
    /// Specified as part of [MSC4262](https://github.com/matrix-org/matrix-spec-proposals/pull/4262).
    #[cfg(feature = "unstable-msc4262")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Profiles {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which user profiles should be enabled.
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which user profiles should be enabled.
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,

        /// Optional filter to control which profile fields to receive updates for. If omitted, all
        /// profile field updates are included.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub fields: Option<Vec<ruma_common::profile::ProfileFieldName>>,
    }

    #[cfg(feature = "unstable-msc4262")]
    impl Profiles {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Presence extension.
    ///
    /// Specified as part of [MSC4559](https://github.com/matrix-org/matrix-spec-proposals/pull/4559).
    #[cfg(feature = "unstable-msc4559")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Presence {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which user presence should be enabled.
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which user presence should be enabled.
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    #[cfg(feature = "unstable-msc4559")]
    impl Presence {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }
}

/// Response type for the `/sync` endpoint.
#[response]
pub struct Response {
    /// Matches the `txn_id` sent by the request (see [`Request::txn_id`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_id: Option<String>,

    /// The token to supply in the `pos` parameter of the next `/sync` request
    /// (see [`Request::pos`]).
    pub pos: String,

    /// Resulting details of the lists.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, response::List>,

    /// The updated rooms.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rooms: BTreeMap<OwnedRoomId, response::Room>,

    /// Extensions.
    #[serde(default, skip_serializing_if = "response::Extensions::is_empty")]
    pub extensions: response::Extensions,
}

impl Response {
    /// Creates a new `Response` with the given `pos`.
    pub fn new(pos: String) -> Self {
        Self {
            txn_id: None,
            pos,
            lists: Default::default(),
            rooms: Default::default(),
            extensions: Default::default(),
        }
    }
}

/// The conversion is lossy in the same way as for [`response::Room`], and `txn_id` is not set.
impl From<v4::Response> for Response {
    fn from(value: v4::Response) -> Self {
        let v4::Response { pos, lists, rooms, extensions } = value;

        Self {
            txn_id: None,
            pos,
            lists: lists
                .into_iter()
                .map(|(key, list)| (key, response::List { count: list.count }))
                .collect(),
            rooms: rooms.into_iter().map(|(room_id, room)| (room_id, room.into())).collect(),
            extensions: extensions.into(),
        }
    }
}

/// HTTP types related to a [`Response`].
pub mod response {
    use as_variant::as_variant;
    use ruma_common::OneTimeKeyAlgorithm;
    use ruma_events::{
        AnyStrippedStateEvent, AnyToDeviceEvent, receipt::SyncReceiptEvent, typing::SyncTypingEvent,
    };
    use serde_json::{json, value::to_raw_value as to_raw_json_value};

    pub use super::super::extensions::response::AccountData;
    #[cfg(feature = "unstable-msc4262")]
    pub use super::super::extensions::response::Profiles;
    #[cfg(feature = "unstable-msc4308")]
    pub use super::super::extensions::response::ThreadSubscriptions;
    #[cfg(feature = "unstable-msc4360")]
    pub use super::super::extensions::response::Threads;
    #[cfg(feature = "unstable-msc4559")]
    pub use super::super::extensions::response::{Presence, PresenceUpdate};
    #[cfg(feature = "unstable-msc4480")]
    pub use super::super::extensions::response::{StickyEvents, StickyEventsRoom};
    use super::{
        super::DeviceLists, AnySyncStateEvent, AnySyncTimelineEvent, BTreeMap, Deserialize,
        JsOption, OwnedMxcUri, OwnedRoomId, OwnedUserId, Raw, Serialize, UInt,
        UnreadNotificationsCount, v4,
    };

    /// A sliding sync response updates to joiend rooms (see
    /// [`super::Response::lists`]).
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The total number of rooms found for this list.
        pub count: UInt,
    }

    /// A sliding sync response updated room (see [`super::Response::rooms`]).
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Room {
        /// The name as calculated by the server.
        ///
        /// If the `unstable-compat-lax-syncv5-deser` feature is enabled,
        /// this field is ignored if its deserialization fails.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "unstable-compat-lax-syncv5-deser",
            serde(default, deserialize_with = "ruma_common::serde::default_on_error")
        )]
        pub name: Option<String>,

        /// The avatar.
        ///
        /// If the `unstable-compat-lax-syncv5-deser` feature is enabled,
        /// this field is ignored if its deserialization fails.
        #[serde(default, skip_serializing_if = "JsOption::is_undefined")]
        #[cfg_attr(
            feature = "unstable-compat-lax-syncv5-deser",
            serde(deserialize_with = "ruma_common::serde::default_on_error")
        )]
        pub avatar: JsOption<OwnedMxcUri>,

        /// Whether it is an initial response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial: Option<bool>,

        /// Whether it is a direct room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_dm: Option<bool>,

        /// If this is `Some(_)`, this is a not-yet-accepted invite containing
        /// the given stripped state events.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invite_state: Option<Vec<Raw<AnyStrippedStateEvent>>>,

        /// Number of unread notifications.
        #[serde(flatten, default, skip_serializing_if = "UnreadNotificationsCount::is_empty")]
        pub unread_notifications: UnreadNotificationsCount,

        /// Message-like events and live state events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub timeline: Vec<Raw<AnySyncTimelineEvent>>,

        /// State events as configured by the request.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub required_state: Vec<Raw<AnySyncStateEvent>>,

        /// The `prev_batch` allowing you to paginate through the messages
        /// before the given ones.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,

        /// True if the number of events returned was limited by the limit on
        /// the filter.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub limited: bool,

        /// The number of users with membership of `join`, including the
        /// client’s own user ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub joined_count: Option<UInt>,

        /// The number of users with membership of `invite`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invited_count: Option<UInt>,

        /// The number of timeline events which have just occurred and are not
        /// historical.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub num_live: Option<UInt>,

        /// The bump stamp of the room.
        ///
        /// It can be interpreted as a “recency stamp” or “streaming order
        /// index”. For example, consider `roomA` with `bump_stamp = 2`, `roomB`
        /// with `bump_stamp = 1` and `roomC` with `bump_stamp = 0`. If `roomC`
        /// receives an update, its `bump_stamp` will be 3.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bump_stamp: Option<UInt>,

        /// Heroes of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub heroes: Option<Vec<Hero>>,
    }

    impl Room {
        /// Creates an empty `Room`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// The conversion is lossy: `membership`, `lists`, `expanded_timeline`, deleted state
    /// entries and the removal of the room name are dropped.
    impl From<v4::response::Room> for Room {
        fn from(value: v4::response::Room) -> Self {
            let v4::response::Room {
                bump_stamp,
                membership: _,
                lists: _,
                name,
                avatar,
                heroes,
                is_dm,
                initial,
                expanded_timeline: _,
                required_state,
                timeline,
                prev_batch,
                limited,
                num_live,
                joined_count,
                invited_count,
                stripped_state,
            } = value;

            Self {
                name: name.into_option(),
                avatar,
                initial: initial.then_some(true),
                is_dm,
                invite_state: stripped_state,
                unread_notifications: UnreadNotificationsCount::new(),
                timeline,
                required_state: required_state
                    .into_iter()
                    .filter_map(|entry| as_variant!(entry, v4::response::RequiredStateEntry::Event))
                    .collect(),
                prev_batch,
                limited,
                joined_count,
                invited_count,
                num_live,
                bump_stamp,
                heroes: heroes.map(|heroes| heroes.into_iter().map(Into::into).collect()),
            }
        }
    }

    /// A sliding sync response room hero (see [`Room::heroes`]).
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Hero {
        /// The user ID.
        pub user_id: OwnedUserId,

        /// The name.
        ///
        /// If the `unstable-compat-lax-syncv5-deser` feature is enabled,
        /// this field is ignored if its deserialization fails.
        #[serde(rename = "displayname", skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "unstable-compat-lax-syncv5-deser",
            serde(default, deserialize_with = "ruma_common::serde::default_on_error")
        )]
        pub name: Option<String>,

        /// The avatar.
        ///
        /// If the `unstable-compat-lax-syncv5-deser` feature is enabled,
        /// this field is ignored if its deserialization fails.
        #[serde(rename = "avatar_url", skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "unstable-compat-lax-syncv5-deser",
            serde(default, deserialize_with = "ruma_common::serde::default_on_error")
        )]
        pub avatar: Option<OwnedMxcUri>,
    }

    impl Hero {
        /// Creates a new `Hero` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, name: None, avatar: None }
        }
    }

    impl From<v4::response::Hero> for Hero {
        fn from(value: v4::response::Hero) -> Self {
            let v4::response::Hero { user_id, name, avatar } = value;
            Self { user_id, name, avatar }
        }
    }

    /// Extensions responses.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Extensions {
        /// To-device extension response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to_device: Option<ToDevice>,

        /// E2EE extension response.
        #[serde(default, skip_serializing_if = "E2EE::is_empty")]
        pub e2ee: E2EE,

        /// Account data extension response.
        #[serde(default, skip_serializing_if = "AccountData::is_empty")]
        pub account_data: AccountData,

        /// Receipts extension response.
        #[serde(default, skip_serializing_if = "Receipts::is_empty")]
        pub receipts: Receipts,

        /// Typing extension response.
        #[serde(default, skip_serializing_if = "Typing::is_empty")]
        pub typing: Typing,

        /// Thread subscriptions extension response.
        #[cfg(feature = "unstable-msc4308")]
        #[serde(
            default,
            skip_serializing_if = "ThreadSubscriptions::is_empty",
            rename = "io.element.msc4308.thread_subscriptions"
        )]
        pub thread_subscriptions: ThreadSubscriptions,

        /// Threads extension response.
        #[cfg(feature = "unstable-msc4360")]
        #[serde(
            default,
            skip_serializing_if = "Threads::is_empty",
            rename = "io.element.msc4360.threads"
        )]
        pub threads: Threads,

        /// Profiles extension response.
        #[cfg(feature = "unstable-msc4262")]
        #[serde(
            default,
            skip_serializing_if = "Profiles::is_empty",
            rename = "org.matrix.msc4262.profiles"
        )]
        pub profiles: Profiles,

        /// Sticky events extension response.
        #[cfg(feature = "unstable-msc4480")]
        #[serde(
            default,
            skip_serializing_if = "StickyEvents::is_empty",
            rename = "org.matrix.msc4354.sticky_events"
        )]
        pub sticky_events: StickyEvents,

        /// Presence extension response.
        #[cfg(feature = "unstable-msc4559")]
        #[serde(
            default,
            skip_serializing_if = "Presence::is_empty",
            rename = "org.continuwuity.presence_v2.msc4559.presence"
        )]
        pub presence: Presence,
    }

    impl Extensions {
        /// Whether the extension data is empty.
        ///
        /// True if neither to-device, e2ee nor account data are to be found.
        pub fn is_empty(&self) -> bool {
            #[allow(unused_mut)]
            let mut empty = self.to_device.is_none()
                && self.e2ee.is_empty()
                && self.account_data.is_empty()
                && self.receipts.is_empty()
                && self.typing.is_empty();

            #[cfg(feature = "unstable-msc4308")]
            {
                empty = empty && self.thread_subscriptions.is_empty();
            }

            #[cfg(feature = "unstable-msc4360")]
            {
                empty = empty && self.threads.is_empty();
            }

            #[cfg(feature = "unstable-msc4262")]
            {
                empty = empty && self.profiles.is_empty();
            }

            #[cfg(feature = "unstable-msc4480")]
            {
                empty = empty && self.sticky_events.is_empty();
            }

            #[cfg(feature = "unstable-msc4559")]
            {
                empty = empty && self.presence.is_empty();
            }

            empty
        }
    }

    /// The typing and receipts events are rebuilt from their content.
    impl From<v4::response::Extensions> for Extensions {
        fn from(value: v4::response::Extensions) -> Self {
            let v4::response::Extensions {
                to_device,
                e2ee,
                account_data,
                receipts,
                typing,
                #[cfg(feature = "unstable-msc4308")]
                thread_subscriptions,
                #[cfg(feature = "unstable-msc4360")]
                threads,
                #[cfg(feature = "unstable-msc4262")]
                profiles,
                #[cfg(feature = "unstable-msc4480")]
                sticky_events,
                #[cfg(feature = "unstable-msc4559")]
                presence,
            } = value;

            Self {
                to_device: to_device.map(|to_device| ToDevice {
                    next_batch: to_device.next_batch,
                    events: to_device.messages,
                }),
                e2ee: e2ee
                    .map(|e2ee| E2EE {
                        device_lists: e2ee.device_lists,
                        device_one_time_keys_count: e2ee.device_one_time_keys_count,
                        device_unused_fallback_key_types: Some(
                            e2ee.device_unused_fallback_key_types,
                        ),
                    })
                    .unwrap_or_default(),
                account_data,
                receipts: Receipts {
                    rooms: receipts
                        .rooms
                        .into_iter()
                        .map(|(room_id, content)| {
                            let event = to_raw_json_value(&json!({
                                "type": "m.receipt",
                                "content": content,
                            }))
                            .expect("serialized receipt event should be valid JSON");
                            (room_id, Raw::from_json(event))
                        })
                        .collect(),
                },
                typing: Typing {
                    rooms: typing
                        .rooms
                        .into_iter()
                        .map(|(room_id, content)| {
                            let event = Raw::new(&SyncTypingEvent::new(content))
                                .expect("serialized typing event should be valid JSON");
                            (room_id, event)
                        })
                        .collect(),
                },
                #[cfg(feature = "unstable-msc4308")]
                thread_subscriptions,
                #[cfg(feature = "unstable-msc4360")]
                threads,
                #[cfg(feature = "unstable-msc4262")]
                profiles,
                #[cfg(feature = "unstable-msc4480")]
                sticky_events,
                #[cfg(feature = "unstable-msc4559")]
                presence,
            }
        }
    }

    /// To-device extension response.
    ///
    /// According to [MSC3885](https://github.com/matrix-org/matrix-spec-proposals/pull/3885).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ToDevice {
        /// Fetch the next batch from this entry.
        pub next_batch: String,

        /// The to-device events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events: Vec<Raw<AnyToDeviceEvent>>,
    }

    /// E2EE extension response.
    ///
    /// According to [MSC3884](https://github.com/matrix-org/matrix-spec-proposals/pull/3884).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct E2EE {
        /// Information on E2EE device updates.
        #[serde(default, skip_serializing_if = "DeviceLists::is_empty")]
        pub device_lists: DeviceLists,

        /// For each key algorithm, the number of unclaimed one-time keys
        /// currently held on the server for a device.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub device_one_time_keys_count: BTreeMap<OneTimeKeyAlgorithm, UInt>,

        /// The unused fallback key algorithms.
        ///
        /// The presence of this field indicates that the server supports
        /// fallback keys.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_unused_fallback_key_types: Option<Vec<OneTimeKeyAlgorithm>>,
    }

    impl E2EE {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.device_lists.is_empty()
                && self.device_one_time_keys_count.is_empty()
                && self.device_unused_fallback_key_types.is_none()
        }
    }

    /// Receipt extension response.
    ///
    /// According to [MSC3960](https://github.com/matrix-org/matrix-spec-proposals/pull/3960)
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Receipts {
        /// The ephemeral receipt room event for each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, Raw<SyncReceiptEvent>>,
    }

    impl Receipts {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.rooms.is_empty()
        }
    }

    /// Typing extension response.
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/typing.go>
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Typing {
        /// The ephemeral typing event for each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, Raw<SyncTypingEvent>>,
    }

    impl Typing {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.rooms.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use ruma_common::{assert_to_canonical_json_eq, owned_room_id, user_id};
    use ruma_events::StateEventType;
    use serde_json::{from_value as from_json_value, json};

    use super::{Request, request, request::ExtensionRoomConfig, response, v4};

    #[test]
    fn serialize_request_extension_room_config() {
        let entry = ExtensionRoomConfig::AllSubscribed;
        assert_eq!(serde_json::to_string(&entry).unwrap().as_str(), r#""*""#);

        let entry = ExtensionRoomConfig::Room(owned_room_id!("!foo:bar.baz"));
        assert_eq!(serde_json::to_string(&entry).unwrap().as_str(), r#""!foo:bar.baz""#);
    }

    #[test]
    fn deserialize_request_extension_room_config() {
        assert_eq!(
            serde_json::from_str::<ExtensionRoomConfig>(r#""*""#).unwrap(),
            ExtensionRoomConfig::AllSubscribed
        );

        assert_eq!(
            serde_json::from_str::<ExtensionRoomConfig>(r#""!foo:bar.baz""#).unwrap(),
            ExtensionRoomConfig::Room(owned_room_id!("!foo:bar.baz"))
        );
    }

    #[test]
    fn request_into_v4() {
        let mut request = Request::new();
        request.txn_id = Some("txn".to_owned());
        request.lists.insert(
            "all".to_owned(),
            request::List {
                ranges: vec![(uint!(10), uint!(19)), (uint!(0), uint!(9))],
                room_details: request::RoomDetails {
                    required_state: vec![
                        ("*".into(), "*".to_owned()),
                        (StateEventType::RoomMember, "$LAZY".to_owned()),
                        (StateEventType::RoomMember, "$ME".to_owned()),
                        (StateEventType::RoomTopic, String::new()),
                    ],
                    timeline_limit: uint!(10),
                },
                filters: Some(request::ListFilters {
                    is_invite: Some(false),
                    ..Default::default()
                }),
            },
        );

        let mut request = request.into_v4(user_id!("@alice:example.com"));

        assert_to_canonical_json_eq!(
            request.lists.remove("all").unwrap(),
            json!({
                "range": [0, 19],
                "required_state": {
                    "include": [
                        {},
                        { "type": "m.room.member", "state_key": "@alice:example.com" },
                        { "type": "m.room.topic", "state_key": "" },
                    ],
                    "lazy_members": true,
                },
                "timeline_limit": 10,
                "filters": { "is_invited": false },
            })
        );
    }

    #[test]
    fn extensions_into_v4() {
        let mut request = Request::new();
        request.extensions.typing.enabled = Some(true);
        request.extensions.typing.rooms = Some(vec![
            ExtensionRoomConfig::AllSubscribed,
            ExtensionRoomConfig::Room(owned_room_id!("!a:example.com")),
        ]);
        request.extensions.receipts.enabled = Some(true);
        request.extensions.receipts.rooms =
            Some(vec![ExtensionRoomConfig::Room(owned_room_id!("!a:example.com"))]);

        let request = request.into_v4(user_id!("@alice:example.com"));

        assert_to_canonical_json_eq!(
            request.extensions,
            json!({
                "typing": { "enabled": true },
                "receipts": { "enabled": true, "rooms": ["!a:example.com"] },
            })
        );
    }

    #[test]
    fn extensions_from_v4() {
        let extensions: v4::response::Extensions = from_json_value(json!({
            "to_device": { "next_batch": "1", "messages": [] },
            "e2ee": { "device_one_time_keys_count": {}, "device_unused_fallback_key_types": [] },
            "receipts": {
                "rooms": {
                    "!a:example.com": {
                        "$e:example.com": { "m.read": { "@b:example.com": { "ts": 1 } } },
                    },
                },
            },
            "typing": { "rooms": { "!a:example.com": { "user_ids": ["@b:example.com"] } } },
        }))
        .unwrap();

        assert_to_canonical_json_eq!(
            response::Extensions::from(extensions),
            json!({
                "to_device": { "next_batch": "1" },
                "e2ee": { "device_unused_fallback_key_types": [] },
                "receipts": {
                    "rooms": {
                        "!a:example.com": {
                            "type": "m.receipt",
                            "content": {
                                "$e:example.com": { "m.read": { "@b:example.com": { "ts": 1 } } },
                            },
                        },
                    },
                },
                "typing": {
                    "rooms": {
                        "!a:example.com": {
                            "type": "m.typing",
                            "content": { "user_ids": ["@b:example.com"] },
                        },
                    },
                },
            })
        );
    }

    #[cfg(feature = "unstable-msc4480")]
    #[test]
    fn serialize_sticky_events_request() {
        let mut extensions = request::Extensions::default();
        extensions.sticky_events = request::StickyEvents {
            enabled: Some(true),
            rooms: Some(vec![ExtensionRoomConfig::AllSubscribed]),
            ..Default::default()
        };

        assert_to_canonical_json_eq!(
            extensions,
            json!({ "org.matrix.msc4354.sticky_events": { "enabled": true, "rooms": ["*"] } })
        );
    }

    #[test]
    fn room_from_v4() {
        let room: v4::response::Room = from_json_value(json!({
            "name": null,
            "initial": true,
            "membership": "join",
            "required_state": [
                {
                    "content": {},
                    "event_id": "$create:example.com",
                    "origin_server_ts": 1,
                    "sender": "@alice:example.com",
                    "state_key": "",
                    "type": "m.room.create",
                },
                { "type": "m.room.topic", "state_key": "" },
            ],
            "stripped_state": [],
        }))
        .unwrap();

        let room = response::Room::from(room);

        assert_eq!(room.name, None);
        assert_eq!(room.initial, Some(true));
        assert_eq!(room.required_state.len(), 1);
        assert_eq!(room.invite_state.unwrap().len(), 0);
    }

    #[test]
    #[cfg(feature = "unstable-compat-lax-syncv5-deser")]
    fn deserialize_room_ignores_invalid_string_fields() {
        use super::response::Room;

        let room: Room = serde_json::from_str(
            r#"{
                "name": {},
                "avatar": {},
                "heroes": [{ "user_id": "@alice:localhost", "displayname": {}, "avatar_url": {} }]
            }"#,
        )
        .unwrap();

        assert_eq!(room.name, None);
        assert!(room.avatar.is_undefined());
        let hero = &room.heroes.unwrap()[0];
        assert_eq!(hero.name, None);
        assert_eq!(hero.avatar, None);

        // Valid values are still kept.
        let room: Room = serde_json::from_str(
            r#"{ "name": "Room", "avatar": "mxc://localhost/a", "heroes": [{ "user_id": "@alice:localhost", "displayname": "Alice", "avatar_url": "mxc://localhost/b" }] }"#,
        )
        .unwrap();

        assert_eq!(room.name.as_deref(), Some("Room"));
        assert_eq!(room.avatar.into_option().unwrap().as_str(), "mxc://localhost/a");
        let hero = &room.heroes.unwrap()[0];
        assert_eq!(hero.name.as_deref(), Some("Alice"));
        assert_eq!(hero.avatar.as_ref().unwrap().as_str(), "mxc://localhost/b");
    }
}
