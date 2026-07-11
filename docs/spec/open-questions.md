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

   > **Re-opening concern (deferred).** [Selectors, Symbols & References §7
   > item 1](selectors.md#7-open-questions-not-decided) re-raises this: if
   > uninitialized `var x` is `None`, every variable is effectively `T | None`
   > and `nil` returns under a new name; the alternative floated there is a
   > VM-only `Uninit` sentinel that traps on read, keeping `None` a *chosen*
   > absence. This is **not** adopted — the resolution above stands — but is
   > recorded here as a live concern for a future revisit.

   > **Related re-opening concern (deferred), `ifTrue`/`ifFalse` → `Option`.**
   > [Values & Absence §3](values-and-absence.md#3-absence-is-option) resolves
   > `ifTrue`/`ifFalse` to return `Option`. [Selectors §7 item
   > 2](selectors.md#7-open-questions-not-decided) flags that this makes
   > chaining unsound (`cond.ifTrue { a }.ifFalse { b }` sends `ifFalse` to an
   > `Option`, not a `Bool`; `ifTrue { None }` is indistinguishable from the
   > branch not being taken), and floats a paired `ifTrue(_)ifFalse(_)`-style
   > selector as primary with single-branch forms as `Option`-returning sugar.
   > Not adopted here — the `Option`-returning resolution stands — but flagged
   > for a future revisit alongside the inliner's sacred-selector list
   > ([Control Flow §3](control-flow.md)).

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

12. **Default arguments.** Raised in [Selectors §7 item
    3](selectors.md#7-open-questions-not-decided). Largely incompatible with
    selector-identity dispatch: a call that omits a defaulted argument
    produces a *different* selector, so lookup misses on the full-arity
    method. Candidate resolutions are arity-family expansion (combinatorial —
    one method per omitted-argument combination) or static callee knowledge
    (unavailable under dynamic dispatch). Flagged there as **decide before
    shipping** — retrofitting after selector identity is load-bearing
    elsewhere would be expensive.

13. **`Option` bootstrap.** Raised in [Selectors §7 item
    4](selectors.md#7-open-questions-not-decided). If `Option` is a plain
    stdlib class and fields default to `None` ([Values & Absence
    §3](values-and-absence.md#3-absence-is-option)), constructing `None`
    requires a class whose own fields default to `None` — a bootstrap cycle.
    `Option` likely needs to be VM-blessed / niche-encoded directly in
    `Value` ([ADR-0010](../adr/0010-tagged-value-enum.md)), which also removes
    an allocation from every optional. Not yet decided how `Option`'s
    construction is special-cased relative to ordinary classes.

14. **`Family` introspection.** Raised in [Selectors §7 item
    5](selectors.md#7-open-questions-not-decided). Whether `Family` (the
    value produced by `::`, [Selectors §3](selectors.md#3-method-references-))
    exposes arity, candidate lists, etc. as a first-class reflective object,
    beyond its current role of enriching `doesNotUnderstand` error messages.

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
