use hyper::Error as HyperError;
use ruma_client_api::Error as RumaClientApiError;
use serde_json::Error as SerdeJsonError;
use serde_urlencoded::ser::Error as SerdeUrlEncodedSerializeError;
use url::ParseError;

/// An error that occurs during client operations.
#[derive(Debug)]
pub enum Error {
    /// An error at the HTTP layer.
    Hyper(HyperError),
    /// An error when parsing a string as a URL.
    Url(ParseError),
    /// An error converting between ruma_client_api types and Hyper types.
    RumaClientApi(RumaClientApiError),
    /// An error when serializing or deserializing a JSON value.
    SerdeJson(SerdeJsonError),
    /// An error when serializing a query string value.
    SerdeUrlEncodedSerialize(SerdeUrlEncodedSerializeError),
    /// Queried endpoint requires authentication but was called on an anonymous client
    AuthenticationRequired,
}

impl From<HyperError> for Error {
    fn from(error: HyperError) -> Error {
        Error::Hyper(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        Error::Url(error)
    }
}

impl From<RumaClientApiError> for Error {
    fn from(error: RumaClientApiError) -> Error {
        Error::RumaClientApi(error)
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
