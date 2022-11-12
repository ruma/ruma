//! `PUT /_matrix/client/*/user/{userId}/account_data/{type}`
//!
//! Sets global account data.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3useruseridaccount_datatype

    use ruma_common::{
        api::{request, response, Metadata},
        events::{
            AnyGlobalAccountDataEventContent, GlobalAccountDataEventContent,
            GlobalAccountDataEventType,
        },
        metadata,
        serde::Raw,
        UserId,
    };
    use serde_json::value::to_raw_value as to_raw_json_value;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/:user_id/account_data/:event_type",
            1.1 => "/_matrix/client/v3/user/:user_id/account_data/:event_type",
        }
    };

    /// Request type for the `set_global_account_data` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The ID of the user to set account_data for.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The event type of the account_data to set.
        ///
        /// Custom types should be namespaced to avoid clashes.
        #[ruma_api(path)]
        pub event_type: GlobalAccountDataEventType,

        /// Arbitrary JSON to store as config data.
        ///
        /// To create a `RawJsonValue`, use `serde_json::value::to_raw_value`.
        #[ruma_api(body)]
        pub data: Raw<AnyGlobalAccountDataEventContent>,
    }

    /// Response type for the `set_global_account_data` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given data, event type and user ID.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`Serialize`][serde::Serialize] implementation can fail.
        pub fn new<T>(user_id: &'a UserId, data: &'a T) -> serde_json::Result<Self>
        where
            T: GlobalAccountDataEventContent,
        {
            Ok(Self {
                user_id,
                event_type: data.event_type(),
                data: Raw::from_json(to_raw_json_value(data)?),
            })
        }

        /// Creates a new `Request` with the given raw data, event type and user ID.
        pub fn new_raw(
            user_id: &'a UserId,
            event_type: GlobalAccountDataEventType,
            data: Raw<AnyGlobalAccountDataEventContent>,
        ) -> Self {
            Self { user_id, event_type, data }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
