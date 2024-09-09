//! Types for MatrixRTC state events ([MSC3401]).
//!
//! This implements a newer/updated version of MSC3401.
//!
//! [MSC3401]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

mod focus;
mod member_data;

pub use focus::*;
pub use member_data::*;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use crate::{
    PossiblyRedactedStateEventContent, PrivOwnedStr, RedactContent, RedactedStateEventContent,
    StateEventType,
};

/// The member state event for a MatrixRTC session.
///
/// This is the object containing all the data related to a Matrix users participation in a
/// MatrixRTC session.
///
/// This is a unit struct with the enum [`CallMemberEventContent`] because a Ruma state event cannot
/// be an enum and we need this to be an untagged enum for parsing purposes. (see
/// [`CallMemberEventContent`])
///
/// This struct also exposes allows to call the methods from [`CallMemberEventContent`].
#[derive(Clone, Debug, Serialize, Deserialize, EventContent, PartialEq)]
#[ruma_event(type = "org.matrix.msc3401.call.member", kind = State, state_key_type = String, custom_redacted, custom_possibly_redacted)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum CallMemberEventContent {
    /// The legacy format for m.call.member events. (An array of memberships. The devices of one
    /// user.)
    LegacyContent(LegacyMembershipContent),
    /// Normal membership events. One event per membership. Multiple state keys will
    /// be used to describe multiple devices for one user.
    SessionContent(SessionMembershipData),
    /// An empty content means this user has been in a rtc session but is not anymore.
    Empty(EmptyMembershipData),
}

impl CallMemberEventContent {
    /// Creates a new [`CallMemberEventContent`] with [`LegacyMembershipData`].
    pub fn new_legacy(memberships: Vec<LegacyMembershipData>) -> Self {
        Self::LegacyContent(LegacyMembershipContent {
            memberships, //: memberships.into_iter().map(MembershipData::Legacy).collect(),
        })
    }

    /// Creates a new [`CallMemberEventContent`] with [`SessionMembershipData`].
    pub fn new(
        application: Application,
        device_id: String,
        focus_active: ActiveFocus,
        foci_preferred: Vec<Focus>,
        created_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        Self::SessionContent(SessionMembershipData {
            application,
            device_id,
            focus_active,
            foci_preferred,
            created_ts,
        })
    }

    /// Creates a new Empty [`CallMemberEventContent`] representing a left membership.
    pub fn new_empty(leave_reason: Option<LeaveReason>) -> Self {
        Self::Empty(EmptyMembershipData { leave_reason })
    }

    /// All non expired memberships in this member event.
    ///
    /// In most cases you want to use this method instead of the public memberships field.
    /// The memberships field will also include expired events.
    ///
    /// This copies all the memberships and converts them
    /// # Arguments
    ///
    /// * `origin_server_ts` - optionally the `origin_server_ts` can be passed as a fallback in the
    ///   Membership does not contain [`LegacyMembershipData::created_ts`]. (`origin_server_ts` will
    ///   be ignored if [`LegacyMembershipData::created_ts`] is `Some`)
    pub fn active_memberships(
        &self,
        origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Vec<MembershipData<'_>> {
        match self {
            CallMemberEventContent::LegacyContent(content) => {
                content.active_memberships(origin_server_ts)
            }
            CallMemberEventContent::SessionContent(content) => {
                [content].map(MembershipData::Session).to_vec()
            }
            CallMemberEventContent::Empty(_) => Vec::new(),
        }
    }

    /// All the memberships for this event. Can only contain multiple elements in the case of legacy
    /// `m.call.member` state events.
    pub fn memberships(&self) -> Vec<MembershipData<'_>> {
        match self {
            CallMemberEventContent::LegacyContent(content) => {
                content.memberships.iter().map(MembershipData::Legacy).collect()
            }
            CallMemberEventContent::SessionContent(content) => {
                [content].map(MembershipData::Session).to_vec()
            }
            CallMemberEventContent::Empty(_) => Vec::new(),
        }
    }

    /// Set the `created_ts` of each [`MembershipData::Legacy`] in this event.
    ///
    /// Each call member event contains the `origin_server_ts` and `content.create_ts`.
    /// `content.create_ts` is undefined for the initial event of a session (because the
    /// `origin_server_ts` is not known on the client).
    /// In the rust sdk we want to copy over the `origin_server_ts` of the event into the content.
    /// (This allows to use `MinimalStateEvents` and still be able to determine if a membership is
    /// expired)
    pub fn set_created_ts_if_none(&mut self, origin_server_ts: MilliSecondsSinceUnixEpoch) {
        match self {
            CallMemberEventContent::LegacyContent(content) => {
                content.memberships.iter_mut().for_each(|m: &mut LegacyMembershipData| {
                    m.created_ts.get_or_insert(origin_server_ts);
                });
            }
            CallMemberEventContent::SessionContent(m) => {
                m.created_ts.get_or_insert(origin_server_ts);
            }
            _ => (),
        }
    }
}

