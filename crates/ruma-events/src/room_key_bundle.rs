//! Types for the `m.room_key_bundle` event defined in [MSC4268].
//!
//! [MSC4268]: https://github.com/matrix-org/matrix-spec-proposals/pull/4268

use ruma_common::OwnedRoomId;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::room::EncryptedFile;

/// The content of an `m.room_key_bundle` event.
///
/// Typically encrypted as an `m.room.encrypted` event, then sent as a to-device event.
///
/// This event is defined in [MSC4268](https://github.com/matrix-org/matrix-spec-proposals/pull/4268)
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "io.element.msc4268.room_key_bundle", alias = "m.room_key_bundle", kind = ToDevice)]
pub struct ToDeviceRoomKeyBundleEventContent {
    /// The room that these keys are for.
    pub room_id: OwnedRoomId,

    /// The location and encryption info of the key bundle.
    pub file: EncryptedFile,
}

impl ToDeviceRoomKeyBundleEventContent {
    /// Creates a new `ToDeviceRoomKeyBundleEventContent` with the given room ID, and
    /// [`EncryptedFile`] which contains the room keys from the bundle.
    pub fn new(room_id: OwnedRoomId, file: EncryptedFile) -> Self {
        Self { room_id, file }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_mxc_uri, owned_room_id, serde::Base64,
    };
    use serde_json::json;

    use super::ToDeviceRoomKeyBundleEventContent;
    use crate::room::{EncryptedFile, EncryptedFileHash, V2EncryptedFileInfo};

    #[test]
    fn serialization() {
        let content = ToDeviceRoomKeyBundleEventContent {
            room_id: owned_room_id!("!testroomid:example.org"),
            file: EncryptedFile::new(
                owned_mxc_uri!("mxc://example.org/FHyPlCeYUSFFxlgbQYZmoEoe"),
                V2EncryptedFileInfo::new(
                    Base64::parse("aWF6-32KGYaC3A_FEUCk1Bt0JA37zP0wrStgmdCaW-0").unwrap(),
                    Base64::parse("w+sE15fzSc0AAAAAAAAAAA").unwrap(),
                )
                .into(),
                std::iter::once(EncryptedFileHash::Sha256(
                    Base64::parse("fdSLu/YkRx3Wyh3KQabP3rd6+SFiKg5lsJZQHtkSAYA").unwrap(),
                ))
                .collect(),
            ),
        };

        assert_to_canonical_json_eq!(
            content,
            json!({
                "room_id": "!testroomid:example.org",
                "file": {
                    "v": "v2",
                    "url": "mxc://example.org/FHyPlCeYUSFFxlgbQYZmoEoe",
                    "key": {
                        "alg": "A256CTR",
                        "ext": true,
                        "k": "aWF6-32KGYaC3A_FEUCk1Bt0JA37zP0wrStgmdCaW-0",
                        "key_ops": ["decrypt", "encrypt"],
                        "kty": "oct"
                    },
                    "iv": "w+sE15fzSc0AAAAAAAAAAA",
                    "hashes": {
                        "sha256": "fdSLu/YkRx3Wyh3KQabP3rd6+SFiKg5lsJZQHtkSAYA"
                    }
                }
            }),
        );
    }
}
