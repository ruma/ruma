//! Errors that can be sent from the homeserver.

use std::{fmt, sync::Arc};

use as_variant::as_variant;
use bytes::{BufMut, Bytes};
use ruma_common::api::{
    EndpointError, OutgoingResponse,
    error::{ErrorBody as MatrixErrorBody, FromHttpResponseError, IntoHttpError},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, from_slice as from_json_slice};

mod kind;
mod kind_serde;
#[cfg(test)]
mod tests;

pub use self::kind::*;

/// The body of a Matrix Client API error.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_enums)]
pub enum ErrorBody {
    /// A JSON body with the fields expected for Client API errors.
    Standard(StandardErrorBody),

    /// A JSON body with an unexpected structure.
    Json(JsonValue),

    /// A response body that is not valid JSON.
    NotJson {
        /// The raw bytes of the response body.
        bytes: Bytes,

        /// The error from trying to deserialize the bytes as JSON.
        deserialization_error: Arc<serde_json::Error>,
    },
}

/// A JSON body with the fields expected for Client API errors.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct StandardErrorBody {
    /// A value which can be used to handle an error message.
    #[serde(flatten)]
    pub kind: ErrorKind,

    /// A human-readable error message, usually a sentence explaining what went wrong.
    #[serde(rename = "error")]
    pub message: String,
}

impl StandardErrorBody {
    /// Construct a new `StandardErrorBody` with the given kind and message.
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }
}

/// A Matrix Error
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct Error {
    /// The http status code.
    pub status_code: http::StatusCode,

    /// The http response's body.
    pub body: ErrorBody,
}

impl Error {
    /// Constructs a new `Error` with the given status code and body.
    ///
    /// This is equivalent to calling `body.into_error(status_code)`.
    pub fn new(status_code: http::StatusCode, body: ErrorBody) -> Self {
        Self { status_code, body }
    }

    /// If `self` is a server error in the `errcode` + `error` format expected
    /// for client-server API endpoints, returns the error kind (`errcode`).
    pub fn error_kind(&self) -> Option<&ErrorKind> {
        as_variant!(&self.body, ErrorBody::Standard(StandardErrorBody { kind, .. }) => kind)
    }
}

impl EndpointError for Error {
    fn from_http_response<T: AsRef<[u8]>>(response: http::Response<T>) -> Self {
        let status = response.status();

        let body_bytes = &response.body().as_ref();
        let error_body: ErrorBody = match from_json_slice::<StandardErrorBody>(body_bytes) {
            Ok(mut standard_body) => {
                let headers = response.headers();

                if let ErrorKind::LimitExceeded(LimitExceededErrorData { retry_after }) =
                    &mut standard_body.kind
                {
                    // The Retry-After header takes precedence over the retry_after_ms field in
                    // the body.
                    if let Some(Ok(retry_after_header)) =
                        headers.get(http::header::RETRY_AFTER).map(RetryAfter::try_from)
                    {
                        *retry_after = Some(retry_after_header);
                    }
                }

                ErrorBody::Standard(standard_body)
            }
            Err(_) => match MatrixErrorBody::from_bytes(body_bytes) {
                MatrixErrorBody::Json(json) => ErrorBody::Json(json),
                MatrixErrorBody::NotJson { bytes, deserialization_error, .. } => {
                    ErrorBody::NotJson { bytes, deserialization_error }
                }
            },
        };

        error_body.into_error(status)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_code = self.status_code.as_u16();
        match &self.body {
            ErrorBody::Standard(StandardErrorBody { kind, message }) => {
                let errcode = kind.errcode();
                write!(f, "[{status_code} / {errcode}] {message}")
            }
            ErrorBody::Json(json) => write!(f, "[{status_code}] {json}"),
            ErrorBody::NotJson { .. } => write!(f, "[{status_code}] <non-json bytes>"),
        }
    }
}

impl std::error::Error for Error {}

impl ErrorBody {
    /// Convert the ErrorBody into an Error by adding the http status code.
    ///
    /// This is equivalent to calling `Error::new(status_code, self)`.
    pub fn into_error(self, status_code: http::StatusCode) -> Error {
        Error { status_code, body: self }
    }
}

impl OutgoingResponse for Error {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        let mut builder = http::Response::builder()
            .header(http::header::CONTENT_TYPE, ruma_common::http_headers::APPLICATION_JSON)
            .status(self.status_code);

        // Add data in headers.
        if let Some(ErrorKind::LimitExceeded(LimitExceededErrorData {
            retry_after: Some(retry_after),
        })) = self.error_kind()
        {
            let header_value = http::HeaderValue::try_from(retry_after)?;
            builder = builder.header(http::header::RETRY_AFTER, header_value);
        }

        builder
            .body(match self.body {
                ErrorBody::Standard(standard_body) => {
                    ruma_common::serde::json_to_buf(&standard_body)?
                }
                ErrorBody::Json(json) => ruma_common::serde::json_to_buf(&json)?,
                ErrorBody::NotJson { .. } => {
                    return Err(IntoHttpError::Json(serde::ser::Error::custom(
                        "attempted to serialize ErrorBody::NotJson",
                    )));
                }
            })
            .map_err(Into::into)
    }
}

/// Extension trait for `FromHttpResponseError<ruma_client_api::Error>`.
pub trait FromHttpResponseErrorExt {
    /// If `self` is a server error in the `errcode` + `error` format expected
    /// for client-server API endpoints, returns the error kind (`errcode`).
    fn error_kind(&self) -> Option<&ErrorKind>;
}

impl FromHttpResponseErrorExt for FromHttpResponseError<Error> {
    fn error_kind(&self) -> Option<&ErrorKind> {
        as_variant!(self, Self::Server)?.error_kind()
    }
}
