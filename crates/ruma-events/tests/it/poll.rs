#![cfg(feature = "unstable-msc3381")]

use std::{collections::BTreeMap, ops::Range};

use assert_matches2::assert_matches;
use js_int::{uint, UInt};
use ruma_common::{owned_event_id, MilliSecondsSinceUnixEpoch};
use ruma_events::{
    message::TextContentBlock,
    poll::{
        compile_poll_results, compile_unstable_poll_results,
        end::PollEndEventContent,
        response::{OriginalSyncPollResponseEvent, PollResponseEventContent},
        start::{
            OriginalSyncPollStartEvent, PollAnswer, PollAnswers, PollAnswersError,
            PollContentBlock, PollKind, PollStartEventContent,
        },
        unstable_end::UnstablePollEndEventContent,
        unstable_response::{
            OriginalSyncUnstablePollResponseEvent, UnstablePollResponseEventContent,
        },
        unstable_start::{
            NewUnstablePollStartEventContent, OriginalSyncUnstablePollStartEvent,
            ReplacementUnstablePollStartEventContent, UnstablePollAnswer,
            UnstablePollStartContentBlock, UnstablePollStartEventContent,
        },
    },
    relation::Reference,
    room::message::{Relation, RelationWithoutReplacement},
    AnyMessageLikeEvent, MessageLikeEvent,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn poll_answers_deserialization_valid() {
    let json_data = json!([
        { "m.id": "aaa", "m.text": [{ "body": "First answer" }] },
        { "m.id": "bbb", "m.text": [{ "body": "Second answer" }] },
    ]);

    let answers = from_json_value::<PollAnswers>(json_data).unwrap();
    assert_eq!(answers.len(), 2);
}

#[test]
fn poll_answers_deserialization_truncate() {
    let json_data = json!([
        { "m.id": "aaa", "m.text": [{ "body": "1st answer" }] },
        { "m.id": "bbb", "m.text": [{ "body": "2nd answer" }] },
        { "m.id": "ccc", "m.text": [{ "body": "3rd answer" }] },
        { "m.id": "ddd", "m.text": [{ "body": "4th answer" }] },
        { "m.id": "eee", "m.text": [{ "body": "5th answer" }] },
        { "m.id": "fff", "m.text": [{ "body": "6th answer" }] },
        { "m.id": "ggg", "m.text": [{ "body": "7th answer" }] },
        { "m.id": "hhh", "m.text": [{ "body": "8th answer" }] },
        { "m.id": "iii", "m.text": [{ "body": "9th answer" }] },
        { "m.id": "jjj", "m.text": [{ "body": "10th answer" }] },
        { "m.id": "kkk", "m.text": [{ "body": "11th answer" }] },
        { "m.id": "lll", "m.text": [{ "body": "12th answer" }] },
        { "m.id": "mmm", "m.text": [{ "body": "13th answer" }] },
        { "m.id": "nnn", "m.text": [{ "body": "14th answer" }] },
        { "m.id": "ooo", "m.text": [{ "body": "15th answer" }] },
        { "m.id": "ppp", "m.text": [{ "body": "16th answer" }] },
        { "m.id": "qqq", "m.text": [{ "body": "17th answer" }] },
        { "m.id": "rrr", "m.text": [{ "body": "18th answer" }] },
        { "m.id": "sss", "m.text": [{ "body": "19th answer" }] },
        { "m.id": "ttt", "m.text": [{ "body": "20th answer" }] },
        { "m.id": "uuu", "m.text": [{ "body": "21th answer" }] },
        { "m.id": "vvv", "m.text": [{ "body": "22th answer" }] },
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
            "m.text": [{ "body": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!" }],
            "m.poll": {
                "question": { "m.text": [{ "body": "How's the weather?" }] },
                "answers": [
                    { "m.id": "not-bad", "m.text": [{ "body": "Not bad…" }] },
                    { "m.id": "fine", "m.text":  [{ "body": "Fine." }] },
                    { "m.id": "amazing", "m.text":  [{ "body": "Amazing!" }] },
                ],
            },
        })
    );
}

#[test]
fn start_content_other_serialization() {
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
            "m.text": [{ "body": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!" }],
            "m.poll": {
                "question": { "m.text": [{ "body": "How's the weather?" }] },
                "kind": "m.disclosed",
                "max_selections": 2,
                "answers": [
                    { "m.id": "not-bad", "m.text":  [{ "body": "Not bad…" }] },
                    { "m.id": "fine", "m.text":  [{ "body": "Fine." }] },
                    { "m.id": "amazing", "m.text":  [{ "body": "Amazing!" }] },
                ]
            },
        })
    );
}

#[test]
fn start_event_deserialization() {
    let json_data = json!({
        "content": {
            "m.text": [
                { "body": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!" }
            ],
            "m.poll": {
                "question": { "m.text": [{ "body": "How's the weather?" }] },
                "max_selections": 2,
                "answers": [
                    {
                        "m.id": "not-bad",
                        "m.text": [{ "body": "Not bad…" }],
                    },
                    {
                        "m.id": "fine",
                        "m.text": [{ "body": "Fine." }],
                    },
                    {
                        "m.id": "amazing",
                        "m.text": [{ "body": "Amazing!" }],
                    },
                ]
            },
            "m.new_content": {
                "m.text": [
                    { "body": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!" }
                ],
                "m.poll": {
                    "question": { "m.text": [{ "body": "How's the weather?" }] },
                    "max_selections": 2,
                    "answers": [
                        {
                            "m.id": "not-bad",
                            "m.text": [{ "body": "Not bad…" }],
                        },
                        {
                            "m.id": "fine",
                            "m.text": [{ "body": "Fine." }],
                        },
                        {
                            "m.id": "amazing",
                            "m.text": [{ "body": "Amazing!" }],
                        },
                    ]
                },
            },
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": "$previous_event_id",
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "m.poll.start",
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
    assert_matches!(message_event.content.relates_to, Some(Relation::Replacement(_)));
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
            "m.selections": ["my-answer"],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        })
    );
}

#[test]
fn response_content_other_serialization() {
    let content = PollResponseEventContent::new(
        vec!["first-answer".to_owned(), "second-answer".to_owned()].into(),
        owned_event_id!("$related_event:notareal.hs"),
    );

    assert_eq!(
        to_json_value(&content).unwrap(),
        json!({
            "m.selections": ["first-answer", "second-answer"],
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
            "m.selections": ["my-answer"],
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
            "m.text":  [{ "body": "The poll has closed. Top answer: Amazing!" }],
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        })
    );
}

#[test]
fn end_content_with_results_serialization() {
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
            "m.text":  [{ "body": "The poll has closed. Top answer: Amazing!" }],
            "m.poll.results": {
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
            "m.text": [
                { "body": "The poll has closed. Top answer: Amazing!" },
            ],
            "m.poll.results": {
                "not-bad": 1,
                "fine": 5,
                "amazing": 14,
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
        "type": "m.poll.end",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(event, AnyMessageLikeEvent::PollEnd(MessageLikeEvent::Original(message_event)));
    assert_eq!(message_event.content.text[0].body, "The poll has closed. Top answer: Amazing!");
    assert_matches!(message_event.content.relates_to, Reference { event_id, .. });
    assert_eq!(event_id, "$related_event:notareal.hs");
}

#[test]
fn new_unstable_start_content_serialization() {
    let event_content = NewUnstablePollStartEventContent::plain_text(
        "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
        UnstablePollStartContentBlock::new(
            "How's the weather?",
            vec![
                UnstablePollAnswer::new("not-bad", "Not bad…"),
                UnstablePollAnswer::new("fine", "Fine."),
                UnstablePollAnswer::new("amazing", "Amazing!"),
            ]
            .try_into()
            .unwrap(),
        ),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
            "org.matrix.msc3381.poll.start": {
                "kind": "org.matrix.msc3381.poll.undisclosed",
                "max_selections": 1,
                "question": { "org.matrix.msc1767.text": "How's the weather?" },
                "answers": [
                    { "id": "not-bad", "org.matrix.msc1767.text": "Not bad…" },
                    { "id": "fine", "org.matrix.msc1767.text":  "Fine." },
                    { "id": "amazing", "org.matrix.msc1767.text":  "Amazing!" },
                ],
            },
        })
    );
}

#[test]
fn replacement_unstable_start_content_serialization() {
    let replaces = owned_event_id!("$replacedevent");
    let event_content = ReplacementUnstablePollStartEventContent::plain_text(
        "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
        UnstablePollStartContentBlock::new(
            "How's the weather?",
            vec![
                UnstablePollAnswer::new("not-bad", "Not bad…"),
                UnstablePollAnswer::new("fine", "Fine."),
                UnstablePollAnswer::new("amazing", "Amazing!"),
            ]
            .try_into()
            .unwrap(),
        ),
        replaces.clone(),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "m.new_content": {
                "org.matrix.msc1767.text": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
                "org.matrix.msc3381.poll.start": {
                    "kind": "org.matrix.msc3381.poll.undisclosed",
                    "max_selections": 1,
                    "question": { "org.matrix.msc1767.text": "How's the weather?" },
                    "answers": [
                        { "id": "not-bad", "org.matrix.msc1767.text": "Not bad…" },
                        { "id": "fine", "org.matrix.msc1767.text":  "Fine." },
                        { "id": "amazing", "org.matrix.msc1767.text":  "Amazing!" },
                    ],
                },
            },
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": replaces,
            },
        })
    );
}

