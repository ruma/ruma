# [unreleased]

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
