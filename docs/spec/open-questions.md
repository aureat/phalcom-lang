# Open Questions

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

Undecided design points. Each must be resolved here before its dependent
implementation begins.

> **RESOLVED questions** are struck through and annotated with the deciding ADR.
> **Open questions** remain for future design sessions.

---

1. ~~**`let` vs `var`.**~~ **RESOLVED** → [ADR-0014](../adr/0014-let-and-var-bindings.md):
   `let` introduces an immutable binding; `var` introduces a mutable one.
   `var x` with no initializer reads as `None` (consistent with an unassigned
   field, see Q5 / absence → [ADR-0007](../adr/0007-option-as-abstract-with-some-none.md)).
   `let x` with no initializer is rejected at the declaration site.
   The lexer now needs both `let` and `var` keywords.

2. **`Number`.** One numeric type, or `Int` / `Float` split? Affects the VM value
   representation and every arithmetic opcode. Decide before the inliner.
   ([ADR-0005](../adr/0005-number-as-flat-f64.md) settled the single-type side
   as `f64`; the Int/Float surface split remains open.)

3. **External vs internal parameter names.** Swift allows `move(to target:)` —
   external label `to`, internal binding `target`. Phalcom currently binds the
   label name directly. Worth it? ([ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md)
   reserves a field for this in `Signature` so it can be added without changing
   selector identity — the question is still open, but it won't require a
   redesign once answered.)

4. **Class hierarchy mutability.** Is `Test.superclass = Test` legal at runtime
   (Smalltalk: yes) or sealed after definition (Wren: no)? Affects whether slot
   layouts and inline caches can assume stability.
   ([ADR-0009](../adr/0009-handle-arena-heap.md) notes that the handle heap keeps
   this implementable; the policy is not yet decided.)

5. **String interpolation syntax.** `"{name}"` is assumed. `"${name}"` and
   `"\(name)"` are alternatives.

6. **Set literal.** Currently `Set(...)`. `#{1, 2, 3}` remains available if the
   ceremony becomes annoying.

7. **Destructuring.** Tuples exist, so `let (a, b) = point` and
   `let [first, *rest] = list` are natural. Not yet specified.

8. **Modules / imports.** The `import` token exists; semantics are unspecified.

9. ~~**Error handling.** `throw` / `try` / `catch`, or `Result` as a sibling of
   `Option`?~~ **RESOLVED** → [ADR-0008](../adr/0008-layered-exceptions-and-result.md)
   (see also [Error Handling](error-handling.md)):
   both, layered — unwinding `throw`/`Error` for the exceptional path, `Result`
   for expected failure, with bridges. Terminating (non-resumable) semantics;
   `throw`/`return`/`abort` unify as one unwind primitive.

10. **Traits / mixins / multiple inheritance.** Unspecified. Single inheritance is
    the current invariant ([Object Model](object-model.md)).

11. ~~**`Behavior` in the kernel.**~~ **RESOLVED** → [ADR-0003](../adr/0003-introduce-behavior-kernel-class.md):
   `Behavior` is the shared superclass of `Class`/`Metaclass`.

---

## Resolved (summary)

| Q  | Decision | ADR |
|----|----------|-----|
| Q1 | `let` (immutable) / `var` (mutable); `var x` without initializer = `None` | [ADR-0014](../adr/0014-let-and-var-bindings.md) |
| Q5 / absence | `Option` is abstract; `Some`/`None` subclasses; `None` is a singleton | [ADR-0007](../adr/0007-option-as-abstract-with-some-none.md) |
| Q9 | Layered exceptions + `Result`; terminating, not resumable | [ADR-0008](../adr/0008-layered-exceptions-and-result.md) |
| heap/ownership | Handle/arena heap; no `Rc`/`RefCell`; `ObjRef`/`ClassId` are `Copy` integers | [ADR-0009](../adr/0009-handle-arena-heap.md) |
| Value repr | Tagged `enum` with private `Nil` sentinel; `Number(f64)`, `Bool(bool)`, `Obj(ObjRef)`, `Symbol(…)` | [ADR-0010](../adr/0010-tagged-value-enum.md) |
| instance `toString` | Default renders `"<ClassName>"`; class `toString` returns its own name | [ADR-0015](../adr/0015-object-default-tostring.md) |
| Q11 | `Behavior` is the shared superclass of `Class`/`Metaclass` | [ADR-0003](../adr/0003-introduce-behavior-kernel-class.md) |
