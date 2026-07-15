// Imported by `compile-errors/decorators_sealed_cross_unit_needs_isolation.ph`
// — not a standalone test driver.
//
// A **user** `@sealed` class in its own compilation unit. `@sealed` records
// `Shape` in `VM::sealed_classes` keyed to THIS module, so any subclass
// compiled in another module should trip `attr.sealed_violation`
// (`class_decl.rs`'s check). The driver documents why that check cannot
// currently be reached.
@sealed class Shape {}
