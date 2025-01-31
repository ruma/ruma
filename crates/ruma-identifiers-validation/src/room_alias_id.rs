use crate::{localpart_is_backwards_compatible, parse_id, Error};

/// Validate a [room alias] as used by clients and servers.
///
/// [room alias]: https://spec.matrix.org/latest/appendices/#room-aliases
pub fn validate(s: &str) -> Result<(), Error> {
    let colon_idx = parse_id(s, b'#')?;
    let localpart = &s[1..colon_idx];

    localpart_is_backwards_compatible(localpart)?;

    Ok(())
}
