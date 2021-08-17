use std::num::NonZeroU8;

use crate::{error::MxcUriError, server_name, Error};

const PROTOCOL: &str = "mxc://";

pub fn validate(uri: &str) -> Result<NonZeroU8, Error> {
    let uri = match uri.strip_prefix(PROTOCOL) {
        Some(uri) => uri,
        None => return Err(MxcUriError::WrongSchema.into()),
    };

    let index = match uri.find('/') {
        Some(index) => index,
        None => return Err(MxcUriError::MissingSlash.into()),
    };

    let server_name = &uri[..index];
    let media_id = &uri[index + 1..];
    // See: https://matrix.org/docs/spec/client_server/r0.6.1#id69
    let media_id_is_valid =
        media_id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' ));

    if !media_id_is_valid {
        Err(MxcUriError::MediaIdMalformed.into())
    } else if !server_name::validate(server_name).is_ok() {
        Err(MxcUriError::ServerNameMalformed.into())
    } else {
        Ok(NonZeroU8::new((index + 6) as u8).unwrap())
    }
}
