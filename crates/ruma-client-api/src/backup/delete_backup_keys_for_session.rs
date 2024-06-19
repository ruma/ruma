//! `DELETE /_matrix/client/*/room_keys/keys/{roomId}/{sessionId}`
//!
//! Delete keys from a backup for a given session.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#delete_matrixclientv3room_keyskeysroomidsessionid

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    const METADATA: Metadata = metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys/{room_id}/{session_id}",
            1.0 => "/_matrix/client/r0/room_keys/keys/{room_id}/{session_id}",
            1.1 => "/_matrix/client/v3/room_keys/keys/{room_id}/{session_id}",
        }
    };

    /// Request type for the `delete_backup_keys_for_session` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version from which to delete keys.
        #[ruma_api(query)]
        pub version: String,

        /// The ID of the room to delete keys from.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The ID of the megolm session to delete keys from.
        #[ruma_api(path)]
        pub session_id: String,
    }

    /// Response type for the `delete_backup_keys_for_session` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// An opaque string representing stored keys in the backup.
        ///
        /// Clients can compare it with  the etag value they received in the request of their last
        /// key storage request.
        pub etag: String,

        /// The number of keys stored in the backup.
        pub count: UInt,
    }

    impl Request {
        /// Creates a new `Request` with the given version, room_id and session_id.
        pub fn new(version: String, room_id: OwnedRoomId, session_id: String) -> Self {
            Self { version, room_id, session_id }
        }
    }

    impl Response {
        /// Creates an new `Response` with the given etag and count.
        pub fn new(etag: String, count: UInt) -> Self {
            Self { etag, count }
        }
    }
}
