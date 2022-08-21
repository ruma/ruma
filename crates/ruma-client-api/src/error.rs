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

    /// M_UNABLE_TO_AUTHORISE_JOIN
    UnableToAuthorizeJoin,

    /// M_UNABLE_TO_GRANT_JOIN
    UnableToGrantJoin,

    /// FI.MAU.MSC2246_NOT_YET_UPLOADED
    #[cfg(feature = "unstable-msc2246")]
    NotYetUploaded,

    /// FI.MAU.MSC2246_CANNOT_OVERWRITE_MEDIA
    #[cfg(feature = "unstable-msc2246")]
    CannotOverwriteMedia,

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
            Self::UnableToAuthorizeJoin => "M_UNABLE_TO_AUTHORISE_JOIN",
            Self::UnableToGrantJoin => "M_UNABLE_TO_GRANT_JOIN",
            #[cfg(feature = "unstable-msc2246")]
            Self::NotYetUploaded => "FI.MAU.MSC2246_NOT_YET_UPLOADED",
            #[cfg(feature = "unstable-msc2246")]
            Self::CannotOverwriteMedia => "FI.MAU.MSC2246_CANNOT_OVERWRITE_MEDIA",
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

    /// The `WWW-Authenticate` header error message.
    #[cfg(feature = "unstable-msc2967")]
    pub authenticate: Option<AuthenticateError>,
}

impl EndpointError for Error {
    fn try_from_http_response<T: AsRef<[u8]>>(
        response: http::Response<T>,
    ) -> Result<Self, DeserializationError> {
        let status = response.status();
        let error_body: ErrorBody = from_json_slice(response.body().as_ref())?;

        #[cfg(not(feature = "unstable-msc2967"))]
        {
            Ok(error_body.into_error(status))
        }

        #[cfg(feature = "unstable-msc2967")]
        {
            use ruma_common::api::error::HeaderDeserializationError;

            let mut error = error_body.into_error(status);

            error.authenticate = response
                .headers()
                .get(http::header::WWW_AUTHENTICATE)
                .map(|val| val.to_str().map_err(HeaderDeserializationError::ToStrError))
                .transpose()?
                .and_then(AuthenticateError::from_str);

            Ok(error)
        }
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
        Error {
            kind: self.kind,
            message: self.message,
            status_code,
            #[cfg(feature = "unstable-msc2967")]
            authenticate: None,
        }
    }
}

impl OutgoingResponse for Error {
    fn try_into_http_response<T: Default + BufMut>(
        self,
    ) -> Result<http::Response<T>, IntoHttpError> {
        let builder = http::Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(self.status_code);

        #[cfg(feature = "unstable-msc2967")]
        let builder = if let Some(auth_error) = &self.authenticate {
            builder.header(http::header::WWW_AUTHENTICATE, auth_error)
        } else {
            builder
        };

        builder.body(ruma_common::serde::json_to_buf(&ErrorBody::from(self))?).map_err(Into::into)
    }
}

/// Errors in the `WWW-Authenticate` header.
///
/// To construct this use `::from_str()`. To get its serialized form, use its
/// `TryInto<http::HeaderValue>` implementation.
#[cfg(feature = "unstable-msc2967")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthenticateError {
    /// insufficient_scope
    ///
    /// Encountered when authentication is handled by OpenID Connect and the current access token
    /// isn't authorized for the proper scope for this request. It should be paired with a
    /// `401` status code and a `M_FORBIDDEN` error.
    InsufficientScope {
        /// The new scope to request an authorization for.
        scope: String,
    },

    #[doc(hidden)]
    _Custom { errcode: PrivOwnedStr, attributes: AuthenticateAttrs },
}

#[cfg(feature = "unstable-msc2967")]
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthenticateAttrs(BTreeMap<String, String>);

