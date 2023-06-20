#![cfg(feature = "unstable-msc3381")]

use std::collections::BTreeMap;

use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{
    events::{
        message::TextContentBlock,
        poll::{
            end::PollEndEventContent,
            response::PollResponseEventContent,
            start::{
                PollAnswer, PollAnswers, PollAnswersError, PollContentBlock, PollKind,
                PollStartEventContent,
            },
        },
        relation::Reference,
        AnyMessageLikeEvent, MessageLikeEvent,
    },
    owned_event_id,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn poll_answers_deserialization_valid() {
    let json_data = json!([
        { "org.matrix.msc3381.v2.id": "aaa", "org.matrix.msc1767.text": [{ "body": "First answer" }] },
        { "org.matrix.msc3381.v2.id": "bbb", "org.matrix.msc1767.text": [{ "body": "Second answer" }] },
    ]);

    let answers = from_json_value::<PollAnswers>(json_data).unwrap();
    assert_eq!(answers.len(), 2);
}

#[test]
fn poll_answers_deserialization_truncate() {
    let json_data = json!([
        { "org.matrix.msc3381.v2.id": "aaa", "org.matrix.msc1767.text": [{ "body": "1st answer" }] },
        { "org.matrix.msc3381.v2.id": "bbb", "org.matrix.msc1767.text": [{ "body": "2nd answer" }] },
        { "org.matrix.msc3381.v2.id": "ccc", "org.matrix.msc1767.text": [{ "body": "3rd answer" }] },
        { "org.matrix.msc3381.v2.id": "ddd", "org.matrix.msc1767.text": [{ "body": "4th answer" }] },
        { "org.matrix.msc3381.v2.id": "eee", "org.matrix.msc1767.text": [{ "body": "5th answer" }] },
        { "org.matrix.msc3381.v2.id": "fff", "org.matrix.msc1767.text": [{ "body": "6th answer" }] },
        { "org.matrix.msc3381.v2.id": "ggg", "org.matrix.msc1767.text": [{ "body": "7th answer" }] },
        { "org.matrix.msc3381.v2.id": "hhh", "org.matrix.msc1767.text": [{ "body": "8th answer" }] },
        { "org.matrix.msc3381.v2.id": "iii", "org.matrix.msc1767.text": [{ "body": "9th answer" }] },
        { "org.matrix.msc3381.v2.id": "jjj", "org.matrix.msc1767.text": [{ "body": "10th answer" }] },
        { "org.matrix.msc3381.v2.id": "kkk", "org.matrix.msc1767.text": [{ "body": "11th answer" }] },
        { "org.matrix.msc3381.v2.id": "lll", "org.matrix.msc1767.text": [{ "body": "12th answer" }] },
        { "org.matrix.msc3381.v2.id": "mmm", "org.matrix.msc1767.text": [{ "body": "13th answer" }] },
        { "org.matrix.msc3381.v2.id": "nnn", "org.matrix.msc1767.text": [{ "body": "14th answer" }] },
        { "org.matrix.msc3381.v2.id": "ooo", "org.matrix.msc1767.text": [{ "body": "15th answer" }] },
        { "org.matrix.msc3381.v2.id": "ppp", "org.matrix.msc1767.text": [{ "body": "16th answer" }] },
        { "org.matrix.msc3381.v2.id": "qqq", "org.matrix.msc1767.text": [{ "body": "17th answer" }] },
        { "org.matrix.msc3381.v2.id": "rrr", "org.matrix.msc1767.text": [{ "body": "18th answer" }] },
        { "org.matrix.msc3381.v2.id": "sss", "org.matrix.msc1767.text": [{ "body": "19th answer" }] },
        { "org.matrix.msc3381.v2.id": "ttt", "org.matrix.msc1767.text": [{ "body": "20th answer" }] },
        { "org.matrix.msc3381.v2.id": "uuu", "org.matrix.msc1767.text": [{ "body": "21th answer" }] },
        { "org.matrix.msc3381.v2.id": "vvv", "org.matrix.msc1767.text": [{ "body": "22th answer" }] },
    ]);

    let answers = from_json_value::<PollAnswers>(json_data).unwrap();
    assert_eq!(answers.len(), 20);
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
    let event_content = PollStartEventContent::with_plain_text(
        "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
        PollContentBlock::new(
            TextContentBlock::plain("How's the weather?"),
            vec![
                PollAnswer::new("not-bad".to_owned(), TextContentBlock::plain("Not bad…")),
                PollAnswer::new("fine".to_owned(), TextContentBlock::plain("Fine.")),
                PollAnswer::new("amazing".to_owned(), TextContentBlock::plain("Amazing!")),
            ]
            .try_into()
            .unwrap(),
        ),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!" }
            ],
            "org.matrix.msc3381.v2.poll": {
                "question": { "org.matrix.msc1767.text": [{ "body": "How's the weather?" }] },
                "answers": [
                    { "org.matrix.msc3381.v2.id": "not-bad", "org.matrix.msc1767.text": [{ "body": "Not bad…" }] },
                    { "org.matrix.msc3381.v2.id": "fine", "org.matrix.msc1767.text": [{ "body": "Fine." }] },
                    { "org.matrix.msc3381.v2.id": "amazing", "org.matrix.msc1767.text": [{ "body": "Amazing!" }] },
                ],
            },
        })
    );
}

