# [unreleased]

Improvements:

* Add support for historical uppercase MXIDs 

# 0.14.1

Breaking changes:

* Our Minimum Supported Rust Version is now 1.36.0
  * This is done in a patch version because it is only a documentation change. Practially, a new
    project using even ruma-identifiers 0.14 won't build out of the box on older versions of Rust
    because of an MSRV bump in a minor release of an indirect dependency. Using ruma-identifiers
    with older versions of Rust will potentially continue to work with some crates pinned to older
    versions, but won't be tested in CI.

Improvements:

* Remove the dependency on `lazy_static` and `regex`
* We now support [historical user IDs](https://matrix.org/docs/spec/appendices#historical-user-ids)
