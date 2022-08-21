#![cfg(feature = "unstable-msc2746")]

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        call::{
            answer::CallAnswerEventContent,
            candidates::{CallCandidatesEventContent, Candidate},
            hangup::{CallHangupEventContent, Reason},
            invite::CallInviteEventContent,
            negotiate::CallNegotiateEventContent,
            reject::CallRejectEventContent,
            select_answer::CallSelectAnswerEventContent,
            AnswerSessionDescription, CallCapabilities, OfferSessionDescription,
            SessionDescription, SessionDescriptionType,
        },
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned, OriginalMessageLikeEvent,
    },
    room_id, user_id, MilliSecondsSinceUnixEpoch, VoipVersionId,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn invite_content_serialization() {
    let event_content = CallInviteEventContent::version_0(
        "abcdef".into(),
        uint!(30000),
        OfferSessionDescription::new("not a real sdp".to_owned()),
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
fn invite_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: CallInviteEventContent::version_1(
            "abcdef".into(),
            "9876".into(),
            uint!(60000),
            OfferSessionDescription::new("not a real sdp".to_owned()),
            CallCapabilities::new(),
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
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
        })
    );
}

#[test]
fn invite_event_deserialization() {
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
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::CallInvite(MessageLikeEvent::Original(message_event)) => message_event
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.lifetime, uint!(60000));
    assert_eq!(content.version, VoipVersionId::V1);
    assert_eq!(content.offer.sdp, "not a real sdp");
    assert!(!content.capabilities.dtmf);
}

#[test]
fn answer_content_serialization() {
    let event_content = CallAnswerEventContent::version_0(
        AnswerSessionDescription::new("not a real sdp".to_owned()),
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
fn answer_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: CallAnswerEventContent::version_1(
            AnswerSessionDescription::new("not a real sdp".to_owned()),
            "abcdef".into(),
            "9876".into(),
            assign!(CallCapabilities::new(), { dtmf: true }),
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
            "content": {
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
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.call.answer",
        })
    );
}

#[test]
fn answer_event_deserialization() {
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
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event)) => message_event
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.version.as_ref(), "org.matrix.1b");
    assert_eq!(content.answer.sdp, "not a real sdp");
    assert!(content.capabilities.dtmf);
}

#[test]
fn candidates_content_serialization() {
    let event_content = CallCandidatesEventContent::version_0(
        "abcdef".into(),
        vec![Candidate::new("not a real candidate".to_owned(), "0".to_owned(), uint!(0))],
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
fn candidates_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: CallCandidatesEventContent::version_1(
            "abcdef".into(),
            "9876".into(),
            vec![
                Candidate::new("not a real candidate".to_owned(), "0".to_owned(), uint!(0)),
                Candidate::new("another fake candidate".to_owned(), "0".to_owned(), uint!(1)),
            ],
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
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
                ],
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.call.candidates",
        })
    );
}

#[test]
fn candidates_event_deserialization() {
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
            ],
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.call.candidates",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::CallCandidates(MessageLikeEvent::Original(message_event)) => message_event
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.version, VoipVersionId::V1);
    assert_eq!(content.candidates.len(), 2);
    assert_eq!(content.candidates[0].candidate, "not a real candidate");
    assert_eq!(content.candidates[0].sdp_mid, "0");
    assert_eq!(content.candidates[0].sdp_m_line_index, uint!(0));
    assert_eq!(content.candidates[1].candidate, "another fake candidate");
    assert_eq!(content.candidates[1].sdp_mid, "0");
    assert_eq!(content.candidates[1].sdp_m_line_index, uint!(1));
}

#[test]
fn hangup_content_serialization() {
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
fn hangup_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: CallHangupEventContent::version_1(
            "abcdef".into(),
            "9876".into(),
            Reason::IceFailed,
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
            "content": {
                "call_id": "abcdef",
                "party_id": "9876",
                "version": "1",
                "reason": "ice_failed",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.call.hangup",
        })
    );
}

#[test]
fn hangup_event_deserialization() {
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
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::CallHangup(MessageLikeEvent::Original(message_event)) => message_event
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id.unwrap(), "9876");
    assert_eq!(content.version, VoipVersionId::V1);
    assert_eq!(content.reason, Some(Reason::UserHangup));
}

#[test]
fn negotiate_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: CallNegotiateEventContent::new(
            "abcdef".into(),
            "9876".into(),
            uint!(30000),
            SessionDescription::new(SessionDescriptionType::Offer, "not a real sdp".to_owned()),
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
            "content": {
                "call_id": "abcdef",
                "party_id": "9876",
                "lifetime": 30000,
                "description": {
                    "type": "offer",
                    "sdp": "not a real sdp",
                }
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.call.negotiate",
        })
    );
}

#[test]
fn negotiate_event_deserialization() {
    let json_data = json!({
        "content": {
            "call_id": "abcdef",
            "party_id": "9876",
            "lifetime": 30000,
            "description": {
                "type": "pranswer",
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
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::CallNegotiate(MessageLikeEvent::Original(message_event)) => message_event
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id, "9876");
    assert_eq!(content.lifetime, uint!(30000));
    assert_eq!(content.description.session_type, SessionDescriptionType::PrAnswer);
    assert_eq!(content.description.sdp, "not a real sdp");
}

#[test]
fn reject_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: CallRejectEventContent::version_1("abcdef".into(), "9876".into()),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
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
        })
    );
}

#[test]
fn reject_event_deserialization() {
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
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::CallReject(MessageLikeEvent::Original(message_event)) => message_event
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id, "9876");
    assert_eq!(content.version, VoipVersionId::V1);
}

#[test]
fn select_answer_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: CallSelectAnswerEventContent::version_1(
            "abcdef".into(),
            "9876".into(),
            "6336".into(),
        ),
        event_id: event_id!("$event:notareal.hs").to_owned(),
        sender: user_id!("@user:notareal.hs").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(134_829_848)),
        room_id: room_id!("!roomid:notareal.hs").to_owned(),
        unsigned: MessageLikeUnsigned::default(),
    };

    assert_eq!(
        to_json_value(&event).unwrap(),
        json!({
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
        })
    );
}

#[test]
fn select_answer_event_deserialization() {
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
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::CallSelectAnswer(MessageLikeEvent::Original(message_event)) => message_event
    );
    let content = message_event.content;
    assert_eq!(content.call_id, "abcdef");
    assert_eq!(content.party_id, "9876");
    assert_eq!(content.selected_party_id, "6336");
    assert_eq!(content.version, VoipVersionId::V1);
}