#[cfg(feature = "unstable-msc2967")]
impl AuthenticateError {
    /// Construct an `AuthenticateError` from a string.
    ///
    /// Returns `None` if the string doesn't contain an error.
    fn from_str(s: &str) -> Option<Self> {
        if let Some(val) = s.strip_prefix("Bearer").map(str::trim) {
            let mut errcode = None;
            let mut attrs = BTreeMap::new();

            // Split the attributes separated by commas and optionally spaces, then split the keys
            // and the values, with the values optionally surrounded by double quotes.
            for (key, value) in val
                .split(',')
                .filter_map(|attr| attr.trim().split_once('='))
                .map(|(key, value)| (key, value.trim_matches('"')))
            {
                if key == "error" {
                    errcode = Some(value);
                } else {
                    attrs.insert(key.to_owned(), value.to_owned());
                }
            }

            if let Some(errcode) = errcode {
                let error = if let Some(scope) =
                    attrs.get("scope").filter(|_| errcode == "insufficient_scope")
                {
                    AuthenticateError::InsufficientScope { scope: scope.to_owned() }
                } else {
                    AuthenticateError::_Custom {
                        errcode: PrivOwnedStr(errcode.into()),
                        attributes: AuthenticateAttrs(attrs),
                    }
                };

                return Some(error);
            }
        }

        None
    }
}

#[cfg(feature = "unstable-msc2967")]
impl TryFrom<&AuthenticateError> for http::HeaderValue {
    type Error = http::header::InvalidHeaderValue;

    fn try_from(error: &AuthenticateError) -> Result<Self, Self::Error> {
        let s = match error {
            AuthenticateError::InsufficientScope { scope } => {
                format!("Bearer error=\"insufficient_scope\", scope=\"{scope}\"")
            }
            AuthenticateError::_Custom { errcode, attributes } => {
                let mut s = format!("Bearer error=\"{}\"", errcode.0);

                for (key, value) in attributes.0.iter() {
                    s.push_str(&format!(", {key}=\"{value}\""));
                }

                s
            }
        };

        s.try_into()
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

        assert_eq!(deserialized.kind, ErrorKind::Forbidden);
        assert_eq!(deserialized.message, "You are not authorized to ban users in this room.");
    }

    #[cfg(feature = "unstable-msc2967")]
    #[test]
    fn custom_authenticate_error_sanity() {
        use super::AuthenticateError;

        let s = "Bearer error=\"custom_error\", misc=\"some content\"";

        let error = AuthenticateError::from_str(s).unwrap();
        let error_header = http::HeaderValue::try_from(&error).unwrap();

        assert_eq!(error_header.to_str().unwrap(), s);
    }

    #[cfg(feature = "unstable-msc2967")]
    #[test]
    fn serialize_insufficient_scope() {
        use super::AuthenticateError;

        let error =
            AuthenticateError::InsufficientScope { scope: "something_privileged".to_owned() };
        let error_header = http::HeaderValue::try_from(&error).unwrap();

        assert_eq!(
            error_header.to_str().unwrap(),
            "Bearer error=\"insufficient_scope\", scope=\"something_privileged\""
        );
    }

    #[cfg(feature = "unstable-msc2967")]
    #[test]
    fn deserialize_insufficient_scope() {
        use ruma_common::api::EndpointError;

        use super::{AuthenticateError, Error};

        let response = http::Response::builder()
            .header(
                http::header::WWW_AUTHENTICATE,
                "Bearer error=\"insufficient_scope\", scope=\"something_privileged\"",
            )
            .status(http::StatusCode::UNAUTHORIZED)
            .body(
                serde_json::to_string(&json!({
                    "errcode": "M_FORBIDDEN",
                    "error": "Insufficient privilege",
                }))
                .unwrap(),
            )
            .unwrap();
        let error = Error::try_from_http_response(response).unwrap();

        assert_eq!(error.status_code, http::StatusCode::UNAUTHORIZED);
        assert_eq!(error.kind, ErrorKind::Forbidden);
        assert_eq!(error.message, "Insufficient privilege");
        let scope = assert_matches::assert_matches!(
            error.authenticate,
            Some(AuthenticateError::InsufficientScope { scope }) => scope
        );
        assert_eq!(scope, "something_privileged");
    }
}
