use crate::error::Error;

pub fn validate(server_name: &str) -> Result<(), Error> {
    use std::net::Ipv6Addr;

    if server_name.is_empty() {
        return Err(Error::InvalidServerName);
    }

    let end_of_host = if server_name.starts_with('[') {
        let end_of_ipv6 = match server_name.find(']') {
            Some(idx) => idx,
            None => return Err(Error::InvalidServerName),
        };

        if server_name[1..end_of_ipv6].parse::<Ipv6Addr>().is_err() {
            return Err(Error::InvalidServerName);
        }

        end_of_ipv6 + 1
    } else {
        let end_of_host = server_name.find(':').unwrap_or_else(|| server_name.len());

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