/// This describes the CallMember event if the user is not part of the current session.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct EmptyMembershipData {
    /// An empty call member state event can optionally contain a leave reason.
    /// If it is `None` the user has left the call ordinarily. (Intentional hangup)  
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leave_reason: Option<LeaveReason>,
}

/// This is the optional value for an empty membership event content:
/// [`CallMemberEventContent::Empty`].
///
/// It is used when the user disconnected and a Future ([MSC4140](https://github.com/matrix-org/matrix-spec-proposals/pull/4140))
/// was used to update the membership after the client was not reachable anymore.  
#[derive(Clone, PartialEq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_enum(rename_all = "m.snake_case")]
pub enum LeaveReason {
    /// The user left the call by losing network connection or closing  
    /// the client before it was able to send the leave event.
    LostConnection,
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl RedactContent for CallMemberEventContent {
    type Redacted = RedactedCallMemberEventContent;

    fn redact(self, _version: &ruma_common::RoomVersionId) -> Self::Redacted {
        RedactedCallMemberEventContent {}
    }
}

/// The PossiblyRedacted version of [`CallMemberEventContent`].
///
/// Since [`CallMemberEventContent`] has the [`CallMemberEventContent::Empty`] state it already is
/// compatible with the redacted version of the state event content.
pub type PossiblyRedactedCallMemberEventContent = CallMemberEventContent;

impl PossiblyRedactedStateEventContent for PossiblyRedactedCallMemberEventContent {
    type StateKey = String;
}

/// The Redacted version of [`CallMemberEventContent`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactedCallMemberEventContent {}

impl ruma_events::content::EventContent for RedactedCallMemberEventContent {
    type EventType = StateEventType;
    fn event_type(&self) -> Self::EventType {
        StateEventType::CallMember
    }
}

impl RedactedStateEventContent for RedactedCallMemberEventContent {
    type StateKey = String;
}

/// Legacy content with an array of memberships. See also: [`CallMemberEventContent`]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LegacyMembershipContent {
    /// A list of all the memberships that user currently has in this room.
    ///
    /// There can be multiple ones in case the user participates with multiple devices or there
    /// are multiple RTC applications running.
    ///
    /// e.g. a call and a spacial experience.
    ///
    /// Important: This includes expired memberships.
    /// To retrieve a list including only valid memberships,
    /// see [`active_memberships`](CallMemberEventContent::active_memberships).
    memberships: Vec<LegacyMembershipData>,
}

