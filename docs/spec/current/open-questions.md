# Open Questions

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

Design points that gate dependent implementation. Each must be resolved here
before the work it blocks begins.

> **RESOLVED questions** are struck through and annotated with the deciding ADR
> (or, for low-ceremony rulings, a one-line resolution). **Open questions** remain
> for future design sessions.

> **Status (2026-07-12):** all fifteen questions are now resolved. Q2/Q3/Q4/Q8
> were ratified this session as [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md)–[ADR-0027](../../adr/0027-modules-as-files-with-public-by-default-imports.md);
> Q15 (concurrency execution model) was added from the forward-compat audit and
> ratified as [ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md);
> Q6/Q7/Q10/Q12/Q13/Q14 were resolved as doc-only rulings (recorded inline). Several
> carry deliberately **non-foreclosed** deferrals (bignum `Int` migration is not one —
> it is the chosen default; but stateful mixins, `Some` niche-encoding, list/rest
> destructuring, `Family` reflection, and default arguments are all reserved for
> later without being precluded).
>
> This file is the **decision record**; the postponed work those deferrals point to —
> plus genuinely-still-open decisions and unbuilt units — is indexed in
> [Deferred & Future Work](deferred-work.md).

---

1. ~~**`let` vs `var`.**~~ **RESOLVED** → [ADR-0014](../../adr/0014-let-and-var-bindings.md):
   `let` introduces an immutable binding; `var` introduces a mutable one.
   `var x` with no initializer reads as `None` (consistent with an unassigned
   field, see Q5 / absence → [ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md)).
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

2. ~~**`Number`.** One numeric type, or `Int` / `Float` split?~~ **RESOLVED** →
   [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md):
   split `Number` (abstract) into **exact, unbounded `Int`** (auto-promoting
   bignum — tagged `i64` immediate that boxes to a heap `LargeInt` on overflow,
   never wraps or traps) and **`Float`** (`f64`, retained from
   [ADR-0005](../../adr/0005-number-as-flat-f64.md)). `1` is an `Int`, `1.0` a
   `Float`; `1 == 1.0` and `2.hash == 2.0.hash` (value-based, per ADR-0023).
   **`/` is true division** (`Int / Int → Float`); **`~/` is integer division**
   (floor semantics, sign agrees with `%`; spelled `~/` because `//` is the
   line-comment token). ADR-0024 supersedes ADR-0005 in part.

3. ~~**External vs internal parameter names.** Swift allows `move(to target:)`.~~
   **RESOLVED** → [ADR-0025](../../adr/0025-external-internal-parameter-names.md):
   **yes** — a labeled parameter may declare a separate internal binding, spelled
   `move(to target:)` (label `to`, binding `target`); the single-word form
   `width:` is sugar for `width width:`. Selector identity is unchanged (the label,
   not the binding, is encoded — [ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md)),
   so this is a parser + frame-binding change with zero dispatch impact.

4. ~~**Class hierarchy mutability.** Is `Test.superclass = Test` legal at runtime?~~
   **RESOLVED** → [ADR-0026](../../adr/0026-class-hierarchy-mutability.md): split the
   two axes. **Methods are open** (add/replace after definition, via the ADR-0018
   override-epoch guard); **superclass reparenting is sealed** at definition (it
   would shift the ADR-0011/0017 fixed slot offsets). Reparenting stays sealed *by
   policy*, not impossibility — a future opt-in `reshape`-with-migration primitive
   is left explicitly non-foreclosed ([ADR-0009](../../adr/0009-handle-arena-heap.md)
   keeps it implementable).

5. ~~**String interpolation syntax.**~~ **RESOLVED** →
   [ADR-0022](../../adr/0022-string-interpolation-backslash-paren-sigil.md): the
   `\(expr)` sigil (Swift-style), landed with U-LEX.

6. ~~**Set literal.**~~ **RESOLVED** (ruling, no ADR — additive sugar): construct
   sets via the **`Set(...)` constructor** (`Set(*xs)`, using U9 variadics); no
   set *literal* ships now. A dedicated literal sigil is **reserved** for the
   future collections unit (candidate `#{ }`, Clojure-precedented, **not**
   committed — `#` is left free). Sets themselves are gated on `Object#hash`
   (U-CORE-1) regardless. The collections umbrella
   [ADR-0032](../../adr/0032-collections-representation-and-literals.md) confirms
   this and formally reserves `#{…}` (inactive, committed meaning) alongside the
   ratified `Map` `{k:v}` and `Tuple` `(a,b)` literals.

