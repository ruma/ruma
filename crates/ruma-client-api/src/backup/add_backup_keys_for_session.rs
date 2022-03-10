//! `PUT /_matrix/client/*/room_keys/keys/{roomId}/{sessionId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3room_keyskeysroomidsessionid

    use js_int::UInt;
    use ruma_common::{api::ruma_api, serde::Raw, RoomId};

    use crate::backup::KeyBackupData;

    ruma_api! {
        metadata: {
            description: "Store keys in the backup for a session.",
            method: PUT,
            name: "add_backup_keys_for_session",
            unstable_path: "/_matrix/client/unstable/room_keys/keys/:room_id/:session_id",
            r0_path: "/_matrix/client/r0/room_keys/keys/:room_id/:session_id",
            stable_path: "/_matrix/client/v3/room_keys/keys/:room_id/:session_id",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The backup version to add keys to.
            ///
            /// Must be the current backup.
            #[ruma_api(query)]
            pub version: &'a str,

            /// The ID of the room to add keys to.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The ID of the megolm session to add keys to.
            #[ruma_api(path)]
            pub session_id: &'a str,

            /// The key information to store.
            #[ruma_api(body)]
            pub session_data: Raw<KeyBackupData>,
        }

        response: {
            /// An opaque string representing stored keys in the backup.
            ///
            /// Clients can compare it with the etag value they received in the request of their last
            /// key storage request.
            pub etag: String,

            /// The number of keys stored in the backup.
            pub count: UInt,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given version, room_id, session_id and session_data.
        pub fn new(
            version: &'a str,
            room_id: &'a RoomId,
            session_id: &'a str,
            session_data: Raw<KeyBackupData>,
        ) -> Self {
            Self { version, room_id, session_id, session_data }
        }
    }

    impl Response {
        /// Creates an new `Response` with the given etag and count.
        pub fn new(etag: String, count: UInt) -> Self {
            Self { etag, count }
        }
    }
}
