//! Types for matrixRTC state events ([MSC3401]).
//!
//! This implements a newer/updated version of MSC3401.
//!
//! [MSC3401]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

use std::time::Duration;

use as_variant::as_variant;
use ruma_common::{serde::StringEnum, MilliSecondsSinceUnixEpoch, OwnedUserId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::PrivOwnedStr;

/// The member state event for a matrixRTC session.
///
/// This is the object containing all the data related to a matrix users participation in a
/// matrixRTC session. It consists of memberships / sessions.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3401.call.member", kind = State, state_key_type = OwnedUserId)]
pub struct CallMemberEventContent {
    /// A list of all the memberships that user currently has in this room.
    ///
    /// There can be multiple ones in cases the user participates with multiple devices or there
    /// are multiple RTC applications running.
    ///
    /// e.g. a call and a spacial experience.
    ///
    /// Important: This includes expired memberships.
    /// To retrieve a list including only valid memberships,
    /// see [`active_memberships`](CallMemberEventContent::active_memberships).
    pub memberships: Vec<Membership>,
}

impl CallMemberEventContent {
    /// Creates a new `CallMemberEventContent`.
    pub fn new(memberships: Vec<Membership>) -> Self {
        Self { memberships }
    }

    /// All non expired memberships in this member event.
    ///
    /// In most cases you want tu use this method instead of the public memberships field.
    /// The memberships field will also include expired events.
    ///
    /// # Arguments
    ///
    /// * `origin_server_ts` - optionally the `origin_server_ts` can be passed as a fallback in case
    ///   the Membership does not contain `created_ts`. (`origin_server_ts` will be ignored if
    ///   `created_ts` is `Some`)
    pub fn active_memberships(
        &self,
        origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Vec<&Membership> {
        self.memberships.iter().filter(|m| !m.is_expired(origin_server_ts)).collect()
    }

    /// Set the `created_ts` of each [Membership] in this event.
    ///
    /// Each call member event contains the `origin_server_ts` and `content.create_ts`.
    /// `content.create_ts` is undefined for the initial event of a session (because the
    /// `origin_server_ts` is not known on the client).
    /// In the rust sdk we want to copy over the `origin_server_ts` of the event into the content.
    /// (This allows to use `MinimalStateEvents` and still be able to determine if a membership is
    /// expired)
    pub fn set_created_ts_if_none(&mut self, origin_server_ts: MilliSecondsSinceUnixEpoch) {
        self.memberships.iter_mut().for_each(|m| {
            m.created_ts.get_or_insert(origin_server_ts);
        });
    }
}

/// A membership describes one of the sessions this user currently partakes.
///
/// The application defines the type of the session.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Membership {
    /// The type of the matrixRTC session the membership belongs to.
    ///
    /// e.g. call, spacial, document...
    #[serde(flatten)]
    pub application: Application,

    /// The device id of this membership.
    ///
    /// The same user can join with their phone/computer.
    pub device_id: String,

    /// The duration in milliseconds relative to the time this membership joined
    /// during which the membership is valid.
    ///
    /// The time a member has joined is defined as:
    /// `MIN(content.created_ts, event.origin_server_ts)`
    #[serde(with = "ruma_common::serde::duration::ms")]
    pub expires: Duration,

    /// Stores a copy of the `origin_server_ts` of the initial session event.
    ///
    /// If the membership is updated this field will be used to track to
    /// original `origin_server_ts`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ts: Option<MilliSecondsSinceUnixEpoch>,

    /// A list of the foci in use for this membership.
    pub foci_active: Vec<Focus>,

    /// The id of the membership.
    ///
    /// This is required to guarantee uniqueness of the event.
    /// Sending the same state event twice to synapse makes the HS drop the second one and return
    /// 200.
    #[serde(rename = "membershipID")]
    pub membership_id: String,
}

impl Membership {
    /// The application of the membership is "m.call" and the scope is "m.room".
    pub fn is_room_call(&self) -> bool {
        as_variant!(&self.application, Application::Call)
            .is_some_and(|call| call.scope == CallScope::Room)
    }

    /// The application of the membership is "m.call".
    pub fn is_call(&self) -> bool {
        as_variant!(&self.application, Application::Call).is_some()
    }

    /// Checks if the event is expired.
    ///
    /// Defaults to using `created_ts` of the `Membership`.
    /// If no `origin_server_ts` is provided and the event does not contain `created_ts`
    /// the event will be considered as not expired.
    /// In this case, a warning will be logged.
    ///
    /// # Arguments
    ///
    /// * `origin_server_ts` - a fallback if `created_ts` is not present
    pub fn is_expired(&self, origin_server_ts: Option<MilliSecondsSinceUnixEpoch>) -> bool {
        let ev_created_ts = self.created_ts.or(origin_server_ts);

        if let Some(ev_created_ts) = ev_created_ts {
            let now = MilliSecondsSinceUnixEpoch::now().to_system_time();
            let expire_ts = ev_created_ts.to_system_time().map(|t| t + self.expires);
            now > expire_ts
        } else {
            // This should not be reached since we only allow events that have copied over
            // the origin server ts. `set_created_ts_if_none`
            warn!("Encountered a Call Member state event where the origin_ts (or origin_server_ts) could not be found.\
            It is treated as a non expired event but this might be wrong.");
            false
        }
    }
}

/// Initial set of fields of [`Membership`].
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct MembershipInit {
    /// The type of the matrixRTC session the membership belongs to.
    ///
    /// e.g. call, spacial, document...
    pub application: Application,

    /// The device id of this membership.
    ///
    /// The same user can join with their phone/computer.
    pub device_id: String,

    /// The duration in milliseconds relative to the time this membership joined
    /// during which the membership is valid.
    ///
    /// The time a member has joined is defined as:
    /// `MIN(content.created_ts, event.origin_server_ts)`
    pub expires: Duration,

    /// A list of the focuses (foci) in use for this membership.
    pub foci_active: Vec<Focus>,

    /// The id of the membership.
    ///
    /// This is required to guarantee uniqueness of the event.
    /// Sending the same state event twice to synapse makes the HS drop the second one and return
    /// 200.
    pub membership_id: String,
}

impl From<MembershipInit> for Membership {
    fn from(init: MembershipInit) -> Self {
        let MembershipInit { application, device_id, expires, foci_active, membership_id } = init;
        Self { application, device_id, expires, created_ts: None, foci_active, membership_id }
    }
}

/// Description of the SFU/Focus a membership can be connected to.
///
/// A focus can be any server powering the matrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Focus {
    /// Livekit is one possible type of SFU/Focus that can be used for a matrixRTC session.
    Livekit(LivekitFocus),
}

