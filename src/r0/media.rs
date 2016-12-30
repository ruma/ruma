//! Endpoints for the media repository.

/// [GET /_matrix/media/r0/download/{serverName}/{mediaId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-media-r0-download-servername-mediaid)
pub mod get_content {
    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug)]
    pub struct PathParams {
        /// The media ID from the mxc:// URI (the path component).
        pub media_id: String,
        /// The server name from the mxc:// URI (the authoritory component).
        pub server_name: String,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = (); // TODO: How should a file be represented as a response?

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/media/r0/download/{}/{}",
                params.server_name,
                params.media_id
            )
        }

        fn router_path() -> String {
            "/_matrix/media/r0/download/:server_name/:media_id".to_string()
        }
    }
}

/// [POST /_matrix/media/r0/upload](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-media-r0-upload)
pub mod create_content {
    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        /// The MXC URI for the uploaded content.
        pub content_uri: String,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = (); // TODO: How should a file be represented as the request body?
        type PathParams = ();
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path()
        }

        fn router_path() -> String {
            "/_matrix/media/r0/upload".to_string()
        }
    }
}

/// [GET /_matrix/media/r0/thumbnail/{serverName}/{mediaId}](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-media-r0-thumbnail-servername-mediaid)
pub mod get_content_thumbnail {
    use std::fmt::{Display, Error as FmtError, Formatter};

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// The desired resizing method.
    #[derive(Clone, Copy, Debug)]
    pub enum Method {
        /// Crop the original to produce the requested image dimensions.
        Crop,
        /// Maintain the original aspect ratio of the source image.
        Scale,
    }

    /// This API endpoint's path parameters.
    pub struct PathParams {
        /// The media ID from the mxc:// URI (the path component).
        pub media_id: String,
        /// The server name from the mxc:// URI (the authoritory component).
        pub server_name: String,
    }

    /// This API endpoint's query string parameters.
    #[derive(Clone, Debug)]
    pub struct QueryParams {
        /// The *desired* height of the thumbnail. The actual thumbnail may not match the size
        /// specified.
        pub height: Option<u64>,
        /// The desired resizing method.
        pub method: Option<Method>,
        /// The *desired* width of the thumbnail. The actual thumbnail may not match the size
        /// specified.
        pub width: Option<u64>,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = QueryParams;
        type Response = (); // TODO: How should a file be represented as a response?

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/media/r0/thumbnail/{}/{}",
                params.server_name,
                params.media_id
            )
        }

        fn router_path() -> String {
            "/_matrix/media/r0/thumbnail/:server_name/:media_id".to_string()
        }
    }

    impl Display for Method {
        fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
            match *self {
                Method::Crop => write!(f, "crop"),
                Method::Scale => write!(f, "scale"),
            }
        }
    }
}
