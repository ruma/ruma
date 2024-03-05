#[cfg(feature = "unstable-msc4075")]
use std::collections::BTreeSet;

use assert_matches2::assert_matches;
#[cfg(feature = "unstable-msc2747")]
use assign::assign;
use js_int::uint;
use ruma_common::{room_id, serde::CanBeEmpty, MilliSecondsSinceUnixEpoch, VoipVersionId};
#[cfg(feature = "unstable-msc2747")]
use ruma_events::call::CallCapabilities;
#[cfg(feature = "unstable-msc4075")]
use ruma_events::{
    call::notify::{ApplicationType, CallNotifyEventContent, NotifyType},
    Mentions,
};
use ruma_events::{
    call::{
        answer::CallAnswerEventContent,
        candidates::{CallCandidatesEventContent, Candidate},
        hangup::{CallHangupEventContent, Reason},
        invite::CallInviteEventContent,
        negotiate::CallNegotiateEventContent,
        reject::CallRejectEventContent,
        select_answer::CallSelectAnswerEventContent,
        SessionDescription,
    },
    AnyMessageLikeEvent, AnySyncMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn answer_v0_content_serialization() {
    let event_content = CallAnswerEventContent::version_0(
        SessionDescription::new("answer".to_owned(), "not a real sdp".to_owned()),
        "abcdef".into(),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "call_id": "abcdef",
            "version": 0,
            "answer": {
                "type": "answer",
                "sdp": "not a real sdp",
            },
        })
    );
}

#[test]
fn answer_v0_content_deserialization() {
    let json_data = json!({
        "answer": {
            "type": "answer",
            "sdp": "Hello"
        },
        "call_id": "foofoo",
        "version": 0
    });

    let content = from_json_value::<CallAnswerEventContent>(json_data).unwrap();

    assert_eq!(content.answer.session_type, "answer");
    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    assert_eq!(content.version, VoipVersionId::V0);
}

#[test]
fn answer_v0_event_deserialization() {
    let json_data = json!({
        "content": {
            "answer": {
                "type": "answer",
                "sdp": "Hello"
            },
            "call_id": "foofoo",
            "version": 0
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.call.answer"
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event))
    );
    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.answer.session_type, "answer");
    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    assert_eq!(content.version, VoipVersionId::V0);
}

#[test]
fn answer_v0_event_deserialization_then_convert_to_full() {
    let rid = room_id!("!roomid:room.com");
    let json_data = json!({
        "content": {
            "answer": {
                "type": "answer",
                "sdp": "Hello",
            },
            "call_id": "foofoo",
            "version": 0,
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "type": "m.call.answer",
    });

    let sync_ev: AnySyncMessageLikeEvent = from_json_value(json_data).unwrap();

    assert_matches!(
        sync_ev.into_full_event(rid.to_owned()),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event))
    );
    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    assert_eq!(content.version, VoipVersionId::V0);
}

#[test]
fn invite_v0_content_serialization() {
    let event_content = CallInviteEventContent::version_0(
        "abcdef".into(),
        uint!(30000),
        SessionDescription::new("offer".to_owned(), "not a real sdp".to_owned()),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "call_id": "abcdef",
            "lifetime": 30000,
            "version": 0,
            "offer": {
                "type": "offer",
                "sdp": "not a real sdp",
            },
        })
    );
}

#[test]
fn candidates_v0_content_serialization() {
    let event_content = CallCandidatesEventContent::version_0(
        "abcdef".into(),
        vec![Candidate::version_0("not a real candidate".to_owned(), "0".to_owned(), uint!(0))],
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "call_id": "abcdef",
            "version": 0,
            "candidates": [
                {
                    "candidate": "not a real candidate",
                    "sdpMid": "0",
                    "sdpMLineIndex": 0,
                },
            ],
        })
    );
}

#[test]
fn hangup_v0_content_serialization() {
    let event_content = CallHangupEventContent::version_0("abcdef".into());

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "call_id": "abcdef",
            "version": 0,
            "reason": "user_hangup",
        })
    );
}

#[test]
fn invite_v1_event_serialization() {
    let content = CallInviteEventContent::version_1(
        "abcdef".into(),
        "9876".into(),
        uint!(60000),
        SessionDescription::new("offer".to_owned(), "not a real sdp".to_owned()),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "lifetime": 60000,
            "version": "1",
            "offer": {
                "type": "offer",
                "sdp": "not a real sdp",
            },
        })
    );
}

#[test]
fn invite_v1_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "lifetime": 60000,
            "version": "1",
            "offer": {
                "type": "offer",
                "sdp": "not a real sdp",
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.invite",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::CallInvite(MessageLikeEvent::Original(message_event))
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.lifetime, uint!(60000));
    assert_eq!(content.version, VoipVersionId::V1);
    assert_eq!(content.offer.session_type, "offer");
    assert_eq!(content.offer.sdp, "not a real sdp");
}

#[test]
fn answer_v1_event_serialization() {
    let content = CallAnswerEventContent::version_1(
        SessionDescription::new("answer".to_owned(), "not a real sdp".to_owned()),
        "abcdef".into(),
        "9876".into(),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
            "answer": {
                "type": "answer",
                "sdp": "not a real sdp",
            },
        })
    );
}

