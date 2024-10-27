use serde::{Deserialize, Serialize};

use crate::room::{EncryptedFile, OwnedMxcUri, UInt};

/// The Source of the PreviewImage.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PreviewImageSource {
    #[serde(rename = "beeper:image:encryption", alias = "matrix:image:encryption")]
    EncryptedImage(EncryptedFile),
    #[serde(rename = "og:image", alias = "og:image:url")]
    Url(OwnedMxcUri),
}

/// Metadata and [`PreviewImageSource`] of an [`UrlPreview`] image.
///
/// Modelled after [OpenGraph Image Properties](https://ogp.me/#structured).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PreviewImage {
    /// Source information for the image.
    #[serde(flatten)]
    pub source: PreviewImageSource,

    /// The size of the image in bytes.
    #[serde(
        rename = "matrix:image:size",
        alias = "og:image:size",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<UInt>,

    /// The width of the image in pixels.
    #[serde(rename = "og:image:width", skip_serializing_if = "Option::is_none")]
    pub width: Option<UInt>,

    /// The height of the image in pixels.
    #[serde(rename = "og:image:height", skip_serializing_if = "Option::is_none")]
    pub height: Option<UInt>,

    /// The mime_type of the image.
    #[serde(rename = "og:image:type", skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,
}

impl PreviewImage {
    /// Construct a PreviewImage with the given [`OwnedMxcUri`] as the source.
    pub fn plain(url: OwnedMxcUri) -> Self {
        Self::with_image(PreviewImageSource::Url(url))
    }

    /// Construct the PreviewImage for the given [`EncryptedFile`] as the source.
    pub fn encrypted(file: EncryptedFile) -> Self {
        Self::with_image(PreviewImageSource::EncryptedImage(file))
    }

    fn with_image(source: PreviewImageSource) -> Self {
        PreviewImage { source, size: None, width: None, height: None, mimetype: None }
    }
}

/// Preview Information for a URL matched in the message's text, according to
/// [MSC 4095](https://github.com/matrix-org/matrix-spec-proposals/pull/4095).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct UrlPreview {
    /// The url this was matching on.
    #[serde(alias = "matrix:matched_url")]
    pub matched_url: Option<String>,

    /// Canonical URL according to open graph data.
    #[serde(rename = "og:url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Title to use for the preview.
    #[serde(rename = "og:title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description to use for the preview.
    #[serde(rename = "og:description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Metadata of a preview image if given.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub image: Option<PreviewImage>,
}

impl UrlPreview {
    /// Construct an preview for a matched_url.
    pub fn matched_url(matched_url: String) -> Self {
        UrlPreview {
            matched_url: Some(matched_url),
            url: None,
            image: None,
            description: None,
            title: None,
        }
    }

    /// Construct an preview for a canonical url.
    pub fn canonical_url(url: String) -> Self {
        UrlPreview {
            matched_url: None,
            url: Some(url),
            image: None,
            description: None,
            title: None,
        }
    }

