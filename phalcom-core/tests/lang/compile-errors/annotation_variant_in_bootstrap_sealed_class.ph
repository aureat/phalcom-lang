// area: compile-errors
// spec: decorators/sealed.md; annotations-data.md §"@variant";
//       drafts/sealed-classes.md S-1; DEFERRED CB-3;
//       docs/forge/units/U-CLASSNS/implementation-spec.md §4.1
// status: NEGATIVE
// **Superseded by U-CLASSNS module-scoped class identity.** Before that
// unit, `VM::sealed_classes` was keyed by bare `Symbol` VM-wide, so this
// file's `class Option { ... }` reopened the *same* row bootstrap wrote for
// the kernel `Option` (`vm/bootstrap.rs` seals it directly — `None` has no
// `.ph` reopen to hang an `@sealed` attribute on). CB-3 was the false
// diagnostic that produced under a table-only gate; the fix was to union the
// attribute list with `VM::sealed_classes` under that shared key.
//
// Under `ClassKey { module, name }` (U-CLASSNS), this file compiles as its
// own module — its `class Option` is a *distinct*, unsealed class, not a
// reopen of the kernel one (decision 0065 ruling 1). Kernel-name reservation
// (ruling 3) is deferred to unit U-CLASSCLOSE, so this is not yet a compile
// error on that ground either. `@variant`'s sealed check correctly consults
// only this module's own `sealed_classes`/attribute-list rows for "the class
// being declared" (§4.1 — no core-module fallback for that site), finds
// neither, and rejects with `attr.illegal_target`. That is now genuinely
// correct, not the CB-3 bug: this `Option` really is unsealed.
//
// This fixture can no longer exercise the original CB-3 shape at all —
// same-symbol cross-module reopening of a bootstrap-sealed class is exactly
// what module-scoped identity removes. Kept as a negative case (still a real
// compile error) with the diagnostic it now actually produces; revisit once
// U-CLASSCLOSE lands kernel-name reservation, which may turn this into an
// `attr.unknown`-adjacent "reserved kernel name" error instead.

class Option {
  @variant Foo(x)
}
