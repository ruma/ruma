//! Matrix-spec compliant server names.

/// A Matrix-spec compliant server name.
#[repr(transparent)]
pub struct ServerName(str);

opaque_identifier_validated!(ServerName, ruma_identifiers_validation::server_name::validate);

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
}
