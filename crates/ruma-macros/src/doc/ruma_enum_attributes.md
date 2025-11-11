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

## Field attributes

These attributes are only valid on unit variants.

* `#[ruma_enum(rename = "value")]` - Override the main string representation of the variant. The
  `value` that is provided is the string representation of the variant.
