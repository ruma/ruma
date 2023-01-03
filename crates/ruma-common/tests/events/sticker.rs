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
        })
    );
}

#[test]
fn content_deserialization() {
    let json_data = json!({
        "body": "Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "info": {},
    });

    let content = from_json_value::<StickerEventContent>(json_data).unwrap();
    assert_eq!(content.body, "Upload: my_image.jpg");
    assert_eq!(content.url, "mxc://notareal.hs/file");
}
