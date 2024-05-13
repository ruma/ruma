# [unreleased]

# 0.10.1

Upgrade `ruma-events` to 0.28.1.

# 0.10.0

- Bump MSRV to 1.75
- The http crate had a major version bump to version 1.1
- The `client-isahc` feature was removed
- Most ruma crates had breaking changes, refer to their changelogs for more
  details

# 0.9.4

Upgrade `ruma-events` and re-export its new `unstable-msc4075` feature.

# 0.9.3

Upgrade `ruma-client-api` and re-export its new `unstable-msc3983` feature.

# 0.9.2

Upgrade `ruma-events` and re-export its new `unstable-msc3401` feature.

# 0.9.1

This release only exists to regenerate documentation to pull in the latest
version of `ruma-events` for the `events` module.

# 0.9.0

- Bump MSRV to 1.70

# 0.8.2

Please refer to the changelogs of:

* ruma-common 0.11.3
* ruma-client-api 0.16.1
* ruma-federation-api 0.7.1
* ruma-identifiers-validation 0.9.1

# 0.8.1

Add the `server-util` feature, which activates a re-export of the new
`ruma_server_util` crate under `ruma::server_util`.

# 0.8.0

Please refer to the changelogs of:

* ruma-common 0.11.0
* ruma-appservice-api 0.8.0
* ruma-client-api 0.16.0
* ruma-federation-api 0.7.0
* ruma-server-util 0.1.0
* ruma-state-res 0.9.0

# 0.7.4

Improvements:

* Fix missing re-exports from `ruma-common`
* Upgrade `ruma-client-api` minimum version to 0.15.1

# 0.7.3

Upgrades ruma-common minimum version to 0.10.3.

# 0.7.2

Upgrades ruma-common minimum version to 0.10.2.

# 0.7.1

Upgrades ruma-common minimum version to 0.10.1.

# 0.7.0

Breaking changes:

* The `receipt` module is no longer exported.
  * `ReceiptType` has been split into two types under `events` and `api::client`.

# 0.6.3

Bug fixes:

* Fix serialization and deserialization of events with a dynamic `event_type`

Improvements:

* Add `From<&UserId>` and `From<&OwnedUserId>` implementations for `UserIdentifier`
* Add `UserIdentifier::email` constructor

# 0.6.2

Improvements:

* Add `StrippedPowerLevelsEvent::power_levels`
* Add (`Sync`)`RoomMemberEvent::membership`
* Export `events::room::member::Change`
  * Prior to this, you couldn't actually do anything with the
    `membership_change` functions on various member event types

# 0.6.1

Improvements:

* Re-export `ruma-common`s `js` Cargo feature

# 0.6.0

Please refer to the changelogs of:

* ruma-common 0.9.0
* ruma-client-api 0.14.0
* ruma-federation-api 0.5.0
* ruma-identity-service-api 0.5.0
* ruma-state-res 0.7.0

# 0.5.0

Please refer to the changelogs of:

* ruma-identifiers 0.21.0 and 0.22.0
* ruma-common 0.7.0 and 0.8.0
* ruma-events 0.25.0 and 0.26.0
* ruma-appservice-api 0.5.0
* ruma-client-api 0.13.0
* ruma-federation-api 0.4.0
* ruma-identity-service-api 0.4.0
* ruma-push-gateway-api 0.4.0
* ruma-client 0.8.0
* ruma-serde 0.6.0
* ruma-signatures 0.10.0
* ruma-state-res 0.6.0

# 0.4.0

Breaking changes:

* Upgrade ruma-state-res to 0.4.0
  * If you are not using state-res, there is no need to upgrade

# 0.3.0

Breaking changes:

* Upgrade sub-crates. The relevant breaking changes can be found in the changelogs of
  * ruma-events 0.24.1 (0.24.0 was yanked)
  * ruma-appservice-api 0.4.0
  * ruma-client-api 0.12.0
  * ruma-federation-api 0.3.0
  * ruma-identity-service-api 0.3.0
  * ruma-push-gateway-api 0.3.0
  * ruma-signatures 0.9.0
  * ruma-state-res 0.3.0

# 0.2.0

Breaking changes:

* Upgrade sub-crates. The relevant breaking changes can be found in the changelogs of
  * ruma-events 0.23.0
  * ruma-appservice-api 0.3.0
  * ruma-client-api 0.11.0
  * ruma-federation-api 0.2.0
  * ruma-identity-service-api 0.2.0
  * ruma-push-gateway-api 0.2.0
  * ruma-signatures 0.8.0
  * ruma-state-res 0.2.0

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
