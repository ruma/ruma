use std::num::NonZeroU8;

use crate::{error::MxcUriError, server_name};

const PROTOCOL: &str = "mxc://";

pub fn validate(uri: &str) -> Result<NonZeroU8, MxcUriError> {
    let uri = match uri.strip_prefix(PROTOCOL) {
        Some(uri) => uri,
        None => return Err(MxcUriError::WrongSchema),
    };

    let index = match uri.find('/') {
        Some(index) => index,
        None => return Err(MxcUriError::MissingSlash),
    };

    let server_name = &uri[..index];
    let media_id = &uri[index + 1..];
    // See: https://spec.matrix.org/v1.10/client-server-api/#security-considerations-5
    let media_id_is_valid = media_id
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' ));

    if !media_id_is_valid {
        Err(MxcUriError::MediaIdMalformed)
    } else if server_name::validate(server_name).is_err() {
        Err(MxcUriError::ServerNameMalformed)
    } else {
        Ok(NonZeroU8::new((index + 6) as u8).unwrap())
    }
}
