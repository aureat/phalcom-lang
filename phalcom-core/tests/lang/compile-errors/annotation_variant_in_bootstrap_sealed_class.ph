// area: compile-errors
// spec: decorators/sealed.md; annotations-data.md §"@variant";
//       drafts/sealed-classes.md S-1; DEFERRED CB-3
// status: NEGATIVE
// **Regression guard for DEFERRED CB-3.** `Option` is sealed by bootstrap
// (`vm/bootstrap.rs` writes `VM::sealed_classes` directly, because `None` has
// no `.ph` reopen to hang an annotation on), so it carries NO `@sealed`
// attribute.
//
// The `@variant` gate used to read the attribute list alone, so this file
// raised the FALSE diagnostic:
//
//     attr.illegal_target: `@variant` requires its enclosing class `Option`
//                          to also carry `@sealed`
//
// — a claim that `Option` is not sealed, about a class that is. The gate now
// takes the union of the attribute list and `VM::sealed_classes`, so `@variant`
// is correctly ALLOWED here, and the arm is then rejected for the true reason:
// the sibling class it expands to (`Foo extends Option`) is not in `Option`'s
// compilation unit.
//
// Reading `VM::sealed_classes` *alone* would not fix this — it would invert it.
// A user's own `@sealed class Shape` is not in that table while its body is
// being expanded (`class_decl.rs` inserts it only after the body compiles), so
// a table-only gate would reject every user `@variant`. Both sources are
// required; see `annotation_variant_requires_sealed.ph` for the other half.

class Option {
  @variant Foo(x:)
}
