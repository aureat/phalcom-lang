# Bootstrapping & self-hosting — stdlib-in-itself and compiler-in-Phalcom

- Status: **Experimental** (design note; not ratified — Rung C reopens a committed position)
- Date: 2026-07-11
- Resolves: untracked — how Phalcom could grow a self-implemented stdlib and (optionally) a self-hosted front-end
- Reopens: overlay §Compiler ("compiler stays in Rust — **NOT** self-hosted") — Rung C only, behind a superseding ADR
- Depends on: **DEC-A** (kernel `List`, unscheduled), open-Q8 (module/import semantics)
- Related: ADR-0002/0003 (metaclass tower, allocate-then-wire), ADR-0006 (callable root), ADR-0007/0010 (absence, VM-blessed roots), ADR-0012 (`encode_selector` shared by compiler + runtime), ADR-0016 (hand-written lexer/parser), ADR-0017 (sacred-selector inliner + deopt guards)
- Siblings: [iteration-protocol.md](iteration-protocol.md), [equality-and-hash.md](equality-and-hash.md), [numeric-and-string-indexing.md](numeric-and-string-indexing.md)

> **Partially superseded (2026-07-12).** Two dependencies of this note are now **closed**:
> open-Q8 (module/import semantics) by
> [ADR-0027](../../adr/0027-modules-as-files-with-public-by-default-imports.md)
> (file-as-module, public-by-default imports), and open-Q2 (numeric surface split) by
> [ADR-0024](../../adr/0024-numeric-surface-split-int-float-and-division.md). The Rung-C
> "import of compiled units" precondition is therefore **partly satisfied** by ADR-0027.
> No body rewrite needed. Index: [deferred-work.md](../deferred-work.md).

## Context

Two requests routinely get bundled and must be kept apart — they sit on
different axes of the bootstrapping design space:

1. **Standard library implemented in itself** — *kernel-in-the-language*. Phalcom
   already does this partially via `core.ph`; the overlay **endorses growing it**.
2. **The language itself implemented in Phalcom** — *self-hosting*. The overlay
   **explicitly locks the opposite** for the compiler, and Smalltalk-family
   precedent locks it for the VM. This is not a settled direction; anything below
   that touches it is a *proposal to reopen*, gated on a new ADR.

"The language itself" further splits into two sub-levels that must never be
conflated:

- **Front-end** — lexer → parser → bytecode compiler. Decoupled from the runtime
  by the bytecode; interchangeable with the Rust compiler as long as both emit the
  same `Chunk`. **Self-hostable (Rung C).**
- **VM** — interpreter loop, object model, moving/tracing GC. Cannot be expressed
  in the full dynamic language without a substrate cycle. **Not self-hostable in
  the full language (Rung D); keep native.**

### Current state (grounding)

| Fact | Value | Source |
|---|---|---|
| `core.ph` size | 37 lines — 10 reopened class shells + 1 method body (`System.print`) | `phalcom-core/core/core.ph` |
| Front-end size | ~3,330 lines Rust (`parser.rs` 1588, `compiler/lib.rs` 921, `lexer.rs` 415, `inliner.rs` 404) | crate tree |
| Bytecode surface | ~25 ops; everything semantic funnels through `Invoke(u8 argc, u16 sel)` | `bytecode.rs` |
| Primitive surface | ~80 methods across 11 modules (`number` 12, `boolean` 11, `nil` 10, `object` 8, `block` 8, …) | `primitive/*.rs` |
| Bytecode trust | Compiler-produced only; **no external loader, no verifier yet** | overlay §Untrusted input |

The structural fact that makes any of this tractable: **the compiler and the
runtime are decoupled by the bytecode.** Self-hosting the front-end never touches
the VM. That is why Rung C is achievable and Rung D is not.

## Decision

Adopt a **four-rung ladder**. Rungs A–B are pure alignment with the overlay and
should proceed regardless. Rung C is a reopening, gated behind a superseding ADR,
a determinism precondition, and a bytecode verifier. Rung D is explicitly **not
pursued** — the Rust VM stays the permanent native trust-and-performance root,
matching Smalltalk (native/Slang VM), PyPy (RPython VM), CPython, and CRuby.

The realistic target is therefore a **metacircular compiler, never a metacircular
VM**: a native-array-floored `List` → a rich `core.ph` stdlib → an optional
self-hosted front-end running on the permanent Rust VM.

### The rungs

| Rung | What | Overlay alignment | Gating blocker |
|---|---|---|---|
| **A** | Grow the stdlib in `core.ph` atop a frozen native floor | **Aligned** — already the committed direction | **DEC-A: kernel `List`** |
| **B** | Reflective self-modification (MOP, `perform`, reified `Message`) | **Already latent** — no new work to *have* it | — |
| **C** | Self-hosted front-end (lexer/parser/compiler in `.ph`) on the Rust VM | **Reopens** overlay §Compiler | Rung A + reified assembler + determinism + verifier |
| **D** | VM / interpreter loop / GC in Phalcom | **Locked out** — keep native | Not pursued (needs Slang-style substrate) |

