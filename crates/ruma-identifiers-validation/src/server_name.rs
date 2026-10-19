use crate::error::Error;

pub fn validate(server_name: &str) -> Result<(), Error> {
    use std::net::Ipv6Addr;

    if server_name.is_empty() {
        return Err(Error::InvalidServerName);
    }

    // As per spec, the server name can be a hostname plus an optional port.
    //
    // The hostname is defined as follows:
    //
    // hostname    = IPv4address / "[" IPv6address "]" / dns-name
    //
    // 1. IPv4 addresses are at most 15 characters long.
    // 2. IPv6 addresses are at most 45 characters long.
    // 3. DNS names are at most 255 characters long.
    //
    // The optional port must be separated by a ":" character followed by at most 5 digits.
    //
    // This brings the maximum allowed length of a server name to 255 + 1 + 5 => 261.
    //
    // [1]: https://spec.matrix.org/v1.19/appendices/#server-name
    if server_name.len() > 261 {
        return Err(Error::InvalidServerName);
    }

    let end_of_host = if server_name.starts_with('[') {
        let Some(end_of_ipv6) = server_name.find(']') else {
            return Err(Error::InvalidServerName);
        };

        if server_name[1..end_of_ipv6].parse::<Ipv6Addr>().is_err() {
            return Err(Error::InvalidServerName);
        }

        end_of_ipv6 + 1
    } else {
        #[allow(clippy::unnecessary_lazy_evaluations)]
        let end_of_host = server_name.find(':').unwrap_or_else(|| server_name.len());

        if end_of_host == 0 {
            return Err(Error::InvalidServerName);
        }

        if server_name[..end_of_host]
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.'))
        {
            return Err(Error::InvalidServerName);
        }

        end_of_host
    };

    if server_name.len() != end_of_host
        && (
            // hostname is followed by something other than ":port"
            server_name.as_bytes()[end_of_host] != b':'
            // the remaining characters after ':' are not a valid port
            || server_name[end_of_host + 1..].parse::<u16>().is_err()
        )
    {
        Err(Error::InvalidServerName)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::user_id;

    #[test]
    fn rejects_hostless_server_name_with_port() {
        assert!(validate(":8448").is_err());
    }

    #[test]
    fn rejects_user_id_with_hostless_server_name() {
        assert!(user_id::validate("@alice::8448").is_err());
    }
}
