# [unreleased]

# 0.10.0

Breaking changes:

* The `url` field of `Registration` is now an `Option<String>`. This should have
  always been the case.
- The http crate had a major version bump to version 1.1

# 0.9.0

Improvements:

- Add support for the appservice ping mechanism (MSC 2659 / Matrix 1.7)

# 0.8.1

Improvements:

* Update links to the latest version of the Matrix spec

# 0.8.0

Improvements:

* Add support for using the Authorization header (MSC2832 / Matrix 1.4)

# 0.7.0

Breaking changes:

* Remove `PartialEq` implementation for `Namespace`
* Remove `push_events::v1::IncomingRequest::try_into_sync_response` and the
  `helper` Cargo feature that was gating it
  * This API is no longer being used by the only known consumer
  * If you were using it, please let us know!

# 0.6.0

Breaking changes:

* Upgrade dependencies

# 0.5.0

Breaking changes:

* Upgrade dependencies

# 0.4.0

Breaking changes:

* Upgrade dependencies

# 0.3.0

Breaking changes:

* Upgrade ruma-client-api to 0.11.0
* Upgrade ruma-events to 0.23.0

# 0.2.0

Breaking changes:

* Fix endpoint versioning
* Upgrade dependencies

Improvements:

* Upgrade dependencies
* Add room visibility management endpoint

Bug fixes:

* Fix `push_events::v1::Request` serialization by sending a dictionary instead of an array on request body

# 0.1.0

Initial release.
