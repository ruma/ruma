# [unreleased]

# 0.13.0

Breaking changes:

- Remove `isahc` feature

Improvements:

- Add `error_kind` accessor method to `Error<E, ruma_client_api::Error>`

# 0.12.0

No changes for this version

# 0.11.0

No changes for this version

# 0.10.0

Breaking changes:

* Upgrade dependencies

# 0.9.0

Breaking changes:

* Upgrade dependencies

# 0.8.0

Breaking changes:

* Upgrade dependencies
* The whole `Client` is now feature-gated (`client-api` feature).
  We may introduce a separate `FederationClient` and possibly other types like
  that in the future.

Improvements:

* Rewrite `Client` initialization and store server-supported Matrix versions in
  it, to determine whether to use stable, unstable or r0 paths for endpoints

# 0.7.0

Breaking changes:

* Upgrade dependencies

# 0.6.0

Breaking changes:

* Upgrade ruma-client-api to 0.11.0

# 0.5.0

Breaking changes:

* Make `Client` generic over the http client
* Make the ruma-client-api dependency optional
* Upgrade dependencies

Improvements:

* Add support for multiple HTTP clients