#[test]
fn start_event_serialization() {
    let mut poll = PollContentBlock::new(
        TextContentBlock::plain("How's the weather?"),
        vec![
            PollAnswer::new("not-bad".to_owned(), TextContentBlock::plain("Not bad…")),
            PollAnswer::new("fine".to_owned(), TextContentBlock::plain("Fine.")),
            PollAnswer::new("amazing".to_owned(), TextContentBlock::plain("Amazing!")),
        ]
        .try_into()
        .unwrap(),
    );
    poll.kind = PollKind::Disclosed;
    poll.max_selections = uint!(2);
    let content = PollStartEventContent::with_plain_text(
        "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
        poll,
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!" }
            ],
            "org.matrix.msc3381.v2.poll": {
                "question": { "org.matrix.msc1767.text": [{ "body": "How's the weather?" }] },
                "kind": "org.matrix.msc3381.v2.disclosed",
                "max_selections": 2,
                "answers": [
                    { "org.matrix.msc3381.v2.id": "not-bad", "org.matrix.msc1767.text": [{ "body": "Not bad…" }] },
                    { "org.matrix.msc3381.v2.id": "fine", "org.matrix.msc1767.text": [{ "body": "Fine." }] },
                    { "org.matrix.msc3381.v2.id": "amazing", "org.matrix.msc1767.text": [{ "body": "Amazing!" }] },
                ]
            },
        })
    );
}

#[test]
fn start_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!" }
            ],
            "org.matrix.msc3381.v2.poll": {
                "question": { "org.matrix.msc1767.text": [{ "body": "How's the weather?" }] },
                "max_selections": 2,
                "answers": [
                    { "org.matrix.msc3381.v2.id": "not-bad", "org.matrix.msc1767.text": [{ "body": "Not bad…" }] },
                    { "org.matrix.msc3381.v2.id": "fine", "org.matrix.msc1767.text": [{ "body": "Fine." }] },
                    { "org.matrix.msc3381.v2.id": "amazing", "org.matrix.msc1767.text": [{ "body": "Amazing!" }] },
                ]
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.v2.poll.start",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::PollStart(MessageLikeEvent::Original(message_event))
    );
    assert_eq!(
        message_event.content.text[0].body,
        "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!"
    );
    let poll = message_event.content.poll;
    assert_eq!(poll.question.text[0].body, "How's the weather?");
    assert_eq!(poll.kind, PollKind::Undisclosed);
    assert_eq!(poll.max_selections, uint!(2));
    let answers = poll.answers;
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
        vec!["my-answer".to_owned()].into(),
        owned_event_id!("$related_event:notareal.hs"),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc3381.v2.selections": ["my-answer"],
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
        vec!["first-answer".to_owned(), "second-answer".to_owned()].into(),
        owned_event_id!("$related_event:notareal.hs"),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc3381.v2.selections": ["first-answer", "second-answer"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            },
        })
    );
}

#[test]
fn response_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc3381.v2.selections": ["my-answer"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.v2.poll.response",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::PollResponse(MessageLikeEvent::Original(message_event))
    );
    let selections = message_event.content.selections;
    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0], "my-answer");
    assert_matches!(message_event.content.relates_to, Reference { event_id, .. });
    assert_eq!(event_id, "$related_event:notareal.hs");
}

#[test]
fn end_content_serialization() {
    let event_content = PollEndEventContent::with_plain_text(
        "The poll has closed. Top answer: Amazing!",
        owned_event_id!("$related_event:notareal.hs"),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "The poll has closed. Top answer: Amazing!" }
            ],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        })
    );
}

#[test]
fn end_event_serialization() {
    let mut content = PollEndEventContent::with_plain_text(
        "The poll has closed. Top answer: Amazing!",
        owned_event_id!("$related_event:notareal.hs"),
    );
    content.poll_results = Some(
        BTreeMap::from([
            ("not-bad".to_owned(), uint!(1)),
            ("fine".to_owned(), uint!(5)),
            ("amazing".to_owned(), uint!(14)),
        ])
        .into(),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "org.matrix.msc1767.text": [
                { "body": "The poll has closed. Top answer: Amazing!" },
            ],
            "org.matrix.msc3381.v2.poll.results": {
                "not-bad": 1,
                "fine": 5,
                "amazing": 14,
            },
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            },
        })
    );
}

#[test]
fn end_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": [
                { "body": "The poll has closed. Top answer: Amazing!" },
            ],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.v2.poll.end",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(event, AnyMessageLikeEvent::PollEnd(MessageLikeEvent::Original(message_event)));
    assert_eq!(message_event.content.text[0].body, "The poll has closed. Top answer: Amazing!");
    assert_matches!(message_event.content.relates_to, Reference { event_id, .. });
    assert_eq!(event_id, "$related_event:notareal.hs");
}
