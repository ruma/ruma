# 0.20.1

Deprecation of the crate. It is now part of ruma-common.

# 0.20.0

Breaking changes:

* Bump dependencies

# 0.19.0

Breaking changes:

* Remove the `RequestDeserializationError` and `ResponseDeserializationError`
  types in favor of using `DeserializationError` directly
* Replace singular `path` metadata field with `stable_path`, `r0_path` and
  `unstable_path`
* Require callers of `OutgoingRequest::try_into_http_request` to pass a list of
  server-supported matrix versions for selecting whether to use r0, unstable or
  stable paths
* Require callers of `IncomingRequest::try_from_http_request` to pass path
  arguments, to avoid unnecessary extra path parsing

# 0.18.5

Bug fixes:

* Stop adding a `Content-Type` header to requests without any fields
  * This fixes usage of the `.well-known` endpoints in a browser WASM
    environment if the server isn't configured to allow that header for
    cross-origin requests

# 0.18.4

Yanked.

# 0.18.3

Improvements:

* Refactor handling of newtype bodies in `ruma_api!`
  * Likely what fixed a bug affecting `ruma-federation-api`

# 0.18.2

Bug fixes:

* Fix (de)serialization of requests with a `#[ruma_api(body)]` field

# 0.18.1

Breaking changes:

* Generate "struct structs" for `Request` and `Response`, even if there are no
  fields
  * We used to generate unit structs when there's no fields
* Upgrade dependencies

Bug fixes:

* Allow all borrowing query and / or body parameters in `ruma_api!` to be
  `cfg`-conditional
  * This used lead to broken code being generated and was fixed by moving all
    `impl` block generation code into a derive macro (derive macros run after
    `cfg` expansion)

# 0.18.0

Yanked, this was the same as 0.17.2 but accidentally released with the minor
rather than the patch version segment incremented. It then turned out that it
did actually contain breaking changes, so 0.17.2 was yanked too.

# 0.17.2

Yanked since it unintentionally contained breaking changes.

# 0.17.1

Improvements:

* Allow `ruma_api!` to be used by crates that depend on matrix-sdk, but not (directly) on ruma or
  ruma-api

# 0.17.0

Breaking changes:

* Update the syntax of the `ruma_api!` macro. Colons are now required after the keywords `metadata`,
  `request` and `response`.
* The `EndpointError` trait now requires `std::error::Error`. This allows integrating
  `EndpointError`s in the common rust error ecosystem like `thiserror` and `anyhow`.
* The `Endpoint` trait has been replaced by four of new traits that each capture a subset of its
  previous functionality: `OutgoingRequest`, `OutgoingResponse`, `IncomingRequest`,
  `IncomingResponse`. See the API documentation for more details.
* Endpoint authentication is now more granularly defined by an enum `AuthScheme`
  instead of a boolean. The `ruma_api!` macro has been updated to require
  `authentication` instead of `requires_authentication`.
* Remove `error::Void`, the default error type for `ruma_api!` is now `error::MatrixError`
  (see below)

Improvements:

* The `EndpointError`s that come with ruma crates now implement `std::error::Error`
* Add a new `MatrixError` type to the `error` module that consists of a HTTP status code and JSON
  `body` and is the new default error type for `ruma_api!`

Bug fixes:

* Prevent panic when building requests with an invalid URI

# 0.16.1

Bug fixes:

* Update ruma-serde to 0.2.0, fixing some issues with query string deserialization (some issues
  still remain but will be fixed in a semver-compatible version)

# 0.16.0

Breaking changes:

* Update ruma-identifiers to 0.16.1
* Remove the `Outgoing` trait and update the `Endpoint` trait and code generation accordingly

Improvements:

* Remove dependency on the `url` crate

# 0.15.1

Bug fixes:

* Write `{}` to the body of responses without body fields (fix from ruma-api-macros)

# 0.15.0

Breaking changes:

* Emit an error on non-UTF8 characters in path segments
  * Before, they would be replaced by the unknown character codepoint
* `FromHttpResponseError` now has a generic parameter for the expected type of
  error the homeserver could return

