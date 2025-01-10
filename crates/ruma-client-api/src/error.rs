//! Errors that can be sent from the homeserver.

use std::{collections::BTreeMap, fmt, str::FromStr, sync::Arc};

use as_variant::as_variant;
use bytes::{BufMut, Bytes};
use ruma_common::{
    api::{
        error::{
            FromHttpResponseError, HeaderDeserializationError, HeaderSerializationError,
            IntoHttpError, MatrixErrorBody,
        },
        EndpointError, OutgoingResponse,
    },
    serde::StringEnum,
    RoomVersionId,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice as from_json_slice, Value as JsonValue};
use web_time::{Duration, SystemTime};

use crate::{
    http_headers::{http_date_to_system_time, system_time_to_http_date},
    PrivOwnedStr,
};

/// Deserialize and Serialize implementations for ErrorKind.
/// Separate module because it's a lot of code.
mod kind_serde;

/// An enum for the error kind.
///
/// Items may contain additional information.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
// Please keep the variants sorted alphabetically.
pub enum ErrorKind {
    /// M_BAD_ALIAS
    BadAlias,

    /// M_BAD_JSON
    BadJson,

    /// M_BAD_STATE
    BadState,

    /// M_BAD_STATUS
    BadStatus {
        /// The HTTP status code of the response.
        status: Option<http::StatusCode>,

        /// The body of the response.
        body: Option<String>,
    },

    /// M_CANNOT_LEAVE_SERVER_NOTICE_ROOM
    CannotLeaveServerNoticeRoom,

    /// M_CANNOT_OVERWRITE_MEDIA
    CannotOverwriteMedia,

    /// M_CAPTCHA_INVALID
    CaptchaInvalid,

    /// M_CAPTCHA_NEEDED
    CaptchaNeeded,

    /// M_CONNECTION_FAILED
    ConnectionFailed,

    /// M_CONNECTION_TIMEOUT
    ConnectionTimeout,

    /// M_DUPLICATE_ANNOTATION
    DuplicateAnnotation,

    /// M_EXCLUSIVE
    Exclusive,

    /// M_FORBIDDEN
    #[non_exhaustive]
    Forbidden {
        /// The `WWW-Authenticate` header error message.
        #[cfg(feature = "unstable-msc2967")]
        authenticate: Option<AuthenticateError>,
    },

    /// M_GUEST_ACCESS_FORBIDDEN
    GuestAccessForbidden,

    /// M_INCOMPATIBLE_ROOM_VERSION
    IncompatibleRoomVersion {
        /// The room's version.
        room_version: RoomVersionId,
    },

    /// M_INVALID_PARAM
    InvalidParam,

    /// M_INVALID_ROOM_STATE
    InvalidRoomState,

    /// M_INVALID_USERNAME
    InvalidUsername,

    /// M_LIMIT_EXCEEDED
    LimitExceeded {
        /// How long a client should wait before they can try again.
        retry_after: Option<RetryAfter>,
    },

    /// M_MISSING_PARAM
    MissingParam,

    /// M_MISSING_TOKEN
    MissingToken,

    /// M_NOT_FOUND
    NotFound,

    /// M_NOT_JSON
    NotJson,

    /// M_NOT_YET_UPLOADED
    NotYetUploaded,

    /// M_RESOURCE_LIMIT_EXCEEDED
    ResourceLimitExceeded {
        /// A URI giving a contact method for the server administrator.
        admin_contact: String,
    },

    /// M_ROOM_IN_USE
    RoomInUse,

    /// M_SERVER_NOT_TRUSTED
    ServerNotTrusted,

    /// M_THREEPID_AUTH_FAILED
    ThreepidAuthFailed,

    /// M_THREEPID_DENIED
    ThreepidDenied,

    /// M_THREEPID_IN_USE
    ThreepidInUse,

    /// M_THREEPID_MEDIUM_NOT_SUPPORTED
    ThreepidMediumNotSupported,

    /// M_THREEPID_NOT_FOUND
    ThreepidNotFound,

    /// M_TOO_LARGE
    TooLarge,

