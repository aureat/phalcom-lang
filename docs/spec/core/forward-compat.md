# Forward-Compatibility Checklist — "Must Not Preclude" (U-CORE-0)

> **Status:** Normative constraint set. The core library is being built *before*
> four still-open or out-of-scope subsystems — **concurrency** (`Fiber`/`Future`),
> the full **Error** mechanism, **modules/imports**, and a possible
> **integer/float** split. This document fixes, per subsystem, the design moves a
> U-CORE unit **must not make** because they would force a breaking change when
> that subsystem lands. Every U-CORE implementation spec must pass this checklist
> under **"what must this not preclude."**

> **Baseline:** HEAD `0f84232`; last code-affecting commit `0da64d6`. (Repinned
> 2026-07-12 to fold in U10/U-LEX/U-STD/U11 — none added a floor primitive or
> changed a "must not preclude" hazard below.) Sources:
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

**(a) Coming (decided by [ADR-0027](../../adr/0027-modules-as-files-with-public-by-default-imports.md)).**
The `import` token's semantics are now fixed: **file-as-module**, **public by
default**, with **qualified / selective / aliased** import forms. `Module` is an
ordinary heap class (`new()` floor primitive only). Today all kernel names are
bound as **globals** in one core module (`install_core` + `core.ph`'s per-class
`DefineGlobal`); the constraints below still hold — the core library must not
foreclose the ADR-0027 scoping model.

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

## 4. Integer/Float split ([ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md), [ADR-0005](../../adr/0005-number-as-flat-f64.md))

**(a) Coming (decided by [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md)).** ADR-0005
fixed the *current* substrate as a single flat `f64`. The **surface** split is now
**decided**: abstract `Number` → an exact **unbounded bignum `Int`** and a
**`Float`**, with `/` as true division and `~/` as floor division. object-model
§4's tower rules already accommodate it. The constraints below still bind — the
core library must not foreclose this split, and (see the forward flag in (c)) the
already-shipped `f64` `hash` must be revisited when the bignum `Int` lands.

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
- **Forward flag (revisit when ADR-0024 is implemented):** U-CORE-1's shipped
  `number_hash` masks the `f64` to **53 bits** of mantissa. This is a latent
  `hash`/`==` **soundness gap** once ADR-0024's exact bignum `Int` lands — two
  distinct large ints `> 2^53` can collide, and a bignum `Int` and a `Float` of
  the same mathematical value must still agree on `hash`. `number_hash` must be
  reworked to hash by exact mathematical value (not the 53-bit `f64` digest) when
  the `Int`/`Float` split is implemented.

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
| `import` semantics decided (file-as-module, public-by-default, qualified/selective/aliased) | [ADR-0027](../../adr/0027-modules-as-files-with-public-by-default-imports.md); [`open-questions.md`](../open-questions.md) §8 |
| Current substrate flat `f64`; surface Int/Float split **decided** (bignum `Int` + `Float`, `/` true, `~/` floor) | [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md); [ADR-0005](../../adr/0005-number-as-flat-f64.md); [`open-questions.md`](../open-questions.md) §2 |
| `Value` is an extensible tagged enum | [ADR-0010](../../adr/0010-tagged-value-enum.md) |

---

## 7. Concurrency — code-grounded foreclosure audit (Fiber/Future deep-dive)

> **Extends §1.** §1 was written against [`concurrency.md`](../concurrency.md);
> this section re-checks each *already-landed* VM decision against the **actual
> tree at the U11 baseline** (U1 heap, U4 blocks/closures, U8 dNU/`perform`, U9
> variadics, U10 non-local return) and cites file+line. It **confirms** §1's
> verdict for six decisions, **corrects** §1(a) on one point (no new `Value`
> arm), and **surfaces one hazard §1 did not name**: the VM dispatch loop is
> *re-entrant across native frames*, which is the single largest complication for
> `Fiber.yield`. Owner: forward-compat pass for concurrency (2026-07-12).
> Design-space grounding: `.claude/skills/language-design/references/vm.md`
> ("Frames on native C stack vs heap/VM stack"; hazard *native-stack frames ⊗
> suspendable control*) and `…/references/concurrency.md` (CROWN-JEWEL hazards
> *stackful-fiber ⊗ moving/native-stack GC* and *non-local-return ⊗ fiber
> boundary*).

