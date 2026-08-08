// area: compile-errors
// spec: docs/spec/v0.2/decorators/native.md §"Legality"
// status: NEGATIVE
// `@native` is legal only on `Method`/`Getter`/`Setter`/`Construct` — a
// class-level `@native` is `attr.illegal_target`.

@native
class Foo { bar => 1
}
