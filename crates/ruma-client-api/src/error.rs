//! Errors that can be sent from the homeserver.

use std::{collections::BTreeMap, fmt, time::Duration};

use bytes::BufMut;
use ruma_common::{
    api::{
        error::{DeserializationError, IntoHttpError},
        EndpointError, OutgoingResponse,
    },
    RoomVersionId,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice as from_json_slice, Value as JsonValue};

use crate::PrivOwnedStr;

/// Deserialize and Serialize implementations for ErrorKind.
/// Separate module because it's a lot of code.
mod kind_serde;

/// An enum for the error kind.
///
/// Items may contain additional information.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// M_FORBIDDEN
    Forbidden,

    /// M_UNKNOWN_TOKEN
    UnknownToken {
        /// If this is `true`, the client can acquire a new access token by specifying the device
        /// ID it is already using to the login API.
        ///
        /// For more information, see [the spec].
        ///
        /// [the spec]: https://spec.matrix.org/v1.2/client-server-api/#soft-logout
        soft_logout: bool,
    },

    /// M_MISSING_TOKEN
    MissingToken,

    /// M_BAD_JSON
    BadJson,

    /// M_NOT_JSON
    NotJson,

    /// M_NOT_FOUND
    NotFound,

    /// M_LIMIT_EXCEEDED
    LimitExceeded {
        /// How long a client should wait in milliseconds before they can try again.
        retry_after_ms: Option<Duration>,
    },

    /// M_UNKNOWN
    Unknown,

    /// M_UNRECOGNIZED
    Unrecognized,

    /// M_UNAUTHORIZED
    Unauthorized,

    /// M_USER_DEACTIVATED
    UserDeactivated,

    /// M_USER_IN_USE
    UserInUse,

    /// M_INVALID_USERNAME
    InvalidUsername,

    /// M_ROOM_IN_USE
    RoomInUse,

    /// M_INVALID_ROOM_STATE
    InvalidRoomState,

    /// M_THREEPID_IN_USE
    ThreepidInUse,

    /// M_THREEPID_NOT_FOUND
    ThreepidNotFound,

    /// M_THREEPID_AUTH_FAILED
    ThreepidAuthFailed,

    /// M_THREEPID_DENIED
    ThreepidDenied,

    /// M_SERVER_NOT_TRUSTED
    ServerNotTrusted,

    /// M_UNSUPPORTED_ROOM_VERSION
    UnsupportedRoomVersion,

    /// M_INCOMPATIBLE_ROOM_VERSION
    IncompatibleRoomVersion {
        /// The room's version.
        room_version: RoomVersionId,
    },

    /// M_BAD_STATE
    BadState,

    /// M_GUEST_ACCESS_FORBIDDEN
    GuestAccessForbidden,

    /// M_CAPTCHA_NEEDED
    CaptchaNeeded,

    /// M_CAPTCHA_INVALID
    CaptchaInvalid,

    /// M_MISSING_PARAM
    MissingParam,

    /// M_INVALID_PARAM
    InvalidParam,

    /// M_TOO_LARGE
    TooLarge,

    /// M_EXCLUSIVE
    Exclusive,

    /// M_RESOURCE_LIMIT_EXCEEDED
    ResourceLimitExceeded {
        /// A URI giving a contact method for the server administrator.
        admin_contact: String,
    },

    /// M_CANNOT_LEAVE_SERVER_NOTICE_ROOM
    CannotLeaveServerNoticeRoom,

    /// M_WEAK_PASSWORD
    WeakPassword,

    #[doc(hidden)]
    _Custom { errcode: PrivOwnedStr, extra: Extra },
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extra(BTreeMap<String, JsonValue>);

