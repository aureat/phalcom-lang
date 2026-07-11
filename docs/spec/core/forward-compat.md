# Forward-Compatibility Checklist — "Must Not Preclude" (U-CORE-0)

> **Status:** Normative constraint set. The core library is being built *before*
> four still-open or out-of-scope subsystems — **concurrency** (`Fiber`/`Future`),
> the full **Error** mechanism, **modules/imports**, and a possible
> **integer/float** split. This document fixes, per subsystem, the design moves a
> U-CORE unit **must not make** because they would force a breaking change when
> that subsystem lands. Every U-CORE implementation spec must pass this checklist
> under **"what must this not preclude."**

> **Baseline:** HEAD `76b5f35`; last code-affecting commit `0da64d6`. Sources:
> [`concurrency.md`](../concurrency.md), [ADR-0008](../../adr/0008-layered-exceptions-and-result.md)
> + [`error-handling.md`](../error-handling.md), [`open-questions.md`](../open-questions.md)
> §2 (numbers) / §8 (modules), [ADR-0010](../../adr/0010-tagged-value-enum.md).

## How to use this

Each section is **(a)** what is coming, **(b)** the concrete preclusion hazards,
**(c)** the passing constraint. A unit "passes" a section when nothing in its
diff trips a hazard. Sections are independent — a unit need only address the ones
it touches (e.g. U-CORE-4 touches Numbers §4; U-CORE-6 touches Errors §2).

---

## 1. Concurrency — `Fiber` / `Future` ([concurrency.md](../concurrency.md))

**(a) Coming.** `Fiber` is a cooperative coroutine backed by a `FiberObject`
holding its **own** `stack: Vec<Value>` + `frames: Vec<CallFrame>`, a `status`, a
`resumer`, and an entry closure — introduced as a **new `Value::Fiber(PhRef<…>)`
arm**. `call`/`yield` become a swap of which fiber's stack/frames the VM points
at. `Future` is a *library-level* `InstanceObject` (no new `Value` arm). Errors
cross fiber boundaries only through a fiber's result slot.

**(b) Preclusion hazards.**
- Assuming a **single global** VM stack/frame vector that cannot be relocated
  into a fiber (concurrency.md §1 relocates "current stack/frames" behind a
  `current: PhRef<FiberObject>` pointer).
- Baking a **closed** `Value` enum assumption into a primitive (an exhaustive
  `match` with no room for a `Fiber` arm that would need editing everywhere).
- Making non-local `return` / unwind **globally** frame-indexed rather than
  frame-token identified, so it cannot stay *fiber-local*.
- Giving `Object`/`Module` a layout that a plain-`InstanceObject` `Future`
  cannot reuse.

**(c) Passing constraint.**
- **Callables (U-CORE-3):** the call/`arity`/frame-token protocol must be
  expressible without assuming one global stack. U4/U10 already use frame tokens
  (ADR-0013) — U-CORE-3's *surface* additions must not reintroduce a global-frame
  assumption. `Fiber` will implement the **same `call` protocol** — do not close
  `Function`/`Block` against a third callable subclass.
- **Errors (U-CORE-6):** the raise/unwind path must be a *payload of the unified
  unwind* (ADR-0008 "one unwind primitive"), so a fiber can capture a propagating
  `Error` into its result slot rather than crashing the host. Do not special-case
  `throw` as host-process termination.
- **Value repr:** treat `Value` as **open for extension** (ADR-0010 already
  anticipates arms); a new `Fiber` arm must not require touching a U-CORE
  primitive's `match` (use a helper/`_ =>` default where a primitive doesn't care).

## 2. Error mechanism ([ADR-0008](../../adr/0008-layered-exceptions-and-result.md))

**(a) Coming (partly decided).** Layered: `throw`/`Error` **unwind** for
exceptional failure; `Result`/`Ok`/`Err` **values** for expected failure; bridges
(`{…}.attempt() → Result`, `result.unwrap()` re-throws, `option.okOr(err)`).
Handling is a **`Block` protocol** — `blk.on(ErrorClass){…}`, `blk.ensure{…}` —
with `try`/`catch`/`finally` as sugar. **Terminating, not resumable.** `return`,
`throw`, and fiber `abort` are three payloads of **one** unwind primitive; `ensure`
fires on *any* unwind. New kernel value classes `Result`/`Ok`/`Err` bootstrap
alongside `Option`/`Some`/`None`.

**(b) Preclusion hazards.**
- Building a **second, parallel** error channel that is not `Error`-subclass-based
  (ADR-0008: only `Error` subclasses are throwable; typed `on(_)` handlers require
  it).
- Implementing `ensure`/`finally` as **exception-only** (must fire on non-local
  `return` and `abort` too — the one subtle rule).
- Wiring dNU (U-CORE-6) to raise a value that is **not** an `Error` subclass, or
  to terminate the host instead of unwinding.
- Modeling `Result` with a shape **incompatible** with `Option`'s
  abstract-root + two-subclasses layout (they must mirror, ADR-0008 / ADR-0007) —
  e.g. giving `Ok`/`Err` an ad-hoc representation that the `Some`/`None` machinery
  and the `WrapSome`-style helpers cannot share.

