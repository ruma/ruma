//! `GET /_matrix/client/*/room_keys/keys/{roomId}/{sessionId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3room_keyskeysroomidsessionid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        RoomId,
    };

    use crate::backup::KeyBackupData;

    const METADATA: Metadata = metadata! {
        description: "Retrieve a key from the backup for a given session.",
        method: GET,
        name: "get_backup_keys_for_session",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys/:room_id/:session_id",
            1.0 => "/_matrix/client/r0/room_keys/keys/:room_id/:session_id",
            1.1 => "/_matrix/client/v3/room_keys/keys/:room_id/:session_id",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
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

    #[response(error = crate::Error)]
    pub struct Response {
        /// Information about the requested backup key.
        #[ruma_api(body)]
        pub key_data: Raw<KeyBackupData>,
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
