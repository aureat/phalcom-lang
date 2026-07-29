# U-INH — Work order: single inheritance — `extends` superclass, parallel metaclass, `super` sends

_Self-contained implementation plan for **one** implementer. Load-bearing object-model unit
(edits `compiler/lib.rs`, `vm.rs`, `bytecode.rs`, and the class-creation wiring) — **Reviewer ON**;
hand the diff to `phalcom-reviewer`, do not self-approve. Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean + `verify_invariants()` green. Grounded in normative
**[object-model.md §5 (metaclass rules 1–5)](../../../spec/current/object-model.md)** and
**[method-lookup.md §1.14 / §2](../../../spec/current/method-lookup.md)**, and in **[ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md)**
(parallel metaclass rule), **[ADR-0011](../../../adr/0011-fixed-instance-slot-layout.md)** (fixed slot
layout), **[ADR-0012](../../../adr/0012-label-encoded-selectors.md)** (label-encoded selectors, IC-ready
dispatch). **New governing ADR required** for the `SuperSend` opcode — see DEC-INH-D._

> **Why a new grouped unit (user ruling, 2026-07-12).** No in-flight plan owns these three pieces.
> **U13** is a *policy* unit (ratify hierarchy mutability + traits/MI) that **assumes** a correct tower;
> it does not build the mechanism. **U12** explicitly says "do not touch the metaclass tower." **U16**
> (family flattening through inheritance) and **U15** (IC invalidation on `superclass=`) are downstream
> *consumers*. So surface subclassing + `super` + the user-class metaclass repair are grouped here, in
> `U-INH`. This unit lands the mechanism U13's policy later governs. User decisions baked in: syntax =
> **`extends`**, scope = **all-in-one**, `super` lowering = **new `SuperSend` opcode**.

---

## 1. Mission (one sentence)
Make single inheritance real end-to-end — the **`class Dog is Animal { }`** surface (object-model
§5.1), the compiler/runtime wiring so a user class's `superclass` **and** its parallel metaclass
superclass are set per **ADR-0002 rule 4**, and **`super.sel(…)`** sends via a new **`SuperSend`** opcode
that starts method lookup at the *superclass of the defining class* with the original receiver
(method-lookup §1.14) — including **super-construct chaining** — with **zero regression** of the native
tower's already-green parallel-metaclass invariant.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **The native tower already satisfies ADR-0002 rule 4 and is TESTED.**
  `metaclass_superclass_parallels_instance_superclass` ([invariants.rs:214](../../../../phalcom-core/tests/invariants.rs))
  asserts `Number.class.superclass == Object.class`; `universe.rs::create_core_classes` wires
  metaclass-side superclasses by the parallel rule. **The bug is on the *surface `class` path* only**
  (user classes), where the compiler always pushes `Object` and the new class's metaclass superclass is
  (almost certainly) left defaulted. **Confirm** what the VM `Class` handler currently sets for a new
  user class's metaclass superclass before changing it.
- **`object-model.md §5 lines 210–211` are STALE** — they claim "every metaclass's superclass wired to
  `Class`, breaking it," which was true pre-U2 but the native tower is now fixed + tested. Do **not** take
  that note as current state; log a DEFERRED docs-drift entry to re-point it (and `implementation-status.md`).
- **Compiler stub #1** — [compiler/lib.rs:739-743](../../../../phalcom-core/src/compiler/lib.rs): the
  `ClassDef` arm always `add_constant(Object)` then `emit(Class(name_idx))`. The VM `Class` handler at
  [vm.rs:963-973](../../../../phalcom-core/src/vm.rs) **already pops a superclass off the stack** (with an
  `InvalidSuperClass` guard) — so wiring an explicit superclass is a **compiler-side push**, the runtime
  already consumes it. Confirm the stack contract on HEAD.
- **Compiler stub #2** — [compiler/lib.rs:1194](../../../../phalcom-core/src/compiler/lib.rs): `Expr::SuperVar`
  emits `Nil`. **`super` already parses** as `Expr::SuperVar` (a primary, [parser.rs:1431](../../../../phalcom-ast/src/parser.rs));
  a `super.foo(…)` send therefore arrives at the compiler as an ordinary send whose **receiver is
  `Expr::SuperVar`**. Confirm the parser wraps `super.method(args)` as a message-send over `SuperVar`
  (postfix-send path) so **no parser change is needed for super-sends** — only for the `extends` arm.
