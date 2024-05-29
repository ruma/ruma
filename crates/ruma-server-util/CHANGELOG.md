# [unreleased]

Breaking changes:

- The `XMatrix::new` method now takes `OwnedServerName` instead of `Option<OwnedServerName>`
  for the destination, since servers must always set the destination.

# 0.3.0

Breaking changes:

- The headers dependency was upgraded to 0.4.0

# 0.2.0

No changes for this version

# 0.1.1

Improvements:

* Update links to the latest version of the Matrix spec

# 0.1.0

Improvements:

* Provide `XMatrix` type for Matrix federation authorization headers.
