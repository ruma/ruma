use assert_matches::assert_matches;
use js_int::uint;

use ruma_common::{
    events::{
        call::{
            answer::CallAnswerEventContent,
            candidates::{CallCandidatesEventContent, Candidate},
            hangup::CallHangupEventContent,
            invite::CallInviteEventContent,
            AnswerSessionDescription, OfferSessionDescription,
        },
        AnyMessageLikeEvent, AnySyncMessageLikeEvent, MessageLikeEvent,
    },
    room_id,
    serde::CanBeEmpty,
    MilliSecondsSinceUnixEpoch, VoipVersionId,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

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
fn answer_content_deserialization() {
    let json_data = json!({
        "answer": {
            "type": "answer",
            "sdp": "Hello"
        },
        "call_id": "foofoo",
        "version": 0
    });

    let content = from_json_value::<CallAnswerEventContent>(json_data).unwrap();

    assert_eq!(content.answer.sdp, "Hello");
    assert_eq!(content.call_id, "foofoo");
    assert_eq!(content.version, VoipVersionId::V0);
}

#[test]
fn answer_event_deserialization() {
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

    let message_event = assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data).unwrap(),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event)) => message_event
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
fn answer_event_deserialization_then_convert_to_full() {
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

    let message_event = assert_matches!(
        sync_ev.into_full_event(rid.to_owned()),
        AnyMessageLikeEvent::CallAnswer(MessageLikeEvent::Original(message_event)) => message_event
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

#[cfg(feature = "unstable-msc2746")]
mod msc2746 {
    use assert_matches::assert_matches;
    use assign::assign;
    use js_int::uint;
    use ruma_common::{
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
            AnyMessageLikeEvent, MessageLikeEvent,
        },
        VoipVersionId,
    };
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    #[test]
    fn invite_event_serialization() {
        let content = CallInviteEventContent::version_1(
            "abcdef".into(),
            "9876".into(),
            uint!(60000),
            OfferSessionDescription::new("not a real sdp".to_owned()),
            CallCapabilities::new(),
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
    fn answer_event_serialization() {
        let content = CallAnswerEventContent::version_1(
            AnswerSessionDescription::new("not a real sdp".to_owned()),
            "abcdef".into(),
            "9876".into(),
            assign!(CallCapabilities::new(), { dtmf: true }),
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
    fn candidates_event_serialization() {
        let content = CallCandidatesEventContent::version_1(
            "abcdef".into(),
            "9876".into(),
            vec![
                Candidate::new("not a real candidate".to_owned(), "0".to_owned(), uint!(0)),
                Candidate::new("another fake candidate".to_owned(), "0".to_owned(), uint!(1)),
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
                ],
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
    fn hangup_event_serialization() {
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
        let content = CallNegotiateEventContent::version_1(
            "abcdef".into(),
            "9876".into(),
            uint!(30000),
            SessionDescription::new(SessionDescriptionType::Offer, "not a real sdp".to_owned()),
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
    fn negotiate_event_deserialization() {
        let json_data = json!({
            "content": {
                "call_id": "abcdef",
                "party_id": "9876",
                "version": "1",
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
}
