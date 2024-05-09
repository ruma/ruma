# [unreleased]

# 0.9.0

Breaking changes:

- The http crate had a major version bump to version 1.1

Improvements:

- The type returned by `get_supported_versions::known_versions()` was simplified

# 0.8.0

Breaking changes:

* Fix the format of the keys in `invitation::store_invitation::v2::PublicKeys` according to a spec
  clarification

# 0.7.1

Improvements:

* Update links to the latest version of the Matrix spec

# 0.7.0

No changes for this version

# 0.6.0

Breaking changes:

* Upgrade dependencies

# 0.5.0

Breaking changes:

* Rename `status` to `discovery`

Improvements:

* Add `room_type` to `store_invitation::Request` according to MSC3288
* Add `discovery::get_supported_versions` according to MSC2320

# 0.4.0

Breaking changes:

* Borrow `mxid` in `invitation::sign_invitation_ed25519::v2::Request`

# 0.3.0

Breaking changes:

* Upgrade dependencies

Improvements:

* Add more endpoints:
  
  ```rust
  association::unbind_3pid::v2,
  invitation::store_invitation::v2
  ```

# 0.2.0

Breaking changes:

* Make `tos::get_terms_of_service::v2::Policies` non-exhaustive

Improvements:

* Add more endpoints:

  ```rust
  association::{
      check_3pid_validity::v2,
      bind_3pid::v2,
  },
  invitation::sign_invitation_ed25519::v2,
  ```

# 0.1.0

Breaking changes:

* Upgrade public dependencies

Improvements:

* Add more endpoints:
  ```rust
  association::{
      email::{
          create_email_validation_session::v2,
          validate_email::v2,
          validate_email_by_end_user::v2,
      },
      msisdn::{
          create_msisdn_validation_session::v2,
          validate_msisdn::v2,
          validate_msisdn_by_phone_number::v2,
      },
  },
  key::{
      check_public_key_validity::v2,
      get_public_key::v2,
      validate_ephemeral_key::v2,
  },
  lookup::{
      get_hash_parameters::v2,
      lookup_3pid::v2,
  },
  status::v2,
  tos::{
      accept_terms_of_service::v2,
      get_terms_of_service::v2,
  }
  ```

# 0.0.1

Initial release with the following endpoints:

```rust
authentication::{get_account_information::v2, logout::v2, register::v2}
```
