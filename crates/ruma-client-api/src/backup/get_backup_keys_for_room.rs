//! `GET /_matrix/client/*/room_keys/keys/{roomId}`
//!
//! Retrieve sessions from the backup for a given room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3room_keyskeysroomid

    use std::collections::BTreeMap;

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
            unstable => "/_matrix/client/unstable/room_keys/keys/{room_id}",
            1.0 => "/_matrix/client/r0/room_keys/keys/{room_id}",
            1.1 => "/_matrix/client/v3/room_keys/keys/{room_id}",
        }
    };

    /// Request type for the `get_backup_keys_for_room` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version to retrieve keys from.
        #[ruma_api(query)]
        pub version: String,

        /// The ID of the room that the requested key is for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,
    }

    /// Response type for the `get_backup_keys_for_room` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A map of session IDs to key data.
        pub sessions: BTreeMap<String, Raw<KeyBackupData>>,
    }

    impl Request {
        /// Creates a new `Request` with the given version and room_id.
        pub fn new(version: String, room_id: OwnedRoomId) -> Self {
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
