# [unreleased]

# 0.6.0

Breaking changes:

- Upgrade html5ever to `0.35.0`

# 0.5.0

Upgrade `ruma-common` to 0.16.0.

# 0.4.1

Improvements:

- Add unstable support for the `data-mx-external-payment-details` attribute for
  spans, according to MSC4286.

# 0.4.0

Upgrade `ruma-common` to 0.15.0.

# 0.3.0

Breaking Changes:

- `MatrixElement::Div` is now a newtype variant.
- `AnchorData`'s `name` field was removed, according to MSC4159 / Matrix 1.12.
- html5ever was bumped to a new major version. A breaking change in the parsing
  API required us to rewrite the `Html` type.
  - `Html::sanitize()` and `Html::sanitize_with()` take a non-mutable reference.
  - `NodeRef` and `Children` are now owned types and no longer implement `Copy`.
  - `NodeData::Text`'s inner string and the `attrs` field of `ElementData` are
    now wrapped in `RefCell`s. 

Improvements:

- Add support for mathematical messages, according to MSC2191 / Matrix 1.11

# 0.2.0

Breaking Changes:

- Do not export `Node` in the public API, it is not usable on its own and it is
  not in the output of any public method.
- `Html::sanitize_with` now takes a reference to `SanitizerConfig`.

Improvements:

- Add support for deprecated HTML tags, according to Matrix 1.10
- Allow to navigate through the HTML tree with `Html::first_child()`,
  `Html::last_child()` or `Html::children()`
- Add `ElementData::to_matrix` to convert it to a type using enums for HTML
  elements and attributes suggested by the Matrix Specification, behind the
  `matrix` cargo feature.

# 0.1.0

Initial release