    /// Whether this preview contains an actual preview or the users homeserver
    /// should be asked for preview data instead.
    pub fn contains_preview(&self) -> bool {
        self.url.is_some()
            || self.title.is_some()
            || self.description.is_some()
            || self.image.is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assert_matches2::assert_matches;
    use assign::assign;
    use js_int::uint;
    use ruma_common::{owned_mxc_uri, serde::Base64};
    use ruma_events::room::message::{MessageType, RoomMessageEventContent};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{super::text::TextMessageEventContent, *};
    use crate::room::{EncryptedFile, JsonWebKey};

    fn dummy_jwt() -> JsonWebKey {
        JsonWebKey {
            kty: "oct".to_owned(),
            key_ops: vec!["encrypt".to_owned(), "decrypt".to_owned()],
            alg: "A256CTR".to_owned(),
            k: Base64::new(vec![0; 64]),
            ext: true,
        }
    }

    fn encrypted_file() -> EncryptedFile {
        let mut hashes: BTreeMap<String, Base64> = BTreeMap::new();
        hashes.insert("sha256".to_owned(), Base64::new(vec![1; 10]));
        EncryptedFile {
            url: owned_mxc_uri!("mxc://localhost/encryptedfile"),
            key: dummy_jwt(),
            iv: Base64::new(vec![1; 12]),
            hashes,
            v: "v2".to_owned(),
        }
    }

    #[test]
    fn serialize_preview_image() {
        let expected_result = json!({
            "og:image": "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO"
        });

        let preview =
            PreviewImage::plain(owned_mxc_uri!("mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO"));

        assert_eq!(to_json_value(&preview).unwrap(), expected_result);

        let encrypted_result = json!({
            "beeper:image:encryption": {
                "hashes" : {
                    "sha256": "AQEBAQEBAQEBAQ",
                },
                "iv": "AQEBAQEBAQEBAQEB",
                "key": {
                    "alg": "A256CTR",
                    "ext": true,
                    "k": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                    "key_ops": [
                        "encrypt",
                        "decrypt"
                    ],
                    "kty": "oct",
                },
                "v": "v2",
                "url": "mxc://localhost/encryptedfile",
            },
        });

        let preview = PreviewImage::encrypted(encrypted_file());

        assert_eq!(to_json_value(&preview).unwrap(), encrypted_result);
    }

    #[test]
    fn serialize_room_message_with_url_preview() {
        let expected_result = json!({
            "msgtype": "m.text",
            "body": "Test message",
            "com.beeper.linkpreviews": [
                {
                    "matched_url": "https://matrix.org/",
                    "og:image": "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO",
                }
            ]
        });

        let preview_img =
            PreviewImage::plain(owned_mxc_uri!("mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO"));
        let full_preview = assign!(UrlPreview::matched_url("https://matrix.org/".to_owned()), {image: Some(preview_img)});
        let msg = MessageType::Text(assign!(TextMessageEventContent::plain("Test message"),  {
            url_previews: Some(vec![full_preview])
        }));

        assert_eq!(to_json_value(RoomMessageEventContent::new(msg)).unwrap(), expected_result);
    }

    #[test]
    fn serialize_room_message_with_url_preview_with_encrypted_image() {
        let expected_result = json!({
            "msgtype": "m.text",
            "body": "Test message",
            "com.beeper.linkpreviews": [
                {
                    "matched_url": "https://matrix.org/",
                    "beeper:image:encryption": {
                        "hashes" : {
                            "sha256": "AQEBAQEBAQEBAQ",
                        },
                        "iv": "AQEBAQEBAQEBAQEB",
                        "key": {
                            "alg": "A256CTR",
                            "ext": true,
                            "k": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                            "key_ops": [
                                "encrypt",
                                "decrypt"
                            ],
                            "kty": "oct",
                        },
                        "v": "v2",
                        "url": "mxc://localhost/encryptedfile",
                    }
                }
            ]
        });

        let preview_img = PreviewImage::encrypted(encrypted_file());
        let full_preview = assign!(UrlPreview::matched_url("https://matrix.org/".to_owned()), {
            image: Some(preview_img),
        });

        let msg = MessageType::Text(assign!(TextMessageEventContent::plain("Test message"),  {
            url_previews: Some(vec![full_preview])
        }));

        assert_eq!(to_json_value(RoomMessageEventContent::new(msg)).unwrap(), expected_result);
    }

    #[cfg(feature = "unstable-msc1767")]
    #[test]
    fn serialize_extensible_room_message_with_preview() {
        use crate::message::MessageEventContent;
        let expected_result = json!({
            "org.matrix.msc1767.text": [
                {"body": "matrix.org/support"}
            ],
            "com.beeper.linkpreviews": [
                {
                    "matched_url": "matrix.org/support",
                    "matrix:image:size": 16588,
                    "og:description": "Matrix, the open protocol for secure decentralised communications",
                    "og:image":"mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO",
                    "og:image:height": 400,
                    "og:image:type": "image/jpeg",
                    "og:image:width": 800,
                    "og:title": "Support Matrix",
                    "og:url": "https://matrix.org/support/"
                }
            ],
        });

        let preview_img = assign!(PreviewImage::plain(owned_mxc_uri!("mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO")), {
                height: Some(uint!(400)),
                width: Some(uint!(800)),
                mimetype: Some("image/jpeg".to_owned()),
                size: Some(uint!(16588))
        });
        let full_preview = assign!(UrlPreview::matched_url("matrix.org/support".to_owned()), {
                   image: Some(preview_img),
                   url: Some("https://matrix.org/support/".to_owned()),
                   title: Some("Support Matrix".to_owned()),
                   description: Some("Matrix, the open protocol for secure decentralised communications".to_owned()),
        });
        let msg = assign!(MessageEventContent::plain("matrix.org/support"),  {
            url_previews: Some(vec![full_preview])
        });
        assert_eq!(to_json_value(&msg).unwrap(), expected_result);
    }

    #[test]
    fn deserialize_regular_example() {
        let normal_preview = json!({
            "msgtype": "m.text",
            "body": "https://matrix.org",
            "m.url_previews": [
                {
                    "matrix:matched_url": "https://matrix.org",
                    "matrix:image:size": 16588,
                    "og:description": "Matrix, the open protocol for secure decentralised communications",
                    "og:image": "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO",
                    "og:image:height": 400,
                    "og:image:type": "image/jpeg",
                    "og:image:width": 800,
                    "og:title": "Matrix.org",
                    "og:url": "https://matrix.org/"
                }
            ],
            "m.mentions": {}
        });

        let message_with_preview: TextMessageEventContent =
            from_json_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let UrlPreview { image, matched_url, title, url, description } = previews.first().unwrap();
        assert_eq!(matched_url.as_ref().unwrap(), "https://matrix.org");
        assert_eq!(title.as_ref().unwrap(), "Matrix.org");
        assert_eq!(
            description.as_ref().unwrap(),
            "Matrix, the open protocol for secure decentralised communications"
        );
        assert_eq!(url.as_ref().unwrap(), "https://matrix.org/");

        // Check the preview image parsed:
        let PreviewImage { size, height, width, mimetype, source } = image.clone().unwrap();
        assert_eq!(size.unwrap(), uint!(16588));
        assert_matches!(source, PreviewImageSource::Url(url));
        assert_eq!(url.as_str(), "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO");
        assert_eq!(height.unwrap(), uint!(400));
        assert_eq!(width.unwrap(), uint!(800));
        assert_eq!(mimetype, Some("image/jpeg".to_owned()));
    }

    #[test]
    fn deserialize_under_dev_prefix() {
        let normal_preview = json!({
            "msgtype": "m.text",
            "body": "https://matrix.org",
            "com.beeper.linkpreviews": [
                {
                    "matched_url": "https://matrix.org",
                    "matrix:image:size": 16588,
                    "og:description": "Matrix, the open protocol for secure decentralised communications",
                    "og:image": "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO",
                    "og:image:height": 400,
                    "og:image:type": "image/jpeg",
                    "og:image:width": 800,
                    "og:title": "Matrix.org",
                    "og:url": "https://matrix.org/"
                }
            ],
            "m.mentions": {}
        });

        let message_with_preview: TextMessageEventContent =
            from_json_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let UrlPreview { image, matched_url, title, url, description } = previews.first().unwrap();
        assert_eq!(matched_url.as_ref().unwrap(), "https://matrix.org");
        assert_eq!(title.as_ref().unwrap(), "Matrix.org");
        assert_eq!(
            description.as_ref().unwrap(),
            "Matrix, the open protocol for secure decentralised communications"
        );
        assert_eq!(url.as_ref().unwrap(), "https://matrix.org/");

        // Check the preview image parsed:
        let PreviewImage { size, height, width, mimetype, source } = image.clone().unwrap();
        assert_eq!(size.unwrap(), uint!(16588));
        assert_matches!(source, PreviewImageSource::Url(url));
        assert_eq!(url.as_str(), "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO");
        assert_eq!(height.unwrap(), uint!(400));
        assert_eq!(width.unwrap(), uint!(800));
        assert_eq!(mimetype, Some("image/jpeg".to_owned()));
    }

    #[test]
    fn deserialize_example_no_previews() {
        let normal_preview = json!({
            "msgtype": "m.text",
            "body": "https://matrix.org",
            "m.url_previews": [],
            "m.mentions": {}
        });
        let message_with_preview: TextMessageEventContent =
            from_json_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        assert!(url_previews.clone().unwrap().is_empty(), "Unexpectedly found url previews");
    }

    #[test]
    fn deserialize_example_empty_previews() {
        let normal_preview = json!({
            "msgtype": "m.text",
            "body": "https://matrix.org",
            "m.url_previews": [
                { "matrix:matched_url": "https://matrix.org" }
            ],
            "m.mentions": {}
        });

        let message_with_preview: TextMessageEventContent =
            from_json_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let preview = previews.first().unwrap();
        assert_eq!(preview.matched_url.as_ref().unwrap(), "https://matrix.org");
        assert!(!preview.contains_preview());
    }

    #[test]
    fn deserialize_encrypted_image_dev_example() {
        let normal_preview = json!({
            "msgtype": "m.text",
            "body": "https://matrix.org",
            "com.beeper.linkpreviews": [
                {
                    "matched_url": "https://matrix.org",
                    "og:title": "Matrix.org",
                    "og:url": "https://matrix.org/",
                    "og:description": "Matrix, the open protocol for secure decentralised communications",
                    "matrix:image:size": 16588,
                    "og:image:height": 400,
                    "og:image:type": "image/jpeg",
                    "og:image:width": 800,
                    "beeper:image:encryption": {
                        "key": {
                            "k": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                            "alg": "A256CTR",
                            "ext": true,
                            "kty": "oct",
                            "key_ops": [
                                "encrypt",
                                "decrypt"
                            ]
                        },
                        "iv": "AQEBAQEBAQEBAQEB",
                        "hashes": {
                            "sha256": "AQEBAQEBAQEBAQ"
                        },
                        "v": "v2",
                        "url": "mxc://beeper.com/53207ac52ce3e2c722bb638987064bfdc0cc257b"
                    }
                }
            ],
            "m.mentions": {}
        });

        let message_with_preview: TextMessageEventContent =
            from_json_value(normal_preview).unwrap();
        let TextMessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let UrlPreview { image, matched_url, title, url, description } = previews.first().unwrap();
        assert_eq!(matched_url.as_ref().unwrap(), "https://matrix.org");
        assert_eq!(title.as_ref().unwrap(), "Matrix.org");
        assert_eq!(
            description.as_ref().unwrap(),
            "Matrix, the open protocol for secure decentralised communications"
        );
        assert_eq!(url.as_ref().unwrap(), "https://matrix.org/");

        // Check the preview image parsed:
        let PreviewImage { size, height, width, mimetype, source } = image.as_ref().unwrap();

        assert_eq!(size.unwrap(), uint!(16588));

        assert_matches!(source, PreviewImageSource::EncryptedImage(encrypted_image));
        assert_eq!(
            encrypted_image.url.as_str(),
            "mxc://beeper.com/53207ac52ce3e2c722bb638987064bfdc0cc257b"
        );
        assert_eq!(height.unwrap(), uint!(400));
        assert_eq!(width.unwrap(), uint!(800));
        assert_eq!(mimetype.as_ref().unwrap().as_str(), "image/jpeg");
    }

    #[test]
    #[cfg(feature = "unstable-msc1767")]
    fn deserialize_extensible_example() {
        use crate::message::MessageEventContent;
        let normal_preview = json!({
            "m.text": [
                {"body": "matrix.org/support"}
            ],
            "m.url_previews": [
                {
                    "matrix:matched_url": "matrix.org/support",
                    "matrix:image:size": 16588,
                    "og:description": "Matrix, the open protocol for secure decentralised communications",
                    "og:image": "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO",
                    "og:image:height": 400,
                    "og:image:type": "image/jpeg",
                    "og:image:width": 800,
                    "og:title": "Support Matrix",
                    "og:url": "https://matrix.org/support/"
                }
            ],
            "m.mentions": {}
        });

        let message_with_preview: MessageEventContent = from_json_value(normal_preview).unwrap();
        let MessageEventContent { url_previews, .. } = message_with_preview;
        let previews = url_previews.expect("No url previews found");
        assert_eq!(previews.len(), 1);
        let preview = previews.first().unwrap();
        assert!(preview.contains_preview());
        let UrlPreview { image, matched_url, title, url, description } = preview;
        assert_eq!(matched_url.as_ref().unwrap(), "matrix.org/support");
        assert_eq!(title.as_ref().unwrap(), "Support Matrix");
        assert_eq!(
            description.as_ref().unwrap(),
            "Matrix, the open protocol for secure decentralised communications"
        );
        assert_eq!(url.as_ref().unwrap(), "https://matrix.org/support/");

        // Check the preview image parsed:
        let PreviewImage { size, height, width, mimetype, source } = image.clone().unwrap();
        assert_eq!(size.unwrap(), uint!(16588));
        assert_matches!(source, PreviewImageSource::Url(url));
        assert_eq!(url.as_str(), "mxc://maunium.net/zeHhTqqUtUSUTUDxQisPdwZO");
        assert_eq!(height.unwrap(), uint!(400));
        assert_eq!(width.unwrap(), uint!(800));
        assert_eq!(mimetype, Some("image/jpeg".to_owned()));
    }
}
