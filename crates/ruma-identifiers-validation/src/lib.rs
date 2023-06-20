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
pub mod room_version_id;
pub mod server_name;
pub mod user_id;
pub mod voip_version_id;

pub use error::Error;

/// Checks an identifier that contains a localpart and hostname for validity.
fn parse_id(id: &str) -> Result<usize, Error> {
    let colon_idx = id.find(':').ok_or(Error::MissingColon)?;
    server_name::validate(&id[colon_idx + 1..])?;
    Ok(colon_idx)
}

/// Checks an identifier that contains a localpart and hostname for validity.
fn validate_delimited_id(id: &str) -> Result<(), Error> {
    parse_id(id)?;
    Ok(())
}
