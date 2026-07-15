// area: compile-errors
// spec: docs/spec/v0.2/decorators/ignore.md §"Legality"
// status: NEGATIVE
// `@ignore` is legal only on `Method`/`Getter`/`Setter`/`Construct` — a
// `Field` is excluded because dropping a field changes instance layout
// (ADR-0011, ignore.md I-2). This is the trap fixture: a bare `retain` on
// `ClassDef::members` (checking nothing) would silently drop the field
// instead of raising `attr.illegal_target` — legality must be checked
// *before* the drop.

class Widget {
  @ignore var hidden
}