    /// M_UNABLE_TO_AUTHORISE_JOIN
    UnableToAuthorizeJoin,

    /// M_UNABLE_TO_GRANT_JOIN
    UnableToGrantJoin,

    /// M_UNACTIONABLE
    #[cfg(feature = "unstable-msc3843")]
    Unactionable,

    /// M_UNAUTHORIZED
    Unauthorized,

    /// M_UNKNOWN
    Unknown,

    /// M_UNKNOWN_POS for sliding sync
    #[cfg(any(feature = "unstable-msc3575", feature = "unstable-msc4186"))]
    UnknownPos,

    /// M_UNKNOWN_TOKEN
    UnknownToken {
        /// If this is `true`, the client can acquire a new access token by specifying the device
        /// ID it is already using to the login API.
        ///
        /// For more information, see [the spec].
        ///
        /// [the spec]: https://spec.matrix.org/latest/client-server-api/#soft-logout
        soft_logout: bool,
    },

    /// M_UNRECOGNIZED
    Unrecognized,

    /// M_UNSUPPORTED_ROOM_VERSION
    UnsupportedRoomVersion,

    /// M_URL_NOT_SET
    UrlNotSet,

    /// M_USER_DEACTIVATED
    UserDeactivated,

    /// M_USER_IN_USE
    UserInUse,

    /// M_USER_LOCKED
    UserLocked,

    /// M_USER_SUSPENDED
    UserSuspended,

    /// M_WEAK_PASSWORD
    WeakPassword,

    /// M_WRONG_ROOM_KEYS_VERSION
    WrongRoomKeysVersion {
        /// The currently active backup version.
        current_version: Option<String>,
    },

    #[doc(hidden)]
    _Custom { errcode: PrivOwnedStr, extra: Extra },
}

impl ErrorKind {
    /// Constructs an empty [`ErrorKind::Forbidden`] variant.
    pub fn forbidden() -> Self {
        Self::Forbidden {
            #[cfg(feature = "unstable-msc2967")]
            authenticate: None,
        }
    }

    /// Constructs an [`ErrorKind::Forbidden`] variant with the given `WWW-Authenticate` header
    /// error message.
    #[cfg(feature = "unstable-msc2967")]
    pub fn forbidden_with_authenticate(authenticate: AuthenticateError) -> Self {
        Self::Forbidden { authenticate: Some(authenticate) }
    }

