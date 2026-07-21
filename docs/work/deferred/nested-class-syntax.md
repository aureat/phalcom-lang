# Deferred: nested-class syntax

Nested class declarations remain unsupported. In a block, declaration-shaped
`class Name` is rejected with `class.nested_declaration`; `class.` remains an
expression and lowers through the normal primary-expression path to
`self.class`.

Future support needs a complete block-level declaration grammar before this
route is broadened. That design must define declaration boundaries, which
forms can nest, name binding and lookup, and how decorators attach to nested
declarations. Do not treat the `class.` routing distinction as nested-class
semantics.
