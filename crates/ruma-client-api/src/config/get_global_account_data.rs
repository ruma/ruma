//! `GET /_matrix/client/*/user/{userId}/account_data/{type}`
//!
//! Gets global account data for a user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3useruseridaccount_datatype

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedUserId,
    };
    use ruma_events::{AnyGlobalAccountDataEventContent, GlobalAccountDataEventType};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/account_data/{event_type}",
            1.1 => "/_matrix/client/v3/user/{user_id}/account_data/{event_type}",
        }
    };

    /// Request type for the `get_global_account_data` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// User ID of user for whom to retrieve data.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// Type of data to retrieve.
        #[ruma_api(path)]
        pub event_type: GlobalAccountDataEventType,
    }

    /// Response type for the `get_global_account_data` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Account data content for the given type.
        ///
        /// Since the inner type of the `Raw` does not implement `Deserialize`, you need to use
        /// `.deserialize_as::<T>()` or `.cast_ref::<T>().deserialize_with_type()` for event
        /// types with a variable suffix (like [`SecretStorageKeyEventContent`]) to
        /// deserialize it.
        ///
        /// [`SecretStorageKeyEventContent`]: ruma_events::secret_storage::key::SecretStorageKeyEventContent
        #[ruma_api(body)]
        pub account_data: Raw<AnyGlobalAccountDataEventContent>,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and event type.
        pub fn new(user_id: OwnedUserId, event_type: GlobalAccountDataEventType) -> Self {
            Self { user_id, event_type }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given account data.
        pub fn new(account_data: Raw<AnyGlobalAccountDataEventContent>) -> Self {
            Self { account_data }
        }
    }
}
