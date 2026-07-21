# E006 · Reading an inherited field reports "Read-before-write", and following the message's advice silently creates a second slot

- **Status:** OPEN — confirmed 2026-07-20 (reproduced under `target/debug/phalcom`, both halves)
- **Severity:** major — diagnostics defect whose implied remedy introduces silent field shadowing. The *runtime* behaviour is spec-correct at every step; the defect is the path a user is steered down.
- **Subsystem:** compiler diagnostics × field layout × inheritance
- **Related:** narrative in [`docs/learn/vm/supersend.md`](../learn/vm/supersend.md). Not the same family as E001/E002/E004 — nothing is dropped or leaked here; the bug is that a correct rule is reported by the wrong error.

## Defect

Two spec rules, both deliberate and both stated:

- `docs/spec/current/classes.md:125` — **"Fields are private to the declaring class and not
  inherited-visible."**
- `docs/spec/current/classes.md:172` — **"A subclass gets its own fresh slot."**

A subclass reading a field declared by its parent violates the first rule. But there is **no
field-privacy diagnostic in `CompilerError` at all**. The only error that can fire is a
flow-analysis one (`phalcom-core/src/compiler/lib/error.rs:100-102`):

```rust
/// A field read whose name is in no assignment set in the class (ADR-0011).
#[error("Read-before-write: field '{0}' is used before being assigned anywhere in this class.")]
ReadBeforeWrite(String),
```

The analysis is per-class by design: `Expr::Field` codegen
(`phalcom-core/src/compiler/lib/expr.rs:254-278`) fetches `field_layouts` for `self.current_class`
only — the class being compiled — and raises `ReadBeforeWrite` if the name is absent. `ClassLayout`'s
doc comment (`phalcom-core/src/vm/mod.rs:52-57`) states the intent: fields are non-inherited, "no
superclass merge."

So a privacy violation is reported as a missing assignment. The message's implied remedy — assign the
field in this class — is exactly the action that creates a **second slot**, per `classes.md:172`.
The parent's value and the subclass's value then coexist on one object under one name, each visible
only to the methods of its own declaring class.

This is adjacent to super-construct, whose purpose (`super.new(x)`, ADR-0040 + idempotent
`NewInstance`) is to fill the parent's inherited slots — which the subclass then cannot read.

## Reproduction

Both under `target/debug/phalcom`.

```phalcom
// A — the misleading diagnostic. `_x` IS assigned, by Base's constructor,
//     and super.new(x) demonstrably runs it (see control C).
class Base {
  construct new(x) { _x = x }
}
class Derived extends Base {
  construct new(x) { super.new(x) }
  peek => _x
}
System.print(Derived.new(9).peek)
// -> Error: Read-before-write: field '_x' is used before being assigned anywhere in this class.
```

```phalcom
// B — following the message's advice. Compiles; silently two slots.
class Base {
  construct new(x) { _x = x }
  baseSees => _x
}
class Derived extends Base {
  construct new(x) { super.new(x); _x = 999 }
  derivedSees => _x
}
const d = Derived.new(7)
System.print("Base sees:    " + d.baseSees.toString)
System.print("Derived sees: " + d.derivedSees.toString)
// -> Base sees:    7
// -> Derived sees: 999
```

**Control** (proves the parent slot really is filled, so "was never assigned" is misleading rather
than merely terse):

```phalcom
class Base {
  construct new(x) { _x = x }
  x => _x
}
class Derived extends Base {
  construct new(x, y) { super.new(x); _y = y }
  sum => self.x + _y
}
System.print(Derived.new(3, 4).sum)   // -> 7
```

Reading through the parent's accessor works and returns the value `super.new` wrote. Only the direct
field read is refused, and only with the wrong message.

## Coverage

- `ReadBeforeWrite` has **zero occurrences anywhere under `phalcom-core/tests/`** — the diagnostic is
  untested in any form.
- The two-slot outcome *is* covered, as a **passing feature test**:
  `phalcom-core/tests/lang/inheritance/inheritance_super_construct_same_field.ph` asserts that a
  subclass field sharing an inherited field's name gets its own slot ("fields stack, never alias").
  That is correct and desirable when a subclass *deliberately* declares a same-named field. Nothing
  records that the identical mechanism, arrived at by following `ReadBeforeWrite`'s advice, is a
  trap.

## Fix direction (NOT implemented / NOT verified)

Sketch of the space, not a prescription — a reproduced diagnosis is not a verified fix in this
codebase (see [README](README.md)); E004 is the standing example of a correct diagnosis with an
unimplementable prescription. Re-derive from code.

The information needed to tell the two cases apart is available at the raise site: if the field name
is absent from `self.current_class`'s layout but **present in an ancestor's**, this is a privacy
violation, not a read-before-write, and deserves its own `CompilerError` variant saying so — ideally
naming the declaring class and pointing at the accessor route (the control above), since that is the
supported way to reach the value.

Care is needed on two points. First, the diagnostic must not imply that assigning the field would be
correct, since that is the shadowing path. Second, a subclass declaring a same-named field on purpose
is a **supported feature** with a passing test — any new error must not fire on that case, which
means the distinguishing signal is *read without assignment in this class* versus *assignment in this
class*, not the name collision itself.

Whatever lands: add fixtures for both diagnostics, since `ReadBeforeWrite` currently has none.
