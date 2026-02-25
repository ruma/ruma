//! Matrix-spec compliant server names.

use std::net::Ipv4Addr;

use ruma_macros::ruma_id;

/// A Matrix-spec compliant [server name].
///
/// It consists of a host and an optional port (separated by a colon if present).
///
/// [server name]: https://spec.matrix.org/latest/appendices/#server-name
#[ruma_id(validate = ruma_identifiers_validation::server_name::validate)]
pub struct ServerName;

impl ServerName {
    /// Returns the host of the server name.
    ///
    /// That is: Return the part of the server name before `:<port>` or the full server name if
    /// there is no port.
    pub fn host(&self) -> &str {
        let s = self.as_str();

        if let Some(end_of_ipv6) = s.find(']') {
            &s[..=end_of_ipv6]
        } else {
            // It's not ipv6, so ':' means the port starts
            let end_of_host = s.find(':').unwrap_or(s.len());
            &s[..end_of_host]
        }
    }

    /// Returns the port of the server name, if any.
    pub fn port(&self) -> Option<u16> {
        let s = self.as_str();

        #[allow(clippy::unnecessary_lazy_evaluations)]
        let end_of_host =
            s.find(']').map(|i| i + 1).or_else(|| s.find(':')).unwrap_or_else(|| s.len());

        (s.len() != end_of_host).then(|| {
            assert!(self.as_bytes()[end_of_host] == b':');
            s[end_of_host + 1..].parse().unwrap()
        })
    }

    /// Returns true if and only if the server name is an IPv4 or IPv6 address.
    pub fn is_ip_literal(&self) -> bool {
        self.host().parse::<Ipv4Addr>().is_ok() || self.as_str().starts_with('[')
    }
}

#[cfg(test)]
mod tests {
    use super::ServerName;

    #[test]
    fn ipv4_host() {
        ServerName::try_from("127.0.0.1").unwrap();
    }

    #[test]
    fn ipv4_host_and_port() {
        ServerName::try_from("1.1.1.1:12000").unwrap();
    }

    #[test]
    fn ipv6() {
        ServerName::try_from("[::1]").unwrap();
    }

    #[test]
    fn ipv6_with_port() {
        ServerName::try_from("[1234:5678::abcd]:5678").unwrap();
    }

    #[test]
    fn dns_name() {
        ServerName::try_from("example.com").unwrap();
    }

    #[test]
    fn dns_name_with_port() {
        ServerName::try_from("ruma.io:8080").unwrap();
    }

    #[test]
    fn empty_string() {
        ServerName::try_from("").unwrap_err();
    }

    #[test]
    fn invalid_ipv6() {
        ServerName::try_from("[test::1]").unwrap_err();
    }

    #[test]
    fn ipv4_with_invalid_port() {
        ServerName::try_from("127.0.0.1:").unwrap_err();
    }

    #[test]
    fn ipv6_with_invalid_port() {
        ServerName::try_from("[fe80::1]:100000").unwrap_err();
        ServerName::try_from("[fe80::1]!").unwrap_err();
    }

    #[test]
    fn dns_name_with_invalid_port() {
        ServerName::try_from("matrix.org:hello").unwrap_err();
    }

    #[test]
    fn parse_ipv4_host() {
        let server_name = ServerName::try_from("127.0.0.1").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.host(), "127.0.0.1");
    }

    #[test]
    fn parse_ipv4_host_and_port() {
        let server_name = ServerName::try_from("1.1.1.1:12000").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.host(), "1.1.1.1");
    }

    #[test]
    fn parse_ipv6() {
        let server_name = ServerName::try_from("[::1]").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.host(), "[::1]");
    }

    #[test]
    fn parse_ipv6_with_port() {
        let server_name = ServerName::try_from("[1234:5678::abcd]:5678").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.host(), "[1234:5678::abcd]");
    }

    #[test]
    fn parse_dns_name() {
        let server_name = ServerName::try_from("example.com").unwrap();
        assert!(!server_name.is_ip_literal());
        assert_eq!(server_name.host(), "example.com");
    }

    #[test]
    fn parse_dns_name_with_port() {
        let server_name = ServerName::try_from("ruma.io:8080").unwrap();
        assert!(!server_name.is_ip_literal());
        assert_eq!(server_name.host(), "ruma.io");
    }
}
