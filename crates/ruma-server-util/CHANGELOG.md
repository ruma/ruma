# [unreleased]

Breaking changes:

- The `XMatrix::new` method now takes `OwnedServerName` instead of `Option<OwnedServerName>`
  for the destination, since servers must always set the destination.
- The `sig` field in `XMatrix` has been changed from `String` to `Base64` to more accurately
  mirror its allowed values in the type system.

Bug fixes:

- When encoding to a header value, `XMatrix` fields are now quoted and escaped correctly.
- Use http-auth crate to parse `XMatrix`. Allows to parse the Authorization HTTP
  header with full compatibility with RFC 7235

Improvements:

- Implement `Display`, `FromStr` and conversion to/from `http::HeaderValue` for
  `XMatrix`

# 0.3.0

Breaking changes:

- The headers dependency was upgraded to 0.4.0

# 0.2.0

No changes for this version

# 0.1.1

Improvements:

* Update links to the latest version of the Matrix spec

# 0.1.0

Improvements:

* Provide `XMatrix` type for Matrix federation authorization headers.
