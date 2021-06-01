# [unreleased]

# 0.1.2

Improvements:

* Bump version of `ruma-common` and `ruma-client-api`, switching the canonical
  location of `ThirdPartyIdentifier`
  (now `ruma::thirdparty::ThirdPartyIdentifier`)

  For backwards compatibility, it is still available at the old path in
  `ruma::client::api::r0::contact::get_contacts`

# 0.1.1

Improvements:

* Bump versions of `ruma-client-api` and `ruma-events` for unstable spaces
  support

# 0.1.0

First release with non-prerelease dependencies! 🎉