- **Message-send opcode is `Invoke(u8, u16)`** ([bytecode.rs:66](../../../../phalcom-core/src/bytecode.rs));
  there is **no** super opcode → `SuperSend` is a genuine addition. Confirm the disassembler
  ([bin/phalcom/disasm.rs](../../../../phalcom-core/src/bin/phalcom/disasm.rs)) enumerates opcodes so the new
  arm is added there too.
- **U7 `construct` landed** (fixed slot layout, ADR-0011). **U-CORE-6 landed** the `Error` root + surface
  `MessageNotUnderstood` on a genuine miss — a `SuperSend` that walks off the top of the chain must raise
  the **same** `MessageNotUnderstood`, not panic.
- `./scripts/verify.sh` green before the first edit. Run `graphify affected "ClassObject"` and
  `graphify affected "compiler/lib.rs"`, and **check concurrent `phalcom-ast` / `vm.rs` / `bytecode.rs`
  editors** (§4.1) — this unit must run alone in each.

## 3. Design (realise the spec — semantics are specified; do not re-litigate)

### 3.1 Surface syntax — `class Name is Super { … }` (object-model §5.1)
- `extends` is a **contextual keyword** recognised only in the `class` header position (DEC-INH-A), so it
  does not reserve `extends` as an identifier elsewhere. Grammar: `class` IDENT (`extends` IDENT)? `{` … `}`.
- `ClassDef` gains `superclass: Option<SuperclassRef>` where `SuperclassRef { name: String, range }`
  (full rustdoc, cites object-model §5.1). Absent `extends` ⇒ `None` ⇒ superclass defaults to `Object`
  (unchanged behaviour). The superclass name resolves as an ordinary global/class lookup at compile time.

### 3.2 Superclass wiring — compiler pushes the named class; VM already consumes it
- Compiler: when `ClassDef.superclass` is `Some(name)`, resolve `name` to its class value and push **that**
  (not `Object`) before `emit(Class(name_idx))`. `None` keeps the current `Object` push. **No VM change to
  the pop side** — vm.rs:963 already reads the superclass and guards non-class with `InvalidSuperClass`.
- **Cycle / self-inheritance** (`class A is A`, or a chain that loops): reject at class-creation with a
  clear diagnostic (span on the `extends` clause). A cycle would make method lookup non-terminating —
  non-negotiable guard.

### 3.3 Parallel metaclass for user classes — ADR-0002 rule 4 (the repair)
When a user class `B extends A` is created, in addition to `B.superclass = A` the creation path **must**
set `B.class.superclass = A.class` (rule 4: `(X class).superclass == (X.superclass) class`), anchored by
the existing root rule `(Object class).superclass == Class`. This is what makes **`static` / `construct`
methods inherit**. Fix it **once, in the class-creation path** (DEC-INH-E) so *every* class — surface or
reflectively created — maintains the parallel rule; extend `class_set_superclass`
([primitive/class.rs:34](../../../../phalcom-core/src/primitive/class.rs)) to also relink the metaclass, or
do it in the `Class`-bytecode handler. The native-tower invariant (invariants.rs:214) must stay green **and**
a new user-class invariant must assert the same rule for a compiled `extends`.

### 3.4 `super.sel(args)` → `SuperSend` opcode (method-lookup §1.14)
- Semantics: **start the selector walk at the superclass of the method's *defining* class, with the
  original receiver** (`self`). The defining class is **statically known** — it is the class currently being
  compiled — so the start is resolvable at compile time.
- Lowering: emit `self`, the args, then **`SuperSend(argc: u8, sel: u16, start_class: u16)`** where
  `start_class` is a constant-pool index to the **defining class** (DEC-INH-B — bake the *defining class*,
  not its superclass, so the VM computes `defining.superclass` at dispatch time and stays correct under a
  future `superclass=` mutation, U13). The VM resolves the method from `start_class.superclass` upward with
  the original receiver bound as `self`.
- **Miss handling:** a `SuperSend` that exhausts the chain routes to the **same** `doesNotUnderstand` →
  surface `MessageNotUnderstood` path as an ordinary miss (U-CORE-6 / U8) — never a panic.
- **`super` outside any method** (top-level, or a bare `super` with no send) is a **compile error** with a
  clear span (there is no defining class to anchor the walk). Bare `super` no longer silently emits `Nil`.

