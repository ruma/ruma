use hyper::error::{Error as HyperError, UriError};
use ruma_api::Error as RumaApiError;
use serde_json::Error as SerdeJsonError;
use serde_urlencoded::ser::Error as SerdeUrlEncodedSerializeError;
use url::ParseError;

/// An error that occurs during client operations.
#[derive(Debug)]
pub enum Error {
    /// Queried endpoint requires authentication but was called on an anonymous client
    AuthenticationRequired,
    /// An error at the HTTP layer.
    Hyper(HyperError),
    /// An error when parsing a string as a URI.
    Uri(UriError),
    /// An error when parsing a string as a URL.
    Url(ParseError),
    /// An error converting between ruma_client_api types and Hyper types.
    RumaApi(RumaApiError),
    /// An error when serializing or deserializing a JSON value.
    SerdeJson(SerdeJsonError),
    /// An error when serializing a query string value.
    SerdeUrlEncodedSerialize(SerdeUrlEncodedSerializeError),
}

impl From<HyperError> for Error {
    fn from(error: HyperError) -> Error {
        Error::Hyper(error)
    }
}

impl From<UriError> for Error {
    fn from(error: UriError) -> Error {
        Error::Uri(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        Error::Url(error)
    }
}

impl From<RumaApiError> for Error {
    fn from(error: RumaApiError) -> Error {
        Error::RumaApi(error)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(error: SerdeJsonError) -> Error {
        Error::SerdeJson(error)
    }
}

impl From<SerdeUrlEncodedSerializeError> for Error {
    fn from(error: SerdeUrlEncodedSerializeError) -> Error {
        Error::SerdeUrlEncodedSerialize(error)
    }
}
