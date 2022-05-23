# [unreleased]

Breaking changes:

* Remove `PartialEq` implementation for `NotificationCounts`

# 0.5.0

Breaking changes:

* Upgrade dependencies

# 0.4.0

Breaking changes:

* Upgrade dependencies

# 0.3.0

Breaking changes:

* Upgrade dependencies

# 0.2.0

Breaking changes:

* Upgrade ruma-events to 0.23.0

# 0.1.0

Breaking changes:

* Remove `Copy` implementation for `NotificationCounts` to avoid simple changes
  being breaking
* Change `Box<RawJsonValue>` to `&RawJsonValue` in request types
* Upgrade public dependencies

# 0.0.1

* Add endpoint `send_event_notification::v1`