#[test]
fn unstable_start_event_content_deserialization_missing_poll_start() {
    let json_data = json!({
            "org.matrix.msc1767.text": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
    });

    from_json_value::<UnstablePollStartEventContent>(json_data).unwrap_err();
}

#[test]
fn new_unstable_start_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
            "org.matrix.msc3381.poll.start": {
                "question": { "org.matrix.msc1767.text": "How's the weather?" },
                "max_selections": 2,
                "answers": [
                    {
                        "id": "not-bad",
                        "org.matrix.msc1767.text": "Not bad…",
                    },
                    {
                        "id": "fine",
                        "org.matrix.msc1767.text": "Fine.",
                    },
                    {
                        "id": "amazing",
                        "org.matrix.msc1767.text": "Amazing!",
                    },
                ]
            },
            "m.relates_to": {
                "rel_type": "m.thread",
                "event_id": "$previous_event_id",
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.poll.start",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::UnstablePollStart(MessageLikeEvent::Original(message_event))
    );
    assert_matches!(message_event.content, UnstablePollStartEventContent::New(content));

    assert_eq!(content.text.unwrap(), "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!");
    let poll = content.poll_start;
    assert_eq!(poll.question.text, "How's the weather?");
    assert_eq!(poll.kind, PollKind::Undisclosed);
    assert_eq!(poll.max_selections, uint!(2));
    let answers = poll.answers;
    assert_eq!(answers.len(), 3);
    assert_eq!(answers[0].id, "not-bad");
    assert_eq!(answers[0].text, "Not bad…");
    assert_eq!(answers[1].id, "fine");
    assert_eq!(answers[1].text, "Fine.");
    assert_eq!(answers[2].id, "amazing");
    assert_eq!(answers[2].text, "Amazing!");
    assert_matches!(content.relates_to, Some(RelationWithoutReplacement::Thread(_)));
}

