/// The HTTP method.
pub const METHOD: &'static str = "GET";

/// The URL's path component.
pub const PATH: &'static str = "/versions";

/// The response type.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// A list of Matrix client API protocol versions supported by the homeserver.
    pub versions: Vec<String>,
}