#[cfg(feature = "unstable-msc2747")]
#[test]
fn answer_v1_event_capabilities_serialization() {
    let content = assign!(
        CallAnswerEventContent::version_1(
            SessionDescription::new("answer".to_owned(), "not a real sdp".to_owned()),
            "abcdef".into(),
            "9876".into()
        ),
        {
            capabilities: assign!(CallCapabilities::new(), { dtmf: true }),
        }
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
            "answer": {
                "type": "answer",
                "sdp": "not a real sdp",
            },
            "capabilities": {
                "m.call.dtmf": true,
            },
        })
    );
}

#[test]
fn answer_unknown_version_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "org.matrix.1b",
            "answer": {
                "type": "answer",
                "sdp": "not a real sdp",
            },
            "capabilities": {
                "m.call.dtmf": true,
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.answer",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event))
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.version.as_ref(), "org.matrix.1b");
    assert_eq!(content.answer.session_type, "answer");
    assert_eq!(content.answer.sdp, "not a real sdp");
    #[cfg(feature = "unstable-msc2747")]
    assert!(content.capabilities.dtmf);
}

#[test]
fn candidates_v1_event_serialization() {
    let content = CallCandidatesEventContent::version_1(
        "abcdef".into(),
        "9876".into(),
        vec![
            Candidate::version_0("not a real candidate".to_owned(), "0".to_owned(), uint!(0)),
            Candidate::version_0("another fake candidate".to_owned(), "0".to_owned(), uint!(1)),
            Candidate::new("".to_owned()),
        ],
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
            "candidates": [
                {
                    "candidate": "not a real candidate",
                    "sdpMid": "0",
                    "sdpMLineIndex": 0,
                },
                {
                    "candidate": "another fake candidate",
                    "sdpMid": "0",
                    "sdpMLineIndex": 1,
                },
                {
                    "candidate": "",
                },
            ],
        })
    );
}

#[test]
fn candidates_v1_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
            "candidates": [
                {
                    "candidate": "not a real candidate",
                    "sdpMid": "0",
                    "sdpMLineIndex": 0,
                },
                {
                    "candidate": "another fake candidate",
                    "sdpMid": "0",
                    "sdpMLineIndex": 1,
                },
                {
                    "candidate": "",
                },
            ],
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.candidates",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::CallCandidates(MessageLikeEvent::Original(message_event))
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.version, VoipVersionId::V1);
    assert_eq!(content.candidates.len(), 3);
    assert_eq!(content.candidates[0].candidate, "not a real candidate");
    assert_eq!(content.candidates[0].sdp_mid.as_deref(), Some("0"));
    assert_eq!(content.candidates[0].sdp_m_line_index, Some(uint!(0)));
    assert_eq!(content.candidates[1].candidate, "another fake candidate");
    assert_eq!(content.candidates[1].sdp_mid.as_deref(), Some("0"));
    assert_eq!(content.candidates[1].sdp_m_line_index, Some(uint!(1)));
    assert_eq!(content.candidates[2].candidate, "");
    assert_eq!(content.candidates[2].sdp_mid, None);
    assert_eq!(content.candidates[2].sdp_m_line_index, None);
}

#[test]
fn hangup_v1_event_serialization() {
    let content =
        CallHangupEventContent::version_1("abcdef".into(), "9876".into(), Reason::IceFailed);

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
            "reason": "ice_failed",
        })
    );
}

#[test]
fn hangup_v1_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.hangup",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::CallHangup(MessageLikeEvent::Original(message_event))
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.version, VoipVersionId::V1);
    assert_eq!(content.reason, Reason::UserHangup);
}

#[test]
fn negotiate_v1_event_serialization() {
    let content = CallNegotiateEventContent::version_1(
        "abcdef".into(),
        "9876".into(),
        uint!(30000),
        SessionDescription::new("offer".to_owned(), "not a real sdp".to_owned()),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
            "lifetime": 30000,
            "description": {
                "type": "offer",
                "sdp": "not a real sdp",
            },
        })
    );
}

#[test]
fn negotiate_v1_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
            "lifetime": 30000,
            "description": {
                "type": "answer",
                "sdp": "not a real sdp",
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.negotiate",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::CallNegotiate(MessageLikeEvent::Original(message_event))
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id, "9876");
    assert_eq!(content.lifetime, uint!(30000));
    assert_eq!(content.description.session_type, "answer");
    assert_eq!(content.description.sdp, "not a real sdp");
}

#[test]
fn reject_v1_event_serialization() {
    let content = CallRejectEventContent::version_1("abcdef".into(), "9876".into());

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
        })
    );
}

#[test]
fn reject_v1_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "version": "1",
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.reject",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::CallReject(MessageLikeEvent::Original(message_event))
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id, "9876");
    assert_eq!(content.version, VoipVersionId::V1);
}

#[test]
fn select_v1_answer_event_serialization() {
    let content =
        CallSelectAnswerEventContent::version_1("abcdef".into(), "9876".into(), "6336".into());

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "call_id": "abcdef",
            "party_id": "9876",
            "selected_party_id": "6336",
            "version": "1",
        })
    );
}

#[test]
fn select_v1_answer_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "selected_party_id": "6336",
            "version": "1",
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.select_answer",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::CallSelectAnswer(MessageLikeEvent::Original(message_event))
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id, "9876");
    assert_eq!(content.selected_party_id, "6336");
    assert_eq!(content.version, VoipVersionId::V1);
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
    use ruma_common::owned_user_id;

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
        BTreeSet::from([owned_user_id!("@user:example.com"), owned_user_id!("@user2:example.com")])
    );
}
