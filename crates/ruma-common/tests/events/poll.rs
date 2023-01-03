#![cfg(feature = "unstable-msc3381")]

use assert_matches::assert_matches;
use assign::assign;
use js_int::uint;
use ruma_common::{
    event_id,
    events::{
        message::TextContentBlock,
        poll::{
            end::{PollEndContent, PollEndEventContent},
            response::{PollResponseContent, PollResponseEventContent},
            start::{
                PollAnswer, PollAnswers, PollAnswersError, PollKind, PollStartContent,
                PollStartEventContent,
            },
        },
        relation::Reference,
        AnyMessageLikeEvent, MessageLikeEvent,
    },
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn poll_answers_deserialization_valid() {
    let json_data = json!([
        { "id": "aaa", "org.matrix.msc1767.text": [{ "body": "First answer" }] },
        { "id": "bbb", "org.matrix.msc1767.text": [{ "body": "Second answer" }] },
    ]);

    let answers = from_json_value::<PollAnswers>(json_data).unwrap();
    assert_eq!(answers.answers().len(), 2);
}

#[test]
fn poll_answers_deserialization_truncate() {
    let json_data = json!([
        { "id": "aaa", "org.matrix.msc1767.text": [{ "body": "1st answer" }] },
        { "id": "bbb", "org.matrix.msc1767.text": [{ "body": "2nd answer" }] },
        { "id": "ccc", "org.matrix.msc1767.text": [{ "body": "3rd answer" }] },
        { "id": "ddd", "org.matrix.msc1767.text": [{ "body": "4th answer" }] },
        { "id": "eee", "org.matrix.msc1767.text": [{ "body": "5th answer" }] },
        { "id": "fff", "org.matrix.msc1767.text": [{ "body": "6th answer" }] },
        { "id": "ggg", "org.matrix.msc1767.text": [{ "body": "7th answer" }] },
        { "id": "hhh", "org.matrix.msc1767.text": [{ "body": "8th answer" }] },
        { "id": "iii", "org.matrix.msc1767.text": [{ "body": "9th answer" }] },
        { "id": "jjj", "org.matrix.msc1767.text": [{ "body": "10th answer" }] },
        { "id": "kkk", "org.matrix.msc1767.text": [{ "body": "11th answer" }] },
        { "id": "lll", "org.matrix.msc1767.text": [{ "body": "12th answer" }] },
        { "id": "mmm", "org.matrix.msc1767.text": [{ "body": "13th answer" }] },
        { "id": "nnn", "org.matrix.msc1767.text": [{ "body": "14th answer" }] },
        { "id": "ooo", "org.matrix.msc1767.text": [{ "body": "15th answer" }] },
        { "id": "ppp", "org.matrix.msc1767.text": [{ "body": "16th answer" }] },
        { "id": "qqq", "org.matrix.msc1767.text": [{ "body": "17th answer" }] },
        { "id": "rrr", "org.matrix.msc1767.text": [{ "body": "18th answer" }] },
        { "id": "sss", "org.matrix.msc1767.text": [{ "body": "19th answer" }] },
        { "id": "ttt", "org.matrix.msc1767.text": [{ "body": "20th answer" }] },
        { "id": "uuu", "org.matrix.msc1767.text": [{ "body": "21th answer" }] },
        { "id": "vvv", "org.matrix.msc1767.text": [{ "body": "22th answer" }] },
    ]);

    let answers = from_json_value::<PollAnswers>(json_data).unwrap();
    assert_eq!(answers.answers().len(), 20);
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
        TextContentBlock::plain("How's the weather?"),
        PollKind::Undisclosed,
        vec![
            PollAnswer::new("not-bad".to_owned(), TextContentBlock::plain("Not bad…")),
            PollAnswer::new("fine".to_owned(), TextContentBlock::plain("Fine.")),
            PollAnswer::new("amazing".to_owned(), TextContentBlock::plain("Amazing!")),
        ]
        .try_into()
        .unwrap(),
    ));

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3381.poll.start": {
                "question": { "org.matrix.msc1767.text": [{ "body": "How's the weather?" }] },
                "kind": "org.matrix.msc3381.poll.undisclosed",
                "answers": [
                    { "id": "not-bad", "org.matrix.msc1767.text": [{ "body": "Not bad…" }] },
                    { "id": "fine", "org.matrix.msc1767.text": [{ "body": "Fine." }] },
                    { "id": "amazing", "org.matrix.msc1767.text": [{ "body": "Amazing!" }] },
                ],
            },
        })
    );
}