### 7.1 Per-decision verdicts (verified against code)

| # | Landed decision | Where (file:line / ADR) | Forecloses Fiber/Future? | Constraint to preserve |
|---|---|---|---|---|
| D1 | **Handle/arena heap** — objects live in a central `Heap`, referenced by `Copy` `ObjRef`; designed so a tracing GC can relocate behind the handle. | ADR-0009; `heap.rs` `enum Object` L67. | **No.** A `FiberObject` becomes one more `Object::Fiber(...)` arena variant, reached by `ObjRef`. Owning `Vec<Value>` + `Vec<CallFrame>` inside it is fine. | When a real GC lands, a *suspended* fiber's `stack`/`frames` are live roots the collector must scan (design-space CROWN JEWEL). ADR-0009 already promises arena scanning; keep the fiber's stacks **inside the arena object**, not in native memory, so the future collector reaches them the same way. |
| D2 | **Tagged `Value` enum** — `Nil / Bool(bool) / Number(f64) / Symbol / Obj(ObjRef)`; *every* heap object (incl. native `List`) is `Value::Obj`. | ADR-0010; `value.rs` `enum Value` L31–44. | **No — and §1(a) is imprecise.** §1(a)/§6 say `Fiber` is a *new `Value` arm* (`Value::Fiber(PhRef<FiberObject>)`), echoing `concurrency.md` §1. That phrasing **predates the handle heap.** The landed model gives no heap type its own `Value` arm — `List` proves the pattern. `Fiber` should be `Object::Fiber` + `Value::Obj(ObjRef)`, **no new `Value` arm**. This *removes* §1(b)'s "closed `Value` enum" hazard entirely. | Do **not** add a `Value::Fiber` arm; follow the `List` precedent. `PhRef<FiberObject>` in `concurrency.md` is stale — the handle is `ObjRef`. |
| D3 | **Single VM value stack + frame stack** — `VM { frames: Vec<CallFrame>, stack: Vec<Value> }`. | `vm.rs` L52/L54. | **No (mechanical).** §1's stated relocation: hoist "current stack/frames" behind `current: ObjRef` into the running `FiberObject`; `call`/`yield` swap which fiber the loop reads. `CallFrame.stack_offset` is a **relative** window index (`frame.rs` L75), so per-fiber stacks starting at 0 keep every offset valid — no rebasing. | The relocation must be a pointer/handle swap, **not** a copy (`concurrency.md` §1 pt 2: O(1) switch). Keep `stack_offset` frame-relative. |
| D4 | **Frame-token non-local return** — `FrameToken { frame_index, generation }`; `generation` from a **VM-global** monotonic `next_frame_generation`; `ReturnNonLocal` unwinds eagerly, matching the live home frame by `(index, generation)` else `DeadFrameError`. | ADR-0013; `frame.rs` L19–24; `vm.rs` `next_frame_generation` L72/583, `ReturnNonLocal` L1075–1125. | **No — fiber-locality falls out for free.** Once `self.frames` is the *current fiber's* vector, `ReturnNonLocal`'s `self.frames.get(token.frame_index)` searches only that fiber; a token whose home is on another fiber fails the lookup ⇒ `DeadFrameError` — exactly `concurrency.md` §3 ("`return` across a fiber boundary raises `DeadFrameError`"). Design-space CROWN JEWEL *non-local-return ⊗ fiber boundary* is thus **already handled by construction.** | **Load-bearing invariant: `next_frame_generation` MUST stay VM-global, never relocated into `FiberObject`.** A per-fiber generation counter would let fiber B's `frame_index=k, generation=g` collide with a live frame in fiber A at the same `(k,g)`, silently returning into the **wrong fiber's** frame. The global monotonic counter is the only thing making the cross-fiber token globally non-matching. |
| D5 | **`Invoke`/`call_method` shape** — a Phalcom→Phalcom send *pushes a `CallFrame`* and returns to the same loop (no native recursion); a **primitive** runs `native_fn` in Rust and detects post-call frame-count shrink to reconcile a non-local return that fired inside it. | `vm.rs` `call_method` L390–453 (Closure arm L431–451 = frame push; Primitive arm L393–430 = `frames_before` heuristic). | **Partly (see 7.2).** The Closure arm is trampolined and fiber-friendly. The Primitive arm's `frames_before`/`frames.len()` heuristic assumes *"frame count changed ⇒ a non-local return unwound."* A **fiber switch also changes `frames.len()`** (different fiber, different depth). These two "frame count moved" causes must be disambiguated, or a fiber swap will be misread as a non-local return. | The fiber-switch signal must be **distinct** from the non-local-return signal at this call site — e.g. a typed `ControlFlow`/switch enum out of the primitive, not an implicit length delta. |
| D6 | **Metaclass tower** (parallel rule, `Behavior` kernel, `verify_invariants`). | ADR-0002/0003; `object-model.md` §5–6. | **No.** `Fiber`/`Future` are ordinary heap classes; the class-side selectors `yield(_)`/`current`/`abort(_)`/`async(_)` are metaclass methods like any other. No apex/tower change. | New kernel classes must pass `verify_invariants()` (parallel-rule) at bootstrap, same as every other class. |
| D7 | **Error/unwind is one primitive** — `return`/`throw`/fiber `abort` share one stack unwind; `ensure` fires on any. | ADR-0008; overlay §Unwind. Today only `return`'s half exists (U10 `ReturnNonLocal`); `throw`/`ensure` are unbuilt (U-CORE-6). | **No, but sequences after Errors.** Fiber *failure* (`failed` status, `try`, `error`, `abort`, `Future` reject) is a *payload of the same unwind*: an error unwinds the failed fiber's **own** frames to empty, stores the `Error` in its result slot, and resumes the resumer (`concurrency.md` §1 pt 4). The eager, fiber-local unwind style of `ReturnNonLocal` (L1078: "unwind happens eagerly, here, in one shot") is the correct template. | The U-CORE-6 unwind must operate on **`self.frames` = the current fiber only**, and must be able to **stop at the fiber's floor** (not the process floor) so failure is *captured*, never host-terminating (§2 hazard: "do not special-case `throw` as host-process termination"). |

