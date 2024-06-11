//! Types for matrixRTC state events ([MSC3401]).
//!
//! This implements a newer/updated version of MSC3401.
//!
//! [MSC3401]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

mod focus;
mod member_data;
mod member_event;

pub use focus::*;
pub use member_data::*;
pub use member_event::*;
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use assert_matches2::assert_matches;
    use ruma_common::{MilliSecondsSinceUnixEpoch as TS, OwnedEventId, OwnedRoomId, OwnedUserId};
    use ruma_events::{
        call::notify::{ApplicationType, CallNotifyEventContent, NotifyType},
        Mentions,
    };
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{
        focus::{ActiveFocus, ActiveLivekitFocus, Focus, LivekitFocus},
        member_data::{
            Application, CallApplicationContent, CallScope, LegacyMembershipData, MembershipData,
        },
        member_event::CallMemberEventContent,
    };
    use crate::{
        call::member::{FocusSelection, SessionMembershipData},
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
                focus_select: FocusSelection::OldestMembership,
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
                "focus_select":"oldest_membership"
            }
        });
        assert_eq!(
            call_member_event,
            &serde_json::to_value(create_call_member_event_content()).unwrap()
        );

        let empty_call_member_event = &json!({});
        assert_eq!(
            empty_call_member_event,
            &serde_json::to_value(CallMemberEventContent::Empty { leave_reason: None }).unwrap()
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
                focus_select: FocusSelection::OldestMembership,
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
                "focus_select": "oldest_membership"
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
        let empty = CallMemberEventContent::Empty { leave_reason: None };
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
    #[test]
    fn deserialize_member_event() {
        let ev = json!({
            "content":{
                "application": "m.call",
                "call_id": "",
                "scope": "m.room",
                "device_id": "THIS_DEVICE",
                "focus_active":{
                    "type": "livekit",
                    "focus_select": "oldest_membership"
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
            "state_key":"@user:example.org",
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
        assert_eq!(member_event.state_key, sender);
        assert_eq!(member_event.event_id, event_id);
        assert_eq!(member_event.sender, sender);
        assert_eq!(member_event.room_id, room_id);
        assert_eq!(member_event.origin_server_ts, TS(js_int::UInt::new(111).unwrap()));
        assert_eq!(
            member_event.content,
            CallMemberEventContent::SessionContent(SessionMembershipData {
                application: Application::Call(CallApplicationContent {
                    call_id: "".to_string(),
                    scope: CallScope::Room
                }),
                device_id: "THIS_DEVICE".to_owned(),
                foci_preferred: [Focus::Livekit(LivekitFocus {
                    alias: "room1".to_owned(),
                    service_url: "https://livekit1.com".to_owned()
                })]
                .to_vec(),
                focus_active: ActiveFocus::Livekit(ActiveLivekitFocus {
                    focus_select: FocusSelection::OldestMembership
                }),
                created_ts: None
            })
        );

        assert_eq!(js_int::Int::new(10), member_event.unsigned.age);
        assert_eq!(
            CallMemberEventContent::Empty { leave_reason: None },
            member_event.unsigned.prev_content.unwrap().0
        );

        // assert_eq!(, StateUnsigned { age: 10, transaction_id: None, prev_content:
        // CallMemberEventContent::Empty { leave_reason: None }, relations: None })
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
    fn memberships_do_expire() {
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

    #[cfg(feature = "unstable-msc4075")]
    #[test]
    fn notify_event_serialization() {
        use ruma_common::owned_user_id;

        let content_user_mention = CallNotifyEventContent::new(
            "abcdef".into(),
            ApplicationType::Call,
            NotifyType::Ring,
            Mentions::with_user_ids(vec![
                owned_user_id!("@user:example.com"),
                owned_user_id!("@user2:example.com"),
            ]),
        );

        let content_room_mention = CallNotifyEventContent::new(
            "abcdef".into(),
            ApplicationType::Call,
            NotifyType::Ring,
            Mentions::with_room_mention(),
        );

        assert_eq!(
            to_json_value(&content_user_mention).unwrap(),
            json!({
                "call_id": "abcdef",
                "application": "m.call",
                "m.mentions": {
                    "user_ids": ["@user2:example.com","@user:example.com"],
                },
                "notify_type": "ring",
            })
        );
        assert_eq!(
            to_json_value(&content_room_mention).unwrap(),
            json!({
                "call_id": "abcdef",
                "application": "m.call",
                "m.mentions": { "room": true },
                "notify_type": "ring",
            })
        );
    }

    #[cfg(feature = "unstable-msc4075")]
    #[test]
    fn notify_event_deserialization() {
        use std::collections::BTreeSet;

        use assert_matches2::assert_matches;
        use ruma_common::owned_user_id;

        use crate::{AnyMessageLikeEvent, MessageLikeEvent};

        let json_data = json!({
            "content": {
                "call_id": "abcdef",
                "application": "m.call",
                "m.mentions": {
                    "room": false,
                    "user_ids": ["@user:example.com", "@user2:example.com"],
                },
                "notify_type": "ring",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.call.notify",
        });

        let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
        assert_matches!(
            event,
            AnyMessageLikeEvent::CallNotify(MessageLikeEvent::Original(message_event))
        );
        let content = message_event.content;
        assert_eq!(content.call_id, "abcdef");
        assert!(!content.mentions.room);
        assert_eq!(
            content.mentions.user_ids,
            BTreeSet::from([
                owned_user_id!("@user:example.com"),
                owned_user_id!("@user2:example.com")
            ])
        );
    }
}