## D1 — Freeze the permanent VM-blessed floor

> Drafted as [ADR-0019](../../adr/0019-freeze-vm-blessed-primitive-floor.md) (Proposed).

The native-boundary-creep hazard is a **one-way ratchet**: every primitive kept
in Rust "for speed" shrinks the dogfoodable surface and freezes semantics the
language could otherwise redefine. Decide the floor **once**, here, or it drifts
forever. Recommended permanent floor (everything above is fair game for `.ph`):

- **Allocation & object graph** — `new`, field get/set, class/metaclass wiring.
- **Dispatch** — `Invoke`, `perform`, `doesNotUnderstand` reification.
- **Absence & Bool roots** — `Some`/`None` construction + `match`; `True`/`False`
  + `ifTrue`/`and`/`or`. (Already blessed; Invariants 4/6.)
- **Number arithmetic** — `f64` ops (cannot be expressed without recursing into
  themselves).
- **Block `call`** — the thunk primitive control flow is built on.
- **`System` I/O** — `print`, source-file read (the compiler needs to read input).

Everything else — `List`, `Map`, `Set`, higher-order `String` manipulation,
Option/Result combinators, the iteration protocol, and eventually the compiler —
is `core.ph`.

**Precludes:** pulling a hot op back into the VM later is a semantics freeze, so
D1 is also a *performance* commitment — accept slower `.ph` hot paths, or fund
inline caches / a JIT above the floor. It does **not** preclude the Int/Float
surface split (open-Q2) or NaN-boxing — both live *below* the floor behind the
same `Value`/`Number` API.

## D2 — `List` as a native-array-backed Phalcom protocol (DEC-A)

> Drafted as [ADR-0020](../../adr/0020-kernel-list-native-array-protocol.md) (Proposed).

`List` is the pivot of the entire ladder. It is a hard dependency of
`doesNotUnderstand` (args reified as a `List`), rest-params (`*xs` collects into
one), the iteration protocol, and **the self-hosted compiler itself** (token
stream, AST children, constant pool). It is currently unscheduled and collides
with U-STD in the same wave. **Nothing downstream — dogfooded or self-hosted — is
buildable until `List` lands.**

Representation constraint (avoids re-hitting the absence bootstrap cycle):

- Backing store is a **VM-blessed primitive array** (Rust `Vec` behind a handle).
- Methods (`do`, `collect`, `inject`, `at`, `add`, …) are `.ph`; storage is native.
- **Not** a linked list of `Some`/`None` cells — that would work but drags `Option`
  onto the collection hot path and is slow. This is the Smalltalk
  `OrderedCollection` shape: native storage, self-defined protocol.

**Precludes:** a purely-in-Phalcom `List` (no native array) is rejected — it would
put allocation of every element behind `Some` and couple collections to absence.

## D3 — Kernel load DAG

Load order is a topological sort; one wrong edge is a hard boot failure with no
user frame to blame (crown-jewel: kernel-load-order ⊗ inter-class deps). The DAG:

```
Object
 └─ Behavior ──┬─ Class
               └─ Metaclass          (cyclic apex: allocate-then-patch; verify_invariants ONCE, at end)
Bool (True/False)      ← before ANY control-flow method
Option (Some/None)     ← before any field defaulting to None
Number, Symbol, String ← primitive-backed, no kernel deps
────────── boundary: below = VM-blessed (D1); above = core.ph ──────────
List                   ← DEC-A. HARD DEP of everything below.
Map, Set               ← need List + Object identity/hash
Iteration protocol     ← needs List
Option/Result combinators (map/flatMap/orElse/…)  ← need Block + match
Message.args           ← needs List (dNU reifies args as a List)
rest-params *xs        ← needs List
──────────
[Rung C] Lexer → Parser → Compiler  ← needs String, List, Map, I/O
Reflection / Family introspection    ← consumes the finished tower; loads LAST
```

`verify_invariants()` runs **exactly once** at end-of-bootstrap (patch-order ⊗
verify-timing hazard: earlier = spurious failures, never = missed link ships).

## D4 — Rung C: reify the assembler

Today `Chunk`/`Bytecode`/`Signature` are Rust-private. Writing the compiler in
Phalcom exposes a **bytecode assembler as a Phalcom class** — the single new
native surface Rung C requires. Sketch:

