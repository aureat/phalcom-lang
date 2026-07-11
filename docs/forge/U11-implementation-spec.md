# U11 — Implementation Specification (supersedes U11-plan.md on conflict)

_Grounded against actual HEAD as of commit `5166729` (docs-only, on top of the
last code-affecting commit `454f2b8` "U-STD: discharge DEFERRED #25, mark U-STD
landed"). This document exists because **`U11-plan.md` was written on 2026-07-11,
before U5's sacred-selector inliner (ADR-0018), U-CORE-2's Bool `Some`-lift
(`0da64d6`), U-STD, and the entire `docs/spec/core/` U-CORE-N spec taxonomy
landed.** As a result its scope inventory (§3 write-set, §4 design, §5 build
order) assumes it must **author** the boolean control methods from scratch and
touch `boolean.rs`/`primitive/boolean.rs` — but those methods **already exist,
are native primitives on `Bool`, are sacred/inlined, and were hardened by
U-CORE-2 to `Some`-lift.** The genuinely-remaining U11 work is much narrower,
lower-risk, and mostly Rust-side. **Where this document and `U11-plan.md`
disagree, follow this document.** Where this document is silent, `U11-plan.md`
still governs (mission intent, mandatory-rules shape, return-contract shape).

Written for a **medium-effort implementer**. If you hit a fact that contradicts
this doc, STOP and report the conflict rather than guessing — this doc's job is
to have already done the archaeology so you don't have to.

**Two items need a ruling before a clean parallel schedule — both in §0.1.**
Neither re-carves U11's scope (unlike U-STD): U11's mission is **intact and
explicitly reserved** for it by two newer normative artifacts (see §0.2). The
first item (**D1: method placement**) is a design decision this spec *resolves*
toward the low-risk option and marks implementer-safe; the second (**BD-U11-SCHED:
write-set collision with the newly-specced U-CORE-1/4/6**) is a real
orchestrator-facing sequencing constraint that the forge collision matrix
predates and cannot see. Read §0.1 before scheduling.

---

## 0. Corrections to `U11-plan.md`

### 0.1 The two decisions that gate implementation / scheduling

#### D1 — Method placement: **KEEP on abstract `Bool` (inherited)** vs MOVE per-subclass. **RESOLVED → KEEP.**

`U11-plan.md` §4 (D2) reads ADR-0004's "boolean control-flow methods are
**defined per subclass**, dispatched by class" literally, and instructs the
implementer to write, in `core.ph`, two method definitions each for
`not`/`and`/`or`/`ifTrue`/`ifFalse` (`True>>not → false`, `False>>not → true`, …).
**That instruction is stale and, taken literally, actively harmful at HEAD.** All
six sacred boolean selectors already exist as **native primitives on `Bool`**
(`primitive/boolean.rs`: `bool_and`/`bool_or`/`bool_not`/`bool_if_true`/
`bool_if_false`/`bool_if_true_if_false`; installed at `universe.rs:290-308`), are
**★sacred** (the inliner special-cases them, ADR-0018), and `ifTrue`/`ifFalse`
were **already fixed by U-CORE-2** to return a well-formed `Option` via a
`Some`-lift kept in lockstep with the inliner's `Bytecode::WrapSome` opcode
(`0da64d6`; `boolean.rs:115-140`).

**This spec resolves D1 to KEEP** — the sacred primitives stay on `Bool`; `True`
and `False` are added as its **singleton subclasses and simply inherit** the
methods through the ordinary hierarchy walk. This is not merely the low-risk
option; it is the design the **newest normative artifact already documents**:
[`floor-census.md`](../spec/core/floor-census.md) §2.6 is literally titled
*"`Bool` — abstract, `True`/`False` by dispatch (ADR-0004)"* and lists all six
sacred fns on the `Bool` row, and §5 (R-SACRED) fixes the sacred set + the
`bool_sacred_pristine` epoch to the **`Bool`** receiver as *the compiler-coupled
interface that must not move*. Under KEEP, `true.class == True` and
`false.class == False` (ADR-0004's observable contract) hold, and dispatch **is**
by class — `True`/`False` resolve the selector by walking to their shared
abstract parent. The internal `if expect_bool(receiver)` inside each primitive is
an implementation detail of the shared method, and is **moot on the hot path
anyway**: the inliner (ADR-0018) elides the call entirely for literal-block call
sites, so the "eliminate the runtime `if`" motivation ADR-0004 cites buys nothing
observable here.

**Why MOVE is rejected** (record this; do not silently re-open it):
- It would require extending `note_method_installed` (`universe.rs:214-222`) to
  watch `true_class`/`false_class`, because that hook keys the sacred epoch on
  `class_id == bool_class` **only** (`universe.rs:216`; sole call site
  `vm.rs:907`, instance methods only). If `ifTrue` lived on `True` as a `.ph`
  method and a user reopened `True` to override it, the epoch would **not** flip,
  the inliner would keep taking the pristine fast path, and the override would be
  **silently ignored — an unsoundness** (the exact hazard ADR-0018 exists to
  prevent).
- It would move the `Some`-lift into `.ph` (`True>>ifTrue(b) → Some.new(b.call)`),
  forcing a re-proof that the `.ph` path is observationally identical to the
  inliner's `WrapSome` fast path — re-litigating U-CORE-2's entire invariant
  corpus (R-INV-2.1) for zero surface gain.
- It would add new `(class, selector)` bindings on `True`/`False`, **breaking**
  U-CORE-1's floor-census audit R-INV-0.1 ("installed floor = **80** bindings,
  exact set matches the census"), which KEEP leaves at exactly 80.

If — and only if — the user later wants ADR-0004's *literal* per-subclass form as
a cosmetic refinement, it is a **separate, later** unit with the three tasks
above as its explicit scope. It is **out of scope for U11.** (Surfaced as a soft
gate **BD-U11-D1** in §8; recommendation KEEP; do not escalate unless the user
asks for literal per-subclass bodies.)

#### BD-U11-SCHED — U11's write-set collides with three **newly-specced** U-CORE-N units the forge collision matrix cannot see. **NEEDS ORCHESTRATOR SEQUENCING.**

`PHASE2-INDEX.md`'s collision matrix (§3, "`core.ph` — U6 / U-STD / U11
serialized by wave order") was written before the `docs/spec/core/` taxonomy
existed and therefore only knows about the `core.ph` axis. But U11's real
write-set (§3 below) is **mostly Rust**, and three U-CORE-N units authored on
2026-07-12 edit the **same Rust structures**:

| Unit | Shared file / region with U11 | Nature |
|---|---|---|
| **U-CORE-1** (kernel reflection) | `universe.rs::create_core_classes` (re-parents `Method < Function`, reorders the ordinary-rows block, [U-CORE-1 spec §4/§2.1, L377-393]); `CoreClasses` struct literal; **`verify_invariants`** (extends the parallel-rule loop from `Number`-only to **all** ordinary rows, R-INV-0.2); `value.rs::class` (its `.ph` `isA(_)` walks the class chain). | **Hard collision on `universe.rs`** — must serialize. |
| **U-CORE-4** (value `toString`) | `core.ph` `class Bool {}` block (adds `toString` over `ifTrue(_, ifFalse)`, [U-CORE-4 spec §item 9, L323/L336]); `value.rs` (`Value::to_string` Obj arm); `primitive/boolean.rs` (references L152). | **Collision on `core.ph` Bool area + `value.rs`** — additive but same file/region; serialize. |
| **U-CORE-6** (Error root) | `universe.rs::create_core_classes` (adds `Error`/`MessageNotUnderstood` rows via `make_core_class`, [U-CORE-6 spec §item 2, L154]); `CoreClasses`. | **Collision on `create_core_classes`** — serialize. |

This is **not** a scope re-carve (§0.2) — nobody else does U11's job — but the
orchestrator **must not co-schedule U11 in the same parallel wave as U-CORE-1,
U-CORE-4, or U-CORE-6.** Recommended order (§9): land **U11 first** (it is tiny,
adds *zero* new floor bindings, and touches `create_core_classes`/`CoreClasses`
minimally), then run the U-CORE-N wave starting with U-CORE-1 — whose extended
`verify_invariants` loop and floor-census audit will then **validate U11's
`True`/`False` rows for free**. If the orchestrator instead lands U-CORE-1 first,
U11 must add `True`/`False` to U-CORE-1's already-extended `verify_invariants`
loop and re-confirm the census stays at 80 (still zero net bindings). Either
order works; **co-scheduling does not.** Do **not** edit `PHASE2-INDEX.md` to
record this (it has in-flight uncommitted edits from the concurrent U-LEX agent)
— this section is the record.

### 0.2 U11's scope is INTACT and explicitly reserved — this is the opposite of U-STD.

Unlike U-STD (whose entire mission was quietly re-assigned to U-CORE-N), U11's
mission is **confirmed as a separate, untouched unit by two newer normative
artifacts**:
- [U-CORE-2 spec §0.3](../spec/core/U-CORE-2-implementation-spec.md) out-of-scope
  table: *"Abstract `Bool` + `True`/`False` singleton representation → **U11**
  (separate forge unit); [ADR-0004]; see §5.3."* And §5.3: U-CORE-2 is about the
  `Option`-returning **surface** of `ifTrue`/`ifFalse`, U11 about Bool
  **representation** — *"orthogonal … U11 will re-parent the sacred `Bool`
  selectors onto `True`/`False`; the sacred set and epoch must stay the
  compiler-coupled interface … U11 is not boxed in."*
- [U-CORE-1 spec L74-76](../spec/core/U-CORE-1-implementation-spec.md): *"`Bool`
  abstract + `True`/`False` singletons (the old U11) remain a **separate** later
  unit and are untouched here."*

**Conclusion: no scope conflict. U11 is neither superseded nor partially done.**
`True`/`False` do not exist anywhere at HEAD (see §1). Proceed.

### 0.3 The plan's write-set is wrong on both `.rs` boolean files.

- **`boolean.rs` needs NO change.** `U11-plan.md` §3 says to add "singleton value
  handles / class-selection helper" here. But at HEAD `boolean.rs` is just the
  two consts `TRUE`/`FALSE` (`boolean.rs:10-12`); the class **handles** live as
  `ClassId`s on `CoreClasses` (`universe.rs`), and the class **selection** is one
  branch of `Value::class` (`value.rs:90`). Nothing in `boolean.rs` is on U11's
  path. Leave it untouched (mirrors U10-spec §0 point 2 / U9's `signature.rs`
  correction — a plan naming a file that turns out to need no edit).
- **`primitive/boolean.rs` needs NO change under KEEP.** The plan wants to
  "replace the buggy `bool_class_new`" and "implement the correct `True`/`False`
  primitives." Under the resolved KEEP design (D1) there are **no** `True`/`False`
  primitives, and the sacred primitives stay exactly as U-CORE-2 left them. The
  debug `println!`s in `bool_class_new` (`boolean.rs:33/35`) are **pre-existing
  cosmetic noise that U-CORE-2 §0.3 deliberately left DEFERRED** — do **not**
  fold their removal into U11 (it would put U11 into `primitive/boolean.rs`, which
  U-CORE-4 also touches, for zero U11 benefit). If you want them gone, that is a
  standalone DEFERRED cleanup, not this unit.

### 0.4 The `booleans` test label is already ACTIVE — there is nothing to un-ignore, and `pending/` is empty.

`U11-plan.md` §3/§5 says "un-ignore the `booleans` label (drop the `#[ignore]`)"
and "promote cases from `booleans/pending/`." Both are stale:
- `tests/lang.rs::booleans()` (**L25-28**) is **already active** — `check_pass("booleans")`,
  **no `#[ignore]`**, green today with two fixtures (`bool_short_circuit_and`,
  `bool_short_circuit_or`, both real short-circuit cases). Nothing to un-ignore.
- `tests/lang/booleans/pending/` **exists but is empty** — there is nothing to
  promote. U11 simply **adds new `.ph`/`.expected` pairs** into the active
  `booleans/` directory (`support::collect_cases` globs it).

### 0.5 The `object-model.md` §3/§4 doc drift the plan §6 flagged is real and is assigned to U11 (doc-only).

`PHASE2-INDEX.md` §5 (L119) lists *"object-model.md §3/§4 reconciliation —
contradicts ADR-0004 (True/False visibility) | spec edit (doc-only) | U11 | with
U11."* object-model §3/§4 still say "users see one class, `Bool`" / "true/false →
`Bool`, one class," which conflicts with ADR-0004 + `values-and-absence.md` §3.1
(and now `floor-census.md` §2.6) making `true.class == True` **surface-visible**.
**Resolution: follow ADR-0004** (True/False visible). This is a **doc-only** edit,
**outside U11's code write-set**; either land it alongside U11 as a documentation
patch or file it for the `documentation-and-adrs` skill. Do not let it block or
bloat the code change.

---

## 1. Preconditions — already verified, do not re-check

- U1–U10, U-LIST, U-STD, U-CORE-2 are merged and green (HEAD `5166729`; last code
  commit `454f2b8`). `./scripts/verify.sh` is green on `main`. Run it on your
  starting tree before the first edit to confirm your baseline. **Working model:
  in-tree on `main`, no worktree** — every unit since U2 landed this way
  (STATE.md); the plan's "isolated worktree" instruction is stale. Confirm the
  convention with the orchestrator before starting (mirrors U9/U10/U-STD spec §1).
- **`True`/`False` do not exist anywhere at HEAD. Verified:**
  - `CoreClasses` (`universe.rs:512-570`) has `bool_class` (L528-529) but **no**
    `true_class`/`false_class` field.
  - `create_core_classes` (`universe.rs:93`) builds `bool_class` at **L135**
    (`make_core_class(heap, "Bool", object_class, metaclass_class)`) and no
    `True`/`False` row.
  - `value.rs::class` (**L87-105**) maps **both** `true` and `false` to the single
    `bool_class` (**L90**: `Value::Bool(_) => vm.universe.classes.bool_class`).
  - `ClassName` (`primitive/mod.rs:54-72`) has `Bool = "Bool"` but **no**
    `True`/`False` consts. (The `True`/`False` you may spot in that module are
    under **`ObjectName`** L79-81 and are the lowercase literal spellings
    `"true"`/`"false"` for the *values*, not class names — unrelated.)
  - `install_core` (`vm.rs:317-373`) binds a `"Bool"` class global via
    `add_class!(bool_class)` (**L339**) and no `True`/`False` global.
  - No test fixture anywhere references `.class == True` / `class True` (grep: 0
    hits).
- **What U5/ADR-0018 already built (answering the plan's "does Bool exist as a
  stub" question precisely):** `Bool` is a **real, complete concrete class**, not
  a stub — six sacred primitives + `new()`/`new(_)`. The VM-level Bool machinery
  is done: `Bytecode::GuardBool` (`vm.rs:1133-1139`) keys the inliner fast path on
  **`matches!(top, Value::Bool(_))` AND `bool_sacred_pristine`** — i.e. on the
  `Value` **representation**, *never* on class identity — and the
  `bool_sacred_pristine` epoch flag lives on `Universe` (`universe.rs:56`), flipped
  by `note_method_installed` (`universe.rs:214`) when a sacred selector is
  installed **on `bool_class`**. **None of that is `True`/`False` — it is the
  representation + inliner + epoch layer.** The class *split* (`true.class == True`)
  is exactly the untouched work U11 delivers.
- **The class-of-`Value::Bool` hot path is the single load-bearing edit.**
  `Value::class` (`value.rs:87`) is called on every dispatch; its `Value::Bool(_)`
  arm (L90) is a cheap `ClassId` field read today. U11 turns it into a `bool`
  branch selecting `true_class`/`false_class` — still a field read, still no
  allocation. Keep it that cheap (the plan's §7 hot-path risk, confirmed real).
- **The sacred inliner is transparent to the class split (verified, this is why
  KEEP is safe).** `GuardBool` checks the `Value::Bool` arm, so splitting the
  class changes nothing it sees; its deopt path issues a real `ifTrue`/`and`/…
  send, which now resolves through `true_class`/`false_class` → inherits from
  `Bool` → finds the same native primitive. `to_context` already returns
  `CallContext::Immediate` for `Value::Bool(_)` (`value.rs:190`), so an immediate
  `true`/`false` receiver invoking an inherited method is already handled (U5 added
  `Immediate` for exactly the reopened-sacred-Bool case, ADR-0018 deviation #4).
- **`class True {}` / `class False {}` reopens in `core.ph` are SAFE (unlike
  `None`).** The `DefineGlobal`-clobber trap documented at `core.ph:32-40` bites
  only `None`, whose global is bound to the singleton **value**. For `True`/`False`
  the global is the **class object** (bound by `add_class!`), so a `class True {}`
  reopen re-emits `DefineGlobal "True" → the True class` — identical to what
  `add_class!` already bound, a harmless no-op. **Precondition:** `add_class!`
  must run in `install_core` (which runs at `VM::new`, before `core.ph` executes)
  and insert `True`/`False` into `self.classes`, so the reopen **reopens** the
  bootstrapped row instead of forging a shadow. Under KEEP the reopens are
  **empty** and optional (surface-visibility consistency with the other core
  classes); see §3.

---

## 2. The one architectural fact that makes this unit safe (and its inverse)

**`Value::class` is the only place `Value::Bool → class` is decided, and every
consumer of Bool class identity flows through it — but the inliner deliberately
does NOT.** These two facts, together, are what make the class split a low-risk,
localized change:

1. **Class-identity consumers** (`x.class`, method lookup on a `true`/`false`
   receiver, `isA`, reflective `respondsTo`) all call `Value::class`
   (`value.rs:87`) → `lookup_method` (`value.rs:111`) → `lookup_method_in_hierarchy`.
   Change the single L90 arm and **all** of them see `True`/`False` consistently.
2. **The inliner's fast path** (`GuardBool`, `vm.rs:1133`) is keyed on the
   `Value::Bool` **representation** + the `bool_sacred_pristine` epoch, **not** on
   `Value::class`. So the split is invisible to it: pristine fast path unchanged;
   deopt path resolves through the new subclasses to the same inherited primitive.

**The inverse — the trap MOVE would spring (do not do it):** the epoch hook
`note_method_installed` (`universe.rs:214-222`, sole call site `vm.rs:907`) is
hard-keyed to `class_id == self.classes.bool_class`. If U11 put sacred methods
on `True`/`False`, a user override of one on `True` would flip *nothing*, and the
inliner would keep silently taking the pristine path — an unsound
"override-ignored" bug. KEEP never lands methods on `True`/`False`, so the hook's
`bool_class` key stays correct and untouched. **This is the whole reason D1
resolves to KEEP.**

---

## 3. Confirmed write-set (re-grep line numbers before editing — earlier edits in
this list shift later ones)

| File | Exact change |
|---|---|
| `phalcom-core/src/universe.rs` | **(a)** `create_core_classes` (`L93`): immediately after `bool_class` (`L135`), add `let true_class = make_core_class(heap, "True", bool_class, metaclass_class);` and `let false_class = make_core_class(heap, "False", bool_class, metaclass_class);` — `Bool` is now the superclass of both, making it the abstract parent. **(b)** Add `true_class`/`false_class` to the returned `CoreClasses { … }` literal (`L175-196`). **(c)** Add `pub true_class: ClassId,` / `pub false_class: ClassId,` fields to the `CoreClasses` struct (`L512-570`), rustdoc'd, citing ADR-0004. **Do NOT touch `install_primitives` (the sacred Bool block `L284-308` stays — `True`/`False` inherit).** **Do NOT extend `verify_invariants` (`L404`) — that is U-CORE-1's file/domain (R-INV-0.2); see §0.1 BD-U11-SCHED. You MUST, however, re-run it and confirm it still passes with the two new rows.** |
| `phalcom-core/src/value.rs` | **One arm only.** `Value::class` (`L87-105`), arm `L90`: replace `Value::Bool(_) => vm.universe.classes.bool_class,` with `Value::Bool(b) => if *b { vm.universe.classes.true_class } else { vm.universe.classes.false_class },`. Update the surrounding rustdoc (L76-86) to note the True/False selection (ADR-0004). Keep it allocation-free (hot path). Nothing else in `value.rs` changes (`to_string`/`to_debug`/`value_eq`/`to_context` all remain correct — `Value::Bool` still renders `"true"`/`"false"`, still compares by value, still `Immediate` context). |
| `phalcom-core/src/vm.rs` | **`install_core` only** (`L317-373`): after `add_class!(bool_class);` (`L339`) add `add_class!(true_class);` and `add_class!(false_class);`. The existing macro (`L323-331`) binds the `"True"`/`"False"` globals from `heap.class(id).name` and inserts them into `self.classes` (so the optional `core.ph` reopens resolve). No other `vm.rs` change — `GuardBool` and `note_method_installed` are read-only confirmations (§2). |
| `phalcom-core/src/primitive/mod.rs` | `ClassName` (`L54-72`): add `pub const True: &'static str = "True";` and `pub const False: &'static str = "False";` (capitalized class names — distinct from `ObjectName::True/False` = `"true"/"false"`). Rustdoc each. |
| `phalcom-core/core/core.ph` | **Additive, tiny, OPTIONAL-but-recommended.** Next to `class Bool {}` (`L11`), add empty skeletons `class True {}` and `class False {}` for surface-visibility parity with every other core class (their globals are already bound in Rust; these reopens are safe no-ops per §1). Carry a `//` comment citing ADR-0004 and noting the bodies live on `Bool` by inheritance (D1/KEEP). **Do NOT add method bodies here.** This is the only U11 edit that shares a file with U-CORE-4 (which adds `Bool#toString` inside `class Bool {}`) — additive, different lines, but serialize per §9. |
| `phalcom-core/tests/lang/booleans/*` | New PASS `.ph`/`.expected` pairs (data only; §6) added to the **already-active** `booleans/` dir: class identity (`true.class == True`, `false.class == False`, `True.superclass == Bool`), the sacred selectors still working through the split (`not`/`and`/`or`/`ifTrue`→`Option`), short-circuit preserved, and abstractness. Header convention `// area:` / `// spec:` / `// status: PASS`. |

**Explicitly NOT touched (correcting `U11-plan.md` §3):**
`phalcom-core/src/boolean.rs` (§0.3), `phalcom-core/src/primitive/boolean.rs`
(§0.3 — the sacred primitives + `bool_class_new` stay verbatim; the `println!`s
stay DEFERRED), `universe.rs::install_primitives` (sacred Bool block unchanged),
`universe.rs::verify_invariants` (U-CORE-1's domain — re-run, don't edit),
`tests/lang.rs` (the `booleans` label is already active, §0.4 — no wiring edit
unless you deliberately add a *new* label, which you should not).

---

## 4. Build order (each step verifies green before the next)

1. **`universe.rs` — tower rows.** Add `true_class`/`false_class` to
   `create_core_classes` (after `bool_class`), the `CoreClasses` struct, and the
   returned literal. Build. **Re-run `verify_invariants` (it is `.expect()`ed in
   `VM::new`) and confirm the kernel still boots** — the parallel rule must hold
   for the two new rows exactly as it does for `Number` (`make_core_class` wires
   each metaclass by the same ADR-0002 rule, so this should pass unmodified). No
   observable behavior change yet (`Value::class` still returns `bool_class`).
2. **`primitive/mod.rs` + `vm.rs::install_core`.** Add the `ClassName` consts and
   the two `add_class!` lines so `True`/`False` are bound globals and
   reopen-resolvable. Build; confirm `True`/`False` are now referenceable from a
   `.ph` snippet (`System.print(True)` → prints the class). Still no class-of
   change.
3. **`value.rs::class` — the flip.** Change arm L90 to select `true_class`/
   `false_class`. **This is the behavior-changing checkpoint.** Build + full
   `verify.sh`: now `true.class == True`. Confirm the sacred selectors still work
   (they dispatch `true`/`false` → `True`/`False` → inherit `Bool`'s primitives)
   and no golden shifts (`Value::Bool` still prints `"true"`/`"false"`, so
   `System.print(true)` output is byte-identical).
4. **`core.ph` skeletons** (optional-but-recommended). Add empty `class True {}` /
   `class False {}`. Build; confirm the reopen is a no-op (no clobber, globals
   unchanged) and `verify.sh` green.
5. **Tests.** Add the `booleans/` fixtures (§6). Green + golden byte-identical.

Commit per green checkpoint (never a non-compiling tree). Step 3 is the one to
land atomically with step 1 (a `true_class` field referenced by `value.rs`
without the field existing won't compile — but do steps 1→3 in order and each
compiles).

---

## 5. Design decisions (grounded in ADR-0004 + floor-census §2.6/§5)

- **D1 — Method placement: KEEP (resolved, §0.1).** Sacred primitives stay on
  `Bool`; `True`/`False` inherit. Zero new floor bindings (census stays 80).
- **D2 — Class shape (ADR-0004).** `Bool` abstract (super `Object`); `True`,
  `False` concrete singleton subclasses (super `Bool`). "Abstract" here means *no
  direct instances*: after step 3, **every** `true`/`false` value's class is
  `True`/`False`, so `Bool` is never the direct class of any value — it is only
  ever reached by inheritance. There is no `Bool.new` producing a `Bool`-classed
  value; `bool_class_new` (`Bool.class::new(_)`, a coercion constructor) returns
  `Value::Bool(true/false)`, whose class is now `True`/`False`. That is correct
  and needs no change.
- **D3 — Class-of selection (the flip).** `value.rs:90` branches on the `bool`
  payload: `true → true_class`, `false → false_class`. Handle-select, no
  allocation.
- **D4 — Singleton identity.** `true`/`false` remain the two `Value::Bool`
  immediates; `==` is value equality (`value_eq`, `value.rs:217`) — unchanged.
  There is exactly one `true` and one `false`; `True`/`False` are singleton
  classes by virtue of `Value::Bool` having only two inhabitants. No heap
  singleton instance is needed (unlike `None`), because booleans are immediates,
  not heap objects.
- **D5 — Bootstrap ordering.** `Bool` before `True`/`False` (they need
  `bool_class` as superclass, and `make_core_class` reads
  `heap.class(superclass).class` — `bool_class` already has its metaclass wired at
  L135). `add_class!(true/false)` in `install_core` runs before `core.ph`, so the
  reopens resolve. `verify_invariants` runs last in `VM::new`, after both.

---

## 6. Test strategy — concrete fixtures

All go in the **active** `tests/lang/booleans/` dir (data only; no `lang.rs`
change). Match the header convention. Keep `.expected` keyed on `Bool`/class-name
output, never on internal representation. Build any lists with `List.new()`.

- **Class identity (the core deliverable).**
  `System.print(true.class == True)` → `true`;
  `System.print(false.class == False)` → `true`;
  `System.print(true.class == False)` → `false`;
  `System.print(true.class == Bool)` → `false` (Bool is abstract, not the direct
  class). Also `System.print(True.superclass == Bool)` → `true` and
  `System.print(False.superclass == Bool)` → `true` (proves the subclass wiring;
  `superclass` is a live `Behavior` primitive, `universe.rs:256`).
- **Sacred selectors survive the split (dispatch through inheritance).** Prove
  each still resolves on a `true`/`false` receiver now that its class is
  `True`/`False`:
  `System.print(true.not)` → `false`; `System.print(false.not)` → `true`;
  `System.print(true.and { false })` → `false`; `System.print(false.or { true })`
  → `true`.
- **Short-circuit preserved (regression).** The two existing green fixtures
  (`bool_short_circuit_and` = `false and (1/0)` → `false`;
  `bool_short_circuit_or` = `true or (1/0)` → `true`) must stay byte-identical —
  the dead branch must still not evaluate (proves `bool_and`/`bool_or`'s
  short-circuit reaches through the new class chain unchanged).
- **`ifTrue` still returns a well-formed `Option` (U-CORE-2 regression through the
  split).** `System.print(true.ifTrue { 42 }.isSome)` → `true`;
  `System.print(false.ifTrue { 42 }.isNone)` → `true`. (Confirms the `Some`-lift
  fallback and the inliner's `WrapSome` fast path both still land on the inherited
  `bool_if_true` after the receiver's class became `True`.) Include an inlined
  (pristine) and — optionally — an epoch-deopt variant à la U-CORE-2's
  `absence_iftrue_some_lift_{fast,deopt}_path` if you want belt-and-suspenders.
- **Value rendering unchanged.** `System.print(true)` → `true`,
  `System.print(false)` → `false` (proves `Value::to_string` is untouched by the
  class split — this is what keeps every existing golden byte-identical).
- **`verify_invariants` still passes** (boot check; it is `.expect()`ed in
  `VM::new`, so a broken tower fails every test — but assert it explicitly if
  U-CORE-1's harness is present).
- **No-truthiness regression (ADR-0021).** Any branch/`ifTrue` condition in a new
  fixture must be a real `Bool`; a fixture that branches on an `Option` must be a
  hard error, not a silent pass.
- **Golden byte-identity.** `examples/*.ph`, `tests/fixtures/golden/*` unchanged —
  U11 adds classes and re-points one class-of arm; nothing that any existing
  golden observes (all Bool output flows through `Value::to_string`, untouched).

---

## 7. Mandatory rules (unchanged from `U11-plan.md` §8 — repeated for emphasis)

- **Docs:** full rustdoc on every touched/added Rust item — the two `CoreClasses`
  fields, the two `create_core_classes` bindings, the `ClassName` consts, and the
  updated `Value::class` arm — each citing ADR-0004. `cargo doc --workspace
  --no-deps` clean, zero new warnings. The `core.ph` `True`/`False` skeletons
  carry a spec-referencing `//` comment (ADR-0004; note bodies inherit from `Bool`).
- **Green gate:** `./scripts/verify.sh` exits 0 is your sole sign-off — **reviewer
  OFF for U11** (STATE.md L410 policy). `booleans` green with the new cases,
  `verify_invariants` passes, goldens byte-identical, no new clippy warnings.
  Self-verify: prove `true.class == True` AND that a sacred send (`ifTrue`) still
  Some-lifts through the split, not just class identity.
- **Touch no kernel wiring you don't own:** do **not** edit `verify_invariants`
  (U-CORE-1), the `Option` substrate (U6), or `install_primitives`'s sacred Bool
  block. Add **no `Value` variant** (ADR-0004 is explicit: `Value::Bool(b)`
  stays). Add **no new floor primitive** (census must stay 80 — R-INV-0.1).
- **`graphify update . --no-cluster`** after edits; commit per green checkpoint.
- **Hard stop when green:** do not begin U-CORE-1/4/6 or U-LEX. Record `STATE.md`
  updates the way U8/U9/U10/U-STD did (mark U11 landed; note the object-model.md
  §3/§4 doc-reconciliation as done-or-filed, §0.5). **Do not touch
  `PHASE2-INDEX.md`** (in-flight U-LEX edits, §0.1).

---

## 8. BLOCKED-ON-DECISION register

- **BD-U11-D1 (soft — resolved by this spec toward KEEP; escalate only on user
  request).** Method placement: KEEP sacred primitives on abstract `Bool`
  (inherited by `True`/`False`) vs MOVE to literal per-subclass `.ph` bodies per
  ADR-0004's wording. **Recommendation: KEEP** — it satisfies ADR-0004's
  observable contract (`true.class == True`), is the design `floor-census.md`
  §2.6/§5 already documents, adds zero floor bindings, and avoids the
  epoch-unsoundness + `Some`-lift-parity re-proof that MOVE incurs (§0.1/§2). An
  implementer may proceed on KEEP without further sign-off. **Escalate to the user
  only if they specifically want ADR-0004's literal per-subclass form** — which is
  then a separate later unit scoped to: (1) extend `note_method_installed` to
  watch `true_class`/`false_class`, (2) re-prove inliner ≡ `.ph` `Some`-lift
  parity, (3) update the floor census.
- **BD-U11-SCHED (hard — needs the orchestrator, not the user).** U11's write-set
  collides with the newly-specced **U-CORE-1** (`universe.rs::create_core_classes`
  + `CoreClasses` + `verify_invariants`), **U-CORE-4** (`core.ph` Bool block +
  `value.rs`), and **U-CORE-6** (`create_core_classes` Error rows). **U11 must not
  be co-scheduled in a parallel wave with any of them.** Recommended: land **U11
  first** (tiny, zero net floor bindings), then the U-CORE-N wave (U-CORE-1 first,
  whose extended harness validates U11's rows). See §0.1 / §9. The forge collision
  matrix predates the U-CORE-N taxonomy and does not encode this — this spec is
  the record (do not edit `PHASE2-INDEX.md`).
- **No scope conflict.** Unlike U-STD, U11's mission is not re-carved — it is
  explicitly reserved (§0.2). There is no ownership decision for the user here.

---

## 9. Sequencing note for the orchestrator (record; do not edit PHASE2-INDEX)

Critical-path placement (extends STATE.md's spine `… → Wave F+1 (U9 ∥ U11)`):

```
… U-STD (landed) ──> U11 (Bool tower, serial, ~tiny)
                         └──> U-CORE-N wave:
                                U-CORE-1 (kernel reflection, stands up the invariant
                                          harness — validates U11's True/False rows)
                                  then U-CORE-4 / U-CORE-6 (serialized vs each other
                                  on create_core_classes / core.ph per their own specs)
```

- **U11 and U-LEX are parallel-safe with each other** — U-LEX touches
  `phalcom-ast` + lexer, U11 touches `phalcom-core` kernel wiring; disjoint.
- **U11 alone on its slice.** Its only shared surfaces are `universe.rs`
  (`create_core_classes`/`CoreClasses`) and the `core.ph` Bool area, both shared
  with U-CORE-1/4/6 — so **run U11 by itself**, then fan out the U-CORE-N wave.
- If the orchestrator must invert (U-CORE-1 before U11), the only extra step is
  §0.1's note: U11 then appends `True`/`False` to U-CORE-1's already-extended
  `verify_invariants` loop and re-confirms the census (still 80).

---

## 10. Return contract (answer all of these — mirrors `U11-plan.md` §9)

- Confirm (or report a deviation from) each correction in §0.
- **State the D1 outcome you implemented** (KEEP vs MOVE) and, if MOVE, who
  authorized it and how you handled the epoch-hook + census (§0.1) — an
  implementer must not have moved methods to `True`/`False` without extending
  `note_method_installed`.
- The `True`/`False` tower wiring: the `create_core_classes` additions, the two
  `CoreClasses` fields, the `ClassName` consts, the `add_class!` bindings, and the
  one-arm `value.rs::class` flip — quote the committed `value.rs` arm.
- Proof `verify_invariants()` still passes with the two new rows (you re-ran it;
  you did **not** edit it).
- Proof `true.class == True` / `false.class == False` (fixture output) **and** that
  a sacred send (`ifTrue` → `Some`, `and`/`or` short-circuit) still works through
  the class split — not just class identity.
- Explicit confirmation you added **no `Value` variant**, **no new floor
  primitive** (census unchanged), did **not** edit `install_primitives`'s Bool
  block, `verify_invariants`, `boolean.rs`, or `primitive/boolean.rs` (incl. the
  DEFERRED `println!`s), and did **not** touch `PHASE2-INDEX.md`.
- Whether you added the optional `core.ph` `True`/`False` skeletons, and confirmed
  the reopen is a harmless no-op (no `None`-style clobber).
- The object-model.md §3/§4 reconciliation (§0.5): landed as a doc patch or filed
  for the docs skill.
- Files changed, `verify.sh` tail, `cargo doc` tail, golden byte-identity proof,
  and any new `DEFERRED.md` entries.
