//! `GET /_matrix/client/*/room_keys/keys/{roomId}/{sessionId}`
//!
//! Retrieve a key from the backup for a given session.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3room_keyskeysroomidsessionid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId,
    };

    use crate::backup::KeyBackupData;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys/{room_id}/{session_id}",
            1.0 => "/_matrix/client/r0/room_keys/keys/{room_id}/{session_id}",
            1.1 => "/_matrix/client/v3/room_keys/keys/{room_id}/{session_id}",
        }
    };

    /// Request type for the `get_backup_keys_for_session` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version to retrieve keys from.
        #[ruma_api(query)]
        pub version: String,

        /// The ID of the room that the requested key is for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The ID of the megolm session whose key is requested.
        #[ruma_api(path)]
        pub session_id: String,
    }

    /// Response type for the `get_backup_keys_for_session` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Information about the requested backup key.
        #[ruma_api(body)]
        pub key_data: Raw<KeyBackupData>,
    }

    impl Request {
        /// Creates a new `Request` with the given version, room_id and session_id.
        pub fn new(version: String, room_id: OwnedRoomId, session_id: String) -> Self {
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