### 3.5 super-@constructor
chaining (all-in-one scope)
`super.construct(…)` (or the constructor selector form on HEAD) uses the **same `SuperSend`** mechanism to
invoke the superclass constructor on the *same* instance, so the subclass constructor can initialise
inherited state before its own. Chaining is **explicit** (DEC-INH-C — no implicit auto-chain), matching the
Smalltalk/Wren precedent and keeping ADR-0011 slot semantics simple. **Subclass fields get fresh slots**
(classes.md §… — "a subclass that writes `_name` gets its own new slot"); super-construct must initialise
the parent's slots, never alias the child's. Verify the `construct` selector's exact spelling on HEAD.

### 3.6 Native-vs-`.ph` split & floor delta
- Surface + wiring + `super`: **parser + compiler + VM + class-creation**, all native. **0 new floor
  primitives** (`.ph` floor census unchanged).
- **+1 bytecode** (`SuperSend`). This is a VM-opcode addition, **not** a frozen-floor primitive — note the
  distinction in the return contract; it needs the new ADR (DEC-INH-D), **not** an ADR-0019 floor amendment.

### Rubric — hazards & preclusion (mandatory)
- **Inline-cache soundness (ADR-0012 / ADR-0018 / U15 / U16) — THE load-bearing check.** A `SuperSend` has a
  **static, per-call-site start class**, so its cache key differs from a receiver-polymorphic `Invoke`.
  Confirm the SuperSend dispatch is IC-representable (or explicitly uncached first cut) and that a future
  `superclass=` (U13) / override-epoch bump (ADR-0018) **invalidates** any SuperSend cache the same way it
  invalidates `Invoke`. Coordinate the seam with U15/U16 — **flag, don't silently diverge**.
- **Parallel-metaclass must not regress the native tower.** invariants.rs:214 stays green; add a *user-class*
  analogue (`class B is A` ⇒ `B.class.superclass == A.class`). A regression here silently breaks
  `static`/`construct` inheritance — the exact bug object-model §5 warns about.
- **Method-lookup termination.** Cycle guard (§3.2) is mandatory; without it `super`/lookup can loop forever.
- **`super` miss = `MessageNotUnderstood`, not panic** (§3.4) — pin a negative golden.
- **Slot layout under super-construct** (§3.5): parent init writes parent slots only; no aliasing with the
  child's fresh slots (ADR-0011). Pin a golden where both define a same-named field.
- **Representation/dispatch impact:** one new opcode; **no `Value` tag change, no selector-encoding change.**
  The `Class`-bytecode stack contract is unchanged (VM already pops the superclass).
- **Precedent:** Smalltalk `super` (start-at-defining-class-superclass, original receiver); Wren/Ruby single
  inheritance + `super`. Rejected: dynamic-receiver-class super (breaks the "defining class" rule),
  implicit constructor auto-chaining (DEC-INH-C). Do not reopen.

## 4. Confirmed write-set (tight; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-ast/src/token.rs` | `extends` contextual keyword (DEC-INH-A) | surface |
| `phalcom-ast/src/lexer.rs` | recognise it in the class-header position | surface |
| `phalcom-ast/src/ast.rs` | `ClassDef.superclass: Option<SuperclassRef>` (+ the struct), full rustdoc | surface |
| `phalcom-ast/src/parser.rs` | parse `class N is S { … }`; confirm `super.m(a)` already parses as a send over `SuperVar` (no change if so) | surface |
| `phalcom-core/src/bytecode.rs` **(SPINE)** | new `SuperSend(u8, u16, u16)` opcode | vm |
| `phalcom-core/src/bin/phalcom/disasm.rs` | disassemble `SuperSend` | vm |
| `phalcom-core/src/compiler/lib.rs` **(SPINE — reviewer ON)** | push named superclass (§3.2); `SuperVar`-receiver send ⇒ `SuperSend` (§3.4); bare/`top-level` super ⇒ compile error; super-construct (§3.5); cycle guard | compiler |
| `phalcom-core/src/vm.rs` **(SPINE)** | execute `SuperSend` (start at `start_class.superclass`, original receiver, miss ⇒ dNU); user-class metaclass-parallel wiring in the `Class` handler (or via §3.3 helper) | vm |
| `phalcom-core/src/primitive/class.rs` | extend `class_set_superclass` to relink the metaclass parallel (DEC-INH-E) | wiring |
| `phalcom-core/src/error.rs` | `InheritanceCycle` (and any needed super diagnostics) if not reusing `InvalidSuperClass` | diag |
| `phalcom-core/tests/lang/inheritance/` (**new label**) + `tests/lang/MANIFEST.md` + `tests/invariants.rs` | goldens + negatives + disasm + user-class parallel-metaclass invariant | all |

