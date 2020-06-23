//! Matrix-spec compliant server names.

use crate::error::Error;

/// A Matrix-spec compliant server name.
///
/// It is discouraged to use this type directly – instead use one of the aliases ([`ServerName`](../type.ServerName.html) and
/// [`ServerNameRef`](../type.ServerNameRef.html)) in the crate root.
#[derive(Clone, Copy, Debug)]
pub struct ServerName<T> {
    full_id: T,
}

impl<T> ServerName<T>
where
    T: AsRef<str>,
{
    /// Creates a reference to this `ServerName`.
    pub fn as_ref(&self) -> ServerName<&str> {
        ServerName { full_id: self.full_id.as_ref() }
    }
}

fn try_from<S, T>(server_name: S) -> Result<ServerName<T>, Error>
where
    S: AsRef<str> + Into<T>,
{
    use core::net::Ipv6Addr;

    let name = server_name.as_ref();

    if name.is_empty() {
        return Err(Error::InvalidServerName);
    }

    let end_of_host = if name.starts_with('[') {
        let end_of_ipv6 = match name.find(']') {
            Some(idx) => idx,
            None => return Err(Error::InvalidServerName),
        };

        if name[1..end_of_ipv6].parse::<Ipv6Addr>().is_err() {
            return Err(Error::InvalidServerName);
        }

        end_of_ipv6 + 1
    } else {
        let end_of_host = name.find(':').unwrap_or_else(|| name.len());

        if name[..end_of_host]
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.'))
        {
            return Err(Error::InvalidServerName);
        }

        end_of_host
    };

    if name.len() != end_of_host
        && (
            // hostname is followed by something other than ":port"
            name.as_bytes()[end_of_host] != b':'
            // the remaining characters after ':' are not a valid port
            || name[end_of_host + 1..].parse::<u16>().is_err()
        )
    {
        Err(Error::InvalidServerName)
    } else {
        Ok(ServerName { full_id: server_name.into() })
    }
}

common_impls!(ServerName, try_from, "An IP address or hostname");

#[cfg(test)]
mod tests {
    use core::convert::TryFrom;

    use crate::ServerNameRef;

    #[test]
    fn ipv4_host() {
        assert!(ServerNameRef::try_from("127.0.0.1").is_ok());
    }

    #[test]
    fn ipv4_host_and_port() {
        assert!(ServerNameRef::try_from("1.1.1.1:12000").is_ok());
    }

    #[test]
    fn ipv6() {
        assert!(ServerNameRef::try_from("[::1]").is_ok());
    }

    #[test]
    fn ipv6_with_port() {
        assert!(ServerNameRef::try_from("[1234:5678::abcd]:5678").is_ok());
    }

    #[test]
    fn dns_name() {
        assert!(ServerNameRef::try_from("example.com").is_ok());
    }

    #[test]
    fn dns_name_with_port() {
        assert!(ServerNameRef::try_from("ruma.io:8080").is_ok());
    }

    #[test]
    fn empty_string() {
        assert!(ServerNameRef::try_from("").is_err());
    }

    #[test]
    fn invalid_ipv6() {
        assert!(ServerNameRef::try_from("[test::1]").is_err());
    }

    #[test]
    fn ipv4_with_invalid_port() {
        assert!(ServerNameRef::try_from("127.0.0.1:").is_err());
    }

    #[test]
    fn ipv6_with_invalid_port() {
        assert!(ServerNameRef::try_from("[fe80::1]:100000").is_err());
        assert!(ServerNameRef::try_from("[fe80::1]!").is_err());
    }

    #[test]
    fn dns_name_with_invalid_port() {
        assert!(ServerNameRef::try_from("matrix.org:hello").is_err());
    }
}
