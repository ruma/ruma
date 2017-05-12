//! Endpoints that cannot change with new versions of the Matrix specification.

/// [GET /_matrix/client/versions](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-versions)
pub mod get_supported_versions {
    use std::convert::TryFrom;

    use hyper::{Method, Request as HyperRequest, Response as HyperResponse, StatusCode};
    use ruma_api::{Endpoint as ApiEndpoint, Metadata};

    use Error;

    /// Endpoint
    #[derive(Debug)]
    pub struct Endpoint;

    impl ApiEndpoint for Endpoint {
        type Request = Request;
        type Response = Response;

        const METADATA: Metadata = Metadata {
            description: "Get the versions of the client-server API supported by this homeserver.",
            method: Method::Get,
            name: "api_versions",
            path: "/_matrix/client/versions",
            rate_limited: false,
            requires_authentication: true,
        };
    }

    /// Request
    #[derive(Debug, Clone)]
    pub struct Request;

    impl TryFrom<Request> for HyperRequest {
        type Error = Error;
        fn try_from(_request: Request) -> Result<HyperRequest, Self::Error> {
            let metadata = Endpoint::METADATA;

            let hyper_request = HyperRequest::new(
                metadata.method,
                metadata.path.parse().map_err(|_| Error)?,
            );

            Ok(hyper_request)
        }
    }

    /// Response
    #[derive(Debug, Clone)]
    pub struct Response {
        /// A list of Matrix client API protocol versions supported by the homeserver.
        pub versions: Vec<String>,
    }

    impl TryFrom<HyperResponse> for Response {
        type Error = Error;

        fn try_from(hyper_response: HyperResponse) -> Result<Response, Self::Error> {
            if hyper_response.status() == StatusCode::Ok {
                Ok(Response { versions: vec![] })
            } else {
                Err(Error)
            }
        }
    }
}