/// The fields to describe livekit as an `active_foci`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LivekitFocus {
    /// The alias where the livekit sessions can be reached.
    #[serde(rename = "livekit_alias")]
    pub alias: String,

    /// The url of the jwt server for the livekit instance.
    #[serde(rename = "livekit_service_url")]
    pub service_url: String,
}

impl LivekitFocus {
    /// Initialize a [`LivekitFocus`].
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias where the livekit sessions can be reached.
    /// * `service_url` - The url of the jwt server for the livekit instance.
    pub fn new(alias: String, service_url: String) -> Self {
        Self { alias, service_url }
    }
}

/// The type of the matrixRTC session.
///
/// This is not the application/client used by the user but the
/// type of matrixRTC session e.g. calling (`m.call`), third-room, whiteboard could be
/// possible applications.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "application")]
pub enum Application {
    #[serde(rename = "m.call")]
    /// A VoIP call.
    Call(CallApplicationContent),
}

/// Call specific parameters membership parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CallApplicationContent {
    /// An identifier for calls.
    ///
    /// All members using the same `call_id` will end up in the same call.
    ///
    /// Does not need to be a uuid.
    ///
    /// `""` is used for room scoped calls.
    pub call_id: String,

    /// Who owns/joins/controls (can modify) the call.
    pub scope: CallScope,
}

impl CallApplicationContent {
    /// Initialize a [`CallApplicationContent`].
    ///
    /// # Arguments
    ///
    /// * `call_id` - An identifier for calls. All members using the same `call_id` will end up in
    ///   the same call. Does not need to be a uuid. `""` is used for room scoped calls.
    /// * `scope` - Who owns/joins/controls (can modify) the call.
    pub fn new(call_id: String, scope: CallScope) -> Self {
        Self { call_id, scope }
    }
}

/// The call scope defines different call ownership models.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_enum(rename_all = "m.snake_case")]
pub enum CallScope {
    /// A call which every user of a room can join and create.
    ///
    /// There is no particular name associated with it.
    ///
    /// There can only be one per room.
    Room,

    /// A user call is owned by a user.
    ///
    /// Each user can create one there can be multiple per room. They are started and ended by the
    /// owning user.
    User,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ruma_common::MilliSecondsSinceUnixEpoch as TS;
    use serde_json::json;

    use super::{
        Application, CallApplicationContent, CallMemberEventContent, CallScope, Focus,
        LivekitFocus, Membership,
    };

    fn create_call_member_event_content() -> CallMemberEventContent {
        CallMemberEventContent::new(vec![Membership {
            application: Application::Call(CallApplicationContent {
                call_id: "123456".to_owned(),
                scope: CallScope::Room,
            }),
            device_id: "ABCDE".to_owned(),
            expires: Duration::from_secs(3600),
            foci_active: vec![Focus::Livekit(LivekitFocus {
                alias: "1".to_owned(),
                service_url: "https://livekit.com".to_owned(),
            })],
            membership_id: "0".to_owned(),
            created_ts: None,
        }])
    }

