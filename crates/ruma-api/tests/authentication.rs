mod access_token {
    use matches::assert_matches;
    use ruma_api::{ruma_api, Authentication, IncomingRequest as _, OutgoingRequest as _};

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo/:bar/:baz",
            rate_limited: false,
            authentication: AccessToken,
        }

        request: {}

        response: {}
    }

    #[test]
    fn extract_authentication() -> Result<(), Box<dyn std::error::Error + 'static>> {
        let req = Request {};

        let http_req = req
            .clone()
            .try_into_http_request("https://homeserver.tld", Some("header_access_token"))?;
        let (_, authentication) = Request::try_from_http_request(http_req)?;

        assert_matches!(
            authentication,
            Authentication::AccessToken(token)
            if token == "header_access_token"
        );

        Ok(())
    }
}

mod query_access_token {
    use matches::assert_matches;
    use ruma_api::{ruma_api, Authentication, IncomingRequest as _, OutgoingRequest};

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo",
            rate_limited: false,
            authentication: AccessToken,
        }

        request: {}

        response: {}
    }

    #[test]
    fn extract_authentication() -> Result<(), Box<dyn std::error::Error + 'static>> {
        let http_req = http::Request::builder()
            .method(<Request as OutgoingRequest>::METADATA.method)
            .uri(format!(
                "https://homeserver.tld{}?access_token=query_access_token",
                <Request as OutgoingRequest>::METADATA.path
            ))
            .body("{}".as_bytes().to_vec())?;
        let (_, authentication) = Request::try_from_http_request(http_req)?;

        assert_matches!(
            authentication,
            Authentication::AccessToken(token)
            if token == "query_access_token"
        );

        Ok(())
    }
}

mod query_only_access_token {
    use matches::assert_matches;
    use ruma_api::{ruma_api, Authentication, IncomingRequest as _, OutgoingRequest};

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo/:bar/:baz",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
        }

        request: {}

        response: {}
    }

    #[test]
    fn test_access_token() -> Result<(), Box<dyn std::error::Error + 'static>> {
        let http_req = http::Request::builder()
            .method(<Request as OutgoingRequest>::METADATA.method)
            .uri(format!(
                "https://homeserver.tld{}?access_token=query_only_access_token",
                <Request as OutgoingRequest>::METADATA.path
            ))
            .body("{}".as_bytes().to_vec())?;
        let (_, authentication) = Request::try_from_http_request(http_req)?;

        assert_matches!(
            authentication,
            Authentication::AccessToken(token)
            if token == "query_only_access_token"
        );

        Ok(())
    }
}

mod server_signatures {
    use matches::assert_matches;
    use ruma_api::{ruma_api, Authentication, IncomingRequest as _, OutgoingRequest as _};

    ruma_api! {
        metadata: {
            description: "Does something.",
            method: POST,
            name: "my_endpoint",
            path: "/_matrix/foo/:bar/:baz",
            rate_limited: false,
            authentication: ServerSignatures,
        }

        request: {}

        response: {}
    }

    #[test]
    fn extract_authentication() -> Result<(), Box<dyn std::error::Error + 'static>> {
        let req = Request {};

        let mut http_req = req.clone().try_into_http_request("https://homeserver.tld", None)?;
        let headers = http_req.headers_mut();
        headers.insert(
            http::header::AUTHORIZATION,
            "x-Matrix origin=theirserver.tld,key=ed25519:key1,signature=AAAAAA123232"
                .parse()
                .unwrap(),
        );

        let (_, authentication) = Request::try_from_http_request(http_req)?;

        assert_matches!(
            authentication,
            Authentication::ServerSignatures(headers)
            if headers[0].origin == "theirserver.tld"
                && headers[0].signature == "AAAAAA123232"
                && headers[0].key.as_ref() == "ed25519:key1"
        );

        Ok(())
    }
}
