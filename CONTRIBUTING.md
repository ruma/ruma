Welcome! Thanks for looking into contributing to our project!

# Table of Contents

- [Looking for Help?](#looking-for-help)
  - [Documentation](#documentation)
  - [Chat Rooms](#chat-rooms)
- [Reporting Issues](#reporting-issues)
- [Submitting Code](#submitting-code)
  - [Coding Style](#coding-style)
  - [Modifying Endpoints](#modifying-endpoints)
  - [Submitting PRs](#submitting-prs)
  - [Where do I start?](#where-do-i-start)
- [Testing](#testing)
- [Contact](#contact)

# Looking for Help?

Here is a list of helpful resources you can consult:

## Documentation

- [Matrix spec Documentation](https://spec.matrix.org/latest/)

## Chat Rooms

- Ruma Matrix room: [#ruma:matrix.org](https://matrix.to/#/#ruma:matrix.org)
- Ruma Development Matrix room: [#ruma-dev:matrix.org](https://matrix.to/#/#ruma-dev:matrix.org)
- Matrix Developer room: [#matrix-dev:matrix.org](https://matrix.to/#/#matrix-dev:matrix.org)

# Reporting Issues

If you find any bugs, inconsistencies or other problems, feel free to submit
a GitHub [issue](https://github.com/ruma/ruma/issues/new).

If you have a quick question, it may be easier to leave a message in
[#ruma:matrix.org](https://matrix.to/#/#ruma:matrix.org).

Also, if you have trouble getting on board, let us know so we can help future
contributors to the project overcome that hurdle too.

# Submitting Code

Ready to write some code? Great! Here are some guidelines to follow to
help you on your way:

## Coding Style

In general, try to replicate the coding style that is already present. Specifically:

### Naming

For internal consistency, Ruma uses American spelling for variable names. Names may differ in the
serialized representation, as the Matrix specification has a mix of British and American English.

### Common Types

When writing endpoint definitions, use the following mapping from request /
response field types listed in the specification to Rust types:

Specification type | Rust type
-------------------|---------------------------------------------------------------------------------------------------------------------
`boolean`          | `bool`
`integer`          | `js_int::UInt` (unless denoted as signed, then `js_int::Int`)
`string`           | If for an identifier (e.g. user ID, room ID), use one of the types from `ruma-identifiers`. Otherwise, use `String`.
`object`           | `serde_json::Value`
`[…]`              | `Vec<…>`
`{string: …}`      | `BTreeMap<String, …>` (or `BTreeMap<SomeId, …>`)

### Code Formatting and Linting

We use [rustfmt] to ensure consistent formatting code and [clippy] to catch
common mistakes not caught by the compiler as well as enforcing a few custom
code style choices.

```sh
# if you don't have them installed, install or update the nightly toolchain
rustup install nightly
# … and install prebuilt rustfmt and clippy executables (available for most platforms)
rustup component add rustfmt clippy
```

Before committing your changes, run `cargo +nightly fmt` to format the code (if
your editor / IDE isn't set up to run it automatically) and
`cargo +nightly clippy --workspace`¹ to run lints.

You can also run all of the tests the same way CI does through `cargo xtask ci`.
This will take a while to complete since it runs all of the tests on stable
Rust, formatting and lint checks on nightly Rust, as well as some basic checks
on our minimum supported Rust version. It requires [rustup] to be installed and
the toolchains for those three versions to be set up (in case of a missing
toolchain, rustup will tell you what to do though). There are also
`cargo xtask ci stable|nightly|msrv` subcommands for only running one of the CI
jobs.

¹ If you modified feature-gated code (`#[cfg(feature = "something")]`), you
have to pass `--all-features` or `--features something` to clippy for it to
check that code

[rustfmt]: https://github.com/rust-lang/rustfmt#readme
[clippy]: https://github.com/rust-lang/rust-clippy#readme

### (Type) Privacy and Forwards Compatibility

Generally, all `struct`s that are mirroring types defined in the Matrix specification should have
all their fields `pub`lic. Where there are restrictions to the fields value beyond their type, these
should generally be implemented by creating or using a more constrained type than the spec uses for
that field – for example, we have a number of identifier types but the Matrix spec uses `string` for
fields that hold user IDs / room IDs and so on.

Almost all types in `ruma-common` and the API crates use the `#[non_exhaustive]` attribute, to allow
us to adapt to new minor releases of the Matrix specification without having a major release of our
crates. You can generally just apply `#[non_exhaustive]` to everything – it's a backwards compatible
change to remove it in the rare case it is not warranted.

Due to this combination of public fields and non-exhaustiveness, all `struct`s generally need a
constructor function or `From` / `TryFrom` implementation to be able to create them in a
straight-forward way (always going through `Deserialize` would be quite ugly).

### Import Formatting

Organize your imports into three groups separated by blank lines:

1. `std` imports
1. External imports (from other crates)
1. Local imports (`self::`, `super::`, `crate::` and things like `LocalEnum::*`)

For example,

```rust
use std::collections::BTreeMap;

use ruma_common::api::ruma_api;

use super::MyType;
```

### Commit Messages

The commit message should start with the _area_ that is affected by the change.
An area is usually the name of the affected crate without the `ruma-` prefix,
except for the ruma-common crate, where the area is usually the name of the
top-level module, like `api` or `identifiers`. For example, the description of
a commit that affects the ruma-events crate should look like
"events: Add new event".

Write commit messages using the imperative mood, as if completing the sentence:
"If applied, this commit will \_\_\_." For example, use "Fix some bug" instead
of "Fixed some bug" or "Add a feature" instead of "Added a feature".

(Take a look at this
[blog post](https://www.freecodecamp.org/news/writing-good-commit-messages-a-practical-guide/)
for more information on writing good commit messages.)

## Modifying Endpoints

### Matrix Spec Version

Use the [latest v1.x documentation](https://spec.matrix.org/latest/) when adding or modifying code. We target
the latest minor version of the Matrix specification.

### Endpoint Module Structure

Matrix uses [versioned endpoints](https://spec.matrix.org/latest/#endpoint-versioning) (with a few small exceptions),
we follow this versioning approach in modules as well.

We structure endpoints and their versions like the following;

`endpoint_name::v1`

All bits pertaining a specific version (that can be linked to in the spec) reside in such a module,
some bits may be shared between endpoint versions, but this should be handled on a case-by-case basis.

Endpoint files may have their version modules embedded;

```rs
// endpoint_name.rs

mod v1 {
  // (version-specific stuff)
}
```

This happens if the endpoint either has a single version, or a few versions of sufficiently small size.

### Endpoint Documentation Header

Add a comment to the top of each endpoint file that includes the path
and a link to the documentation of the spec. Replace the version
marker (`v3`) with a `*`, like so;

```rust
//! `GET /_matrix/client/*/sync`
```

Then, in the subsequent version module, embed the version and specification link like so;

```rs
pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3sync
}
```

### Naming Endpoints

When adding new endpoints, select the module that fits the purpose of the
endpoint. When naming the endpoint itself, you can use the following
guidelines:
- The name should be a verb describing what the client is requesting, e.g.
  `get_some_resource`.
- Endpoints which are basic CRUD operations should use the prefixes
  `create`, `get`, `update`, and `delete`.
- The prefix `set` is preferred to create if the resource is a singleton.
  In other words, when there's no distinction between `create` and `update`.
- Try to use names that are as descriptive as possible and distinct from
  other endpoints in all other modules. (For example, instead of
  `r0::room::get_event`, use `r0::room::get_room_event`).
- If you're not sure what to name it, pick any name and we can help you
  with it.

### Tracking Changes

If your changes affect the API of a user-facing crate (all except the `-macros` crates and
`ruma-identifiers-validation`), add an entry about them to the change log (`CHANGELOG.md`)
of that crate. Where applicable, try to find and denote the version of the spec that
included the change you are making.

## Submitting PRs

Once you're ready to submit your code, create a pull request, and one of our
maintainers will review it. Once your PR has passed review, a maintainer will
merge the request and you're done! 🎉

## Where do I start?

If this is your first contribution to the project, we recommend taking a look
at one of the [open issues][] we've marked for new contributors.

[open issues]: https://github.com/ruma/ruma/issues?q=is%3Aissue+is%3Aopen+label%3A"help+wanted"

# Testing

Before committing, run `cargo check` to make sure that your changes can build, as well as running the formatting and linting tools [mentioned above](#code-formatting-and-linting).