### 7.2 The hazard §1 did not name — the re-entrant dispatch loop

**Finding.** The VM loop is **not a flat trampoline.** Pure Phalcom→Phalcom
sends *are* trampolined — `call_method`'s Closure arm pushes a `CallFrame`
(`vm.rs` L450) and the single `run_until` loop (L739) picks it up with no native
recursion. **But every path where a Rust primitive needs a synchronous `Value`
back from Phalcom code re-enters `run_until` recursively, growing the native
Rust stack:**

- `block_call` → `vm.run_until(base_frames)` (`primitive/block.rs` L114);
- `send_dynamic`/`perform` → `run_until(base_frames)` (`vm.rs` L565);
- `forward_does_not_understand` (same re-entrancy pattern, `vm.rs` L499+);
- **transitively, any collection/combinator primitive that calls a block** —
  `List.each`, `Option.map`, `reduce`, etc. — because they bottom out in
  `block_call`.

So when the running fiber is *inside such a primitive*, one or more **native
Rust frames sit between the fiber's entry and the `Fiber.yield` call site.**
`concurrency.md` §1's model — "repoint `current`, then **return to the dispatch
loop**, which resumes at the new fiber's saved `ip`" — reaches only the
**innermost** `run_until`, not the top-level one. You cannot repoint a handle and
keep going, because the suspended fiber's position *is* those native frames, and
returning out of them destroys it. This is precisely the design-space CROWN
JEWEL **native-stack frames ⊗ suspendable control** (`vm.md`): a native-stack
activation forecloses suspension "without a CPS/state-machine rewrite."

**Precise scope (the good news).** The foreclosure is *not* total. It bites
**only** across a re-entrant primitive boundary. A fiber whose body only does
pure Phalcom sends and **inlined** control flow can suspend freely, because those
are trampolined (the U5 sacred-selector inliner lowers `while`/`ifTrue:` to
`Jump`/`Loop` opcodes *within one chunk* — no frame push, no native frame). The
canonical `concurrency.md` §1 generator —

