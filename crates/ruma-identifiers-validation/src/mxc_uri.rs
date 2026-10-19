use std::num::NonZeroU16;

use crate::{error::MxcUriError, server_name};

const PROTOCOL: &str = "mxc://";

pub fn validate(uri: &str) -> Result<NonZeroU16, MxcUriError> {
    let Some(uri) = uri.strip_prefix(PROTOCOL) else {
        return Err(MxcUriError::WrongSchema);
    };

    let Some(index) = uri.find('/') else {
        return Err(MxcUriError::MissingSlash);
    };

    let server_name = &uri[..index];
    let media_id = &uri[index + 1..];
    // See: https://spec.matrix.org/v1.19/client-server-api/#matrix-content-mxc-uris
    let media_id_is_valid = media_id
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' ));

    if !media_id_is_valid {
        Err(MxcUriError::MediaIdMalformed)
    } else if server_name::validate(server_name).is_err() {
        Err(MxcUriError::ServerNameMalformed)
    } else {
        // SAFETY: The `server_name::validate` call above guarantees a maximum length of
        // 261 bytes. The maximum length plus the additional 6 bytes is still representable
        // by a `u16`.
        Ok(NonZeroU16::new((index + 6) as u16).unwrap())
    }
}
