//! `GET /_matrix/client/*/room_keys/version`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3room_keysversion

    use js_int::UInt;
    use ruma_common::{api::ruma_api, serde::Raw};
    use serde::{ser, Deserialize, Deserializer, Serialize};
    use serde_json::value::to_raw_value as to_raw_json_value;

    use crate::backup::{
        get_backup_info::v3::{AlgorithmWithData, RefResponseBodyRepr, ResponseBodyRepr},
        BackupAlgorithm,
    };

    ruma_api! {
        metadata: {
            description: "Get information about the latest backup.",
            method: GET,
            name: "get_latest_backup_info",
            unstable_path: "/_matrix/client/unstable/room_keys/version",
            r0_path: "/_matrix/client/r0/room_keys/version",
            stable_path: "/_matrix/client/v3/room_keys/version",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        #[ruma_api(manual_body_serde)]
        response: {
            /// The algorithm used for storing backups.
            pub algorithm: Raw<BackupAlgorithm>,

            /// The number of keys stored in the backup.
            pub count: UInt,

            /// An opaque string representing stored keys in the backup.
            ///
            /// Clients can compare it with the etag value they received in the request of their last
            /// key storage request.
            pub etag: String,

            /// The backup version.
            pub version: String,
        }

        error: crate::Error
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given algorithm, key count, etag and version.
        pub fn new(
            algorithm: Raw<BackupAlgorithm>,
            count: UInt,
            etag: String,
            version: String,
        ) -> Self {
            Self { algorithm, count, etag, version }
        }
    }

    impl<'de> Deserialize<'de> for ResponseBody {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let ResponseBodyRepr { algorithm, auth_data, count, etag, version } =
                ResponseBodyRepr::deserialize(deserializer)?;

            let algorithm = Raw::from_json(
                to_raw_json_value(&AlgorithmWithData { algorithm, auth_data }).unwrap(),
            );

            Ok(Self { algorithm, count, etag, version })
        }
    }

    impl Serialize for ResponseBody {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let ResponseBody { algorithm, count, etag, version } = self;
            let AlgorithmWithData { algorithm, auth_data } =
                algorithm.deserialize_as().map_err(ser::Error::custom)?;

            let repr = RefResponseBodyRepr {
                algorithm: &algorithm,
                auth_data: &auth_data,
                count: *count,
                etag,
                version,
            };

            repr.serialize(serializer)
        }
    }
}
