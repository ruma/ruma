//! `GET /_matrix/app/*/media/preview_url`
//!
//! Endpoint to query a preview for a given url.

pub mod unstable {
    //! `msc4417` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4417

    use ruma_common::{
        OwnedUserId,
        api::{request, response},
        metadata,
    };
    use serde::Serialize;
    use serde_json::value::{RawValue as RawJsonValue, to_raw_value as to_raw_json_value};

    use crate::HomeserverToken;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: HomeserverToken,
        path: "/_matrix/app/unstable/uk.half-shot.msc4417/preview_url"
    }

    /// Request type for the `preview_url` endpoint.
    #[request]
    pub struct Request {
        /// URL to get a preview of.
        #[ruma_api(query)]
        pub url: String,

        /// UserID of the user requesting this preview.
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_id: Option<OwnedUserId>,
    }

    /// Response type for the `get_media_preview` endpoint.
    ///
    /// Equivalent to `ruma_client_api::authenticated_media::get_media_preview::v1::Response`
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// OpenGraph-like data for the URL.
        ///
        /// Differences from OpenGraph: the image size in bytes is added to the `matrix:image:size`
        /// field, and `og:image` returns the MXC URI to the image, if any.
        #[ruma_api(body)]
        pub data: Option<Box<RawJsonValue>>,
    }

    impl Request {
        /// Creates a new `Request` with the given URL.
        pub fn new(url: String) -> Self {
            Self { url, user_id: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self { data: None }
        }

        /// Creates a new `Response` with the given OpenGraph data (in a
        /// `serde_json::value::RawValue`).
        pub fn from_raw_value(data: Box<RawJsonValue>) -> Self {
            Self { data: Some(data) }
        }

        /// Creates a new `Response` with the given OpenGraph data (in any kind of serializable
        /// object).
        pub fn from_serialize<T: Serialize>(data: &T) -> serde_json::Result<Self> {
            Ok(Self { data: Some(to_raw_json_value(data)?) })
        }
    }
}
