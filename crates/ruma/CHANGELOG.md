# [unreleased]

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
