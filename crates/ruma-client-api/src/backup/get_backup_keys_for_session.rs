//! `GET /_matrix/client/*/room_keys/keys/{roomId}/{sessionId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3room_keyskeysroomidsessionid

    use ruma_common::{api::ruma_api, serde::Raw, RoomId};

    use crate::backup::KeyBackupData;

    ruma_api! {
        metadata: {
            description: "Retrieve a key from the backup for a given session.",
            method: GET,
            name: "get_backup_keys_for_session",
            unstable_path: "/_matrix/client/unstable/room_keys/keys/:room_id/:session_id",
            r0_path: "/_matrix/client/r0/room_keys/keys/:room_id/:session_id",
            stable_path: "/_matrix/client/v3/room_keys/keys/:room_id/:session_id",
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

            /// The ID of the megolm session whose key is requested.
            #[ruma_api(path)]
            pub session_id: &'a str,
        }

        response: {
            /// Information about the requested backup key.
            #[ruma_api(body)]
            pub key_data: Raw<KeyBackupData>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given version, room_id and session_id.
        pub fn new(version: &'a str, room_id: &'a RoomId, session_id: &'a str) -> Self {
            Self { version, room_id, session_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given key_data.
        pub fn new(key_data: Raw<KeyBackupData>) -> Self {
            Self { key_data }
        }
    }
}