**(c) Passing constraint.**
- **U-CORE-6** reifies `Error`/`MessageNotUnderstood` and wires dNU to `raise` an
  `Error` subclass through the **existing** unwind, not a new mechanism. It should
  reserve — not implement — `Result`/`Ok`/`Err` as an `Option`-shaped sibling.
- **U-CORE-2/3:** the unwind that non-local `return` uses (U10) is the *same one*
  `throw` will use. Do not fork it. Any `Option`↔`Result` bridge (`okOr`, `ok`)
  must be layerable later over `match` **without** re-shaping `Option`.
- **Floor:** if U-CORE-6 needs a native raise primitive, that is an **ADR-0019
  amendment** (decisions.md Q2), not a silent `primitive!`.

## 3. Modules / imports (open-questions.md §8)

**(a) Coming (undecided).** The `import` token exists; semantics are unspecified.
`Module` is an ordinary heap class (`new()` floor primitive only). Today all
kernel names are bound as **globals** in one core module (`install_core` +
`core.ph`'s per-class `DefineGlobal`).

**(b) Preclusion hazards.**
- Hard-coding kernel names as **process-global** in a way a future module/namespace
  system cannot scope or shadow (e.g. a primitive that looks a name up in a single
  flat global table by string, with no module indirection).
- Giving `Module` a layout or `Object` a protocol that assumes **exactly one**
  namespace.
- Making the prelude (decisions.md Q4) an *implicit, unnamed* set that a module
  system cannot later re-express as "the core module, auto-imported."

**(c) Passing constraint.**
- Treat the current global set as **"the core module's exports, auto-imported into
  every compilation unit"** (decisions.md Q4), not as an irreducible global
  namespace. A U-CORE unit that adds a name should add it to the **core module**,
  reachable through `install_core`/`core.ph`, so a later `import` can re-scope it.
- Do not add a primitive that resolves globals by raw string without going through
  the module's binding table.

## 4. Integer/Float split (open-questions.md §2, [ADR-0005](../../adr/0005-number-as-flat-f64.md))

**(a) Coming (undecided).** ADR-0005 fixed the substrate as a single flat `f64`.
The **surface** split (abstract `Number` → immediate `Integer`/`Float`) remains
open; object-model §4's tower rules "already accommodate it."

**(b) Preclusion hazards.**
- Writing `Number` surface protocol (arithmetic, `toString`, comparison,
  `toNumber`, `hash`) against the **concrete `f64`** in a way that assumes a single
  numeric class — e.g. a `toString` that hard-assumes float rendering, or a `hash`
  keyed on `f64` bits that would disagree with a future `Integer`'s hash for the
  same mathematical value.
- Making `Number` a **concrete** class that `Integer`/`Float` could not later slot
  under as an abstract root without breaking dispatch identity.

**(c) Passing constraint.**
- **U-CORE-1 (`hash`) / U-CORE-4 (`toString`, value protocol):** write `Number`
  protocol against the **abstract numeric contract** (PHASE2-INDEX soft-flag for
  U-STD), so an `Integer`/`Float` split is additive. `hash` must satisfy
  `a == b ⇒ a.hash == b.hash` in a way that survives `2` and `2.0` becoming
  distinct classes (hash by mathematical value, not raw `f64` bits, where they
  would diverge).
- Keep `Number` positioned as it is in the tower (a row an abstract split can
  refine); do not add a subclass-hostile assumption.

---

## 5. Quick per-unit applicability

| Unit | Must clear sections |
|---|---|
| U-CORE-1 kernel reflection | §4 (`hash` vs int/float), §1 (Value openness for `isA`), §3 (reflection via module, not flat globals) |
| U-CORE-2 absence + Boolean | §2 (`Option`↔`Result` shape parity) |
| U-CORE-3 callables/Block | §1 (fiber-local frames, shared `call` protocol), §2 (unified unwind) |
| U-CORE-4 value classes | §4 (int/float-safe `toString`/`hash`), §3 (names → core module) |
| U-CORE-5 collection contract | §4 (hash/eq for keys), §1 (no global-stack assumption in iteration) |
| U-CORE-6 errors | §2 (**the** section — reify over the unified unwind), §1 (fiber error capture) |

## 6. Traceability

| Claim | Source |
|---|---|
| `Fiber` = new `Value` arm + own stack/frames; `Future` = plain `InstanceObject` | [`concurrency.md`](../concurrency.md) §1–2 |
| Non-local return is frame-local ⇒ fiber-local | [`concurrency.md`](../concurrency.md) §3; [ADR-0013](../../adr/0013-closure-upvalues-and-frame-token-return.md) |
| One unwind primitive; only `Error` throwable; `ensure` on any unwind | [ADR-0008](../../adr/0008-layered-exceptions-and-result.md) |
| `Result`/`Ok`/`Err` mirror `Option`/`Some`/`None` | [ADR-0008](../../adr/0008-layered-exceptions-and-result.md); [ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md) |
| `import` token exists, semantics open | [`open-questions.md`](../open-questions.md) §8 |
| Single flat `f64`; surface Int/Float split open | [ADR-0005](../../adr/0005-number-as-flat-f64.md); [`open-questions.md`](../open-questions.md) §2 |
| `Value` is an extensible tagged enum | [ADR-0010](../../adr/0010-tagged-value-enum.md) |
