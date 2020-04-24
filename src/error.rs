//! Errors that can be sent from the homeserver.

use std::fmt::{self, Display, Formatter};

use ruma_api::{error::ResponseDeserializationError, EndpointError};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice as from_json_slice, to_vec as to_json_vec};
use strum::{AsRefStr, Display, EnumString};

/// An enum for the error kind. Items may contain additional information.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, Display, EnumString)]
#[serde(tag = "errcode")]
#[cfg_attr(test, derive(PartialEq))]
pub enum ErrorKind {
    /// M_FORBIDDEN
    #[serde(rename = "M_FORBIDDEN")]
    #[strum(to_string = "M_FORBIDDEN")]
    Forbidden,
    /// M_UNKNOWN_TOKEN
    #[serde(rename = "M_UNKNOWN_TOKEN")]
    #[strum(to_string = "M_UNKNOWN_TOKEN")]
    UnknownToken,
    /// M_MISSING_TOKEN
    #[serde(rename = "M_MISSING_TOKEN")]
    #[strum(to_string = "M_MISSING_TOKEN")]
    MissingToken,
    /// M_BAD_JSON
    #[serde(rename = "M_BAD_JSON")]
    #[strum(to_string = "M_BAD_JSON")]
    BadJson,
    /// M_NOT_JSON
    #[serde(rename = "M_NOT_JSON")]
    #[strum(to_string = "M_NOT_JSON")]
    NotJson,
    /// M_NOT_FOUND
    #[serde(rename = "M_NOT_FOUND")]
    #[strum(to_string = "M_NOT_FOUND")]
    NotFound,
    /// M_LIMIT_EXCEEDED
    #[serde(rename = "M_LIMIT_EXCEEDED")]
    #[strum(to_string = "M_LIMIT_EXCEEDED")]
    LimitExceeded,
    /// M_UNKNOWN
    #[serde(rename = "M_UNKNOWN")]
    #[strum(to_string = "M_UNKNOWN")]
    Unknown,
    /// M_UNRECOGNIZED
    #[serde(rename = "M_UNRECOGNIZED")]
    #[strum(to_string = "M_UNRECOGNIZED")]
    Unrecognized,
    /// M_UNAUTHORIZED
    #[serde(rename = "M_UNAUTHORIZED")]
    #[strum(to_string = "M_UNAUTHORIZED")]
    Unauthorized,
    /// M_USER_DEACTIVATED
    #[serde(rename = "M_USER_DEACTIVATED")]
    #[strum(to_string = "M_USER_DEACTIVATED")]
    UserDeactivated,
    /// M_USER_IN_USE
    #[serde(rename = "M_USER_IN_USE")]
    #[strum(to_string = "M_USER_IN_USE")]
    UserInUse,
    /// M_INVALID_USERNAME
    #[serde(rename = "M_INVALID_USERNAME")]
    #[strum(to_string = "M_INVALID_USERNAME")]
    InvalidUsername,
    /// M_ROOM_IN_USE
    #[serde(rename = "M_ROOM_IN_USE")]
    #[strum(to_string = "M_ROOM_IN_USE")]
    RoomInUse,
    /// M_INVALID_ROOM_STATE
    #[serde(rename = "M_INVALID_ROOM_STATE")]
    #[strum(to_string = "M_INVALID_ROOM_STATE")]
    InvalidRoomState,
    /// M_THREEPID_IN_USE
    #[serde(rename = "M_THREEPID_IN_USE")]
    #[strum(to_string = "M_THREEPID_IN_USE")]
    ThreepidInUse,
    /// M_THREEPID_NOT_FOUND
    #[serde(rename = "M_THREEPID_NOT_FOUND")]
    #[strum(to_string = "M_THREEPID_NOT_FOUND")]
    ThreepidNotFound,
    /// M_THREEPID_AUTH_FAILED
    #[serde(rename = "M_THREEPID_AUTH_FAILED")]
    #[strum(to_string = "M_THREEPID_AUTH_FAILED")]
    ThreepidAuthFailed,
    /// M_THREEPID_DENIED
    #[serde(rename = "M_THREEPID_DENIED")]
    #[strum(to_string = "M_THREEPID_DENIED")]
    ThreepidDenied,
    /// M_SERVER_NOT_TRUSTED
    #[serde(rename = "M_SERVER_NOT_TRUSTED")]
    #[strum(to_string = "M_SERVER_NOT_TRUSTED")]
    ServerNotTrusted,
    /// M_UNSUPPORTED_ROOM_VERSION
    #[serde(rename = "M_UNSUPPORTED_ROOM_VERSION")]
    #[strum(to_string = "M_UNSUPPORTED_ROOM_VERSION")]
    UnsupportedRoomVersion,
    /// M_INCOMPATIBLE_ROOM_VERSION
    #[serde(rename = "M_INCOMPATIBLE_ROOM_VERSION")]
    #[strum(to_string = "M_INCOMPATIBLE_ROOM_VERSION")]
    IncompatibleRoomVersion,
    /// M_BAD_STATE
    #[serde(rename = "M_BAD_STATE")]
    #[strum(to_string = "M_BAD_STATE")]
    BadState,
    /// M_GUEST_ACCESS_FORBIDDEN
    #[serde(rename = "M_GUEST_ACCESS_FORBIDDEN")]
    #[strum(to_string = "M_GUEST_ACCESS_FORBIDDEN")]
    GuestAccessForbidden,
    /// M_CAPTCHA_NEEDED
    #[serde(rename = "M_CAPTCHA_NEEDED")]
    #[strum(to_string = "M_CAPTCHA_NEEDED")]
    CaptchaNeeded,
    /// M_CAPTCHA_INVALID
    #[serde(rename = "M_CAPTCHA_INVALID")]
    #[strum(to_string = "M_CAPTCHA_INVALID")]
    CaptchaInvalid,
    /// M_MISSING_PARAM
    #[serde(rename = "M_MISSING_PARAM")]
    #[strum(to_string = "M_MISSING_PARAM")]
    MissingParam,
    /// M_INVALID_PARAM
    #[serde(rename = "M_INVALID_PARAM")]
    #[strum(to_string = "M_INVALID_PARAM")]
    InvalidParam,
    /// M_TOO_LARGE
    #[serde(rename = "M_TOO_LARGE")]
    #[strum(to_string = "M_TOO_LARGE")]
    TooLarge,
    /// M_EXCLUSIVE
    #[serde(rename = "M_EXCLUSIVE")]
    #[strum(to_string = "M_EXCLUSIVE")]
    Exclusive,
}