#[test]
fn replacement_unstable_start_event_deserialization() {
    let json_data = json!({
        "content": {
            "m.new_content": {
                "org.matrix.msc1767.text": "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!",
                "org.matrix.msc3381.poll.start": {
                    "question": { "org.matrix.msc1767.text": "How's the weather?" },
                    "max_selections": 2,
                    "answers": [
                        {
                            "id": "not-bad",
                            "org.matrix.msc1767.text": "Not bad…",
                        },
                        {
                            "id": "fine",
                            "org.matrix.msc1767.text": "Fine.",
                        },
                        {
                            "id": "amazing",
                            "org.matrix.msc1767.text": "Amazing!",
                        },
                    ]
                },
            },
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": "$previous_event_id",
            },
        },
        "event_id": "$event:notareal.hs",
        "origin_server_ts": 134_829_848,
        "room_id": "!roomid:notareal.hs",
        "sender": "@user:notareal.hs",
        "type": "org.matrix.msc3381.poll.start",
    });

    let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
    assert_matches!(
        event,
        AnyMessageLikeEvent::UnstablePollStart(MessageLikeEvent::Original(message_event))
    );
    assert_matches!(message_event.content, UnstablePollStartEventContent::Replacement(content));
    assert!(content.text.is_none());
    assert!(content.poll_start.is_none());

    let new_content = content.relates_to.new_content;
    assert_eq!(new_content.text.unwrap(), "How's the weather?\n1. Not bad…\n2. Fine.\n3. Amazing!");
    let poll = new_content.poll_start;
    assert_eq!(poll.question.text, "How's the weather?");
    assert_eq!(poll.kind, PollKind::Undisclosed);
    assert_eq!(poll.max_selections, uint!(2));
    let answers = poll.answers;
    assert_eq!(answers.len(), 3);
    assert_eq!(answers[0].id, "not-bad");
    assert_eq!(answers[0].text, "Not bad…");
    assert_eq!(answers[1].id, "fine");
    assert_eq!(answers[1].text, "Fine.");
    assert_eq!(answers[2].id, "amazing");
    assert_eq!(answers[2].text, "Amazing!");
}

