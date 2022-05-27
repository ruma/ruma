#![cfg(feature = "unstable-msc3381")]

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        message::MessageContent,
        poll::{
            end::{PollEndContent, PollEndEventContent},
            response::{PollResponseContent, PollResponseEventContent},
            start::{
                PollAnswer, PollAnswers, PollAnswersError, PollKind, PollStartContent,
                PollStartEventContent,
            },
            ReferenceRelation,
        },
        AnyMessageLikeEvent, MessageLikeEvent, MessageLikeUnsigned, OriginalMessageLikeEvent,
    },
    room_id, user_id, MilliSecondsSinceUnixEpoch,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn poll_answers_deserialization_valid() {
    let json_data = json!([
        { "id": "aaa", "m.text": "First answer" },
        { "id": "bbb", "m.text": "Second answer" },
    ]);

    assert_matches!(
        from_json_value::<PollAnswers>(json_data),
        Ok(answers) if answers.answers().len() == 2
    );
}

#[test]
fn poll_answers_deserialization_truncate() {
    let json_data = json!([
        { "id": "aaa", "m.text": "1st answer" },
        { "id": "bbb", "m.text": "2nd answer" },
        { "id": "ccc", "m.text": "3rd answer" },
        { "id": "ddd", "m.text": "4th answer" },
        { "id": "eee", "m.text": "5th answer" },
        { "id": "fff", "m.text": "6th answer" },
        { "id": "ggg", "m.text": "7th answer" },
        { "id": "hhh", "m.text": "8th answer" },
        { "id": "iii", "m.text": "9th answer" },
        { "id": "jjj", "m.text": "10th answer" },
        { "id": "kkk", "m.text": "11th answer" },
        { "id": "lll", "m.text": "12th answer" },
        { "id": "mmm", "m.text": "13th answer" },
        { "id": "nnn", "m.text": "14th answer" },
        { "id": "ooo", "m.text": "15th answer" },
        { "id": "ppp", "m.text": "16th answer" },
        { "id": "qqq", "m.text": "17th answer" },
        { "id": "rrr", "m.text": "18th answer" },
        { "id": "sss", "m.text": "19th answer" },
        { "id": "ttt", "m.text": "20th answer" },
        { "id": "uuu", "m.text": "21th answer" },
        { "id": "vvv", "m.text": "22th answer" },
    ]);

    assert_matches!(
        from_json_value::<PollAnswers>(json_data),
        Ok(answers) if answers.answers().len() == 20
    );
}

#[test]
fn poll_answers_deserialization_not_enough() {
    let json_data = json!([]);

    let err = from_json_value::<PollAnswers>(json_data).unwrap_err();
    assert!(err.is_data());
    assert_eq!(err.to_string(), PollAnswersError::NotEnoughValues.to_string());
}

#[test]
fn start_content_serialization() {
    let event_content = PollStartEventContent::new(PollStartContent::new(
        MessageContent::plain("How's the weather?"),
        PollKind::Undisclosed,
        vec![
            PollAnswer::new("not-bad".to_owned(), MessageContent::plain("Not bad…")),
            PollAnswer::new("fine".to_owned(), MessageContent::plain("Fine.")),
            PollAnswer::new("amazing".to_owned(), MessageContent::plain("Amazing!")),
        ]
        .try_into()
        .unwrap(),
    ));

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3381.poll.start": {
                "question": { "org.matrix.msc1767.text": "How's the weather?" },
                "kind": "org.matrix.msc3381.poll.undisclosed",
                "answers": [
                    { "id": "not-bad", "org.matrix.msc1767.text": "Not bad…"},
                    { "id": "fine", "org.matrix.msc1767.text": "Fine."},
                    { "id": "amazing", "org.matrix.msc1767.text": "Amazing!"},
                ],
            },
        })
    );
}

#[test]
fn start_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: PollStartEventContent::new(assign!(
            PollStartContent::new(
                MessageContent::plain("How's the weather?"),
                PollKind::Disclosed,
                vec![
                    PollAnswer::new("not-bad".to_owned(), MessageContent::plain("Not bad…")),
                    PollAnswer::new("fine".to_owned(), MessageContent::plain("Fine.")),
                    PollAnswer::new("amazing".to_owned(), MessageContent::plain("Amazing!")),
                ]
                .try_into()
                .unwrap(),
            ),
            { max_selections: uint!(2) }
        )),
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
                "org.matrix.msc3381.poll.start": {
                    "question": { "org.matrix.msc1767.text": "How's the weather?" },
                    "kind": "org.matrix.msc3381.poll.disclosed",
                    "max_selections": 2,
                    "answers": [
                        { "id": "not-bad", "org.matrix.msc1767.text": "Not bad…"},
                        { "id": "fine", "org.matrix.msc1767.text": "Fine."},
                        { "id": "amazing", "org.matrix.msc1767.text": "Amazing!"},
                    ]
                },
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "org.matrix.msc3381.poll.start",
        })
    );
}

