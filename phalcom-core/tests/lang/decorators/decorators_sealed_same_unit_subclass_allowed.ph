// area: decorators
// spec: decorators/sealed.md; annotations-data.md §"@sealed";
//       drafts/sealed-classes.md §1.3 / S-2
// status: PASS
// `@sealed` is sealed-to-the-compilation-unit, not sealed-to-nobody: a
// subclass declared in the SAME unit as its sealed superclass is legal. This
// is the positive half of the decorator's headline enforcement; the negative
// half (a cross-unit subclass) is NOT reachable for a user class — see
// `decorators_sealed_cross_unit_needs_isolation.ph` for why.
//
// Before DEFERRED CB-3 this file's `@variant` sibling would have been fine but
// the equivalent inside a *bootstrap*-sealed class raised a false
// `@variant requires @sealed`; the gate now reads the attribute list OR
// `VM::sealed_classes`.

@sealed class Shape {}
class Square is Shape { area { 4 }
}

System.print(Square.new().area)
System.print(Square.new().is(Shape))