#[test]
fn unstable_response_content_serialization() {
    let event_content = UnstablePollResponseEventContent::new(
        vec!["my-answer".to_owned()],
        owned_event_id!("$related_event:notareal.hs"),
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
fn unstable_response_event_deserialization() {
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
    assert_matches!(
        event,
        AnyMessageLikeEvent::UnstablePollResponse(MessageLikeEvent::Original(message_event))
    );
    let selections = message_event.content.poll_response.answers;
    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0], "my-answer");
    assert_matches!(message_event.content.relates_to, Reference { event_id, .. });
    assert_eq!(event_id, "$related_event:notareal.hs");
}

#[test]
fn unstable_end_content_serialization() {
    let event_content = UnstablePollEndEventContent::new(
        "The poll has closed. Top answer: Amazing!",
        owned_event_id!("$related_event:notareal.hs"),
    );

    assert_eq!(
        to_json_value(&event_content).unwrap(),
        json!({
            "org.matrix.msc1767.text":  "The poll has closed. Top answer: Amazing!",
            "org.matrix.msc3381.poll.end": {},
            "m.relates_to": {
                "rel_type": "m.reference",
                "event_id": "$related_event:notareal.hs",
            }
        })
    );
}

#[test]
fn unstable_end_event_deserialization() {
    let json_data = json!({
        "content": {
            "org.matrix.msc1767.text":  "The poll has closed. Top answer: Amazing!",
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
    assert_matches!(
        event,
        AnyMessageLikeEvent::UnstablePollEnd(MessageLikeEvent::Original(message_event))
    );
    assert_eq!(message_event.content.text, "The poll has closed. Top answer: Amazing!");
    assert_matches!(message_event.content.relates_to, Reference { event_id, .. });
    assert_eq!(event_id, "$related_event:notareal.hs");
}

fn new_poll_response(
    event_id: &str,
    user_id: &str,
    ts: UInt,
    selections: &[&str],
) -> OriginalSyncPollResponseEvent {
    from_json_value(json!({
      "type": "m.poll.response",
      "sender": user_id,
      "origin_server_ts": ts,
      "event_id": event_id,
      "content": {
        "m.relates_to": {
          "rel_type": "m.reference",
          "event_id": "$poll_start_event_id"
        },
        "m.selections": selections,
      }
    }))
    .unwrap()
}

fn generate_poll_responses(
    range: Range<usize>,
    selections: &[&str],
) -> Vec<OriginalSyncPollResponseEvent> {
    let mut responses = Vec::with_capacity(range.len());

    for i in range {
        let event_id = format!("$valid_event_{i}");
        let user_id = format!("@valid_user_{i}:localhost");
        let ts = 1000 + i as u16;

        responses.push(new_poll_response(&event_id, &user_id, ts.into(), selections));
    }

    responses
}

#[test]
fn compute_results() {
    let poll: OriginalSyncPollStartEvent = from_json_value(json!({
        "type": "m.poll.start",
        "sender": "@alice:localhost",
        "event_id": "$poll_start_event_id",
        "origin_server_ts": 1,
        "content": {
          "m.text": [
            { "body": "What should we order for the party?\n1. Pizza 🍕\n2. Poutine 🍟\n3. Italian 🍝\n4. Wings 🔥" },
          ],
          "m.poll": {
            "kind": "m.disclosed",
            "max_selections": 2,
            "question": {
              "m.text": [{ "body": "What should we order for the party?" }],
            },
            "answers": [
              { "m.id": "pizza", "m.text":  [{ "body": "Pizza 🍕" }] },
              { "m.id": "poutine", "m.text":  [{ "body": "Poutine 🍟" }] },
              { "m.id": "italian", "m.text":  [{ "body": "Italian 🍝" }] },
              { "m.id": "wings", "m.text":  [{ "body": "Wings 🔥" }] },
            ]
          },
        }
      })).unwrap();

    // Populate responses.
    let mut responses = generate_poll_responses(0..5, &["pizza"]);
    responses.extend(generate_poll_responses(5..10, &["poutine"]));
    responses.extend(generate_poll_responses(10..15, &["italian"]));
    responses.extend(generate_poll_responses(15..20, &["wings"]));

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 5);
    assert_eq!(counted.get("poutine").unwrap().len(), 5);
    assert_eq!(counted.get("italian").unwrap().len(), 5);
    assert_eq!(counted.get("wings").unwrap().len(), 5);
    let mut iter = counted.keys();
    assert_eq!(iter.next(), Some(&"pizza"));
    assert_eq!(iter.next(), Some(&"poutine"));
    assert_eq!(iter.next(), Some(&"italian"));
    assert_eq!(iter.next(), Some(&"wings"));
    assert_eq!(iter.next(), None);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(5));
    assert_eq!(*results.get("poutine").unwrap(), uint!(5));
    assert_eq!(*results.get("italian").unwrap(), uint!(5));
    assert_eq!(*results.get("wings").unwrap(), uint!(5));
    assert_eq!(
        results.sorted().as_slice(),
        &[("italian", uint!(5)), ("pizza", uint!(5)), ("poutine", uint!(5)), ("wings", uint!(5))]
    );
    assert_eq!(
        poll_end.text.find_plain(),
        Some("The poll has closed. Top answers: Pizza 🍕, Poutine 🍟, Italian 🍝, Wings 🔥")
    );

    responses.extend(vec![
        new_poll_response(
            "$multi_event_1",
            "@multi_user_1:localhost",
            uint!(2000),
            &["poutine", "wings"],
        ),
        new_poll_response(
            "$multi_event_2",
            "@multi_user_2:localhost",
            uint!(2200),
            &["poutine", "italian"],
        ),
    ]);

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 5);
    assert_eq!(counted.get("poutine").unwrap().len(), 7);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 6);
    let mut iter = counted.keys();
    assert_eq!(iter.next(), Some(&"poutine"));
    assert_eq!(iter.next(), Some(&"italian"));
    assert_eq!(iter.next(), Some(&"wings"));
    assert_eq!(iter.next(), Some(&"pizza"));
    assert_eq!(iter.next(), None);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(5));
    assert_eq!(*results.get("poutine").unwrap(), uint!(7));
    assert_eq!(*results.get("italian").unwrap(), uint!(6));
    assert_eq!(*results.get("wings").unwrap(), uint!(6));
    assert_eq!(
        results.sorted().as_slice(),
        &[("poutine", uint!(7)), ("italian", uint!(6)), ("wings", uint!(6)), ("pizza", uint!(5))]
    );
    assert_eq!(poll_end.text.find_plain(), Some("The poll has closed. Top answer: Poutine 🍟"));

    responses.extend(vec![
        new_poll_response(
            "$multi_same_event_1",
            "@multi_same_user_1:localhost",
            uint!(3000),
            &["poutine", "poutine"],
        ),
        new_poll_response(
            "$multi_same_event_2",
            "@multi_same_user_2:localhost",
            uint!(3300),
            &["pizza", "pizza"],
        ),
    ]);

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 6);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 6);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(6));
    assert_eq!(*results.get("poutine").unwrap(), uint!(8));
    assert_eq!(*results.get("italian").unwrap(), uint!(6));
    assert_eq!(*results.get("wings").unwrap(), uint!(6));

    let changing_user_1 = "@changing_user_1:localhost";
    let changing_user_2 = "@changing_user_2:localhost";
    let changing_user_3 = "@changing_user_3:localhost";

    responses.extend(vec![
        new_poll_response("$valid_for_now_event_1", changing_user_1, uint!(4000), &["wings"]),
        new_poll_response("$valid_for_now_event_2", changing_user_2, uint!(4100), &["wings"]),
        new_poll_response("$valid_for_now_event_3", changing_user_3, uint!(4200), &["wings"]),
    ]);

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 6);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 9);
    let mut iter = counted.keys();
    assert_eq!(iter.next(), Some(&"wings"));
    assert_eq!(iter.next(), Some(&"poutine"));
    assert_eq!(iter.next(), Some(&"pizza"));
    assert_eq!(iter.next(), Some(&"italian"));
    assert_eq!(iter.next(), None);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(6));
    assert_eq!(*results.get("poutine").unwrap(), uint!(8));
    assert_eq!(*results.get("italian").unwrap(), uint!(6));
    assert_eq!(*results.get("wings").unwrap(), uint!(9));
    assert_eq!(
        results.sorted().as_slice(),
        &[("wings", uint!(9)), ("poutine", uint!(8)), ("italian", uint!(6)), ("pizza", uint!(6))]
    );
    assert_eq!(poll_end.text.find_plain(), Some("The poll has closed. Top answer: Wings 🔥"));

    // Change with new selection.
    responses.push(new_poll_response(
        "$change_vote_event",
        changing_user_1,
        uint!(4400),
        &["italian"],
    ));

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 6);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 7);
    assert_eq!(counted.get("wings").unwrap().len(), 8);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(6));
    assert_eq!(*results.get("poutine").unwrap(), uint!(8));
    assert_eq!(*results.get("italian").unwrap(), uint!(7));
    assert_eq!(*results.get("wings").unwrap(), uint!(8));

    // Change with no selection.
    responses.push(new_poll_response(
        "$no_selection_vote_event",
        changing_user_1,
        uint!(4500),
        &[],
    ));

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 6);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 8);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(6));
    assert_eq!(*results.get("poutine").unwrap(), uint!(8));
    assert_eq!(*results.get("italian").unwrap(), uint!(6));
    assert_eq!(*results.get("wings").unwrap(), uint!(8));

    // Change with invalid selection.
    responses.push(new_poll_response(
        "$invalid_vote_event",
        changing_user_2,
        uint!(4500),
        &["indian"],
    ));

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 6);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 7);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(6));
    assert_eq!(*results.get("poutine").unwrap(), uint!(8));
    assert_eq!(*results.get("italian").unwrap(), uint!(6));
    assert_eq!(*results.get("wings").unwrap(), uint!(7));

    // Response older than most recent one is ignored.
    responses.push(new_poll_response("$past_event", changing_user_3, uint!(1), &["pizza"]));

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 6);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 7);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(6));
    assert_eq!(*results.get("poutine").unwrap(), uint!(8));
    assert_eq!(*results.get("italian").unwrap(), uint!(6));
    assert_eq!(*results.get("wings").unwrap(), uint!(7));

    // Response later than end_timestamp is ignored.
    let now = MilliSecondsSinceUnixEpoch::now();
    let future_ts = now.0 + uint!(100_000);
    responses.push(new_poll_response("$future_event", changing_user_3, future_ts, &["pizza"]));

    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), Some(now));
    assert_eq!(counted.get("pizza").unwrap().len(), 6);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 7);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    let results = poll_end.poll_results.unwrap();
    assert_eq!(*results.get("pizza").unwrap(), uint!(6));
    assert_eq!(*results.get("poutine").unwrap(), uint!(8));
    assert_eq!(*results.get("italian").unwrap(), uint!(6));
    assert_eq!(*results.get("wings").unwrap(), uint!(7));

    // Response in the future is not ignored if there is no end_timestamp.
    let counted =
        compile_poll_results(&poll.content.poll, responses.iter().map(|r| r.data()), None);
    assert_eq!(counted.get("pizza").unwrap().len(), 7);
    assert_eq!(counted.get("poutine").unwrap().len(), 8);
    assert_eq!(counted.get("italian").unwrap().len(), 6);
    assert_eq!(counted.get("wings").unwrap().len(), 6);
}

