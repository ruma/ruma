# 0.18.0 (unreleased)

Breaking changes:

* Remove deprecated constructors
* Remove deprecated `is_` methods
* Rename `ServerKeyAlgorithm` to `SigningKeyAlgorithm`
* Rename `ServerKeyId` to `ServerSigningKeyId`
* Rename `Error::MissingSigil` to `Error::MissingLeadingSigil`
* Merge all missing delimiter error variants
* Update the set of conversion trait implementations for crypto algorithms

Improvements:

* Add `DeviceKeyId::from_parts` and `SigningKeyId::from_parts`
* Add `DeviceIdBox` and `ServerNameBox` type aliases
* Clean up debug logs for identifiers
* Allow crypto algorithm enums to contain any string, for forwards compatibility
  and to allow experimentation with non-specified ones

# 0.17.4

Bug fixes:

* Fix key algorithm serde (de)serialization
* Fix `key_algorithms::{DeviceKeyAlgorithm, ServerKeyAlgorithm}` not implementing `Deserialize` and
  `Serialize` with `default-features = false` + `features = ["serde", …]`

# 0.17.3

Bug fixes:

* Fix `key_algorithms::{DeviceKeyAlgorithm, ServerKeyAlgorithm}` not implementing `Deserialize` and
  `Serialize` with the default feature set
  * This bug was introduced in 0.17.2
  * This problem still persists with `default-features = false` + `features = ["serde", …]`, but
    there was never a reason to use ruma-identifiers like that

# 0.17.2 (yanked)

Bug fixes:

* Fix `RoomVersionId::is_version_6` actually checking for version 5
* Fix `TryFrom` implementations for `RoomVersionId`, which previously returned
  `RoomVersionId::Custom` for the input `6`

Improvements:

* Add macros to construct identifiers with compile-time validation

# 0.17.1

Improvements:

* Add `.as_bytes()` to all identifier types that already had `.as_str()`

# 0.17.0

Breaking changes:

* Change `RoomVersionId` from being an opaque struct to a non-exhaustive enum
  * The constructor functions and `is_` predicates are now deprecated
  * Remove `RoomVersionId::custom`. It could be used to create invalid room versions (empty or
    exceeding 32 code points in length). Use the `TryFrom<&str>` or `TryFrom<String>` implementation
    instead.
* Remove diesel integration. If you were using it, please comment on the corresponding issue:
  [#22](https://github.com/ruma/ruma-identifiers/issues/22)
* Remove `TryFrom<Cow<'_, str>>` implementations for identifier types
* Update `parse_with_server_name`s signature (instead of `Into<String>` it now requires
  `Into<Box<str>>` of the id type). This is technically a breaking change, but extremely unlikely
  to affect any existing code.
* Modify identifier types to use the new `ServerName` type:
  * Change signature of `new()` methods of `EventId`, `RoomId`, and `UserId` from
    ```rust
    fn new(&str) -> Result<Self, Error>
    //...
    ```
    to
    ```rust
    fn new(&ServerName) -> Self
    ```

  * Change signature of `server_name()` for `EventId`, `RoomAliasId`, `RoomId`, `RoomIdOrAliasId`, `UserId` from
    ```rust
    fn server_name() -> &str
    ```
    to
    ```rust
    fn server_name() -> &ServerName
    ```
* Change `DeviceId` from being an alias for `String` to being a newtype around `str`

  This means owned device IDs are now `Box<DeviceId>`.

Deprecations:

* Mark `server_name::is_valid_server_name` as deprecated in favor of `<&ServerName>::try_from()`

Improvements:

* Add `DeviceKeyId`, `DeviceKeyAlgorithm`, `ServerKeyId`, and `ServerKeyAlgorithm`
* Add a `ServerName` type
  * This is a dynamically-sized type, if you need to store it use `Box<ServerName>`

# 0.16.2

Improvements:

* Update the internal representation of identifiers to be more compact
* Add `RoomVersionId::version_6` and `RoomVersionId::is_version_6`
* Add `PartialOrd` and `Ord` implementations for `RoomVersionId`

# 0.16.1

Bug fixes:

* Change `PartialEq` implementations to compare IDs with string literals from `str` to `&str`
  * This is technically a breaking change, but the previous implementations were extremely
    unlikely to actually be used

# 0.16.0

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
