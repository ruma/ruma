//! De-/serialization functions for `Option<MediaSource>` objects representing a thumbnail source.

use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Deserializer,
};

use crate::OwnedMxcUri;

use super::{EncryptedFile, MediaSource};

/// Serializes a MediaSource to a thumbnail source.
pub(crate) fn serialize<S>(source: &Option<MediaSource>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(source) = source {
        let mut st = serializer.serialize_struct("ThumbnailSource", 1)?;
        match source {
            MediaSource::Plain(url) => st.serialize_field("thumbnail_url", url)?,
            MediaSource::Encrypted(file) => st.serialize_field("thumbnail_file", file)?,
        }
        st.end()
    } else {
        serializer.serialize_none()
    }
}

/// Deserializes a thumbnail source to a MediaSource.
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<MediaSource>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct ThumbnailSourceJsonRepr {
        thumbnail_url: Option<OwnedMxcUri>,
        thumbnail_file: Option<Box<EncryptedFile>>,
    }

    match ThumbnailSourceJsonRepr::deserialize(deserializer)? {
        ThumbnailSourceJsonRepr { thumbnail_url: None, thumbnail_file: None } => Ok(None),
        // Prefer file if it is set
        ThumbnailSourceJsonRepr { thumbnail_file: Some(file), .. } => {
            Ok(Some(MediaSource::Encrypted(file)))
        }
        ThumbnailSourceJsonRepr { thumbnail_url: Some(url), .. } => {
            Ok(Some(MediaSource::Plain(url)))
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::{
        events::room::{EncryptedFileInit, JsonWebKeyInit, MediaSource},
        mxc_uri,
        serde::Base64,
    };

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct ThumbnailSourceTest {
        #[serde(flatten, with = "super", skip_serializing_if = "Option::is_none")]
        source: Option<MediaSource>,
    }

    #[test]
    fn deserialize_plain() {
        let json = json!({ "thumbnail_url": "mxc://notareal.hs/abcdef" });

        assert_matches!(
            serde_json::from_value::<ThumbnailSourceTest>(json),
            Ok(ThumbnailSourceTest { source: Some(MediaSource::Plain(url)) })
        );
        assert_eq!(url, "mxc://notareal.hs/abcdef");
    }

    #[test]
    fn deserialize_encrypted() {
        let json = json!({
            "thumbnail_file": {
                "url": "mxc://notareal.hs/abcdef",
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
        });

        assert_matches!(
            serde_json::from_value::<ThumbnailSourceTest>(json),
            Ok(ThumbnailSourceTest { source: Some(MediaSource::Encrypted(file)) })
        );
        assert_eq!(file.url, "mxc://notareal.hs/abcdef");
    }

    #[test]
    fn deserialize_none_by_absence() {
        let json = json!({});

        assert_matches!(
            serde_json::from_value::<ThumbnailSourceTest>(json).unwrap(),
            ThumbnailSourceTest { source: None }
        );
    }

    #[test]
    fn deserialize_none_by_null_plain() {
        let json = json!({ "thumbnail_url": null });

        assert_matches!(
            serde_json::from_value::<ThumbnailSourceTest>(json).unwrap(),
            ThumbnailSourceTest { source: None }
        );
    }

    #[test]
    fn deserialize_none_by_null_encrypted() {
        let json = json!({ "thumbnail_file": null });

        assert_matches!(
            serde_json::from_value::<ThumbnailSourceTest>(json).unwrap(),
            ThumbnailSourceTest { source: None }
        );
    }

    #[test]
    fn serialize_plain() {
        let request = ThumbnailSourceTest {
            source: Some(MediaSource::Plain(mxc_uri!("mxc://notareal.hs/abcdef").into())),
        };
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "thumbnail_url": "mxc://notareal.hs/abcdef" })
        );
    }

    #[test]
    fn serialize_encrypted() {
        let request = ThumbnailSourceTest {
            source: Some(MediaSource::Encrypted(Box::new(
                EncryptedFileInit {
                    url: mxc_uri!("mxc://notareal.hs/abcdef").to_owned(),
                    key: JsonWebKeyInit {
                        kty: "oct".to_owned(),
                        key_ops: vec!["encrypt".to_owned(), "decrypt".to_owned()],
                        alg: "A256CTR".to_owned(),
                        k: Base64::parse("TLlG_OpX807zzQuuwv4QZGJ21_u7weemFGYJFszMn9A").unwrap(),
                        ext: true,
                    }
                    .into(),
                    iv: Base64::parse("S22dq3NAX8wAAAAAAAAAAA").unwrap(),
                    hashes: [(
                        "sha256".to_owned(),
                        Base64::parse("aWOHudBnDkJ9IwaR1Nd8XKoI7DOrqDTwt6xDPfVGN6Q").unwrap(),
                    )]
                    .into(),
                    v: "v2".to_owned(),
                }
                .into(),
            ))),
        };
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "thumbnail_file": {
                    "url": "mxc://notareal.hs/abcdef",
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
            })
        );
    }

    #[test]
    fn serialize_none() {
        let request = ThumbnailSourceTest { source: None };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({}));
    }
}
