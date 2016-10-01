/// Details about this API endpoint.
pub struct Endpoint;

/// This API endpoint's response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// A list of Matrix client API protocol versions supported by the homeserver.
    pub versions: Vec<String>,
}

impl ::Endpoint for Endpoint {
    type BodyParams = ();
    type PathParams = ();
    type QueryParams = ();
    type Response = Response;

    fn method() -> ::Method {
        ::Method::Get
    }

    fn request_path(params: Self::PathParams) -> String {
        Self::router_path()
    }

    fn router_path() -> String {
        "/_matrix/client/versions".to_string()
    }
}
