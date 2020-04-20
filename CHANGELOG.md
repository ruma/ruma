# [unreleased]

Breaking changes:

* Update `RoomId::parse_with_server_name`s bounds from `Into<Cow<'_, str>>` to
  `AsRef<str> + Into<String>`. While this is a breaking change, it is not expected to actually
  require code changes.

Improvements:

* Add conversion functions for `RoomIdOrAliasId`
  * `impl From<RoomId> for RoomIdOrAliasId`
  * `impl From<RoomAliasId> for RoomIdOrAliasId`
  * `impl TryFrom<RoomIdOrAliasId> for RoomId`
  * `impl TryFrom<RoomIdOrAliasId> for RoomAliasId`
  * `RoomIdOrAliasId::into_either` (if the optional dependency `either` is activated with the
    identically named feature)

# 0.15.1

Bug fixes:

* Fix docs.rs build

# 0.15.0

Breaking changes:

* All identifiers now allocate at maximum one string (localpart and host are no longer stored
  separately)
  * Because of this, these traits are now implemented for them and only allocate in the obvious
    case:
    * `impl From<…Id> for String`
    * `impl AsRef<str> for …Id`
    * `impl TryFrom<Cow<'_, str>> for …Id`
    * `impl TryFrom<String> for …Id`
    * `PartialEq` for `String`s and string slices
  * Additionally, the `Hash` implementations will now yield the same hashes as hashing the string
    representation
    * Note that hashes are generally only guaranteed consistent in the lifetime of the program
      though, so do not persist them!
  * The `hostname` methods have been rename to `server_name` and updated to return string slices
    instead of `&url::Host`
* `Error::InvalidHost` has been renamed to `Error::InvalidServerName`, because it also covers errors
  in the port, not just the host part section of the server name
* The random identifier generation functions (`Id::new`) are now only available if the `rand`
  feature of this crate is enabled

Improvements:

* Add support for historical uppercase MXIDs
* Made all dependencies optional
  * `serde` is the only one that is enabled by default
* The `user_id` module is now public and contains `fn localpart_is_fully_conforming`
  * This function can be used to determine whether a user name (the localpart of a user ID) is valid
    without actually constructing a full user ID first
* Add `UserId::parse_with_server_name`

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
