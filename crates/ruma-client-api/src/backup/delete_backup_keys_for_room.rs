//! `DELETE /_matrix/client/*/room_keys/keys/{roomId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#delete_matrixclientv3room_keyskeysroomid

    use js_int::UInt;
    use ruma_common::{api::ruma_api, RoomId};

    ruma_api! {
        metadata: {
            description: "Delete keys from a backup for a given room.",
            method: DELETE,
            name: "delete_backup_keys_for_room",
            unstable_path: "/_matrix/client/unstable/room_keys/keys/:room_id",
            r0_path: "/_matrix/client/r0/room_keys/keys/:room_id",
            stable_path: "/_matrix/client/v3/room_keys/keys/:room_id",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The backup version from which to delete keys.
            #[ruma_api(query)]
            pub version: &'a str,

            /// The ID of the room to delete keys from.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,
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
        /// Creates a new `Request` with the given version and room_id.

        pub fn new(version: &'a str, room_id: &'a RoomId) -> Self {
            Self { version, room_id }
        }
    }

    impl Response {
        /// Creates an new `Response` with the given etag and count.
        pub fn new(etag: String, count: UInt) -> Self {
            Self { etag, count }
        }
    }
}
