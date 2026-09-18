// area: compile-errors
// spec: docs/specs/source-declarations.md §6
// status: NEGATIVE
// Retired `@native` is not a declaration modifier or semantic authority.

@native
class Foo { bar { 1 }
}
