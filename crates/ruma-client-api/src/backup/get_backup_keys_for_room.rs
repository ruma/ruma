//! `GET /_matrix/client/*/room_keys/keys/{roomId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3room_keyskeysroomid

    use std::collections::BTreeMap;

    use ruma_common::{api::ruma_api, serde::Raw, RoomId};

    use crate::backup::KeyBackupData;

    ruma_api! {
        metadata: {
            description: "Retrieve sessions from the backup for a given room.",
            method: GET,
            name: "get_backup_keys_for_room",
            unstable_path: "/_matrix/client/unstable/room_keys/keys/:room_id",
            r0_path: "/_matrix/client/r0/room_keys/keys/:room_id",
            stable_path: "/_matrix/client/v3/room_keys/keys/:room_id",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The backup version to retrieve keys from.
            #[ruma_api(query)]
            pub version: &'a str,

            /// The ID of the room that the requested key is for.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,
        }

        response: {
            /// A map of session IDs to key data.
            pub sessions: BTreeMap<String, Raw<KeyBackupData>>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given version and room_id.
        pub fn new(version: &'a str, room_id: &'a RoomId) -> Self {
            Self { version, room_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given sessions.
        pub fn new(sessions: BTreeMap<String, Raw<KeyBackupData>>) -> Self {
            Self { sessions }
        }
    }
}
