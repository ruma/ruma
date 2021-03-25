use crate::{server_name, Error};

const PROTOCOL: &str = "mxc://";

pub fn validate(uri: &str) -> Result<(&str, &str), Error> {
    let uri = match uri.strip_prefix(PROTOCOL) {
        Some(uri) => uri,
        None => return Err(Error::InvalidMxcUri),
    };

    let index = match uri.find('/') {
        Some(index) => index,
        None => return Err(Error::InvalidMxcUri),
    };

    let server_name = &uri[..index];
    let media_id = &uri[index + 1..];
    // See: https://matrix.org/docs/spec/client_server/r0.6.1#id69
    let media_id_is_valid =
        media_id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' ));

    if media_id_is_valid && server_name::validate(server_name).is_ok() {
        Ok((media_id, server_name))
    } else {
        Err(Error::InvalidMxcUri)
    }
}
