//! [POST /_matrix/client/r0/user/{userId}/filter](https://matrix.org/docs/spec/client_server/r0.6.1#post-matrix-client-r0-user-userid-filter)

use ruma_api::ruma_api;
use ruma_identifiers::UserId;

use super::{FilterDefinition, IncomingFilterDefinition};

ruma_api! {
    metadata: {
        description: "Create a new filter for event retrieval.",
        method: POST,
        name: "create_filter",
        path: "/_matrix/client/r0/user/:user_id/filter",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The ID of the user uploading the filter.
        ///
        /// The access token must be authorized to make requests for this user ID.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The filter definition.
        #[ruma_api(body)]
        pub filter: FilterDefinition<'a>,
    }

    response: {
        /// The ID of the filter that was created.
        pub filter_id: String,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID and filter definition.
    pub fn new(user_id: &'a UserId, filter: FilterDefinition<'a>) -> Self {
        Self { user_id, filter }
    }
}

impl Response {
    /// Creates a new `Response` with the given filter ID.
    pub fn new(filter_id: String) -> Self {
        Self { filter_id }
    }
}

#[cfg(all(test, any(feature = "client", feature = "server")))]
mod tests {
    use matches::assert_matches;

    #[cfg(feature = "server")]
    #[test]
    fn deserialize_request() {
        use ruma_api::IncomingRequest as _;

        use super::IncomingRequest;

        assert_matches!(
            IncomingRequest::try_from_http_request(
                http::Request::builder()
                    .method(http::Method::POST)
                    .uri("https://matrix.org/_matrix/client/r0/user/@foo:bar.com/filter")
                    .body(b"{}" as &[u8])
                    .unwrap(),
                &["@foo:bar.com"]
            ),
            Ok(IncomingRequest { user_id, filter })
            if user_id == "@foo:bar.com" && filter.is_empty()
        );
    }

    #[cfg(feature = "client")]
    #[test]
    fn serialize_request() {
        use ruma_api::{OutgoingRequest, SendAccessToken};
        use ruma_identifiers::user_id;

        use crate::r0::filter::FilterDefinition;

        assert_matches!(
            super::Request::new(user_id!("@foo:bar.com"), FilterDefinition::default())
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.org",
                    SendAccessToken::IfRequired("tok"),
                    ruma_api::EndpointPath::PreferStable
                ),
            Ok(res) if res.body() == b"{}"
        );
    }
}
