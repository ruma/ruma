//! Matrix-spec compliant server names.

use std::net::Ipv4Addr;

/// A Matrix-spec compliant server name.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServerName(str);

opaque_identifier_validated!(ServerName, ruma_identifiers_validation::server_name::validate);

impl ServerName {
    /// Returns the server name without the port.
    pub fn without_port(&self) -> &str {
        if let Some(end_of_ipv6) = self.0.find(']') {
            &self.0[0..=end_of_ipv6]
        } else {
            // It's not ipv6, so ':' means the port starts
            let end_of_host = self.0.find(':').unwrap_or_else(|| self.0.len());
            &self.0[0..end_of_host]
        }
    }

    /// Returns true if and only if the server name is an IPv4 or IPv6 address.
    pub fn is_ip_literal(&self) -> bool {
        self.without_port().parse::<Ipv4Addr>().is_ok() || self.0.starts_with('[')
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::ServerName;

    #[test]
    fn ipv4_host() {
        assert!(<&ServerName>::try_from("127.0.0.1").is_ok());
    }

    #[test]
    fn ipv4_host_and_port() {
        assert!(<&ServerName>::try_from("1.1.1.1:12000").is_ok());
    }

    #[test]
    fn ipv6() {
        assert!(<&ServerName>::try_from("[::1]").is_ok());
    }

    #[test]
    fn ipv6_with_port() {
        assert!(<&ServerName>::try_from("[1234:5678::abcd]:5678").is_ok());
    }

    #[test]
    fn dns_name() {
        assert!(<&ServerName>::try_from("example.com").is_ok());
    }

    #[test]
    fn dns_name_with_port() {
        assert!(<&ServerName>::try_from("ruma.io:8080").is_ok());
    }

    #[test]
    fn empty_string() {
        assert!(<&ServerName>::try_from("").is_err());
    }

    #[test]
    fn invalid_ipv6() {
        assert!(<&ServerName>::try_from("[test::1]").is_err());
    }

    #[test]
    fn ipv4_with_invalid_port() {
        assert!(<&ServerName>::try_from("127.0.0.1:").is_err());
    }

    #[test]
    fn ipv6_with_invalid_port() {
        assert!(<&ServerName>::try_from("[fe80::1]:100000").is_err());
        assert!(<&ServerName>::try_from("[fe80::1]!").is_err());
    }

    #[test]
    fn dns_name_with_invalid_port() {
        assert!(<&ServerName>::try_from("matrix.org:hello").is_err());
    }

    #[test]
    fn parse_ipv4_host() {
        let server_name = <&ServerName>::try_from("127.0.0.1").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.without_port(), "127.0.0.1");
    }

    #[test]
    fn parse_ipv4_host_and_port() {
        let server_name = <&ServerName>::try_from("1.1.1.1:12000").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.without_port(), "1.1.1.1");
    }

    #[test]
    fn parse_ipv6() {
        let server_name = <&ServerName>::try_from("[::1]").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.without_port(), "[::1]");
    }

    #[test]
    fn parse_ipv6_with_port() {
        let server_name = <&ServerName>::try_from("[1234:5678::abcd]:5678").unwrap();
        assert!(server_name.is_ip_literal());
        assert_eq!(server_name.without_port(), "[1234:5678::abcd]");
    }

    #[test]
    fn parse_dns_name() {
        let server_name = <&ServerName>::try_from("example.com").unwrap();
        assert!(!server_name.is_ip_literal());
        assert_eq!(server_name.without_port(), "example.com");
    }

    #[test]
    fn parse_dns_name_with_port() {
        let server_name = <&ServerName>::try_from("ruma.io:8080").unwrap();
        assert!(!server_name.is_ip_literal());
        assert_eq!(server_name.without_port(), "ruma.io");
    }
}