```phalcom
class Assembler {
  emitConstant(idx)            // -> Constant(u16)
  emitInvoke(argc, selIdx)     // -> Invoke(u8, u16)   — the workhorse
  emitGetLocal(slot)  emitSetLocal(slot)
  emitGetField(slot)  emitSetField(slot)
  emitJumpIfFalse()  -> Label  // patchable forward jump
  patch(label)
  emitReturn()
  addConstant(value) -> idx
  internSelector(name, labels) -> idx   // MUST equal Rust encode_selector, byte-for-byte
  finish() -> Chunk
}
```

**Hard correctness constraint:** `internSelector` and constant-pool ordering in
Phalcom must produce **byte-identical** output to the Rust `encode_selector`
(overlay: "compiler & every runtime builder share one `encode_selector`",
ADR-0012). If they diverge, a Phalcom-compiled method and a Rust-compiled call
site compute different selector symbols and dispatch silently misses. A
byte-parity test between the two encoders is a Rung C gate. This is the
identity-dispatch invariant reaching into the assembler.

### Pipeline, mirrored

```
String (source)
  → Lexer.ph    → List<Token>      (needs String byte access + List)
  → Parser.ph   → AST objects      (recursive-descent/Pratt; mirrors parser.rs)
  → Compiler.ph → Assembler calls  (name resolution → slots, lowering, inliner)
  → Chunk       → VM runs it (Rust interpreter loop unchanged)
```

The **inliner** is the subtle stage: a Phalcom emitter that inlines sacred
selectors (`ifTrue`, `whileTrue`, `and`) **must emit `GuardBool`/`GuardBlock`
with the same override-epoch deopt semantics** (ADR-0017), or it produces unsound
fast paths the Rust compiler would not. Reuse the guard contract; do not
reinvent it. (Speculative-inlining ⊗ late-binding hazard.)

## D5 — Staging & the fixpoint gate

Three-stage build (rustc/Go model) is how self-hosting is *proven*, not merely
asserted:

```
stage0 = Rust compiler (frozen, ARCHIVED seed)
stage1 = stage0 compiles Compiler.ph → Phalcom-compiler-A
stage2 = stage1 compiles Compiler.ph → Phalcom-compiler-B
stage3 = stage2 compiles Compiler.ph → Phalcom-compiler-C
GATE:  stage2 bytecode == stage3 bytecode   (byte-identical ⇒ compiler is a fixpoint)
```

Two preconditions, both hazards that fire here:

- **Deterministic codegen (precondition, not nice-to-have).** Phalcom dispatch is
  hashmap-keyed; if the compiler ever iterates a `Map` to emit constants or
  selectors, order is nondeterministic and `stage2 ≠ stage3` *even for a correct
  compiler*. Emission must be sorted/insertion-ordered **before** the gate means
  anything. Land this in the *Rust* compiler first (compiling the same source
  twice → byte-identical `Chunk`).
- **Seed ⊗ new-feature use.** The moment `Compiler.ph` uses a Phalcom feature it
  just taught itself to compile, `stage0` can no longer build it. Rule: **a
  feature ships in the Rust seed one release before `Compiler.ph` consumes it.**
  Archive `stage0` permanently — it becomes the trust root (Thompson: reading
  `Compiler.ph` proves nothing; only the archived seed or diverse
  double-compilation refutes a backdoor).

## D6 — Bytecode verifier (security precondition)

Bytecode is currently **trusted by construction** — only the Rust compiler makes
it. Rung C breaks that invariant: Phalcom code now produces bytecode the VM runs.

- **Compile-and-run, in-process, no serialization** → verifier may be *deferred*;
  the Phalcom compiler is as trusted as any `.ph` in the image.
- **Serialize / load a `.pcbc` (snapshot, or `import` of a compiled unit — open-Q8)**
  → verifier **mandatory**: bounds-check every `Get/SetLocal`/`Field` slot,
  validate jump targets land in range, cap stack depth, reject malformed
  selectors, convert every malformed input into a *defined* error (never a raw
  `panic!`). An out-of-range `SetField(u16)` is a type-confusion primitive. This
  gates any distribution of compiled Phalcom. (Dynamic-power ⊗ untrusted-input
  hazard.)

## D7 — Boot artifact: snapshot, not image

`core.ph` is re-parsed/compiled each boot (37 lines — free today). A real stdlib +
a Phalcom compiler is thousands of lines → cold-start cost becomes real.

- Stay **source-based** while small — reproducible, no drift.
- When it hurts, add a **deterministic compiled snapshot** (a `.pcbc` build
  artifact rebuilt from source in CI) — fast boot **and** source-is-truth.
- **Reject a live Smalltalk-style image**: it accumulates state no source file
  records, and the heap becomes the only source of truth (image-staleness hazard).

## Precedent (with the cost each paid)

