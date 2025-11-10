<!-- Keep this comment so the content is always included as a new paragraph -->
## Container Attributes

* `#[ruma_enum(rename_all = "rule")]` or
  `#[ruma_enum(rename_all(prefix = "prefix", rule = "rule"))]` - Override the string representation
  of all unit variants by using the given `prefix` and `rule`. By default, the string representation
  uses the unit variant name, with the same case. This attribute allows to change the string
  representation by adding the given prefix or applying case transformation according to one of the
  following rules. When using the first syntax, the prefix is assumed to be empty. The case of the
  name of the rule always matches the transformation of the string.

  * `lowercase` - Convert to lowercase. `MyVariant` becomes `myvariant`.
  * `UPPERCASE` - Convert to uppercase. `MyVariant` becomes `MYVARIANT`.
  * `camelCase` - Convert the first character to lowercase. `MyVariant` becomes `myVariant`.
  * `snake_case` - Add a `_` before all uppercase characters except at the start and convert all
    characters to lowercase. `MyVariant` becomes `my_variant`.
  * `SCREAMING_SNAKE_CASE` - Add a `_` before all uppercase characters except at the start and
    convert all characters to uppercase. `MyVariant` becomes `MY_VARIANT`.
  * `kebab-case` - Add a `-` before all uppercase characters except at the start and convert all
    characters to lowercase. `MyVariant` becomes `my-variant`.
  * `M_MATRIX_ERROR_CASE` - The case usually used for error codes in the Matrix specification.
    This is the same as `SCREAMING_SNAKE_CASE`, prepended with `M_`. `MyVariant` becomes
    `M_MY_VARIANT`.
  * `m.snake_case` => The case usually used for namespaced fields in the Matrix specification.
    This is the same as `snake_case`, prepended with `m.`. `MyVariant` becomes `m.my_variant`.
  * `m.lowercase` => A variant of `m.snake_case` based on the `lowercase` rule. `MyVariant`
    becomes `m.myvariant`.
  * `.m.rule.snake_case` => A variant of `m.snake_case` where the prefix is `.m.rule.`, usually
    used for push rules `rule_id`s in the Matrix Specification. `MyVariant` becomes
    `.m.rule.my_variant`.
  * `m.role.snake_case` => A variant of `m.snake_case` where the prefix is `m.role.`, usually used
    for contact methods roles in the Matrix Specification. `MyVariant` becomes
    `m.role.my_variant`.

## Field attributes

These attributes are only valid on unit variants.

* `#[ruma_enum(rename = "value")]` - Override the main string representation of the variant. The
  `value` that is provided is the string representation of the variant.