#[test]
fn start_event_serialization() {
    let content = PollStartEventContent::new(assign!(
        PollStartContent::new(
            TextContentBlock::plain("How's the weather?"),
            PollKind::Disclosed,
            vec![
                PollAnswer::new("not-bad".to_owned(), TextContentBlock::plain("Not bad…")),
                PollAnswer::new("fine".to_owned(), TextContentBlock::plain("Fine.")),
                PollAnswer::new("amazing".to_owned(), TextContentBlock::plain("Amazing!")),
            ]
            .try_into()
            .unwrap(),
        ),
        { max_selections: uint!(2) }
    ));

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc3381.poll.start": {
                "question": { "org.matrix.msc1767.text": [{ "body": "How's the weather?" }] },
                "kind": "org.matrix.msc3381.poll.disclosed",
                "max_selections": 2,
                "answers": [
                    { "id": "not-bad", "org.matrix.msc1767.text": [{ "body": "Not bad…" }] },
                    { "id": "fine", "org.matrix.msc1767.text": [{ "body": "Fine." }] },
                    { "id": "amazing", "org.matrix.msc1767.text": [{ "body": "Amazing!" }] },
                ]
            },
        })
    );
}

#[test]
fn start_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc3381.poll.start": {
                "question": { "org.matrix.msc1767.text": [{ "body": "How's the weather?" }] },
                "kind": "org.matrix.msc3381.poll.undisclosed",
                "max_selections": 2,
                "answers": [
                    { "id": "not-bad", "org.matrix.msc1767.text": [{ "body": "Not bad…" }] },
                    { "id": "fine", "org.matrix.msc1767.text": [{ "body": "Fine." }] },
                    { "id": "amazing", "org.matrix.msc1767.text": [{ "body": "Amazing!" }] },
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
    assert_eq!(poll_start.question.text[0].body, "How's the weather?");
    assert_eq!(poll_start.kind, PollKind::Undisclosed);
    assert_eq!(poll_start.max_selections, uint!(2));
    let answers = poll_start.answers.answers();
    assert_eq!(answers.len(), 3);
    assert_eq!(answers[0].id, "not-bad");
    assert_eq!(answers[0].text[0].body, "Not bad…");
    assert_eq!(answers[1].id, "fine");
    assert_eq!(answers[1].text[0].body, "Fine.");
    assert_eq!(answers[2].id, "amazing");
    assert_eq!(answers[2].text[0].body, "Amazing!");
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
    let content = PollResponseEventContent::new(
        PollResponseContent::new(vec!["first-answer".to_owned(), "second-answer".to_owned()]),
        event_id!("$related_event:notareal.hs").to_owned(),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc3381.poll.response": {
                "answers": ["first-answer", "second-answer"],
            },
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            },
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
    let event_id = assert_matches!(
        message_event.content.relates_to,
        Reference { event_id, .. } => event_id
    );
    assert_eq!(event_id, "$related_event:notareal.hs");
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
    let event_id = assert_matches!(
        message_event.content.relates_to,
        Reference { event_id, .. } => event_id
    );
    assert_eq!(event_id, "$related_event:notareal.hs");
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
    let content = PollEndEventContent::new(
        PollEndContent::new(),
        event_id!("$related_event:notareal.hs").to_owned(),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc3381.poll.end": {},
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            },
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
    let event_id = assert_matches!(
        message_event.content.relates_to,
        Reference { event_id, .. } => event_id
    );
    assert_eq!(event_id, "$related_event:notareal.hs");
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
    let event_id = assert_matches!(
        message_event.content.relates_to,
        Reference { event_id, .. } => event_id
    );
    assert_eq!(event_id, "$related_event:notareal.hs");
}