| Language | Self-hosts | Does **not** | Lesson for Phalcom |
|---|---|---|---|
| Smalltalk (Squeak/Pharo) | compiler + full class library | **VM in Slang** (static subset → C) | Exact shape to copy: dogfood above the VM; the full dynamic language can't express its own moving GC |
| rustc | whole compiler; 3-stage fixpoint | std relies on LLVM | Staging + `stage2==stage3` is the proof; seed archival mandatory |
| Go | compiler in Go | bootstrapped from a frozen older Go | "feature one release ahead of use" is a real, painful constraint |
| PyPy | interpreter in **RPython** (restricted) → C/JIT | not full-Python-in-Python at runtime | Same Slang lesson: the metacircle runs in a *subset* a translator can lower |
| Wren / Lua | ~nothing; permanent C core | — | Baseline for "small embeddable VM — don't bother"; Phalcom is closer to Smalltalk, so A/C pay off |
| CPython / CRuby | large stdlib in-language | VM + core types in C | Confirms Rung D is not the norm even for big dynamic languages |

The pattern is unanimous: **self-host the compiler and library; keep a native (or
restricted-subset-translated) VM.** No production dynamic language runs its own
interpreter loop in its own full dynamic semantics.

## Staging plan (ordered; each rung independently shippable)

| # | Step | Gate |
|---|---|---|
| 1 | **ADR: freeze VM-blessed floor** (D1) | Written "native forever" list |
| 2 | **`List` as native-array-backed `.ph` protocol** (D2 / DEC-A) | `List` on critical path; U-STD reordered behind it |
| 3 | **Grow `core.ph` stdlib** — collections, iteration, Option/Result combinators, `Message.args`, rest-params | `verify_invariants()` green; dNU + variadics work in `.ph` |
| 4 | **Deterministic codegen** in the Rust compiler | Compile same source twice → byte-identical `Chunk` |
| 5 | **ADR superseding "compiler NOT self-hosted"** — only if Rung C earns it | Explicit decision; open-Q8 resolved enough to load compiled units |
| 6 | **Reify `Assembler` + `Chunk`** (D4) | `encode_selector` byte-parity test (Phalcom vs Rust) |
| 7 | **Port lexer → parser → compiler to `.ph`** (D4), staged one feature behind the seed | `stage2 == stage3` fixpoint |
| 8 | **Bytecode verifier** (D6) | Before any serialized `.pcbc` / `import` of compiled units |
| 9 | **Deterministic snapshot** (D7) | Cold-start artifact; never a live image |

Steps 1–3 are alignment (do regardless). Steps 5–8 are the reopening — behind an
ADR, gated on the fixpoint and verifier.

## What this precludes (mandatory step-5 check)

- Publishing Phalcom-emitted bytecode **precludes skipping the verifier** —
  trusted-by-construction dies the moment non-Rust code produces bytecode (D6).
- A rich `core.ph` on a shrinking native floor **precludes fast cold start**
  unless a deterministic snapshot is added; a live image would preclude
  source-as-truth (D7).
- A self-hosted front-end **freezes the bytecode format into a public ABI**
  between the Phalcom compiler and the Rust VM — churning it becomes a breaking
  change.
- It does **not** preclude open-Q2 (Int/Float surface split) or NaN-boxing —
  provided the stdlib is written against the *abstract* `Number`/dispatch
  protocols, both stay open below the floor.

## Test strategy

- **Kernel load**: `verify_invariants()` green after each `core.ph` growth step;
  a negative test that a method sending to a not-yet-installed primitive fails at
  *load* with a defined diagnostic, not a panic.
- **`List`**: golden `.ph` corpus for `do`/`collect`/`inject`/`at`/`add`; dNU
  `Message.args` and `*xs` rest-params exercised end-to-end.
- **Assembler parity (Rung C)**: property test — for a corpus of selectors,
  Phalcom `internSelector` output == Rust `encode_selector` output, byte-for-byte.
- **Determinism (Rung C)**: compile the corpus twice, assert identical `Chunk`
  bytes (precondition for the fixpoint).
- **Fixpoint (Rung C)**: `stage2 == stage3` byte-identical in CI.
- **Verifier (Rung C)**: fuzz malformed `.pcbc` — every input yields a defined
  error, never UB / panic (folds into the existing fuzz + miri lanes).

## Open questions

- **B1** — Does Rung C earn its cost? (dogfooding + user-extensible compilation +
  macro/DSL leverage vs a permanent second compiler to maintain and keep in
  lockstep with the Rust seed.) Decide before step 5.
- **B2** — `import` of compiled units (open-Q8) is the first real consumer of a
  loaded `Chunk`; its module semantics must land before serialized bytecode, and
  it forces D6.
- **B3** — If a Slang-style translatable subset is ever wanted for Rung D, what is
  the subset boundary? (Out of scope here; recorded so it is not assumed away.)