    #[test]
    fn serialize_call_member_event_content() {
        let call_member_event = &json!({
            "memberships": [
                {
                    "application": "m.call",
                    "call_id": "123456",
                    "scope": "m.room",
                    "device_id": "ABCDE",
                    "expires": 3_600_000,
                    "foci_active": [
                        {
                            "livekit_alias": "1",
                            "livekit_service_url": "https://livekit.com",
                            "type": "livekit"
                        }
                    ],
                    "membershipID": "0"
                }
            ]
        });

        assert_eq!(
            call_member_event,
            &serde_json::to_value(create_call_member_event_content()).unwrap()
        );
    }

    #[test]
    fn deserialize_call_member_event_content() {
        let call_member_ev: CallMemberEventContent = CallMemberEventContent::new(vec![
            Membership {
                application: Application::Call(CallApplicationContent {
                    call_id: "123456".to_owned(),
                    scope: CallScope::Room,
                }),
                device_id: "THIS_DEVICE".to_owned(),
                expires: Duration::from_secs(3600),
                foci_active: vec![Focus::Livekit(LivekitFocus {
                    alias: "room1".to_owned(),
                    service_url: "https://livekit1.com".to_owned(),
                })],
                membership_id: "0".to_owned(),
                created_ts: None,
            },
            Membership {
                application: Application::Call(CallApplicationContent {
                    call_id: "".to_owned(),
                    scope: CallScope::Room,
                }),
                device_id: "OTHER_DEVICE".to_owned(),
                expires: Duration::from_secs(3600),
                foci_active: vec![Focus::Livekit(LivekitFocus {
                    alias: "room2".to_owned(),
                    service_url: "https://livekit2.com".to_owned(),
                })],
                membership_id: "0".to_owned(),
                created_ts: None,
            },
        ]);

        let call_member_ev_json = json!({
            "memberships": [
                {
                    "application": "m.call",
                    "call_id": "123456",
                    "scope": "m.room",
                    "device_id": "THIS_DEVICE",
                    "expires": 3_600_000,
                    "foci_active": [
                        {
                            "livekit_alias": "room1",
                            "livekit_service_url": "https://livekit1.com",
                            "type": "livekit"
                        }
                    ],
                    "membershipID": "0",
                },
                {
                    "application": "m.call",
                    "call_id": "",
                    "scope": "m.room",
                    "device_id": "OTHER_DEVICE",
                    "expires": 3_600_000,
                    "foci_active": [
                        {
                            "livekit_alias": "room2",
                            "livekit_service_url": "https://livekit2.com",
                            "type": "livekit"
                        }
                    ],
                    "membershipID": "0"
                }
            ]
        });

        let ev_content: CallMemberEventContent =
            serde_json::from_value(call_member_ev_json).unwrap();
        assert_eq!(ev_content, call_member_ev);
    }

    fn timestamps() -> (TS, TS, TS) {
        let now = TS::now();
        let one_second_ago =
            now.to_system_time().unwrap().checked_sub(Duration::from_secs(1)).unwrap();
        let two_hours_ago =
            now.to_system_time().unwrap().checked_sub(Duration::from_secs(60 * 60 * 2)).unwrap();
        (
            now,
            TS::from_system_time(one_second_ago).unwrap(),
            TS::from_system_time(two_hours_ago).unwrap(),
        )
    }

    #[test]
    fn membership_do_expire() {
        let content = create_call_member_event_content();
        let (now, one_second_ago, two_hours_ago) = timestamps();
        assert_eq!(
            content.active_memberships(Some(one_second_ago)),
            content.memberships.iter().collect::<Vec<&Membership>>()
        );
        assert_eq!(
            content.active_memberships(Some(now)),
            content.memberships.iter().collect::<Vec<&Membership>>()
        );
        assert_eq!(content.active_memberships(Some(two_hours_ago)), vec![] as Vec<&Membership>);
    }

    #[test]
    fn set_created_ts() {
        let mut content_now = create_call_member_event_content();
        let mut content_two_hours_ago = create_call_member_event_content();
        let mut content_one_second_ago = create_call_member_event_content();
        let (now, one_second_ago, two_hours_ago) = timestamps();

        content_now.set_created_ts_if_none(now);
        content_one_second_ago.set_created_ts_if_none(one_second_ago);
        content_two_hours_ago.set_created_ts_if_none(two_hours_ago);
        assert_eq!(
            content_now.active_memberships(None),
            content_now.memberships.iter().collect::<Vec<&Membership>>()
        );

        assert_eq!(content_two_hours_ago.active_memberships(None), vec![] as Vec<&Membership>);
        assert_eq!(
            content_one_second_ago.active_memberships(None),
            content_one_second_ago.memberships.iter().collect::<Vec<&Membership>>()
        );

        // created_ts should not be overwritten.
        content_two_hours_ago.set_created_ts_if_none(one_second_ago);
        // There still should be no active membership.
        assert_eq!(content_two_hours_ago.active_memberships(None), vec![] as Vec<&Membership>);
    }
}
