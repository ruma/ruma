# [unreleased]

# 0.9.0

Breaking changes:

- The http crate had a major version bump to version 1.1

# 0.8.0

No changes for this version

# 0.7.1

Improvements:

* Update links to the latest version of the Matrix spec

# 0.7.0

No changes for this version

# 0.6.0

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
