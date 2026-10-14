// area: compile-errors
// spec: objects/enums.md
// status: NEGATIVE
// Class case markers remain unsupported for sealed classes, including names
// used by sealed bootstrap declarations.

@sealed class Option {
  @variant Foo(x:)
}
