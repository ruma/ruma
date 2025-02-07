use assert_matches2::assert_matches;
use assign::assign;
use js_int::{uint, UInt};
use ruma_common::{mxc_uri, owned_event_id, serde::CanBeEmpty, MilliSecondsSinceUnixEpoch};
use ruma_events::{
    relation::Replacement,
    room::{message::Relation, ImageInfo, MediaSource, ThumbnailInfo},
    sticker::{StickerEventContent, StickerEventContentWithoutRelation, StickerMediaSource},
    AnyMessageLikeEvent, MessageLikeEvent,
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
fn replace_content_serialization() {
    let mut message_event_content = StickerEventContent::new(
        "* Upload: my_image.jpg".to_owned(),
        ImageInfo::new(),
        mxc_uri!("mxc://notareal.hs/file").to_owned(),
    );
    let old_event_id = owned_event_id!("$15827405538098VGFWH:example.com");
    let new_message_event_content = StickerEventContent::new(
        "Upload: my_image.jpg".to_owned(),
        ImageInfo::new(),
        mxc_uri!("mxc://notareal.hs/file").to_owned(),
    );
    let new_content = StickerEventContentWithoutRelation::from(new_message_event_content);
    let replacement = Replacement::new(old_event_id.clone(), new_content);
    let relation = Relation::Replacement(replacement);
    message_event_content.relates_to = Some(relation);

    assert_eq!(
        to_json_value(&message_event_content).unwrap(),
        json!({
            "body": "* Upload: my_image.jpg",
            "url": "mxc://notareal.hs/file",
            "info": {},
            "m.new_content": {
                "body": "Upload: my_image.jpg",
                "url": "mxc://notareal.hs/file",
                "info": {},
            },
            "m.relates_to": {
                "rel_type": "m.replace",
                "event_id": old_event_id,
            }
        })
    );
}

#[test]
fn event_serialization() {
    let content = StickerEventContent::new(
        "Hello".into(),
        assign!(ImageInfo::new(), {
            height: UInt::new(423),
            width: UInt::new(1011),
            mimetype: Some("image/png".into()),
            size: UInt::new(84242),
            thumbnail_info: Some(Box::new(assign!(ThumbnailInfo::new(), {
                width: UInt::new(800),
                height: UInt::new(334),
                mimetype: Some("image/png".into()),
                size: UInt::new(82595),
            }))),
            thumbnail_source: Some(MediaSource::Plain(mxc_uri!("mxc://matrix.org/irsns989Rrsn").to_owned())),
        }),
        mxc_uri!("mxc://matrix.org/rnsldl8srs98IRrs").to_owned(),
    );

    let actual = to_json_value(&content).unwrap();

    let expected = json!({
        "body": "Hello",
        "info": {
            "h": 423,
            "mimetype": "image/png",
            "size": 84242,
            "thumbnail_info": {
                "h": 334,
                "mimetype": "image/png",
                "size": 82595,
                "w": 800
            },
            "thumbnail_url": "mxc://matrix.org/irsns989Rrsn",
            "w": 1011
            },
        "url": "mxc://matrix.org/rnsldl8srs98IRrs",
    });
    assert_eq!(actual, expected);
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
    assert_matches!(content.source, StickerMediaSource::Plain(sticker_url));
    assert_eq!(sticker_url, "mxc://notareal.hs/file");

    let encrypted_json_data = json!({
        "body": "Upload: my_image.jpg",
        "file": {
            "url": "mxc://notareal.hs/file",
            "key": {
                "kty": "oct",
                "key_ops": ["encrypt", "decrypt"],
                "alg": "A256CTR",
                "k": "TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A",
                "ext": true
            },
            "iv": "S22dq3NAX8wAAAAAAAAAAA",
            "hashes": {
                "sha256": "aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q"
            },
            "v": "v2",
        },
        "info": {},
    });

    #[cfg(not(feature = "compat-encrypted-stickers"))]
    {
        from_json_value::<StickerEventContent>(encrypted_json_data).unwrap_err();
    }
    #[cfg(feature = "compat-encrypted-stickers")]
    {
        let encrypted_content =
            from_json_value::<StickerEventContent>(encrypted_json_data).unwrap();
        assert_eq!(encrypted_content.body, "Upload: my_image.jpg");
        assert_matches!(
            encrypted_content.source,
            StickerMediaSource::Encrypted(encrypted_sticker_url)
        );
        assert_eq!(encrypted_sticker_url.url, "mxc://notareal.hs/file");
    }
}

#[test]
fn replace_content_deserialization() {
    let old_event_id = owned_event_id!("$15827405538098VGFWH:example.com");
    let json_data = json!({
        "body": "* Upload: my_image.jpg",
        "url": "mxc://notareal.hs/file",
        "info": {},
        "m.new_content": {
            "body": "Upload: my_image.jpg",
            "url": "mxc://notareal.hs/file",
            "info": {},
        },
        "m.relates_to": {
            "rel_type": "m.replace",
            "event_id": old_event_id,
        }
    });

    let content = from_json_value::<StickerEventContent>(json_data).unwrap();
    assert_eq!(content.body, "* Upload: my_image.jpg");
    assert_matches!(content.source, StickerMediaSource::Plain(sticker_url));
    assert_eq!(sticker_url, "mxc://notareal.hs/file");

    assert_matches!(content.relates_to, Some(Relation::Replacement(replacement)));
    assert_eq!(replacement.new_content.body, "Upload: my_image.jpg");
    assert_matches!(replacement.new_content.source, StickerMediaSource::Plain(sticker_url));
    assert_eq!(sticker_url, "mxc://notareal.hs/file");

    let encrypted_json_data = json!({
        "body": "* Upload: my_image.jpg",
        "file": {
            "url": "mxc://notareal.hs/file",
            "key": {
                "kty": "oct",
                "key_ops": ["encrypt", "decrypt"],
                "alg": "A256CTR",
                "k": "TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A",
                "ext": true
            },
            "iv": "S22dq3NAX8wAAAAAAAAAAA",
            "hashes": {
                "sha256": "aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q"
            },
            "v": "v2",
        },
        "m.new_content": {
            "body": "Upload: my_image.jpg",
            "url": "mxc://notareal.hs/file",
            "info": {},
        },
        "m.relates_to": {
            "event_id": old_event_id,
            "rel_type": "m.replace"
        },
        "info": {},
    });

    #[cfg(not(feature = "compat-encrypted-stickers"))]
    {
        from_json_value::<StickerEventContent>(encrypted_json_data).unwrap_err();
    }
    #[cfg(feature = "compat-encrypted-stickers")]
    {
        let encrypted_content =
            from_json_value::<StickerEventContent>(encrypted_json_data).unwrap();
        assert_eq!(encrypted_content.body, "* Upload: my_image.jpg");
        assert_matches!(
            encrypted_content.source,
            StickerMediaSource::Encrypted(encrypted_sticker_url)
        );
        assert_eq!(encrypted_sticker_url.url, "mxc://notareal.hs/file");

        assert_matches!(
            encrypted_content.relates_to,
            Some(Relation::Replacement(encrypted_replacement))
        );
        assert_eq!(encrypted_replacement.new_content.body, "Upload: my_image.jpg");
        assert_matches!(
            encrypted_replacement.new_content.source,
            StickerMediaSource::Plain(encrypted_sticker_url)
        );
        assert_eq!(encrypted_sticker_url, "mxc://notareal.hs/file");
    }
}

#[test]
fn event_deserialization() {
    let json_data = json!({
        "content": {
            "body": "Hello",
            "info": {
                "h": 423,
                "mimetype": "image/png",
                "size": 84242,
                "thumbnail_info": {
                  "h": 334,
                  "mimetype": "image/png",
                  "size": 82595,
                  "w": 800
                },
                "thumbnail_url": "mxc://matrix.org/irnsNRS2879",
                "w": 1011
              },
            "url": "mxc://matrix.org/jxPXTKpyydzdHJkdFNZjTZrD"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "type": "m.sticker"
    });

    assert_matches!(
        from_json_value::<AnyMessageLikeEvent>(json_data),
        Ok(AnyMessageLikeEvent::Sticker(MessageLikeEvent::Original(message_event)))
    );

    assert_eq!(message_event.event_id, "$h29iv0s8:example.com");
    assert_eq!(message_event.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(message_event.room_id, "!roomid:room.com");
    assert_eq!(message_event.sender, "@carl:example.com");
    assert!(message_event.unsigned.is_empty());

    let content = message_event.content;
    assert_eq!(content.body, "Hello");
    assert_eq!(content.info.height, Some(uint!(423)));
    assert_eq!(content.info.width, Some(uint!(1011)));
    assert_eq!(content.info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(content.info.size, Some(uint!(84242)));
    assert_matches!(content.source, StickerMediaSource::Plain(sticker_url));
    assert_eq!(sticker_url, "mxc://matrix.org/jxPXTKpyydzdHJkdFNZjTZrD");

    assert_matches!(content.info.thumbnail_source, Some(MediaSource::Plain(thumbnail_url)));
    assert_eq!(thumbnail_url, "mxc://matrix.org/irnsNRS2879");
    let thumbnail_info = content.info.thumbnail_info.unwrap();
    assert_eq!(thumbnail_info.width, Some(uint!(800)));
    assert_eq!(thumbnail_info.height, Some(uint!(334)));
    assert_eq!(thumbnail_info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(thumbnail_info.size, Some(uint!(82595)));
}