#[test]
fn start_event_unstable_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc3381.poll.start": {
                "question": { "org.matrix.msc1767.text": "How's the weather?" },
                "kind": "org.matrix.msc3381.poll.undisclosed",
                "max_selections": 2,
                "answers": [
                    { "id": "not-bad", "org.matrix.msc1767.text": "Not bad…"},
                    { "id": "fine", "org.matrix.msc1767.text": "Fine."},
                    { "id": "amazing", "org.matrix.msc1767.text": "Amazing!"},
                ]
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.poll.start",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::PollStart(MessageLikeEvent::Original(message_event)) => message_event
    );
    let poll_start = message_event.content.poll_start;
    assert_eq!(poll_start.question[0].body, "How's the weather?");
    assert_eq!(poll_start.kind, PollKind::Undisclosed);
    assert_eq!(poll_start.max_selections, uint!(2));
    let answers = poll_start.answers.answers();
    assert_eq!(answers.len(), 3);
    assert_eq!(answers[0].id, "not-bad");
    assert_eq!(answers[0].answer[0].body, "Not bad…");
    assert_eq!(answers[1].id, "fine");
    assert_eq!(answers[1].answer[0].body, "Fine.");
    assert_eq!(answers[2].id, "amazing");
    assert_eq!(answers[2].answer[0].body, "Amazing!");
}

#[test]
fn start_event_stable_deserialization() {
    let json_data = json!({
        "content": {
            "m.poll.start": {
                "question": { "m.text": "How's the weather?" },
                "kind": "m.poll.disclosed",
                "answers": [
                    { "id": "not-bad", "m.text": "Not bad…"},
                    { "id": "fine", "m.text": "Fine."},
                    { "id": "amazing", "m.text": "Amazing!"},
                ]
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.poll.start",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::PollStart(MessageLikeEvent::Original(message_event)) => message_event
    );
    let poll_start = message_event.content.poll_start;
    assert_eq!(poll_start.question[0].body, "How's the weather?");
    assert_eq!(poll_start.kind, PollKind::Disclosed);
    assert_eq!(poll_start.max_selections, uint!(1));
    let answers = poll_start.answers.answers();
    assert_eq!(answers.len(), 3);
    assert_eq!(answers[0].id, "not-bad");
    assert_eq!(answers[0].answer[0].body, "Not bad…");
    assert_eq!(answers[1].id, "fine");
    assert_eq!(answers[1].answer[0].body, "Fine.");
    assert_eq!(answers[2].id, "amazing");
    assert_eq!(answers[2].answer[0].body, "Amazing!");
}

#[test]
fn response_content_serialization() {
    let event_content = PollResponseEventContent::new(
        PollResponseContent::new(vec!["my-answer".to_owned()]),
        event_id!("$related_event:notareal.hs").to_owned(),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3381.poll.response": {
                "answers": ["my-answer"],
            },
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        })
    );
}

#[test]
fn response_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: PollResponseEventContent::new(
            PollResponseContent::new(vec!["first-answer".to_owned(), "second-answer".to_owned()]),
            event_id!("$related_event:notareal.hs").to_owned(),
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
                "org.matrix.msc3381.poll.response": {
                    "answers": ["first-answer", "second-answer"],
                },
                "m.relates_to": {
                    "rel_type": "m.reference",
                    "event_id": "$related_event:notareal.hs",
                }
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "org.matrix.msc3381.poll.response",
        })
    );
}

#[test]
fn response_event_unstable_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc3381.poll.response": {
                "answers": ["my-answer"],
            },
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.poll.response",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::PollResponse(MessageLikeEvent::Original(message_event))
            => message_event
    );
    let answers = message_event.content.poll_response.answers;
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0], "my-answer");
    assert_matches!(
        message_event.content.relates_to,
        ReferenceRelation { event_id, .. } if event_id == "$related_event:notareal.hs"
    );
}

#[test]
fn response_event_stable_deserialization() {
    let json_data = json!({
        "content": {
            "m.poll.response": {
                "answers": ["first-answer", "second-answer"],
            },
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.poll.response",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::PollResponse(MessageLikeEvent::Original(message_event))
            => message_event
    );
    let answers = message_event.content.poll_response.answers;
    assert_eq!(answers.len(), 2);
    assert_eq!(answers[0], "first-answer");
    assert_eq!(answers[1], "second-answer");
    assert_matches!(
        message_event.content.relates_to,
        ReferenceRelation { event_id, .. } if event_id == "$related_event:notareal.hs"
    );
}

#[test]
fn end_content_serialization() {
    let event_content = PollEndEventContent::new(
        PollEndContent::new(),
        event_id!("$related_event:notareal.hs").to_owned(),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3381.poll.end": {},
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        })
    );
}

#[test]
fn end_event_serialization() {
    let event = OriginalMessageLikeEvent {
        content: PollEndEventContent::new(
            PollEndContent::new(),
            event_id!("$related_event:notareal.hs").to_owned(),
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
                "org.matrix.msc3381.poll.end": {},
                "m.relates_to": {
                    "rel_type": "m.reference",
                    "event_id": "$related_event:notareal.hs",
                }
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "org.matrix.msc3381.poll.end",
        })
    );
}

#[test]
fn end_event_unstable_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc3381.poll.end": {},
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.poll.end",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::PollEnd(MessageLikeEvent::Original(message_event)) => message_event
    );
    assert_matches!(
        message_event.content.relates_to,
        ReferenceRelation { event_id, .. } if event_id == "$related_event:notareal.hs"
    );
}

#[test]
fn end_event_stable_deserialization() {
    let json_data = json!({
        "content": {
            "m.poll.end": {},
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.poll.end",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    let message_event = assert_matches!(
        event,
        AnyMessageLikeEvent::PollEnd(MessageLikeEvent::Original(message_event)) => message_event
    );
    assert_matches!(
        message_event.content.relates_to,
        ReferenceRelation { event_id, .. } if event_id == "$related_event:notareal.hs"
    );
}