7. ~~**Destructuring.**~~ **RESOLVED** → [ADR-0046](../../adr/0046-destructuring-bindings.md)
   (amends this ruling's scope, U14): ship **irrefutable tuple AND list
   destructuring** in `let`/`var` now — `let (a, b) = point`,
   `let (q, r) = divmod(17, 5)`, `let [first, *rest] = list` — desugaring to a
   single evaluation of the initializer followed by positional reads through the
   same `at(_)` selector `List`/`Tuple` already expose (ADR-0020), with an inline
   arity guard that raises a clean `Error` on a shape mismatch (a `List`'s
   `at(_)` is already total — ADR-0020 — so the irrefutable list form costs no
   more to build correctly than the tuple form). U9's `*rest` spelling is reused
   verbatim, and it must be the pattern's last element. Both forms stay
   irrefutable — there is no `match`/`if let` failure branch yet. Fuller pattern
   matching (map patterns, match arms, a genuinely refutable evaluator over the
   same `Pattern` AST node) remains deferred to a future unit.

8. ~~**Modules / imports.**~~ **RESOLVED** →
   [ADR-0027](../../adr/0027-modules-as-files-with-public-by-default-imports.md)
   (design) + [ADR-0045](../../adr/0045-module-import-relative-path-whole-module-binding.md)
   (Draft 0.1 implementation, DEC-U15 = A + A, U15): **file = module**; a
   `Module` is a first-class namespace object whose members are reached by
   **ordinary sends** ([modules.md](modules.md)). ADR-0027's original grammar
   (qualified/selective/aliased forms over logical-name resolution) is
   narrowed for Draft 0.1: **relative file-path resolution**
   (`import "./geometry/point"`, `.ph` appended, canonicalized) +
   **whole-module binding only** (`import "path" as Name`; no bare/selective
   form yet). Every top-level name is a member — no `export`, no `_`-prefix
   privacy enforcement yet (ADR-0027 §2 not enforced). A unit is compiled and
   run exactly once, memoized by canonical path; a mutual import cycle
   terminates (a name read across its not-yet-complete edge is an ordinary
   `doesNotUnderstand` miss, documented). Parameterized/first-class modules
   (beyond the basic `Module` object), logical-name resolution, selective
   import, `export`, and compiled-bytecode imports (no verifier exists) are
   all deferred (non-foreclosed).

9. ~~**Error handling.** `throw` / `try` / `catch`, or `Result` as a sibling of
   `Option`?~~ **RESOLVED** → [ADR-0008](../../adr/0008-layered-exceptions-and-result.md)
   (see also [Error Handling](error-handling.md)):
   both, layered — unwinding `throw`/`Error` for the exceptional path, `Result`
   for expected failure, with bridges. Terminating (non-resumable) semantics;
   `throw`/`return`/`abort` unify as one unwind primitive. The surface **syntax**
   (`throw`/`try`/`catch`/`on`/`ensure`, 1:1 sugar over the block protocol) is
   ratified by [ADR-0031](../../adr/0031-error-handling-surface-syntax.md).

10. ~~**Traits / mixins / multiple inheritance.**~~ **RESOLVED** (ruling, no ADR):
    **single inheritance only** for now — no traits, mixins, or multiple
    inheritance. Revisit after the core is decided and implemented. State-bearing
    mixins/MI stay foreclosed on purpose (they break the ADR-0011/0017 fixed slot
    offsets); **stateless traits** — method-only bundles flattened into the
    existing method table with explicit conflict resolution — are the cheap future
    option if behavior-reuse pain appears, since they touch neither slot layout nor
    the single-super lookup chain.

11. ~~**`Behavior` in the kernel.**~~ **RESOLVED** → [ADR-0003](../../adr/0003-introduce-behavior-kernel-class.md):
   `Behavior` is the shared superclass of `Class`/`Metaclass`.

12. ~~**Default arguments.**~~ **RESOLVED** (ruling, no ADR): **no default
    arguments now.** The urgency in the "decide before shipping" flag was really
    about *mechanism*: the expensive-to-retrofit approach is **call-site
    resolution** (needs static callee knowledge, unavailable under dynamic
    dispatch) — that is **permanently forbidden**. If defaults are ever added they
    must **desugar to real arity-family overloads at definition time** (each
    installed selector a real forwarding method — pure codegen over the
    arity-overloading that already works, no dispatch change) and be restricted to
    **trailing parameters** (keeps the expansion linear, `n` defaults → `n+1`
    selectors, not combinatorial). With that mechanism fixed, adding defaults later
    is non-breaking sugar.

13. ~~**`Option` bootstrap.**~~ **RESOLVED** (ruling, no ADR — confirms the
    as-built): the feared cycle is already broken by the U6 implementation and that
    design stands. `Value::Nil` is a **private** uninitialized-slot sentinel
    ([ADR-0010](../../adr/0010-tagged-value-enum.md), no surface syntax); **`None` is a
    single pre-built heap singleton**; the `Nil` sentinel is **surfaced to `None`**
    one-directionally at read boundaries (`Nil → None`, never the reverse), so an
    uninitialized `var` reads as `None` without any construction regress. `Some(x)`
    remains an **ordinary heap instance** (`_value` field) — the object model stays
    uniform. **Deferred optimization (non-foreclosed):** niche-encoding `Some` into
    `Value` for allocation-free optionals, slotted behind the existing
    `surface_none` boundary once there is a GC + benchmarks to justify it.

14. ~~**`Family` introspection.**~~ **RESOLVED** (ruling, no ADR): ship `Family`
    ([Selectors §3](selectors.md#3-method-references-)) as a **callable value only**
    for now — its candidate list stays a VM-internal detail feeding
    `doesNotUnderstand` messages. Exposing `Family` as a **first-class reflective
    mirror** (`.candidates`, `.arities`, `.name`, `.receiver`, `.respondsTo(_)`,
    `.isPinned`) is **deferred** to a later reflection unit, designed together with
    the U8 `Message`/`perform`/`respondsTo` surface as one coherent API. Reflection
    is easy to add and painful to walk back, so the commitment waits; the candidate
    list already exists internally, so this only decides *when* to expose it.

15. ~~**Concurrency execution model.** Restricted re-entrant loop, full
    trampoline, or stackful coroutines?~~ **RESOLVED** →
    [ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md): the
    **restricted re-entrant loop** (Lua-5.1 style). `Fiber.yield` integrates with
    the top-level dispatch loop only; yielding across a native callback frame
    raises `CannotYieldAcrossNativeFrame`. It is the smallest correct step on the
    current VM, keeps the moving-GC design open (no native fiber stacks), and lifts
    *additively* to a full trampoline later. Surfaced from the
    [forward-compat §7.2](core/forward-compat.md) audit, which is where the
    hazard (`native-stack frames ⊗ suspendable control`) was first named.

---

## Resolved (summary)

| Q  | Decision | ADR / ruling |
|----|----------|-----|
| Q1 | `let` (immutable) / `var` (mutable); `var x` without initializer = `None` | [ADR-0014](../../adr/0014-let-and-var-bindings.md) |
| Q2 | Split `Number` → exact bignum `Int` + `Float`; `/` true division, `~/` floor integer division | [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md) |
| Q3 | Separate external label from internal binding (`move(to target:)`); selector identity unchanged | [ADR-0025](../../adr/0025-external-internal-parameter-names.md) |
| Q4 | Methods open (epoch guard); superclass reparenting sealed; future `reshape` non-foreclosed | [ADR-0026](../../adr/0026-class-hierarchy-mutability.md) |
| Q5 (interp.) | String interpolation uses `\(expr)` | [ADR-0022](../../adr/0022-string-interpolation-backslash-paren-sigil.md) |
| Q5 / absence | `Option` is abstract; `Some`/`None` subclasses; `None` is a singleton | [ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md) |
| Q6 | `Set(...)` constructor; `#{…}` set literal reserved-inactive (`Map`/`Tuple` literals ship) | ruling (Q6 above) + [ADR-0032](../../adr/0032-collections-representation-and-literals.md) |
| Q7 | Irrefutable tuple AND list/`*rest` destructuring now, via `at(_)`; fuller pattern matching (map patterns, match arms) deferred | [ADR-0046](../../adr/0046-destructuring-bindings.md) |
| Q8 | File = module; Draft 0.1: relative file-path resolution + whole-module binding only (`import "./x" as Name`), members via ordinary sends | [ADR-0027](../../adr/0027-modules-as-files-with-public-by-default-imports.md) + [ADR-0045](../../adr/0045-module-import-relative-path-whole-module-binding.md) |
| Q9 | Layered exceptions + `Result`; terminating, not resumable; surface `throw`/`try`/`catch`/`on`/`ensure` | [ADR-0008](../../adr/0008-layered-exceptions-and-result.md) + [ADR-0031](../../adr/0031-error-handling-surface-syntax.md) |
| Q10 | Single inheritance only; traits/mixins deferred (stateless traits the future option) | ruling (Q10 above) |
| Q11 | `Behavior` is the shared superclass of `Class`/`Metaclass` | [ADR-0003](../../adr/0003-introduce-behavior-kernel-class.md) |
| Q12 | No default arguments now; if added → definition-time overload desugar, trailing-only | ruling (Q12 above) |
| Q13 | Keep as-built `None` singleton + private `Nil` sentinel; `Some` heap instance | ruling (Q13 above) |
| Q14 | `Family` callable-only now; reflective mirror deferred to a unified reflection unit | ruling (Q14 above) |
| Q15 | Concurrency execution model: restricted re-entrant loop (Lua-5.1 style); yield across a native frame raises `CannotYieldAcrossNativeFrame` | [ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md) |
| heap/ownership | Handle/arena heap; no `Rc`/`RefCell`; `ObjRef`/`ClassId` are `Copy` integers | [ADR-0009](../../adr/0009-handle-arena-heap.md) |
| Value repr | Tagged `enum` with private `Nil` sentinel; `Int(i64)`/`Float(f64)`, `Bool(bool)`, `Obj(ObjRef)`, `Symbol(…)` | [ADR-0010](../../adr/0010-tagged-value-enum.md) + [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md) |
| instance `toString` | Default renders `"<ClassName>"`; class `toString` returns its own name | [ADR-0015](../../adr/0015-object-default-tostring.md) |