/// A Matrix Error without a status code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ErrorBody {
    /// A value which can be used to handle an error message
    #[serde(flatten)]
    pub kind: ErrorKind,
    /// A human-readable error message, usually a sentence explaining what went wrong.
    #[serde(rename = "error")]
    pub message: String,
}

/// A Matrix Error
#[derive(Debug, Clone)]
pub struct Error {
    /// A value which can be used to handle an error message
    pub kind: ErrorKind,
    /// A human-readable error message, usually a sentence explaining what went wrong.
    pub message: String,
    /// The http status code
    pub status_code: http::StatusCode,
}

impl EndpointError for Error {
    fn try_from_response(
        response: http::Response<Vec<u8>>,
    ) -> Result<Self, ResponseDeserializationError> {
        match from_json_slice::<ErrorBody>(response.body()) {
            Ok(error_body) => Ok(error_body.into_error(response.status())),
            Err(de_error) => Err(ResponseDeserializationError::new(de_error, response)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[{} / {}] {}",
            self.status_code.as_u16(),
            self.kind,
            self.message
        )
    }
}

impl std::error::Error for Error {}

impl From<Error> for ErrorBody {
    fn from(error: Error) -> Self {
        Self {
            kind: error.kind,
            message: error.message,
        }
    }
}

impl ErrorBody {
    /// Convert the ErrorBody into an Error by adding the http status code.
    pub fn into_error(self, status_code: http::StatusCode) -> Error {
        Error {
            kind: self.kind,
            message: self.message,
            status_code,
        }
    }
}

impl From<Error> for http::Response<Vec<u8>> {
    fn from(error: Error) -> http::Response<Vec<u8>> {
        http::Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(error.status_code)
            .body(to_json_vec(&ErrorBody::from(error)).unwrap())
            .unwrap()
    }
}
