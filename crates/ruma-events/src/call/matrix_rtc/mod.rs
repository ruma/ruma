//! Types for matrixRTC state events ([MSC3401]).
//!
//! This implements a newer/updated version of MSC3401.
//!
//! [MSC3401]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401
pub mod focus;
pub mod member_data;
pub mod member_event;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ruma_common::MilliSecondsSinceUnixEpoch as TS;
    use serde_json::json;

    use super::{
        focus::{ActiveFocus, ActiveLivekitFocus, Focus, LivekitFocus},
        member_data::{
            Application, CallApplicationContent, CallScope, LegacyMembershipData, MembershipData,
        },
        member_event::MemberEventContent,
    };

    fn create_call_member_legacy_event_content() -> MemberEventContent {
        MemberEventContent::new_legacy(vec![LegacyMembershipData {
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

    fn create_call_member_event_content() -> MemberEventContent {
        MemberEventContent::new(
            Application::Call(CallApplicationContent {
                call_id: "123456".to_owned(),
                scope: CallScope::Room,
            }),
            "ABCDE".to_owned(),
            ActiveFocus::Livekit(ActiveLivekitFocus {
                focus_select: super::focus::FocusSelection::OldestMembership,
            }),
            vec![Focus::Livekit(LivekitFocus {
                alias: "1".to_owned(),
                service_url: "https://livekit.com".to_owned(),
            })],
        )
    }

    #[test]
    fn serialize_call_member_event_content() {
        let call_member_event = &json!(
                {
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
                }
        );
        assert_eq!(
            call_member_event,
            &serde_json::to_value(create_call_member_event_content()).unwrap()
        );

        let empty_call_member_event = &json!({});
        assert_eq!(
            empty_call_member_event,
            &serde_json::to_value(MemberEventContent::Empty {}).unwrap()
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
        let call_member_ev: MemberEventContent = MemberEventContent::new(
            Application::Call(CallApplicationContent {
                call_id: "123456".to_owned(),
                scope: CallScope::Room,
            }),
            "THIS_DEVICE".to_owned(),
            ActiveFocus::Livekit(ActiveLivekitFocus {
                focus_select: super::focus::FocusSelection::OldestMembership,
            }),
            vec![Focus::Livekit(LivekitFocus {
                alias: "room1".to_owned(),
                service_url: "https://livekit1.com".to_owned(),
            })],
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
                    "membershipID": "0",
                }
        );

        let ev_content: MemberEventContent = serde_json::from_value(call_member_ev_json).unwrap();
        assert_eq!(
            serde_json::to_string(&ev_content).unwrap(),
            serde_json::to_string(&call_member_ev).unwrap()
        );
        let empty = MemberEventContent::Empty {};
        assert_eq!(
            serde_json::to_string(&json!({})).unwrap(),
            serde_json::to_string(&empty).unwrap()
        );
    }
    #[test]
    fn deserialize_legacy_call_member_event_content() {
        let call_member_ev: MemberEventContent = MemberEventContent::new_legacy(vec![
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

        let ev_content: MemberEventContent = serde_json::from_value(call_member_ev_json).unwrap();
        assert_eq!(
            serde_json::to_string(&ev_content).unwrap(),
            serde_json::to_string(&call_member_ev).unwrap()
        );
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
        let content = create_call_member_legacy_event_content();
        let (now, one_second_ago, two_hours_ago) = timestamps();

        assert_eq!(content.active_memberships(Some(one_second_ago)), content.memberships());
        assert_eq!(content.active_memberships(Some(now)), content.memberships());
        assert_eq!(
            content.active_memberships(Some(two_hours_ago)),
            (vec![] as Vec<&MembershipData>)
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

        assert_eq!(content_two_hours_ago.active_memberships(None), vec![] as Vec<&MembershipData>);
        assert_eq!(
            content_one_second_ago.active_memberships(None),
            content_one_second_ago.memberships()
        );

        // created_ts should not be overwritten.
        content_two_hours_ago.set_created_ts_if_none(one_second_ago);
        // There still should be no active membership.
        assert_eq!(content_two_hours_ago.active_memberships(None), vec![] as Vec<&MembershipData>);
    }
}
