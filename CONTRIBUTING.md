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

- [Matrix spec Documentation](https://matrix.org/docs/spec/client_server/latest)
- Documentation to other Ruma modules:
  - [ruma-events](https://docs.rs/ruma-events/)
  - [ruma-api](https://docs.rs/ruma-api/)
  - [ruma-client](https://docs.rs/ruma-client/)

## Chat Rooms

- Ruma Matrix room: [#ruma:matrix.org](https://matrix.to/#/#ruma:matrix.org)
- Matrix Developer room: [#matrix-dev:matrix.org](https://matrix.to/#/#matrix-dev:matrix.org)

# Reporting Issues

If you find any bugs, inconsistencies or other problems, feel free to submit
a GitHub [issue](issues).

If you have a quick question, it may be easier to leave a message on
[#ruma:matrix.org](https://matrix.to/#/#ruma:matrix.org).

Also, if you have trouble getting on board, let us know so we can help future
contributors to the project overcome that hurdle too.

# Submitting Code

Ready to write some code? Great! Here are some guidelines to follow to
help you on your way:

## Coding Style

### Common types

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

### Import Formatting

Organize your imports into three groups separated by blank lines:

1. `std` imports
1. External imports (from other crates)
1. Local imports (`self::`, `super::`, `crate::` and things like `LocalType::*`)

For example,

```rust
use std::collections::BTreeMap;

use ruma_api::ruma_api;

use super::MyType;
```

Also, group imports by module. For example, do this:

```rust
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    fmt::{Debug, Display, Error as FmtError, Formatter},
};
```

as opposed to:

```rust
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};
```

### Serde Imports

When importing methods and types from `serde_json`, methods should be such as
`serde_json::{from,to}_{slice,string,value,vec}` should be imported as
`{from,to}_json_{slice,string,value,vec}`.

For example:

```rust
use serde_json::{
  from_value as from_json_value,
  to_str as to_json_str,
};
```

Also, `serde_json::Value` should be imported as `JsonValue`.

### Code Formatting and Linting

Use `rustfmt` to format your code and `clippy` to lint your code. Before
committing your changes, go ahead and run `cargo fmt` and `cargo clippy
--all-targets --all-features` on the repository to make sure that the
formatting and linting checks pass in CI. Note that `clippy` warnings are
reported as errors in CI builds, so make sure to handle those before
comitting as well. (To install the tools, run `rustup component add rustfmt
clippy`.)

### Commit Messages

Write commit messages using the imperative mood, as if completing the sentence:
"If applied, this commit will \_\_\_." For example, use "Fix some bug" instead
of "Fixed some bug" or "Add a feature" instead of "Added a feature".

(Take a look at this
[blog post](https://www.freecodecamp.org/news/writing-good-commit-messages-a-practical-guide/)
for more information on writing good commit messages.)

## Modifying Endpoints

### Matrix Spec Version

Use the latest r0.x.x documentation when adding or modifying code. We target
the latest minor version of the Matrix specification. (Note: We might
reconsider this when the Client-Server API hits r1.0.0.)

### Endpoint Documentation Header

Add a comment to the top of each endpoint file that includes the path
and a link to the documentation of the spec. You can use the latest
version at the time of the commit. For example:

```rust
//! [GET /.well-known/matrix/client](https://matrix.org/docs/spec/client_server/r0.4.0#get-well-known-matrix-client)
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

Add your changes to the [change log](CHANGELOG.md). If possible, try to
find and denote the version of the spec that included the change you are
making.

## Submitting PRs

Once you're ready to submit your code, create a pull request, and one of our
maintainers will review it. Once your PR has passed review, a maintainer will
merge the request and you're done! 🎉

## Where do I start?

If this is your first contribution to the project, we recommend taking a look
at one of the [open issues][] we've marked for new contributors.

It may be helpful to peruse some of the documentation for `ruma-events` and
`ruma-api` listed above for some context.

[open issues]: https://github.com/ruma/ruma-client-api/issues?q=is%3Aopen+is%3Aissue+label%3Aeffort%2Feasy

# Testing

Before committing, run `cargo check` to make sure that your changes can build, as well as running the formatting and linting tools [mentioned above](#code-formatting-and-linting).

# Contact

Thanks again for being a contributor! If you have any questions, join us at
[#ruma:matrix.org](https://matrix.to/#/#ruma:matrix.org).