fn new_unstable_poll_response(
    event_id: &str,
    user_id: &str,
    ts: UInt,
    selections: &[&str],
) -> OriginalSyncUnstablePollResponseEvent {
    from_json_value(json!({
      "type": "org.matrix.msc3381.poll.response",
      "sender": user_id,
      "origin_server_ts": ts,
      "event_id": event_id,
      "content": {
        "m.relates_to": {
          "rel_type": "m.reference",
          "event_id": "$poll_start_event_id"
        },
        "org.matrix.msc3381.poll.response": {
            "answers": selections,
        },
      }
    }))
    .unwrap()
}

fn generate_unstable_poll_responses(
    range: Range<usize>,
    selections: &[&str],
) -> Vec<OriginalSyncUnstablePollResponseEvent> {
    let mut responses = Vec::with_capacity(range.len());

    for i in range {
        let event_id = format!("$valid_event_{i}");
        let user_id = format!("@valid_user_{i}:localhost");
        let ts = 1000 + i as u16;

        responses.push(new_unstable_poll_response(&event_id, &user_id, ts.into(), selections));
    }

    responses
}

#[test]
fn compute_unstable_results() {
    let poll: OriginalSyncUnstablePollStartEvent = from_json_value(json!({
        "type": "org.matrix.msc3381.poll.start",
        "sender": "@alice:localhost",
        "event_id": "$poll_start_event_id",
        "origin_server_ts": 1,
        "content": {
          "org.matrix.msc1767.text": "What should we order for the party?\n1. Pizza 🍕\n2. Poutine 🍟\n3. Italian 🍝\n4. Wings 🔥",
          "org.matrix.msc3381.poll.start": {
            "kind": "org.matrix.msc3381.poll.disclosed",
            "max_selections": 2,
            "question": {
              "org.matrix.msc1767.text": "What should we order for the party?",
            },
            "answers": [
              { "id": "pizza", "org.matrix.msc1767.text": "Pizza 🍕" },
              { "id": "poutine", "org.matrix.msc1767.text": "Poutine 🍟" },
              { "id": "italian", "org.matrix.msc1767.text": "Italian 🍝" },
              { "id": "wings", "org.matrix.msc1767.text": "Wings 🔥" },
            ]
          },
        }
      })).unwrap();

    // Populate responses.
    let mut responses = generate_unstable_poll_responses(0..5, &["pizza"]);
    responses.extend(generate_unstable_poll_responses(5..6, &["poutine"]));
    responses.extend(generate_unstable_poll_responses(6..8, &["italian"]));
    responses.extend(generate_unstable_poll_responses(8..11, &["wings"]));

    let counted = compile_unstable_poll_results(
        poll.content.poll_start(),
        responses.iter().map(|r| r.data()),
        None,
    );
    assert_eq!(counted.get("pizza").unwrap().len(), 5);
    assert_eq!(counted.get("poutine").unwrap().len(), 1);
    assert_eq!(counted.get("italian").unwrap().len(), 2);
    assert_eq!(counted.get("wings").unwrap().len(), 3);
    let mut iter = counted.keys();
    assert_eq!(iter.next(), Some(&"pizza"));
    assert_eq!(iter.next(), Some(&"wings"));
    assert_eq!(iter.next(), Some(&"italian"));
    assert_eq!(iter.next(), Some(&"poutine"));
    assert_eq!(iter.next(), None);

    let poll_end = poll.compile_results(responses.iter().map(|r| r.data()));
    assert_eq!(poll_end.text, "The poll has closed. Top answer: Pizza 🍕");
}