impl LegacyMembershipContent {
    fn active_memberships(
        &self,
        origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Vec<MembershipData<'_>> {
        self.memberships
            .iter()
            .filter(|m| !m.is_expired(origin_server_ts))
            .map(MembershipData::Legacy)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use assert_matches2::assert_matches;
    use ruma_common::{MilliSecondsSinceUnixEpoch as TS, OwnedEventId, OwnedRoomId, OwnedUserId};
    use serde_json::{from_value as from_json_value, json};

    use super::{
        focus::{ActiveFocus, ActiveLivekitFocus, Focus, LivekitFocus},
        member_data::{
            Application, CallApplicationContent, CallScope, LegacyMembershipData, MembershipData,
        },
        CallMemberEventContent,
    };
    use crate::{
        call::member::{EmptyMembershipData, FocusSelection, SessionMembershipData},
        AnyStateEvent, StateEvent,
    };

    fn create_call_member_legacy_event_content() -> CallMemberEventContent {
        CallMemberEventContent::new_legacy(vec![LegacyMembershipData {
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

    fn create_call_member_event_content() -> CallMemberEventContent {
        CallMemberEventContent::new(
            Application::Call(CallApplicationContent {
                call_id: "123456".to_owned(),
                scope: CallScope::Room,
            }),
            "ABCDE".to_owned(),
            ActiveFocus::Livekit(ActiveLivekitFocus {
                focus_selection: FocusSelection::OldestMembership,
            }),
            vec![Focus::Livekit(LivekitFocus {
                alias: "1".to_owned(),
                service_url: "https://livekit.com".to_owned(),
            })],
            None,
        )
    }

    #[test]
    fn serialize_call_member_event_content() {
        let call_member_event = &json!({
            "application": "m.call",
            "call_id": "123456",
            "scope": "m.room",
            "device_id": "ABCDE",
            "foci_preferred": [
                {
                    "livekit_alias": "1",
                    "livekit_service_url": "https://livekit.com",
                    "type": "livekit"
                }
            ],
            "focus_active":{
                "type":"livekit",
                "focus_selection":"oldest_membership"
            }
        });
        assert_eq!(
            call_member_event,
            &serde_json::to_value(create_call_member_event_content()).unwrap()
        );

        let empty_call_member_event = &json!({});
        assert_eq!(
            empty_call_member_event,
            &serde_json::to_value(CallMemberEventContent::Empty(EmptyMembershipData {
                leave_reason: None
            }))
            .unwrap()
        );
    }

    #[test]
    fn serialize_legacy_call_member_event_content() {
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
            &serde_json::to_value(create_call_member_legacy_event_content()).unwrap()
        );
    }
    #[test]
    fn deserialize_call_member_event_content() {
        let call_member_ev = CallMemberEventContent::new(
            Application::Call(CallApplicationContent {
                call_id: "123456".to_owned(),
                scope: CallScope::Room,
            }),
            "THIS_DEVICE".to_owned(),
            ActiveFocus::Livekit(ActiveLivekitFocus {
                focus_selection: FocusSelection::OldestMembership,
            }),
            vec![Focus::Livekit(LivekitFocus {
                alias: "room1".to_owned(),
                service_url: "https://livekit1.com".to_owned(),
            })],
            None,
        );

        let call_member_ev_json = json!({
            "application": "m.call",
            "call_id": "123456",
            "scope": "m.room",
            "device_id": "THIS_DEVICE",
            "focus_active":{
                "type": "livekit",
                "focus_selection": "oldest_membership"
            },
            "foci_preferred": [
                {
                    "livekit_alias": "room1",
                    "livekit_service_url": "https://livekit1.com",
                    "type": "livekit"
                }
            ],
        });

        let ev_content: CallMemberEventContent =
            serde_json::from_value(call_member_ev_json).unwrap();
        assert_eq!(
            serde_json::to_string(&ev_content).unwrap(),
            serde_json::to_string(&call_member_ev).unwrap()
        );
        let empty = CallMemberEventContent::Empty(EmptyMembershipData { leave_reason: None });
        assert_eq!(
            serde_json::to_string(&json!({})).unwrap(),
            serde_json::to_string(&empty).unwrap()
        );
    }

    #[test]
    fn deserialize_legacy_call_member_event_content() {
        let call_member_ev = CallMemberEventContent::new_legacy(vec![
            LegacyMembershipData {
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
            LegacyMembershipData {
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
        assert_eq!(
            serde_json::to_string(&ev_content).unwrap(),
            serde_json::to_string(&call_member_ev).unwrap()
        );
    }

    fn deserialize_member_event_helper(state_key: &str) {
        let ev = json!({
            "content":{
                "application": "m.call",
                "call_id": "",
                "scope": "m.room",
                "device_id": "THIS_DEVICE",
                "focus_active":{
                    "type": "livekit",
                    "focus_selection": "oldest_membership"
                },
                "foci_preferred": [
                    {
                        "livekit_alias": "room1",
                        "livekit_service_url": "https://livekit1.com",
                        "type": "livekit"
                    }
                ],
            },
            "type": "m.call.member",
            "origin_server_ts": 111,
            "event_id": "$3qfxjGYSu4sL25FtR0ep6vePOc",
            "room_id": "!1234:example.org",
            "sender": "@user:example.org",
            "state_key": state_key,
            "unsigned":{
                "age":10,
                "prev_content": {},
                "prev_sender":"@user:example.org",
            }
        });

        assert_matches!(
            from_json_value(ev),
            Ok(AnyStateEvent::CallMember(StateEvent::Original(member_event)))
        );

        let event_id = OwnedEventId::try_from("$3qfxjGYSu4sL25FtR0ep6vePOc").unwrap();
        let sender = OwnedUserId::try_from("@user:example.org").unwrap();
        let room_id = OwnedRoomId::try_from("!1234:example.org").unwrap();
        assert_eq!(member_event.state_key, state_key);
        assert_eq!(member_event.event_id, event_id);
        assert_eq!(member_event.sender, sender);
        assert_eq!(member_event.room_id, room_id);
        assert_eq!(member_event.origin_server_ts, TS(js_int::UInt::new(111).unwrap()));
        let membership = SessionMembershipData {
            application: Application::Call(CallApplicationContent {
                call_id: "".to_owned(),
                scope: CallScope::Room,
            }),
            device_id: "THIS_DEVICE".to_owned(),
            foci_preferred: [Focus::Livekit(LivekitFocus {
                alias: "room1".to_owned(),
                service_url: "https://livekit1.com".to_owned(),
            })]
            .to_vec(),
            focus_active: ActiveFocus::Livekit(ActiveLivekitFocus {
                focus_selection: FocusSelection::OldestMembership,
            }),
            created_ts: None,
        };
        assert_eq!(
            member_event.content,
            CallMemberEventContent::SessionContent(membership.clone())
        );

        // Correctly computes the active_memberships array.
        assert_eq!(
            member_event.content.active_memberships(None)[0],
            vec![MembershipData::Session(&membership)][0]
        );
        assert_eq!(js_int::Int::new(10), member_event.unsigned.age);
        assert_eq!(
            CallMemberEventContent::Empty(EmptyMembershipData { leave_reason: None }),
            member_event.unsigned.prev_content.unwrap()
        );

        // assert_eq!(, StateUnsigned { age: 10, transaction_id: None, prev_content:
        // CallMemberEventContent::Empty { leave_reason: None }, relations: None })
    }

    #[test]
    fn deserialize_member_event() {
        deserialize_member_event_helper("@user:example.org");
    }

    #[test]
    fn deserialize_member_event_with_scoped_state_key_prefixed() {
        deserialize_member_event_helper("_@user:example.org:THIS_DEVICE");
    }

    #[test]
    fn deserialize_member_event_with_scoped_state_key_unprefixed() {
        deserialize_member_event_helper("@user:example.org:THIS_DEVICE");
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
    fn legacy_memberships_do_expire() {
        let content_legacy = create_call_member_legacy_event_content();
        let (now, one_second_ago, two_hours_ago) = timestamps();

        assert_eq!(
            content_legacy.active_memberships(Some(one_second_ago)),
            content_legacy.memberships()
        );
        assert_eq!(content_legacy.active_memberships(Some(now)), content_legacy.memberships());
        assert_eq!(
            content_legacy.active_memberships(Some(two_hours_ago)),
            (vec![] as Vec<MembershipData<'_>>)
        );
        // session do never expire
        let content_session = create_call_member_event_content();
        assert_eq!(
            content_session.active_memberships(Some(one_second_ago)),
            content_session.memberships()
        );
        assert_eq!(content_session.active_memberships(Some(now)), content_session.memberships());
        assert_eq!(
            content_session.active_memberships(Some(two_hours_ago)),
            content_session.memberships()
        );
    }

    #[test]
    fn set_created_ts() {
        let mut content_now = create_call_member_legacy_event_content();
        let mut content_two_hours_ago = create_call_member_legacy_event_content();
        let mut content_one_second_ago = create_call_member_legacy_event_content();
        let (now, one_second_ago, two_hours_ago) = timestamps();

        content_now.set_created_ts_if_none(now);
        content_one_second_ago.set_created_ts_if_none(one_second_ago);
        content_two_hours_ago.set_created_ts_if_none(two_hours_ago);
        assert_eq!(content_now.active_memberships(None), content_now.memberships());

        assert_eq!(
            content_two_hours_ago.active_memberships(None),
            vec![] as Vec<MembershipData<'_>>
        );
        assert_eq!(
            content_one_second_ago.active_memberships(None),
            content_one_second_ago.memberships()
        );

        // created_ts should not be overwritten.
        content_two_hours_ago.set_created_ts_if_none(one_second_ago);
        // There still should be no active membership.
        assert_eq!(
            content_two_hours_ago.active_memberships(None),
            vec![] as Vec<MembershipData<'_>>
        );
    }
}