    /// Get the [`ErrorCode`] for this `ErrorKind`.
    pub fn errcode(&self) -> ErrorCode {
        match self {
            ErrorKind::BadAlias => ErrorCode::BadAlias,
            ErrorKind::BadJson => ErrorCode::BadJson,
            ErrorKind::BadState => ErrorCode::BadState,
            ErrorKind::BadStatus { .. } => ErrorCode::BadStatus,
            ErrorKind::CannotLeaveServerNoticeRoom => ErrorCode::CannotLeaveServerNoticeRoom,
            ErrorKind::CannotOverwriteMedia => ErrorCode::CannotOverwriteMedia,
            ErrorKind::CaptchaInvalid => ErrorCode::CaptchaInvalid,
            ErrorKind::CaptchaNeeded => ErrorCode::CaptchaNeeded,
            ErrorKind::ConnectionFailed => ErrorCode::ConnectionFailed,
            ErrorKind::ConnectionTimeout => ErrorCode::ConnectionTimeout,
            ErrorKind::DuplicateAnnotation => ErrorCode::DuplicateAnnotation,
            ErrorKind::Exclusive => ErrorCode::Exclusive,
            ErrorKind::Forbidden { .. } => ErrorCode::Forbidden,
            ErrorKind::GuestAccessForbidden => ErrorCode::GuestAccessForbidden,
            ErrorKind::IncompatibleRoomVersion { .. } => ErrorCode::IncompatibleRoomVersion,
            ErrorKind::InvalidParam => ErrorCode::InvalidParam,
            ErrorKind::InvalidRoomState => ErrorCode::InvalidRoomState,
            ErrorKind::InvalidUsername => ErrorCode::InvalidUsername,
            ErrorKind::LimitExceeded { .. } => ErrorCode::LimitExceeded,
            ErrorKind::MissingParam => ErrorCode::MissingParam,
            ErrorKind::MissingToken => ErrorCode::MissingToken,
            ErrorKind::NotFound => ErrorCode::NotFound,
            ErrorKind::NotJson => ErrorCode::NotJson,
            ErrorKind::NotYetUploaded => ErrorCode::NotYetUploaded,
            ErrorKind::ResourceLimitExceeded { .. } => ErrorCode::ResourceLimitExceeded,
            ErrorKind::RoomInUse => ErrorCode::RoomInUse,
            ErrorKind::ServerNotTrusted => ErrorCode::ServerNotTrusted,
            ErrorKind::ThreepidAuthFailed => ErrorCode::ThreepidAuthFailed,
            ErrorKind::ThreepidDenied => ErrorCode::ThreepidDenied,
            ErrorKind::ThreepidInUse => ErrorCode::ThreepidInUse,
            ErrorKind::ThreepidMediumNotSupported => ErrorCode::ThreepidMediumNotSupported,
            ErrorKind::ThreepidNotFound => ErrorCode::ThreepidNotFound,
            ErrorKind::TooLarge => ErrorCode::TooLarge,
            ErrorKind::UnableToAuthorizeJoin => ErrorCode::UnableToAuthorizeJoin,
            ErrorKind::UnableToGrantJoin => ErrorCode::UnableToGrantJoin,
            #[cfg(feature = "unstable-msc3843")]
            ErrorKind::Unactionable => ErrorCode::Unactionable,
            ErrorKind::Unauthorized => ErrorCode::Unauthorized,
            ErrorKind::Unknown => ErrorCode::Unknown,
            #[cfg(any(feature = "unstable-msc3575", feature = "unstable-msc4186"))]
            ErrorKind::UnknownPos => ErrorCode::UnknownPos,
            ErrorKind::UnknownToken { .. } => ErrorCode::UnknownToken,
            ErrorKind::Unrecognized => ErrorCode::Unrecognized,
            ErrorKind::UnsupportedRoomVersion => ErrorCode::UnsupportedRoomVersion,
            ErrorKind::UrlNotSet => ErrorCode::UrlNotSet,
            ErrorKind::UserDeactivated => ErrorCode::UserDeactivated,
            ErrorKind::UserInUse => ErrorCode::UserInUse,
            ErrorKind::UserLocked => ErrorCode::UserLocked,
            ErrorKind::UserSuspended => ErrorCode::UserSuspended,
            ErrorKind::WeakPassword => ErrorCode::WeakPassword,
            ErrorKind::WrongRoomKeysVersion { .. } => ErrorCode::WrongRoomKeysVersion,
            ErrorKind::_Custom { errcode, .. } => errcode.0.clone().into(),
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Extra(BTreeMap<String, JsonValue>);

/// The possible [error codes] defined in the Matrix spec.
///
/// [error codes]: https://spec.matrix.org/latest/client-server-api/#standard-error-response
#[derive(Clone, StringEnum)]
#[non_exhaustive]
#[ruma_enum(rename_all = "M_MATRIX_ERROR_CASE")]
// Please keep the variants sorted alphabetically.
pub enum ErrorCode {
    /// M_BAD_ALIAS
    BadAlias,

    /// M_BAD_JSON
    BadJson,

    /// M_BAD_STATE
    BadState,

    /// M_BAD_STATUS
    BadStatus,

    /// M_CANNOT_LEAVE_SERVER_NOTICE_ROOM
    CannotLeaveServerNoticeRoom,

    /// M_CANNOT_OVERWRITE_MEDIA
    CannotOverwriteMedia,

    /// M_CAPTCHA_INVALID
    CaptchaInvalid,

    /// M_CAPTCHA_NEEDED
    CaptchaNeeded,

    /// M_CONNECTION_FAILED
    ConnectionFailed,

    /// M_CONNECTION_TIMEOUT
    ConnectionTimeout,

    /// M_DUPLICATE_ANNOTATION
    DuplicateAnnotation,

    /// M_EXCLUSIVE
    Exclusive,

    /// M_FORBIDDEN
    Forbidden,

    /// M_GUEST_ACCESS_FORBIDDEN
    GuestAccessForbidden,

    /// M_INCOMPATIBLE_ROOM_VERSION
    IncompatibleRoomVersion,

    /// M_INVALID_PARAM
    InvalidParam,

    /// M_INVALID_ROOM_STATE
    InvalidRoomState,

    /// M_INVALID_USERNAME
    InvalidUsername,

    /// M_LIMIT_EXCEEDED
    LimitExceeded,

    /// M_MISSING_PARAM
    MissingParam,

    /// M_MISSING_TOKEN
    MissingToken,

    /// M_NOT_FOUND
    NotFound,

    /// M_NOT_JSON
    NotJson,

    /// M_NOT_YET_UPLOADED
    NotYetUploaded,

    /// M_RESOURCE_LIMIT_EXCEEDED
    ResourceLimitExceeded,

    /// M_ROOM_IN_USE
    RoomInUse,

    /// M_SERVER_NOT_TRUSTED
    ServerNotTrusted,

    /// M_THREEPID_AUTH_FAILED
    ThreepidAuthFailed,

    /// M_THREEPID_DENIED
    ThreepidDenied,

    /// M_THREEPID_IN_USE
    ThreepidInUse,

    /// M_THREEPID_MEDIUM_NOT_SUPPORTED
    ThreepidMediumNotSupported,

    /// M_THREEPID_NOT_FOUND
    ThreepidNotFound,

    /// M_TOO_LARGE
    TooLarge,

    /// M_UNABLE_TO_AUTHORISE_JOIN
    #[ruma_enum(rename = "M_UNABLE_TO_AUTHORISE_JOIN")]
    UnableToAuthorizeJoin,

    /// M_UNABLE_TO_GRANT_JOIN
    UnableToGrantJoin,

    /// M_UNACTIONABLE
    #[cfg(feature = "unstable-msc3843")]
    Unactionable,

    /// M_UNAUTHORIZED
    Unauthorized,

    /// M_UNKNOWN
    Unknown,

    /// M_UNKNOWN_POS for sliding sync
    #[cfg(any(feature = "unstable-msc3575", feature = "unstable-msc4186"))]
    UnknownPos,

    /// M_UNKNOWN_TOKEN
    UnknownToken,

    /// M_UNRECOGNIZED
    Unrecognized,

    /// M_UNSUPPORTED_ROOM_VERSION
    UnsupportedRoomVersion,

    /// M_URL_NOT_SET
    UrlNotSet,

    /// M_USER_DEACTIVATED
    UserDeactivated,

    /// M_USER_IN_USE
    UserInUse,

    /// M_USER_LOCKED
    UserLocked,

    /// M_USER_SUSPENDED
    UserSuspended,

    /// M_WEAK_PASSWORD
    WeakPassword,

    /// M_WRONG_ROOM_KEYS_VERSION
    WrongRoomKeysVersion,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The body of a Matrix Client API error.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_enums)]
pub enum ErrorBody {
    /// A JSON body with the fields expected for Client API errors.
    Standard {
        /// A value which can be used to handle an error message.
        kind: ErrorKind,

        /// A human-readable error message, usually a sentence explaining what went wrong.
        message: String,
    },

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
#[allow(clippy::exhaustive_structs)]
pub struct StandardErrorBody {
    /// A value which can be used to handle an error message.
    #[serde(flatten)]
    pub kind: ErrorKind,

    /// A human-readable error message, usually a sentence explaining what went wrong.
    #[serde(rename = "error")]
    pub message: String,
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
        as_variant!(&self.body, ErrorBody::Standard { kind, .. } => kind)
    }
}

impl EndpointError for Error {
    fn from_http_response<T: AsRef<[u8]>>(response: http::Response<T>) -> Self {
        let status = response.status();

        let body_bytes = &response.body().as_ref();
        let error_body: ErrorBody = match from_json_slice(body_bytes) {
            Ok(StandardErrorBody { mut kind, message }) => {
                let headers = response.headers();

                match &mut kind {
                    #[cfg(feature = "unstable-msc2967")]
                    ErrorKind::Forbidden { authenticate } => {
                        *authenticate = headers
                            .get(http::header::WWW_AUTHENTICATE)
                            .and_then(|val| val.to_str().ok())
                            .and_then(AuthenticateError::from_str);
                    }
                    ErrorKind::LimitExceeded { retry_after } => {
                        // The Retry-After header takes precedence over the retry_after_ms field in
                        // the body.
                        if let Some(Ok(retry_after_header)) =
                            headers.get(http::header::RETRY_AFTER).map(RetryAfter::try_from)
                        {
                            *retry_after = Some(retry_after_header);
                        }
                    }
                    _ => {}
                }

                ErrorBody::Standard { kind, message }
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
            ErrorBody::Standard { kind, message } => {
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
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(self.status_code);

        #[allow(clippy::collapsible_match)]
        if let ErrorBody::Standard { kind, .. } = &self.body {
            match kind {
                #[cfg(feature = "unstable-msc2967")]
                ErrorKind::Forbidden { authenticate: Some(auth_error) } => {
                    builder = builder.header(http::header::WWW_AUTHENTICATE, auth_error);
                }
                ErrorKind::LimitExceeded { retry_after: Some(retry_after) } => {
                    let header_value = http::HeaderValue::try_from(retry_after)?;
                    builder = builder.header(http::header::RETRY_AFTER, header_value);
                }
                _ => {}
            }
        }

        builder
            .body(match self.body {
                ErrorBody::Standard { kind, message } => {
                    ruma_common::serde::json_to_buf(&StandardErrorBody { kind, message })?
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

/// How long a client should wait before it tries again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum RetryAfter {
    /// The client should wait for the given duration.
    ///
    /// This variant should be preferred for backwards compatibility, as it will also populate the
    /// `retry_after_ms` field in the body of the response.
    Delay(Duration),
    /// The client should wait for the given date and time.
    DateTime(SystemTime),
}

impl TryFrom<&http::HeaderValue> for RetryAfter {
    type Error = HeaderDeserializationError;

    fn try_from(value: &http::HeaderValue) -> Result<Self, Self::Error> {
        if value.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            // It should be a duration.
            Ok(Self::Delay(Duration::from_secs(u64::from_str(value.to_str()?)?)))
        } else {
            // It should be a date.
            Ok(Self::DateTime(http_date_to_system_time(value)?))
        }
    }
}

impl TryFrom<&RetryAfter> for http::HeaderValue {
    type Error = HeaderSerializationError;

    fn try_from(value: &RetryAfter) -> Result<Self, Self::Error> {
        match value {
            RetryAfter::Delay(duration) => Ok(duration.as_secs().into()),
            RetryAfter::DateTime(time) => system_time_to_http_date(time),
        }
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

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::api::{EndpointError, OutgoingResponse};
    use serde_json::{
        from_slice as from_json_slice, from_value as from_json_value, json, Value as JsonValue,
    };
    use web_time::{Duration, UNIX_EPOCH};

    use super::{Error, ErrorBody, ErrorKind, RetryAfter, StandardErrorBody};

    #[test]
    fn deserialize_forbidden() {
        let deserialized: StandardErrorBody = from_json_value(json!({
            "errcode": "M_FORBIDDEN",
            "error": "You are not authorized to ban users in this room.",
        }))
        .unwrap();

        assert_eq!(
            deserialized.kind,
            ErrorKind::Forbidden {
                #[cfg(feature = "unstable-msc2967")]
                authenticate: None
            }
        );
        assert_eq!(deserialized.message, "You are not authorized to ban users in this room.");
    }

    #[test]
    fn deserialize_wrong_room_key_version() {
        let deserialized: StandardErrorBody = from_json_value(json!({
            "current_version": "42",
            "errcode": "M_WRONG_ROOM_KEYS_VERSION",
            "error": "Wrong backup version."
        }))
        .expect("We should be able to deserialize a wrong room keys version error");

        assert_matches!(deserialized.kind, ErrorKind::WrongRoomKeysVersion { current_version });
        assert_eq!(current_version.as_deref(), Some("42"));
        assert_eq!(deserialized.message, "Wrong backup version.");
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
        use super::AuthenticateError;

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
        let error = Error::from_http_response(response);

        assert_eq!(error.status_code, http::StatusCode::UNAUTHORIZED);
        assert_matches!(error.body, ErrorBody::Standard { kind, message });
        assert_matches!(kind, ErrorKind::Forbidden { authenticate });
        assert_eq!(message, "Insufficient privilege");
        assert_matches!(authenticate, Some(AuthenticateError::InsufficientScope { scope }));
        assert_eq!(scope, "something_privileged");
    }

    #[test]
    fn deserialize_limit_exceeded_no_retry_after() {
        let response = http::Response::builder()
            .status(http::StatusCode::TOO_MANY_REQUESTS)
            .body(
                serde_json::to_string(&json!({
                    "errcode": "M_LIMIT_EXCEEDED",
                    "error": "Too many requests",
                }))
                .unwrap(),
            )
            .unwrap();
        let error = Error::from_http_response(response);

        assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
        assert_matches!(
            error.body,
            ErrorBody::Standard { kind: ErrorKind::LimitExceeded { retry_after: None }, message }
        );
        assert_eq!(message, "Too many requests");
    }

    #[test]
    fn deserialize_limit_exceeded_retry_after_body() {
        let response = http::Response::builder()
            .status(http::StatusCode::TOO_MANY_REQUESTS)
            .body(
                serde_json::to_string(&json!({
                    "errcode": "M_LIMIT_EXCEEDED",
                    "error": "Too many requests",
                    "retry_after_ms": 2000,
                }))
                .unwrap(),
            )
            .unwrap();
        let error = Error::from_http_response(response);

        assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
        assert_matches!(
            error.body,
            ErrorBody::Standard {
                kind: ErrorKind::LimitExceeded { retry_after: Some(retry_after) },
                message
            }
        );
        assert_matches!(retry_after, RetryAfter::Delay(delay));
        assert_eq!(delay.as_millis(), 2000);
        assert_eq!(message, "Too many requests");
    }

    #[test]
    fn deserialize_limit_exceeded_retry_after_header_delay() {
        let response = http::Response::builder()
            .status(http::StatusCode::TOO_MANY_REQUESTS)
            .header(http::header::RETRY_AFTER, "2")
            .body(
                serde_json::to_string(&json!({
                    "errcode": "M_LIMIT_EXCEEDED",
                    "error": "Too many requests",
                }))
                .unwrap(),
            )
            .unwrap();
        let error = Error::from_http_response(response);

        assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
        assert_matches!(
            error.body,
            ErrorBody::Standard {
                kind: ErrorKind::LimitExceeded { retry_after: Some(retry_after) },
                message
            }
        );
        assert_matches!(retry_after, RetryAfter::Delay(delay));
        assert_eq!(delay.as_millis(), 2000);
        assert_eq!(message, "Too many requests");
    }

    #[test]
    fn deserialize_limit_exceeded_retry_after_header_datetime() {
        let response = http::Response::builder()
            .status(http::StatusCode::TOO_MANY_REQUESTS)
            .header(http::header::RETRY_AFTER, "Fri, 15 May 2015 15:34:21 GMT")
            .body(
                serde_json::to_string(&json!({
                    "errcode": "M_LIMIT_EXCEEDED",
                    "error": "Too many requests",
                }))
                .unwrap(),
            )
            .unwrap();
        let error = Error::from_http_response(response);

        assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
        assert_matches!(
            error.body,
            ErrorBody::Standard {
                kind: ErrorKind::LimitExceeded { retry_after: Some(retry_after) },
                message
            }
        );
        assert_matches!(retry_after, RetryAfter::DateTime(time));
        assert_eq!(time.duration_since(UNIX_EPOCH).unwrap().as_secs(), 1_431_704_061);
        assert_eq!(message, "Too many requests");
    }

    #[test]
    fn deserialize_limit_exceeded_retry_after_header_over_body() {
        let response = http::Response::builder()
            .status(http::StatusCode::TOO_MANY_REQUESTS)
            .header(http::header::RETRY_AFTER, "2")
            .body(
                serde_json::to_string(&json!({
                    "errcode": "M_LIMIT_EXCEEDED",
                    "error": "Too many requests",
                    "retry_after_ms": 3000,
                }))
                .unwrap(),
            )
            .unwrap();
        let error = Error::from_http_response(response);

        assert_eq!(error.status_code, http::StatusCode::TOO_MANY_REQUESTS);
        assert_matches!(
            error.body,
            ErrorBody::Standard {
                kind: ErrorKind::LimitExceeded { retry_after: Some(retry_after) },
                message
            }
        );
        assert_matches!(retry_after, RetryAfter::Delay(delay));
        assert_eq!(delay.as_millis(), 2000);
        assert_eq!(message, "Too many requests");
    }

    #[test]
    fn serialize_limit_exceeded_retry_after_none() {
        let error = Error::new(
            http::StatusCode::TOO_MANY_REQUESTS,
            ErrorBody::Standard {
                kind: ErrorKind::LimitExceeded { retry_after: None },
                message: "Too many requests".to_owned(),
            },
        );

        let response = error.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(response.status(), http::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers().get(http::header::RETRY_AFTER), None);

        let json_body: JsonValue = from_json_slice(response.body()).unwrap();
        assert_eq!(
            json_body,
            json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
            })
        );
    }

    #[test]
    fn serialize_limit_exceeded_retry_after_delay() {
        let error = Error::new(
            http::StatusCode::TOO_MANY_REQUESTS,
            ErrorBody::Standard {
                kind: ErrorKind::LimitExceeded {
                    retry_after: Some(RetryAfter::Delay(Duration::from_secs(3))),
                },
                message: "Too many requests".to_owned(),
            },
        );

        let response = error.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(response.status(), http::StatusCode::TOO_MANY_REQUESTS);
        let retry_after_header = response.headers().get(http::header::RETRY_AFTER).unwrap();
        assert_eq!(retry_after_header.to_str().unwrap(), "3");

        let json_body: JsonValue = from_json_slice(response.body()).unwrap();
        assert_eq!(
            json_body,
            json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
                "retry_after_ms": 3000,
            })
        );
    }

    #[test]
    fn serialize_limit_exceeded_retry_after_datetime() {
        let error = Error::new(
            http::StatusCode::TOO_MANY_REQUESTS,
            ErrorBody::Standard {
                kind: ErrorKind::LimitExceeded {
                    retry_after: Some(RetryAfter::DateTime(
                        UNIX_EPOCH + Duration::from_secs(1_431_704_061),
                    )),
                },
                message: "Too many requests".to_owned(),
            },
        );

        let response = error.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(response.status(), http::StatusCode::TOO_MANY_REQUESTS);
        let retry_after_header = response.headers().get(http::header::RETRY_AFTER).unwrap();
        assert_eq!(retry_after_header.to_str().unwrap(), "Fri, 15 May 2015 15:34:21 GMT");

        let json_body: JsonValue = from_json_slice(response.body()).unwrap();
        assert_eq!(
            json_body,
            json!({
                "errcode": "M_LIMIT_EXCEEDED",
                "error": "Too many requests",
            })
        );
    }

    #[test]
    fn serialize_user_locked() {
        let error = Error::new(
            http::StatusCode::UNAUTHORIZED,
            ErrorBody::Standard {
                kind: ErrorKind::UserLocked,
                message: "This account has been locked".to_owned(),
            },
        );

        let response = error.try_into_http_response::<Vec<u8>>().unwrap();

        assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
        let json_body: JsonValue = from_json_slice(response.body()).unwrap();
        assert_eq!(
            json_body,
            json!({
                "errcode": "M_USER_LOCKED",
                "error": "This account has been locked",
                "soft_logout": true,
            })
        );
    }
}
