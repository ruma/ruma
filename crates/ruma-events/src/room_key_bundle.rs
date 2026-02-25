//! Types for the `m.room_key_bundle` event defined in [MSC4268].
//!
//! [MSC4268]: https://github.com/matrix-org/matrix-spec-proposals/pull/4268

use ruma_common::RoomId;
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
    pub room_id: RoomId,

    /// The location and encryption info of the key bundle.
    pub file: EncryptedFile,
}

impl ToDeviceRoomKeyBundleEventContent {
    /// Creates a new `ToDeviceRoomKeyBundleEventContent` with the given room ID, and
    /// [`EncryptedFile`] which contains the room keys from the bundle.
    pub fn new(room_id: RoomId, file: EncryptedFile) -> Self {
        Self { room_id, file }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ruma_common::{owned_mxc_uri, owned_room_id, serde::Base64};
    use serde_json::json;

    use super::ToDeviceRoomKeyBundleEventContent;
    use crate::room::{EncryptedFile, JsonWebKey};

    #[test]
    fn serialization() {
        let content = ToDeviceRoomKeyBundleEventContent {
            room_id: owned_room_id!("!testroomid:example.org"),
            file: EncryptedFile {
                url: owned_mxc_uri!("mxc://example.org/FHyPlCeYUSFFxlgbQYZmoEoe"),
                key: JsonWebKey {
                    kty: "A256CTR".to_owned(),
                    key_ops: vec!["encrypt".to_owned(), "decrypt".to_owned()],
                    alg: "A256CTR".to_owned(),
                    k: Base64::parse("aWF6-32KGYaC3A_FEUCk1Bt0JA37zP0wrStgmdCaW-0").unwrap(),
                    ext: true,
                },
                iv: Base64::parse("w+sE15fzSc0AAAAAAAAAAA").unwrap(),
                hashes: BTreeMap::from([(
                    "sha256".to_owned(),
                    Base64::parse("fdSLu/YkRx3Wyh3KQabP3rd6+SFiKg5lsJZQHtkSAYA").unwrap(),
                )]),
                v: "v2".to_owned(),
            },
        };

        let serialized = serde_json::to_value(content).unwrap();

        assert_eq!(
            serialized,
            json!({
                "room_id": "!testroomid:example.org",
                "file": {
                    "v": "v2",
                    "url": "mxc://example.org/FHyPlCeYUSFFxlgbQYZmoEoe",
                    "key": {
                        "alg": "A256CTR",
                        "ext": true,
                        "k": "aWF6-32KGYaC3A_FEUCk1Bt0JA37zP0wrStgmdCaW-0",
                        "key_ops": ["encrypt","decrypt"],
                        "kty": "A256CTR"
                    },
                    "iv": "w+sE15fzSc0AAAAAAAAAAA",
                    "hashes": {
                        "sha256": "fdSLu/YkRx3Wyh3KQabP3rd6+SFiKg5lsJZQHtkSAYA"
                    }
                }
            }),
            "The serialized value should match the declared JSON Value"
        );
    }
}
