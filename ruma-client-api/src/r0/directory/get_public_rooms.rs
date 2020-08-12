//! [GET /_matrix/client/r0/publicRooms](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-publicrooms)

use js_int::UInt;
use ruma_api::ruma_api;

use super::PublicRoomsChunk;

ruma_api! {
    metadata: {
        description: "Get the list of rooms in this homeserver's public directory.",
        method: GET,
        name: "get_public_rooms",
        path: "/_matrix/client/r0/publicRooms",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {
        /// Limit for the number of results to return.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub limit: Option<UInt>,

        /// Pagination token from a previous request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub since: Option<&'a str>,

        /// The server to fetch the public room lists from.
        ///
        /// `None` means the server this request is sent to.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub server: Option<&'a str>,
    }

    response: {
        /// A paginated chunk of public rooms.
        pub chunk: Vec<PublicRoomsChunk>,

        /// A pagination token for the response.
        pub next_batch: Option<String>,

        /// A pagination token that allows fetching previous results.
        pub prev_batch: Option<String>,

        /// An estimate on the total number of public rooms, if the server has an estimate.
        pub total_room_count_estimate: Option<UInt>,
    }

    error: crate::Error
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use js_int::uint;
    use ruma_api::OutgoingRequest as _;

    use super::{Request, Response};

    #[test]
    fn construct_request_from_refs() {
        let req: http::Request<Vec<u8>> =
            Request { limit: Some(uint!(10)), since: Some("hello"), server: Some("address") }
                .try_into_http_request("https://homeserver.tld", Some("auth_tok"))
                .unwrap();

        let uri = req.uri();
        let query = uri.query().unwrap();

        assert_eq!(uri.path(), "/_matrix/client/r0/publicRooms");
        assert!(query.contains("since=hello"));
        assert!(query.contains("limit=10"));
        assert!(query.contains("server=address"));
    }

    #[test]
    fn construct_response_from_refs() {
        let res: http::Response<Vec<u8>> = Response {
            chunk: vec![],
            next_batch: Some("next_batch_token".into()),
            prev_batch: Some("prev_batch_token".into()),
            total_room_count_estimate: Some(uint!(10)),
        }
        .try_into()
        .unwrap();

        assert_eq!(
            String::from_utf8_lossy(res.body()),
            r#"{"chunk":[],"next_batch":"next_batch_token","prev_batch":"prev_batch_token","total_room_count_estimate":10}"#
        );
    }
}
