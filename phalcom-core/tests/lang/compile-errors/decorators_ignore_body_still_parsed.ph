// area: compile-errors
// spec: docs/spec/v0.2/decorators/ignore.md §"Semantics"
// status: NEGATIVE
// `@ignore` mutes *compilation*, not *lexing and parsing* — it is not a
// comment and not `#if 0`. A syntax error inside an `@ignore` body must
// still be a parse error, proving the body is parsed before it is dropped.

class Draft {
  @ignore broken( {
    1 +
  }
}