**Adopted debt (incidental — fix in this unit's `vm.rs` pass; was orphaned, no prior owner).**
- `vm.rs:107-110` — `impl Default for VM` is a bare `todo!()` that panics if VM is ever constructed via
  `Default`. While in `vm.rs` for the `Class`-handler / `SuperSend` work, either give it a real default
  (delegate to the actual constructor) or delete the `Default` impl if nothing needs it — confirm first
  with `graphify affected "VM"` that no caller depends on `VM::default()`. Small, no ADR; land it as its
  **own tidy commit**, kept out of the load-bearing `SuperSend` diff.

**Deliberately NOT in scope:** traits / mixins / multiple inheritance (U13 open-Q10); **runtime `superclass=`
mutability** (U13 open-Q4 — this unit sets it at creation only); `value.rs`/heap tag changes; the
`each`/family combinators. `docs/forge/units/README.md` and the phase INDEX are **not edited** (shared files,
concurrent-session hazard) — roster update is a follow-up docs pass (return contract).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`phalcom-ast/src/parser.rs` + `token.rs`** are contended by the live U12/U14/U15/U16/U18/U-COLL cluster
  (all `phalcom-ast` editors). **Serialize** — U-INH takes its own `phalcom-ast` slot.
- **`vm.rs` / `bytecode.rs` / `compiler/lib.rs`** are the runtime spine — keep serial; confirm no concurrent
  runtime unit holds them before dispatch.
- **`primitive/class.rs` + `universe.rs` wiring** overlaps U12 (numeric tower) and the U-CORE track. Confirm
  neither is mid-flight in the class-creation path.
- **No `core.ph` edit** in this unit ⇒ free of the U-CORE `core.ph` serialization point.

## 5. Build order (small, independently-green diffs)
1. **Surface** — `extends` token + lexer + `ClassDef.superclass` + parser; parse-only/AST goldens. Green.
   *(collides only in `phalcom-ast`.)*
2. **Superclass wiring** — compiler pushes named superclass; cycle guard; runtime already consumes it.
   Golden: `class B is A` ⇒ `B.superclass == A`, an inherited instance method resolves on a `B`. Green.
3. **Parallel metaclass** — set `B.class.superclass = A.class` in the creation path; new user-class invariant
   + native-tower invariant both green; golden: an inherited `static`/`construct` resolves on `B`. Green.
4. **`SuperSend`** — opcode + disasm + compiler lowering (`SuperVar` receiver ⇒ `SuperSend`, defining-class
   constant) + VM execution + bare/top-level-super compile error + miss ⇒ `MessageNotUnderstood`. Goldens +
   disasm golden + negative. Green.
5. **super-construct chaining** — `super.construct(…)` initialises inherited slots; same-named-field slot
   golden (no aliasing). Green.

Each step is a self-verifiable commit (commit-frequently: never leave the tree non-compiling).

## 6. Mandatory rules
- **Docs:** `///` on every new AST node/field, opcode variant, and parser/compiler/VM fn, citing object-model
  §5 / method-lookup §1.14 / ADR-0002 / the new SuperSend ADR. `cargo doc --workspace --no-deps` adds no
  warnings.
- **Green gate:** `./scripts/verify.sh` exits 0; `verify_invariants()` green; no new clippy; no `unsafe`.
  Follow `rust-best-practices`.
- **Reviewer ON** (spine + object-model) — `phalcom-reviewer` gates the diff; the writer never self-approves.

## 7. Test strategy (the green gate must assert) — new `inheritance` label
- **Superclass wiring (PASS):** `class B is A` ⇒ `B.superclass == A`; a method defined on `A` resolves
  on a `B` instance; an override on `B` wins over `A`.
- **Parallel metaclass (INVARIANT):** user-class analogue of invariants.rs:214 — `B.class.superclass ==
  A.class`; a `static`/`construct` defined on `A` is reachable from `B`. Native-tower invariant still green.
- **`super` send (PASS):** `B#m` calls `super.m` ⇒ runs `A#m` with `self` still the `B` instance; two-level
  `C extends B extends A` chains correctly; `super` to a selector only on `A` skips `B`.
- **`super` disasm (PASS):** a `super.m(…)` body emits `SuperSend` with the defining-class constant, **not**
  `Invoke`.
- **super-construct (PASS):** `B extends A` with `super.construct(…)` initialises `A`'s slots then `B`'s;
  same-named field in both ⇒ two distinct slots, no aliasing.
- **NEGATIVES:** `class A is A` (and a longer cycle) ⇒ compile/creation error; `super` at top level or
  outside a method ⇒ compile error with span; `super.unknownSel` ⇒ surface `MessageNotUnderstood` (not panic).
- **Regression:** full `verify_invariants()` + the golden `.ph` corpus stay green (no `Object`-root class
  behaves differently when `extends` is absent).

## 8. Decisions flagged (architect recommendations — the 4 top-level choices are already ruled)
| ID | Decision | Options | Recommendation |
|---|---|---|---|
| **DEC-INH-A** | `extends` keyword form | (A) contextual (only in the `class` header); (B) reserved word | **(A)** — no existing identifier `extends` breaks; only the header needs it. |
| **DEC-INH-B** | `SuperSend` start-class encoding | (A) bake the **defining class** constant, VM computes `.superclass` at dispatch; (B) bake the superclass directly | **(A)** — matches "superclass of the defining class" literally and stays correct under a future `superclass=` (U13). |
| **DEC-INH-C** | super-construct chaining | (A) explicit `super.construct(…)`; (B) implicit auto-chain to the parent constructor | **(A)** — explicit; Smalltalk/Wren precedent; keeps ADR-0011 slot init predictable. |
| **DEC-INH-D** | ADR for the `SuperSend` opcode | (A) small new ADR (grab next-free number, **coordinate — concurrent-session ADR-collision hazard**); (B) amend ADR-0012 | **(A)** — a new dispatch opcode is ADR-worthy; cross-link ADR-0012. Confirm the next free number at author time. |
| **DEC-INH-E** | Where to fix the metaclass parallel | (A) in the shared class-creation path / `class_set_superclass`; (B) only in the `Class` bytecode handler | **(A)** — one site so surface *and* reflective class creation both maintain rule 4. |
| **DEC-INH-F** | `SuperSend` inline-cache treatment | (A) uncached first cut (correct, simple), IC follow-on with U15/U16; (B) cache immediately | **(A)** — ship correctness; wire the IC seam when U15's `superclass=` invalidation lands. Flag to U15/U16. |

## 9. Must-not-preclude check
- **U13 (hierarchy stability policy):** *served*, not precluded — U-INH lands the creation-time mechanism;
  U13 later rules runtime `superclass=` mutability + traits/MI on top. DEC-INH-B's defining-class encoding is
  chosen precisely so a later mutable `superclass=` stays correct.
- **U15 (modules) / U16 (family):** IC-invalidation seam and inheritance-flattened base-name index become
  *reachable* (there are finally real subclasses); DEC-INH-F leaves the cache wiring to them. Not precluded.
- **U12 (numeric split):** its assertion `Integer.class.superclass == Number.class` (native) is unaffected;
  the user-class parallel fix generalises the same rule. Not precluded.
- **Traits / MI (U13 open-Q10):** the single-superclass field + single-dictionary dispatch (ADR-0012) are
  preserved; a future MI must still flatten into the one dictionary — this unit adds no second lookup path.
- **`abstract` classes / `Bool`/`Number` tower:** untouched; `extends` is the surface form the native tower
  already expresses natively.

## 10. Return contract (report to `phalcom-reviewer`)
The `extends` token/AST/parser arm (and confirmation `super.m(…)` needed **no** parser change) · the compiler
named-superclass push + cycle guard · the user-class **parallel-metaclass** fix site (DEC-INH-E) + the new
invariant proving `B.class.superclass == A.class` with the native-tower invariant still green · the
`SuperSend` opcode shape (argc/sel/start-class encoding, DEC-INH-B) + disasm arm + the **disasm golden**
proving `super` emits `SuperSend` not `Invoke` · `super`-miss ⇒ `MessageNotUnderstood` (not panic) ·
super-construct slot-init golden (no aliasing) · the negatives (self/cyclic inheritance, top-level `super`) ·
how DEC-INH-A..F resolved and the **exact ADR number** grabbed for `SuperSend` (DEC-INH-D) · confirmation
**net floor primitives = 0, +1 opcode** · the `inheritance` corpus label + MANIFEST bump · `verify.sh` +
`cargo doc` + `verify_invariants()` tails · new `DEFERRED.md` entries: the **stale object-model §5:210-211 /
implementation-status.md docs-drift** re-point, the SuperSend **IC follow-on** (DEC-INH-F → U15/U16), and the
`README`/INDEX roster row for U-INH (not edited here — shared-file hazard).
