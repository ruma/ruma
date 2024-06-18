#![allow(clippy::exhaustive_structs)]

use http::header::CONTENT_TYPE;
use ruma_common::{
    api::{
        request, response, IncomingRequest as _, MatrixVersion, Metadata, OutgoingRequest as _,
        OutgoingRequestAppserviceExt, SendAccessToken,
    },
    metadata, owned_user_id, user_id, OwnedUserId,
};

const METADATA: Metadata = metadata! {
    method: POST,
    rate_limited: false,
    authentication: None,
    history: {
        unstable => "/_matrix/foo/{bar}/{user}",
    }
};

/// Request type for the `my_endpoint` endpoint.
#[request]
pub struct Request {
    pub hello: String,

    #[ruma_api(header = CONTENT_TYPE)]
    pub world: String,

    #[ruma_api(query)]
    pub q1: String,

    #[ruma_api(query)]
    pub q2: u32,

    #[ruma_api(path)]
    pub bar: String,

    #[ruma_api(path)]
    pub user: OwnedUserId,
}

/// Response type for the `my_endpoint` endpoint.
#[response]
pub struct Response {
    pub hello: String,

    #[ruma_api(header = CONTENT_TYPE)]
    pub world: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_flag: Option<bool>,
}

#[test]
fn request_serde() {
    let req = Request {
        hello: "hi".to_owned(),
        world: "test".to_owned(),
        q1: "query_param_special_chars %/&@!".to_owned(),
        q2: 55,
        bar: "barVal".to_owned(),
        user: owned_user_id!("@bazme:ruma.io"),
    };

    let http_req = req
        .clone()
        .try_into_http_request::<Vec<u8>>(
            "https://homeserver.tld",
            SendAccessToken::None,
            &[MatrixVersion::V1_1],
        )
        .unwrap();
    let req2 = Request::try_from_http_request(http_req, &["barVal", "@bazme:ruma.io"]).unwrap();

    assert_eq!(req.hello, req2.hello);
    assert_eq!(req.world, req2.world);
    assert_eq!(req.q1, req2.q1);
    assert_eq!(req.q2, req2.q2);
    assert_eq!(req.bar, req2.bar);
    assert_eq!(req.user, req2.user);
}

#[test]
fn invalid_uri_should_not_panic() {
    let req = Request {
        hello: "hi".to_owned(),
        world: "test".to_owned(),
        q1: "query_param_special_chars %/&@!".to_owned(),
        q2: 55,
        bar: "barVal".to_owned(),
        user: owned_user_id!("@bazme:ruma.io"),
    };

    let result = req.try_into_http_request::<Vec<u8>>(
        "invalid uri",
        SendAccessToken::None,
        &[MatrixVersion::V1_1],
    );
    result.unwrap_err();
}

#[test]
fn request_with_user_id_serde() {
    let req = Request {
        hello: "hi".to_owned(),
        world: "test".to_owned(),
        q1: "query_param_special_chars %/&@!".to_owned(),
        q2: 55,
        bar: "barVal".to_owned(),
        user: owned_user_id!("@bazme:ruma.io"),
    };

    let user_id = user_id!("@_virtual_:ruma.io");
    let http_req = req
        .try_into_http_request_with_user_id::<Vec<u8>>(
            "https://homeserver.tld",
            SendAccessToken::None,
            user_id,
            &[MatrixVersion::V1_1],
        )
        .unwrap();

    let query = http_req.uri().query().unwrap();

    assert_eq!(
        query,
        "q1=query_param_special_chars+%25%2F%26%40%21&q2=55&user_id=%40_virtual_%3Aruma.io"
    );
}

mod without_query {
    use http::header::CONTENT_TYPE;
    use ruma_common::{
        api::{
            request, response, MatrixVersion, Metadata, OutgoingRequestAppserviceExt,
            SendAccessToken,
        },
        metadata, owned_user_id, user_id, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/foo/{bar}/{user}",
        }
    };

    /// Request type for the `my_endpoint` endpoint.
    #[request]
    pub struct Request {
        pub hello: String,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,

        #[ruma_api(path)]
        pub bar: String,

        #[ruma_api(path)]
        pub user: OwnedUserId,
    }

    /// Response type for the `my_endpoint` endpoint.
    #[response]
    pub struct Response {
        pub hello: String,

        #[ruma_api(header = CONTENT_TYPE)]
        pub world: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub optional_flag: Option<bool>,
    }

    #[test]
    fn request_without_query_with_user_id_serde() {
        let req = Request {
            hello: "hi".to_owned(),
            world: "test".to_owned(),
            bar: "barVal".to_owned(),
            user: owned_user_id!("@bazme:ruma.io"),
        };

        let user_id = user_id!("@_virtual_:ruma.io");
        let http_req = req
            .try_into_http_request_with_user_id::<Vec<u8>>(
                "https://homeserver.tld",
                SendAccessToken::None,
                user_id,
                &[MatrixVersion::V1_1],
            )
            .unwrap();

        let query = http_req.uri().query().unwrap();

        assert_eq!(query, "user_id=%40_virtual_%3Aruma.io");
    }
}
