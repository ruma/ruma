#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]

pub mod client_secret;
pub mod device_key_id;
pub mod error;
pub mod event_id;
pub mod key_id;
pub mod mxc_uri;
pub mod room_alias_id;
pub mod room_id;
pub mod room_id_or_alias_id;
pub mod room_name;
pub mod room_version_id;
pub mod server_name;
pub mod session_id;
pub mod user_id;
pub mod voip_version_id;

pub use error::Error;

/// All identifiers must be 255 bytes or less.
const MAX_BYTES: usize = 255;

/// Checks if an identifier is valid.
fn validate_id(id: &str, valid_sigils: &[char]) -> Result<(), Error> {
    if id.len() > MAX_BYTES {
        return Err(Error::MaximumLengthExceeded);
    }

    if !id.starts_with(valid_sigils) {
        return Err(Error::MissingLeadingSigil);
    }

    Ok(())
}

/// Checks an identifier that contains a localpart and hostname for validity.
fn parse_id(id: &str, valid_sigils: &[char]) -> Result<usize, Error> {
    validate_id(id, valid_sigils)?;
    let colon_idx = id.find(':').ok_or(Error::MissingColon)?;
    server_name::validate(&id[colon_idx + 1..])?;
    Ok(colon_idx)
}

/// Checks an identifier that contains a localpart and hostname for validity.
fn validate_delimited_id(id: &str, valid_sigils: &[char]) -> Result<(), Error> {
    parse_id(id, valid_sigils)?;
    Ok(())
}
