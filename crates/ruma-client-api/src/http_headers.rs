//! Helpers for HTTP headers with the `http` crate.
#![allow(clippy::declare_interior_mutable_const)]

use http::{header::HeaderName, HeaderValue};
use ruma_common::api::error::{HeaderDeserializationError, HeaderSerializationError};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

/// The [`Cross-Origin-Resource-Policy`] HTTP response header.
///
/// [`Cross-Origin-Resource-Policy`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy
pub const CROSS_ORIGIN_RESOURCE_POLICY: HeaderName =
    HeaderName::from_static("cross-origin-resource-policy");

/// Convert as `SystemTime` to a HTTP date header value.
pub fn system_time_to_http_date(
    time: &SystemTime,
) -> Result<HeaderValue, HeaderSerializationError> {
    let mut buffer = [0; 29];

    let duration =
        time.duration_since(UNIX_EPOCH).map_err(|_| HeaderSerializationError::InvalidHttpDate)?;
    date_header::format(duration.as_secs(), &mut buffer)
        .map_err(|_| HeaderSerializationError::InvalidHttpDate)?;

    Ok(HeaderValue::from_bytes(&buffer).expect("date_header should produce a valid header value"))
}

/// Convert a header value representing a HTTP date to a `SystemTime`.
pub fn http_date_to_system_time(
    value: &HeaderValue,
) -> Result<SystemTime, HeaderDeserializationError> {
    let bytes = value.as_bytes();

    let ts = date_header::parse(bytes).map_err(|_| HeaderDeserializationError::InvalidHttpDate)?;

    UNIX_EPOCH
        .checked_add(Duration::from_secs(ts))
        .ok_or(HeaderDeserializationError::InvalidHttpDate)
}
