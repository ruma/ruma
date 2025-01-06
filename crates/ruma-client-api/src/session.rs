//! Endpoints for user session management.

#[cfg(feature = "unstable-msc3824")]
use ruma_common::serde::StringEnum;

#[cfg(feature = "unstable-msc3824")]
use crate::PrivOwnedStr;

pub mod get_login_token;
pub mod get_login_types;
pub mod login;
pub mod login_fallback;
pub mod logout;
pub mod logout_all;
pub mod refresh_token;
pub mod sso_login;
pub mod sso_login_with_provider;

/// Possible purposes for using the SSO redirect URL for OIDC-aware compatibility ([MSC3824]).
///
/// [MSC3824]: https://github.com/matrix-org/matrix-spec-proposals/pull/3824
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
#[cfg(feature = "unstable-msc3824")]
pub enum SsoRedirectOidcAction {
    /// The SSO redirect is for the purpose of signing an existing user in.
    Login,

    /// The SSO redirect is for the purpose of registering a new user account.
    Register,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
