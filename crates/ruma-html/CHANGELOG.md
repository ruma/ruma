# [unreleased]

Breaking Changes:

- Do not export `Node` in the public API, it is not usable on its own and it is
  not in the output of any public method.

Improvements:

- Add support for deprecated HTML tags, according to Matrix 1.10
- Allow to navigate through the HTML tree with `Html::first_child()`,
  `Html::last_child()` or `Html::children()`

# 0.1.0

Initial release