Improvements:

* Enable deserialization of unsuccessful responses

# 0.14.0

Breaking changes:

* Update ruma-api-macros to 0.11.0
  * This includes a fix that uses `TryFrom<&str>` instead of serde_json for path segment
    deserialization

# 0.13.1

Improvements:

* Update ruma-api-macros to 0.10.1
  * `Incoming` types will now implement `Debug`

# 0.13.0

Breaking changes:

* Instead of one `Error` type, there is now many
  * The new types live in their own `error` module
  * They provide access to details that were previously hidden
* Our Minimum Supported Rust Version is now 1.40.0

# 0.12.1

Improvements:

* Update ruma-api-macros to 0.9.1 to support `#[ruma_api(raw_body)]`

# 0.12.0

Breaking changes:

* Our Minimum Supported Rust Version is now 1.39.0
* Support for the server-side use case has been restored. For details, see the documentation for
  `ruma_api!`, the new `Outgoing` trait and its derive macro

# 0.11.2

Improvements:

* Update ruma-api-macros to 0.8.2

# 0.11.1

Improvements:

* Update ruma-api-macros to 0.8.1

# 0.11.0

Breaking changes:

* To be able to use ruma-event's `EventResult` in ruma-client without large-ish refactorings to ruma-api, we removed support for the server-side use case in ruma-api 0.11.0. It will be added back in a future release.

Improvements:

* Our CI now tests ruma-api on Rust 1.34.2, beta and nightly in addition to stable
* Updated syn and quote to 1.0

# 0.10.0

Breaking changes:

* The `Endpoint` trait is now implemented directly on the relevant request type rather than having both the request and response be associated types.

Improvements:

* ruma-api now re-exports the `ruma_api` macro from ruma-api-macros. Downstream crates no longer need to depend on ruma-api-macros directly.
* The ruma-api and ruma-api-macros repositories have been merged into one Cargo workspace for easier dependency management and development.

# 0.9.0

Breaking changes:

* The `Request` and `Response` associated types on the `Endpoint` trait are now bounded by `std::convert::TryFrom` instead of `futures::future::FutureFrom`. This was done in preparation for futures 0.3 which does not have this trait.
* The conversions required to and from `http::Request` and `http::Response` for the `Request` and `Response` associated types on the `Endpoint` trait now use `Vec<u8>` as the body type. This removes the dependency on hyper. It's possible this will change again in a future release. See https://github.com/rustasync/team/issues/84 for details.

Improvements:

* Internal code quality improvements via clippy and rustfmt.

# 0.8.0

Breaking changes:

* The `Error` type is now an opaque struct that hides implementation details.
* Updates to ruma-identifiers 0.13.

Improvements:

* ruma-api now uses clippy to improve code quality.

# 0.7.0

Improvements:

* ruma-api now runs on stable Rust, requiring version 1.34 or higher.
* Updated all dependencies for upstream improvements.
* Updated all code to use Rust edition 2018.

# 0.6.0

Breaking changes:

* Hyper has been updated to version 0.12.
* A new variant to the `Error` enum for hyper errors has been added.
* Conversions between this crate's request and response types and the http crate's request and response types are now bidirectional.

# 0.5.0

Breaking changes:

* Types from hyper have been replaced with types from the http crate.
* The `Error` enum can no longer be matched exhaustively, to allow for future expansion without breaking the crate's API.

# 0.4.0

Breaking changes:

The crate has been redesign to focus on conversions between an endpoint's request and response types and Hyper request and response types. Implementations are expected to be generated via [ruma-api-macros].

[ruma-api-macros]: https://github.com/ruma/ruma/tree/main/ruma-api-macros

# 0.3.0

Breaking changes:

* `Endpoint::router_path` now returns a `&'static str`
* Added new required methods to `Endpoint`: `name`, `description`, `requires_authentication`, and `rate_limited`.

# 0.2.0

Breaking changes:

* `Endpoint::Query_params` must now be `Deserialize + Serialize`.

# 0.1.0

Initial release.