```phalcom
Fiber.new { let n = 0; while (true) { Fiber.yield(n); n = n + 1 } }
```

— uses an **inlined `while`**, so it suspends across only the *one* native frame
of the resuming `call` primitive itself. The form that is foreclosed under the
cheap model is the **callback generator**: `Fiber.new { list.each { Fiber.yield(x) } }`,
where `yield` sits under `each`'s native `block_call`.

**This is the load-bearing open decision** and belongs in
[`open-questions.md`](../open-questions.md) (it is **not** currently listed there;
overlay §OPEN lists only structured-concurrency / `select`-`race` / scheduler
fairness). The three positions, and which pieces each foreclosure gate:

| Option | What it is | Cost | Forecloses later? |
|---|---|---|---|
| **A — restricted (Lua 5.1 style)** | Fiber switch integrates with the **top-level** `run_until` only; `Fiber.yield` while a re-entrant primitive is on the native stack raises `CannotYieldAcrossNativeFrame`. | Small VM change (a switch signal + top-loop honoring it). No collection rewrite. | **No.** Lifting the restriction later (Option B) is purely additive. Keeps ADR-0009's GC-ready arena claim intact (no native stacks to scan). |
| **B — full trampoline** | De-recurse *every* callback primitive (`block_call`, `each`, `map`, `reduce`, `perform`, dNU forward) so they push work onto the VM frame stack instead of calling `run_until`. Yield works anywhere. | Large, invasive rewrite of the primitive/callback protocol. | No — strictly more capable than A. |
| **C — stackful** | Give each fiber a real native stack (`corosensei`/`makecontext`-style switch). Yield crosses native frames. | Adds an `unsafe` stack-switch dependency; **directly collides** with the CROWN-JEWEL *stackful-fiber ⊗ moving/native-stack GC* hazard — every suspended fiber's native stack becomes a root the future collector must scan/relocate, weakening ADR-0009. | Constrains the GC design permanently. |

**Recommendation (audit, non-binding — this is for the user / an ADR to decide):**
ship **Option A first.** It is the smallest correct step, keeps the spec's own
canonical example working (inlined `while`), avoids the GC crown-jewel entirely,
and does not foreclose Option B (the restriction is a *guard* later removed).
This is captured as **BLOCKED-ON-DECISION** in the U14 plan and must be resolved
**before** the fiber execution-model unit (U14.2) is scheduled.

### 7.3 Invariants a pre-Fiber unit MUST NOT break

1. **Keep `next_frame_generation` VM-global** (D4). Any refactor that moves it
   into per-frame or per-context state breaks cross-fiber `DeadFrameError`.
2. **No new `Value` arm for heap types** (D2). Reach `FiberObject` through
   `Value::Obj(ObjRef)` + an `Object::Fiber` variant, matching `List`.
3. **Do not conflate "frame count changed" with "non-local return"** at the
   primitive call site (D5). Leave room for a *third* cause (fiber switch) by
   keeping the reconciliation an explicit signal, not a length heuristic that a
   fiber switch would trip.
4. **Keep the eager, fiber-local unwind shape** (D4/D7). The U-CORE-6 error
   unwind must be expressible as "unwind `self.frames` to a floor," where the
   floor can be a *fiber* floor, not only the process floor.
5. **Keep `stack_offset` frame-relative** (D3), so a per-fiber stack needs no
   rebasing.

### 7.4 Corrections to §1 (record)

- §1(a)/§6: "`Fiber` = **new `Value` arm**" → **superseded**: `Fiber` is an
  `Object::Fiber` heap variant reached via `Value::Obj`; **no new `Value` arm**
  (D2). `concurrency.md` §1's `Value::Fiber(PhRef<FiberObject>)` predates
  ADR-0009/0010 and should be read as "a `FiberObject` on the heap."
- §1(b) hazard "closed `Value` enum" is **moot** under the handle heap — heap
  types don't take `Value` arms.
- §1 omitted the **re-entrant `run_until`** hazard (7.2), which is the actual
  dominant complication. §1's "single global VM stack" worry is the *easy*
  (mechanical) part; the *hard* part is native-frame re-entrancy.