impl AsRef<str> for ErrorKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Forbidden => "M_FORBIDDEN",
            Self::UnknownToken { .. } => "M_UNKNOWN_TOKEN",
            Self::MissingToken => "M_MISSING_TOKEN",
            Self::BadJson => "M_BAD_JSON",
            Self::NotJson => "M_NOT_JSON",
            Self::NotFound => "M_NOT_FOUND",
            Self::LimitExceeded { .. } => "M_LIMIT_EXCEEDED",
            Self::Unknown => "M_UNKNOWN",
            Self::Unrecognized => "M_UNRECOGNIZED",
            Self::Unauthorized => "M_UNAUTHORIZED",
            Self::UserDeactivated => "M_USER_DEACTIVATED",
            Self::UserInUse => "M_USER_IN_USE",
            Self::InvalidUsername => "M_INVALID_USERNAME",
            Self::RoomInUse => "M_ROOM_IN_USE",
            Self::InvalidRoomState => "M_INVALID_ROOM_STATE",
            Self::ThreepidInUse => "M_THREEPID_IN_USE",
            Self::ThreepidNotFound => "M_THREEPID_NOT_FOUND",
            Self::ThreepidAuthFailed => "M_THREEPID_AUTH_FAILED",
            Self::ThreepidDenied => "M_THREEPID_DENIED",
            Self::ServerNotTrusted => "M_SERVER_NOT_TRUSTED",
            Self::UnsupportedRoomVersion => "M_UNSUPPORTED_ROOM_VERSION",
            Self::IncompatibleRoomVersion { .. } => "M_INCOMPATIBLE_ROOM_VERSION",
            Self::BadState => "M_BAD_STATE",
            Self::GuestAccessForbidden => "M_GUEST_ACCESS_FORBIDDEN",
            Self::CaptchaNeeded => "M_CAPTCHA_NEEDED",
            Self::CaptchaInvalid => "M_CAPTCHA_INVALID",
            Self::MissingParam => "M_MISSING_PARAM",
            Self::InvalidParam => "M_INVALID_PARAM",
            Self::TooLarge => "M_TOO_LARGE",
            Self::Exclusive => "M_EXCLUSIVE",
            Self::ResourceLimitExceeded { .. } => "M_RESOURCE_LIMIT_EXCEEDED",
            Self::CannotLeaveServerNoticeRoom => "M_CANNOT_LEAVE_SERVER_NOTICE_ROOM",
            Self::WeakPassword => "M_WEAK_PASSWORD",
            Self::_Custom { errcode, .. } => &errcode.0,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// A Matrix Error without a status code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::exhaustive_structs)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ErrorBody {
    /// A value which can be used to handle an error message.
    #[serde(flatten)]
    pub kind: ErrorKind,

    /// A human-readable error message, usually a sentence explaining what went wrong.
    #[serde(rename = "error")]
    pub message: String,
}

/// A Matrix Error
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct Error {
    /// A value which can be used to handle an error message.
    pub kind: ErrorKind,

    /// A human-readable error message, usually a sentence explaining what went wrong.
    pub message: String,

    /// The http status code.
    pub status_code: http::StatusCode,
}

impl EndpointError for Error {
    fn try_from_http_response<T: AsRef<[u8]>>(
        response: http::Response<T>,
    ) -> Result<Self, DeserializationError> {
        let status = response.status();
        let error_body: ErrorBody = from_json_slice(response.body().as_ref())?;
        Ok(error_body.into_error(status))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} / {}] {}", self.status_code.as_u16(), self.kind, self.message)
    }
}

impl std::error::Error for Error {}

impl From<Error> for ErrorBody {
    fn from(error: Error) -> Self {
        Self { kind: error.kind, message: error.message }
    }
}

impl ErrorBody {
    /// Convert the ErrorBody into an Error by adding the http status code.
    pub fn into_error(self, status_code: http::StatusCode) -> Error {
        Error { kind: self.kind, message: self.message, status_code }
    }
}

impl OutgoingResponse for Error {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        http::Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(self.status_code)
            .body(ruma_common::serde::json_to_buf(&ErrorBody::from(self))?)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json};

    use super::{ErrorBody, ErrorKind};

    #[test]
    fn deserialize_forbidden() {
        let deserialized: ErrorBody = from_json_value(json!({
            "errcode": "M_FORBIDDEN",
            "error": "You are not authorized to ban users in this room.",
        }))
        .unwrap();

        assert_eq!(
            deserialized,
            ErrorBody {
                kind: ErrorKind::Forbidden,
                message: "You are not authorized to ban users in this room.".into(),
            }
        );
    }
}
