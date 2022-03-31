#![cfg(feature = "unstable-msc3552")]

use ruma_common::{
    events::{room::ImageInfo, sticker::StickerEventContent},
    mxc_uri,
};
use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

#[test]
fn content_serialization() {
    let message_event_content = StickerEventContent::new(
        "Upload: my_image.jpg".to_owned(),
        ImageInfo::new(),
        mxc_uri!("mxc://notareal.hs/file").to_owned(),
    );

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "Upload: my_image.jpg",
            "url": "mxc://notareal.hs/file",
            "info": {},
            "org.matrix.msc1767.text": "Upload: my_image.jpg",
            "org.matrix.msc1767.file": {
                "url": "mxc://notareal.hs/file",
            },
            "org.matrix.msc1767.image": {},
        })
    );
}

#[test]
fn content_stable_deserialization() {
    let json_data = json!({
        "body": "Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "info": {},
        "m.text": "Upload: my_image.jpg",
        "m.file": {
            "url": "mxc://notareal.hs/file",
        },
        "m.image": {},
    });

    let content = from_json_value::<StickerEventContent>(json_data).unwrap();
    assert_eq!(content.body, "Upload: my_image.jpg");
    assert_eq!(content.url, "mxc://notareal.hs/file");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "Upload: my_image.jpg");
    let file = content.file.unwrap();
    assert_eq!(file.url, "mxc://notareal.hs/file");
    assert!(!file.is_encrypted());
}

#[test]
fn content_unstable_deserialization() {
    let json_data = json!({
        "body": "Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "info": {},
        "org.matrix.msc1767.text": "Upload: my_image.jpg",
        "org.matrix.msc1767.file": {
            "url": "mxc://notareal.hs/file",
        },
        "org.matrix.msc1767.image": {},
    });

    let content = from_json_value::<StickerEventContent>(json_data).unwrap();
    assert_eq!(content.body, "Upload: my_image.jpg");
    assert_eq!(content.url, "mxc://notareal.hs/file");
    let message = content.message.unwrap();
    assert_eq!(message.len(), 1);
    assert_eq!(message[0].body, "Upload: my_image.jpg");
    let file = content.file.unwrap();
    assert_eq!(file.url, "mxc://notareal.hs/file");
    assert!(!file.is_encrypted());
}
