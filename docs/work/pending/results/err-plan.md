# U-ERR — Work order: the error-handling surface + `Result`/`Ok`/`Err`

_Self-contained implementation plan for **one** implementer. **Reviewer ON** (touches the two spine
surfaces `compiler/lib.rs` + `phalcom-ast/src/parser.rs`, the frozen-floor `primitive/block.rs`, and
`core.ph`) — hand the diff to `phalcom-reviewer`; never self-approve. **Worktree isolation** (mutates
`block.rs` / a small `vm.rs` helper / `core.ph` / `compiler/lib.rs` while U-ITER, U-FIBER, and the
U-CORE / `phalcom-ast` clusters are live). Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean. Grounded in **[ADR-0008](../../../adr/0008-layered-exceptions-and-result.md)**
(error **model** — terminating, one unwind, handling as a `Block` protocol),
**[ADR-0031](../../../adr/0031-error-handling-surface-syntax.md)** (surface **syntax** —
`throw`/`try`/`catch`/`on`/`ensure`), **[ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md)**
(the abstract-root-plus-two-subclasses machinery `Result` reuses), and the normative
**[error-handling.md](../../../spec/current/error-handling.md)** §1–§6 + **[result.md](../../../spec/current/result.md)** §1–§5.
**New governing ADR REQUIRED:** one ADR-0019 floor amendment for the two native `Block` handler
primitives (`on(_)(_)`, `ensure(_)`) — see §2.1 / §7 / DEC-ERR-A. The `documentation-and-adrs` skill
drafts it; claim the next free number at dispatch._

> **Unit-name note.** This is the "later unit" that `docs/forge/units/U-CORE-0/decision-register.md` §Q2 and
> [U-CORE-6/as-built.md](../U-CORE-6/as-built.md) §0 explicitly **reserved** ("Reserve — do not
> implement — `Result`/`Ok`/`Err` and the full `try`/`catch`/`on`/`ensure` block protocol; those are a
> later unit."). ADR-0031's ratification of the surface spelling unblocks it. Named **`U-ERR`**, verified
> free at plan time (`U-ERR`/`U-THROW`/`U-EXC` all absent). It does **not** edit
> `docs/forge/PHASE2-INDEX.md` or `docs/forge/units/README.md` (shared files, concurrent editors).

---

## 0. Preconditions (verify on actual HEAD — do not assume; these were confirmed at plan time but the tree is live)

- **U-CORE-6 landed** — `Error` (raisable root) + `MessageNotUnderstood` are surface classes with native
  `Error#message` / `Error#raise`, and the unified unwind's **`Raise` payload** exists:
  `RuntimeError::Raise { error: Value, rendered: String }` (`error.rs:87`, confirmed present) inside
  `PhError::Runtime`. `error` is the catchable surface `Error` value; `rendered` is a display snapshot
  only. `primitive/error.rs` (`error_raise`/`error_message`) is registered. **This unit builds the
  *catch* side of the payload U-CORE-6 produces.**
- **U4 / U-CORE-3 landed** — first-class `Block` + `block_call` (`primitive/block.rs:117`) re-enters
  `run_until(base_frames)` for one activation. `Block` today ships **only** `arity`/`name`/`call`/
  `callWith`/`whileTrue` — **`on(_)(_)`, `ensure(_)`, and `attempt()` do NOT exist** (confirmed:
  `universe.rs:427–436`). This unit adds them. **This is the load-bearing finding: the entire handling
  protocol is greenfield native work, not a wiring-up of existing sends.**
- **U6 landed** — `Option`/`Some`/`None` exist (abstract `Option` + `Some(_value)` + `None` singleton;
  `Some(_)` construction + `match(some:,none:)` are native, `option`). `Result` **mirrors** this shape;
  `result.ok()` returns an `Option`, `option.okOr(_)` returns a `Result` — both must round-trip.
- **U7 landed** — `.ph` `construct` + `_`-prefixed instance fields + fixed slot layout (ADR-0011),
  `561f7e2`. **Decisive for the floor split:** because user-facing `construct`/fields now work in `.ph`,
  `Result`/`Ok`/`Err` are **pure `.ph`** (unlike `Some`, which predated U7 and had to be native — see
  `nil.rs:7` "bootstrapped in Rust so U6 does not depend on U7's user-facing `construct`"). **Confirm U7
  is committed on HEAD before relying on `construct`.**
- **U10 landed** — `Bytecode::ReturnNonLocal` + eager frame-token unwind + `DeadFrameError` (`vm.rs:1154`).
  **Read this handler before touching `on`/`ensure` — its asymmetry with `Raise` is §2.3's crux:** a
  non-local `return` unwinds **eagerly in place** (mutates `self.frames`/`self.stack`, truncates to the
  home frame, then lets the loop continue — it does **not** return `Err`), so it surfaces to a nested
  `block_call` as an **`Ok(value)` with a shrunk frame stack**; a `throw`/`Raise` propagates as
  **`Err(RuntimeError::Raise)` via Rust `?` with frames left intact** (`run_until` never truncates on
  error — that is how the top-level renders a source-mapped trace). `ensure` must fire on **both**; `on`
  catches only the `Err(Raise)`.
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected "core.ph"`,
  `graphify affected "compiler/lib.rs"`, `graphify affected "primitive/block.rs"`, and **check concurrent
  `phalcom-ast` / `core.ph` / `vm.rs` editors** (§3.1).

---

## 1. Mission (one sentence)
Land Phalcom's error-handling **surface** and value channel on top of U-CORE-6's `Raise` payload — the
`throw` prefix and the `try`/`on`/`catch`/`ensure` statement (ADR-0031, 1:1 sugar over `Block` sends), the
two **native** `Block` handler primitives `on(_)(_)`/`ensure(_)` that intercept a `RuntimeError::Raise` at
a block boundary and restore VM frame/stack state (the **+2 floor amendment**, the only new primitives),
and the **pure-`.ph`** `Result`/`Ok`/`Err` mirror of `Option` with the `attempt`/`unwrap`/`okOr`/`ok`
bridges — with `attempt()` itself derived in `.ph` over `on`, so the net floor delta is exactly **+2**.

## 2. Design (realise ADR-0008 model + ADR-0031 surface; do not re-litigate either)

### 2.1 The native-vs-`.ph` split & the frozen floor (the headline finding — flag, do not silently add)

| Concern | Native (Rust) | `.ph` / compiler | Floor Δ |
|---|---|---|---|
| `throw expr` → `expr.raise()`; `throw <non-Error literal>` compile error | — | compiler lowering + compile check (§2.2) | **0** |
| `try`/`on`/`catch`/`ensure` statement | — | parser + compiler nested-desugar (§2.4) | **0** |
| `Block#on(_)(_)` — catch a `Raise` matching a class, run handler, else re-propagate | ✅ `block_on` (§2.3) | — | **+1** |
| `Block#ensure(_)` — run cleanup on *any* exit (normal / `Raise` / non-local `return` / future `abort`) | ✅ `block_ensure` (§2.3) | — | **+1** |
| `Block#attempt()` — throw→value bridge → `Result` | — | `.ph` over `on` (§2.5) | **0** |
| `Result` / `Ok` / `Err` + `isOk`/`isErr`/`map`/`mapErr`/`andThen`/`unwrap`/`unwrapOr`/`unwrapErr`/`ok`/`okOr` | — | **pure `.ph`** (U7 `construct`+fields; §2.5) | **0** |
| `option.okOr(_)` → `Result` (reopen `Option`) | — | `.ph` on `Option`/`Some`/`None` | **0** |

**Net floor delta: +2 bindings** (`Block#on(_)(_)`, `Block#ensure(_)`). Both fail the ADR-0019 §1
derivability test — a `.ph` body has **no way** to observe a `throw` (a Rust-level `Err`) or a non-local
`return` (an invisible in-place unwind); the whole point of these primitives is to convert those into a
value/handler dispatch. So they are an **ADR-0019 amendment**. **This is NOT pre-cleared** — the omnibus
[ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md) named hash + kernel
reflection + `Error#message`/`raise`, but **not** `Block#on`/`ensure` (confirmed: no match at plan time).
So author a **new per-unit ADR-0019 landing-record amendment** (mirrors [ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md)
for U-CORE-3): cite ADR-0008 as the design authority, record `+2 bindings` / `+2 distinct fns`
(`block_on`, `block_ensure`), and bump `floor-census.md`. **Claim the next free ADR number at dispatch
(`ls docs/adr/`).** See DEC-ERR-A. *A small `pub(crate)` `vm.rs` restore helper (§2.3) is **plumbing**,
not a bound selector — it does not count toward the floor, exactly as U-CORE-6's `Raise` payload did not.*

### 2.2 `throw` — prefix lowering + the only-`Error`-throwable rule (error-handling.md §1)
`throw expr` compiles to: emit `expr`, then `Invoke` the `raise()` selector (0-arity) — i.e. `expr.raise()`
(ADR-0031 §1, U-CORE-6's `error_raise`). Because `raise` is installed **only on `Error`**, a runtime
`throw 42` (`42.raise()`) misses → dNU → `MessageNotUnderstood` (the runtime half of the rule, already
live). The **compile-time** half is this unit's job: `throw` of a **syntactic non-`Error` literal**
(`throw "oops"`, `throw 42`, `throw true`) is a **compile error** (error-handling.md §1: "`throw "oops"`
is a compile error"). Scope the static check to literals the compiler can prove are non-`Error`
(`String`/`Number`/`Bool`/list/map literals); a `throw someVariable` cannot be statically typed and defers
to the runtime dNU (do **not** attempt flow typing). `throw` is a prefix form usable in statement position;
since `raise` unwinds, its "expression value" never materialises — treat it as a diverging expression.

### 2.3 `Block#on(_)(_)` and `Block#ensure(_)` — the catch machinery (THE crux; error-handling.md §2, §4)
Both are native primitives on the `block_cls` (and `function_cls`, mirroring how `call`/`whileTrue` are
installed on both). **The receiver is always a `Block`** (each desugar layer re-wraps in `{ }`, §2.4), so
`block_call` applies.

**Snapshot-and-restore is mandatory** because `run_until` leaves a throwing block's frames live:
1. On entry, snapshot `let (s0, f0) = (vm.stack.len(), vm.frames.len());`.
2. Run the protected block: `let outcome = block_call(vm, receiver, &[]);`.
3. Dispatch on `outcome`:
   - **`Ok(v)` with `vm.frames.len() == f0`** — normal completion. `on`: return `Ok(v)` (no handler).
     `ensure`: run cleanup, then return `Ok(v)` (see cleanup rule below).
   - **`Ok(v)` with `vm.frames.len() < f0`** — a **non-local `return`** unwound *through* the protected
     block (U10 already truncated frames below `f0`, `v` is the home method's return value). `on`: this is
     **not** a catch — return `Ok(v)` unchanged; the enclosing `call_method` Primitive arm re-pushes it and
     the unwind continues to the home frame (ADR-0008 §4.2: a `return` in a protected block returns to *its*
     home). `ensure`: **must still fire** — run cleanup, then return `Ok(v)` so the unwind resumes. This is
     the one subtle rule (ADR-0008 §4.1: "`ensure` fires on *any* unwind through it").
   - **`Err(PhError::Runtime(RuntimeError::Raise { error, .. }))`** — a `throw`. `on(T, handler)`: test
     `error isA T` by **walking `error`'s class→superclass chain in Rust** (mirror core.ph `isA`; `T` must
     be a `Class` else type-error). **Match** → restore VM state to the snapshot (§ restore rule), then
     `block_call(vm, handler, &[error])` and return its result. **No match** → return the `Err` **unchanged**
     (frames untouched — so the next `on` in the nested chain, or the top-level trace renderer, sees the full
     stack). `ensure`: run cleanup, then re-propagate the **same** `Err` (unless cleanup itself diverges).
   - **any other `Err`** (`DeadFrameError`, a future `abort` payload, etc.) — `on`: re-propagate unchanged
     (`on` catches only `Raise`). `ensure`: run cleanup, then re-propagate.

**Restore rule (the borrow-model fragility — get this exactly right).** Before running a handler/cleanup
after a caught `Raise`, discard the abandoned frames of the throwing block: **close upvalues first, then
truncate** — `vm.close_upvalues_from(<stack offset of frame f0>)` then `vm.frames.truncate(f0)` and
`vm.stack.truncate(s0)` — mirroring the exact order the `Return`/`ReturnNonLocal` handlers use
(`close_upvalues_from` *before* `stack.truncate`, `vm.rs:1147`/`1195`). Skipping the upvalue close is a
use-after-free for any closure that escaped the throwing block. **Prefer adding one small `pub(crate)`
helper to `vm.rs`** — e.g. `fn unwind_to(&mut self, stack_len: usize, frames_len: usize)` that does
close-upvalues + double-truncate — rather than reaching into VM internals from `block.rs`; this keeps the
restore logic beside the existing unwind handlers and is **plumbing, not a new unwind mechanism** (no
ADR-0019 impact beyond the +2 bindings). The `s0` stack floor equals the block's `stack_offset`; confirm
against `block_call`'s `stack_offset = vm.stack.len()` convention.

**Cleanup-supersedes rule.** If the `ensure` cleanup block itself `throw`s or `return`s non-locally, that
new unwind **supersedes** the pending one (ADR-0008 §4.2: cleanup is an ordinary block; a `throw` inside it
unwinds outward). Implement by running cleanup and, if it yields `Err`/shrinks frames, propagating **that**
outcome instead of the saved one. Pin this with a golden.

### 2.4 The `try` statement — nested re-wrapping desugar (ADR-0031 §3; the non-obvious compiler detail)
`try { P } on A a { HA } on B b { HB } catch e { HC } ensure { C }` does **not** compile to a flat
left-associative chain — `{P}.on(A){}.on(B){}` would send `on(B)` to the *value* `on(A)` returns, not a
block. Because `on`/`ensure` are **eager** (they run their receiver block immediately), each successive
clause must **wrap the accumulated expression in a fresh block literal**. The compiler (which has all
clauses at once) emits, inside-out:

```phalcom
{ { { { P }.on(A) { a => HA } }.on(B) { b => HB } }.on(Error) { e => HC } }.ensure { C }
```

- `{ P }` runs **once** (innermost). `on(A)` catches `A` from `P`; whatever escapes (including a `throw`
  inside `HA`) is re-wrapped and offered to `on(B)`, then `on(Error)` (`catch`), giving **first-listed =
  innermost = first-checked = first-match-wins** (error-handling.md §2). `catch e` ≡ `.on(Error){e=>}`
  (catch-all, since `Error` is the root).
- `ensure { C }` is the **outermost** wrapper, so `C` runs on every exit path of the whole construct —
  normal value, throw, or non-local `return` (§2.3).
- A `try` with only `ensure` (no `on`/`catch`) → `{ P }.ensure { C }`. A `try` with only `on`/`catch` →
  the `on`-nest with no `.ensure`. `try` with neither is a compile error (empty handler set).
- **`on`/`catch`/`ensure` are contextual keywords** — reserved *only* as `try`-clauses. Everywhere else
  they stay ordinary identifiers/selectors: the desugar targets `on(_)(_)`/`ensure(_)` must remain callable,
  and the `Fiber>>try` message must survive (`try` is reserved only at **statement-leading** position, not
  in message position). Implement with the lexer emitting ordinary identifiers and the **parser** treating
  `on`/`catch`/`ensure` as keywords solely while parsing a `try` tail (ADR-0031 §4).

### 2.5 `Result`/`Ok`/`Err` + bridges — pure `.ph` (result.md §1–§3; ADR-0007 machinery)
Mirror `Option`/`Some`/`None` exactly, but in `.ph` (U7 gives `construct`+fields; no native construction
needed — this is the floor win over `Some`). Abstract `Result` holds the combinators over a `.ph`
`match(ok:, err:)`; `Ok`/`Err` each define `match` + hold one field:

```phalcom
class Result {
  isOk    => self.match(ok: { v => true },  err: { e => false })
  isErr   => self.match(ok: { v => false }, err: { e => true })
  map(f)     => self.match(ok: { v => Ok(f.call(v)) },  err: { e => self })      // Err passes through
  mapErr(f)  => self.match(ok: { v => self },           err: { e => Err(f.call(e)) })
  andThen(f) => self.match(ok: { v => f.call(v) },      err: { e => self })      // flat-map; Err short-circuits
  unwrap     => self.match(ok: { v => v },              err: { e => e.raise() }) // re-throw (throw expr ≡ .raise())
  unwrapOr(default) => self.match(ok: { v => v },       err: { e => default })
  unwrapErr  => self.match(ok: { v => v.raise() },      err: { e => e })         // symmetric: throw if Ok
  ok()       => self.match(ok: { v => Some(v) },        err: { e => None })      // Result → Option
  okOr(err)  => self                                                             // Result → Result identity guard? no — okOr is on Option
  toString   => self.match(ok: { v => "Ok(" + v.toString + ")" }, err: { e => "Err(" + e.toString + ")" })
}
class Ok  { construct(v) { _value = v }  match(ok: o, err: e) => o.call(_value) }
class Err { construct(e) { _error = e }  match(ok: o, err: e) => e.call(_error) }
```
- `okOr(_)` lives on **`Option`** (not `Result`): reopen `Option`/`Some`/`None` — `Some(v).okOr(err) => Ok(v)`,
  `None.okOr(err) => Err(err)` (result.md §2, absence→error bridge). Reuse `Option#match`.
- `unwrap`/`unwrapErr` are the **only** methods that cross into the exception channel; they re-throw via
  `raise` (already on `Error`). If `_error`/`_value` is a **non-`Error`** reason (a user built `Err(42)`),
  `raise` misses → dNU → `MessageNotUnderstood` — consistent with the only-`Error`-throwable rule; note in
  a fixture, do not special-case.
- **`Ok(v)` / `Err(e)` construction surface.** `Ok`/`Err` are `.ph` classes; `Ok(v)` is a call-form
  construction send. **Confirm on HEAD how `Some(x)` construction desugars** (compiler lowers `Some(x)` to
  the `Some.new(_)` send / `WrapSome`) and reuse the *same* path for `Ok`/`Err` so `Ok(v)` reads naturally.
  If the `Name(args)` construction sugar is `Some`-specific, either (a) generalise it to any class with a
  matching `construct`, or (b) accept `Ok.new(v)` and add the `Ok(v)`/`Err(v)` sugar as a follow-on. See
  DEC-ERR-B.
- **`Block#attempt()` is `.ph` over `on`** — the throw→value bridge (error-handling.md §5), no new floor:
  ```phalcom
  attempt => { Ok(self.call()) }.on(Error) { e => Err(e) }
  ```
  Success: the inner block returns `Ok(v)`; a `throw` inside `self.call()` is caught by `on(Error)` and
  becomes `Err(e)`. Install on `Block`/`Function` as a `.ph` reopen, or express as a plain method — confirm
  `Block` is reopenable in `core.ph` (it is a native kernel class; the `.ph` `attempt` body must not read a
  native field, only send `call`/`on`, so no read-before-write hazard).

### Rubric — hazards & preclusion (mandatory)
- **`Raise`-catch ⊗ live frames (THE crown-jewel hazard).** `run_until` leaves the throwing block's frames
  and stack live (for the top-level trace). `on`/`ensure` **must** snapshot `(stack.len, frames.len)` on
  entry and, on a caught `Raise`, `close_upvalues_from` **then** double-truncate before running the
  handler/cleanup (§2.3). A missed upvalue-close is a use-after-free; a missed truncate corrupts the handler's
  stack. Pin an invariant + a golden where the handler allocates after a deep throw.
- **non-local `return` ⊗ `ensure` (the §4.1 subtlety).** A `return` through a protected block is an
  `Ok(value)` with `frames.len() < f0`, **not** an `Err`. `on` must not catch it; `ensure` must fire on it
  and re-return so the unwind resumes. Golden: `ensure` fires on normal exit **and** `return` **and** `throw`.
- **first-match-wins ⊗ re-propagation.** A non-matching `on` must re-propagate the `Err` **verbatim**
  (frames untouched) so the next chain layer / top-level sees it. Only a *matching* `on` restores state.
  Golden: `on A` then `on B`, throw `B` → `B`'s handler runs, `A`'s does not; throw `C` → neither, uncaught
  trace intact.
- **cleanup-supersedes.** A `throw`/`return` inside an `ensure` cleanup supersedes the pending unwind
  (§2.3). Golden.
- **fiber-locality (forward-compat §7 D7 / concurrency.md §1).** The snapshot/restore uses **relative**
  `self.frames.len()` / `self.stack.len()`, never absolute indices or `base_frames == 0`, so when U-FIBER
  makes `self.frames` per-fiber the catch stays fiber-local automatically and a `throw` reaching the fiber
  floor (never caught by an inner `on`) is captured into the fiber's result slot rather than terminating the
  host. **Do not hardcode the main stack.** This is the load-bearing preclusion check.
- **`Result` shape ⊗ `Option` (forward-compat §2).** `Result`/`Ok`/`Err` mirror `Option`/`Some`/`None`
  (abstract root + two subclasses, one field each) and must **not** couple to `Option`'s *native* `match`
  — `Result` gets its **own** `.ph` `match`. `result.ok()`↔`option.okOr(_)` must round-trip.
- **Representation/dispatch impact:** none. No new `Value` arm (`Ok`/`Err` are ordinary `InstanceObject`s;
  `Raise` already exists), no selector-encoding change, no new opcode. The only VM touch is an optional
  `pub(crate)` restore helper (plumbing).
- **Precedent:** Smalltalk `on:do:` / `ensure:` block protocol (the primitive, ADR-0008); the C-family
  `try`/`catch`/`ensure` keyword surface (ADR-0031). Rejected: Ruby `begin/rescue/ensure` and resumable
  conditions (`resume:`) — ADR-0008/0031 Alternatives; do not reopen.

## 3. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-ast/src/token.rs` | `throw` + `try` keyword tokens; `on`/`catch`/`ensure` contextual (identifier tokens, parser-recognised) | surface |
| `phalcom-ast/src/lexer.rs` | emit `throw`/`try` keywords; leave `on`/`catch`/`ensure` as identifiers | surface |
| `phalcom-ast/src/ast.rs` | `Throw { expr }`, `Try { protected, handlers: Vec<OnClause>, ensure: Option<Block> }` nodes (full rustdoc) | surface |
| `phalcom-ast/src/parser.rs` **(contended — see §3.1)** | parse `throw expr`; parse the `try` statement + contextual `on`/`catch`/`ensure` tail | surface |
| `phalcom-core/src/compiler/lib.rs` **(SPINE — reviewer ON)** | lower `throw` → `.raise()` + non-`Error`-literal compile error; lower `try` → the §2.4 nested re-wrapping desugar | compiler |
| `phalcom-core/src/primitive/block.rs` **(FROZEN FLOOR — reviewer ON)** | `block_on` (`on(_)(_)`) + `block_ensure` (`ensure(_)`) native primitives (§2.3) | floor |
| `phalcom-core/src/vm.rs` **(SPINE)** | *optional* `pub(crate) fn unwind_to(stack_len, frames_len)` restore helper (plumbing, no unwind-semantics change) | floor |
| `phalcom-core/src/universe.rs` | register `on`/`ensure` on `block_cls`+`function_cls`; **floor census bump (+2)** | floor |
| `phalcom-core/core/core.ph` **(never two editors — serialize)** | `class Result`/`Ok`/`Err` (pure `.ph`); `Option`/`Some`/`None` reopen for `okOr(_)`; `Block#attempt` `.ph` | protocol |
| `docs/adr/00NN-amend-floor-admit-block-handlers.md` (**new**, claim number at dispatch) | ADR-0019 amendment landing-record for `Block#on`/`ensure` (+2) | ADR |
| `docs/spec/current/core/floor-census.md` | +2 census rows, in lockstep with the ADR (same change) | ADR |
| `../../../../phalcom-core/src/primitive/option.rs` (**adopted debt**) | repoint the broken rustdoc intra-doc link at `nil.rs:~64` → private `wrap_some` (was orphaned) | docs |
| `docs/spec/current/core/README.md` (**adopted debt**) | re-baseline the stale "Baseline & drift policy" floor table to the post-U-CORE ceiling (was orphaned; ex-DEFERRED) | docs |
| `phalcom-core/tests/lang/errors/` (**new label**) + `tests/lang/MANIFEST.md` | goldens + negatives (§6) | all |
| `phalcom-core/tests/invariants.rs` | the catch-restore + isA-match invariants (§6) | all |

**Adopted debt (incidental docs fixes folded into this unit — both were orphaned; DEFERRED.md is now empty).**
- `primitive/nil.rs:~64` — a `cargo doc` intra-doc link points at the **private** `wrap_some`, emitting a
  warning that survives the green gate. Repoint or privatise it so `cargo doc --workspace --no-deps` is
  clean (this unit already gates on that, §5).
- `docs/spec/current/core/README.md` "Baseline & drift policy" — still states the pre-U-CORE-3 floor baseline
  (**80 / 64**); the U-CORE track closed at **88**. Re-baseline the table, the "Last floor-affecting
  commit" row, and the `80 + 5 + 1 + 2 = 88` ceiling prose while landing this unit's own `+2` census bump
  — same floor-accounting pass, in lockstep with ADR-0038. *(Migrated from the former DEFERRED.md U-CORE-3
  entry, which touched the same floor-accounting docs.)*

**Deliberately NOT in scope:** reifying the remaining native `RuntimeError` variants (`Arity`→`ArgumentError`,
`Type`→`TypeError`, `DeadFrameError`, `RangeError`, `ZeroDivision`) into surface classes — those stay
native (U-CORE-6 §0 reserved them to a *later* reification unit; they are already catchable-in-principle
once wrapped, but wrapping them is out of scope). No `retry`, no `?` propagation operator (result.md §5
non-goals). No `Future`/`async` (separate track). No third `Result` arm.

### 3.1 Write-set collision risk (flag, don't resolve)
- **`phalcom-ast/src/parser.rs`** — contended by the live U14/U15/U16/U-COLL cluster and U-ITER (all
  `phalcom-ast` editors). **Serialize** — U-ERR takes its own slot; cannot share a parallel wave with any
  `phalcom-ast` unit.
- **`phalcom-core/core/core.ph` — never two editors.** U-ITER (`List#iterate`), U-FIBER (`class Fiber`),
  and the U-CORE / U-STD tracks all edit `core.ph`. U-ERR's `Result`/`Ok`/`Err` + `Option` reopen + `Block#attempt`
  must **serialize** against every one of them. The parser + compiler + `block.rs` slices are free of this
  and can land first behind the `core.ph` slice.
- **`compiler/lib.rs`, `vm.rs`, `primitive/block.rs`** — spine / frozen-floor files; confirm no concurrent
  unit (U-ITER holds `compiler/lib.rs`; U-FIBER holds `vm.rs`) holds them before dispatch. **Worktree
  isolation** is mandated for exactly this reason. U-ERR's `vm.rs` touch is a *tiny additive helper*; if
  U-FIBER is mid-flight on `vm.rs`, sequence after it (or inline the restore in `block.rs` via existing
  `pub(crate)` methods to avoid the `vm.rs` edit — see DEC-ERR-D).

## 4. Build order (small, independently-green diffs)
1. **`throw`** — token + AST + parser + compiler lowering to `.raise()` + the non-`Error`-literal compile
   error. Green: `throw MyError.new("x")` renders/unwinds; `throw "oops"` is a compile error. *(Depends only
   on U-CORE-6's `raise`; touches `phalcom-ast` + `compiler/lib.rs`, not `core.ph`.)*
2. **The handling primitives** — `block_on` + `block_ensure` (native, §2.3) + the optional `vm.rs`
   `unwind_to` helper + `universe.rs` registration + the **ADR-0019 amendment + census bump**. Green via
   **direct sends** — `{ throw E.new("x") }.on(Error) { e => e.message }` and `{ ... }.ensure { ... }` — no
   `try` syntax needed yet. This is the highest-risk diff; land it with the invariant tests (§6). *(Serialize
   vs `vm.rs`/`block.rs` holders.)*
3. **The `try` statement** — parser tail (contextual `on`/`catch`/`ensure`) + compiler nested re-wrapping
   desugar (§2.4). Green: `try/on/catch/ensure` goldens equal the hand-written `.on()/.ensure()` chains from
   step 2. *(Touches `phalcom-ast` + `compiler/lib.rs`.)*
4. **`Result`/`Ok`/`Err` + bridges** — pure `.ph` (§2.5): `class Result`/`Ok`/`Err`, the `Option` reopen for
   `okOr(_)`, and `Block#attempt` over `on`. Green: `map`/`mapErr`/`andThen`/`unwrap`(re-throw)/`ok`/`okOr`
   round-trips + `{ ... }.attempt()` → `Ok`/`Err`. *(Serialize vs every `core.ph` editor; depends on step 2's
   `on` for `attempt` and U-CORE-6's `raise` for `unwrap`.)*

Each step is a self-verifiable commit. Steps 1 and (2-native) can proceed in parallel with step 4's `.ph`
only if their write-sets are confirmed disjoint on HEAD (they are: `phalcom-ast`/`block.rs` vs `core.ph`),
but step 4's `attempt` needs step 2 landed first.

## 5. Mandatory rules
- **Docs:** `///` on every new AST node/field, parser/compiler fn, and the two primitives, citing
  ADR-0008/0031 + error-handling.md §. `cargo doc --workspace --no-deps` adds no warnings.
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; **no `unsafe`** (the restore helper is safe
  `Vec::truncate` + `close_upvalues_from`). Follow `rust-best-practices`.
- **Reviewer ON** (spine + frozen-floor) — `phalcom-reviewer` gates the diff; the writer never self-approves.
- **Floor discipline:** the +2 amendment ADR + `floor-census.md` bump land **in the same change** as the two
  primitives (mirror ADR-0028's pattern). Do not add any primitive beyond `on`/`ensure`.

## 6. Test strategy (the green gate must assert) — new `errors` corpus label
- **`throw` (PASS + NEGATIVE):** `throw MyError.new("boom")` uncaught → the rendered message + trace (exit 70,
  byte-stable). **NEGATIVE:** `throw "oops"` / `throw 42` → compile error with a clear span.
- **`on` typed + catch-all (PASS):** `try { throw A.new() } on A e { "gotA" } catch e { "other" }` → `"gotA"`;
  a `B` throw with the same clauses → `"other"`; **first-match-wins**: `on A` before `on B`, throw `B` →
  `B`'s handler; throw `C` (neither) → uncaught, full trace intact.
- **`ensure` fires on all three exits (PASS — the §4.1 rule):** one fixture proving the `ensure` block runs on
  (a) normal completion, (b) a non-local `return` through the protected block, (c) a `throw` — asserting
  observable side-effect order in each. Plus **cleanup-supersedes**: a `throw` inside `ensure` overrides the
  pending unwind.
- **catch-restore (INVARIANT, `invariants.rs`):** after `{ deeplyNested(); throw E.new() }.on(Error){ e =>
  alloc-and-return }`, assert `vm.frames.len()` / `vm.stack.len()` are back to the pre-`on` snapshot and the
  handler's freshly-allocated value survives (upvalue-close correctness). Assert the isA-match walks the
  superclass chain (`on(SuperError)` catches a `SubError` throw).
- **`Result` combinators (PASS):** `Ok(2).map { n => n * 2 }` → `Ok(4)`; `Err(e).map { … }` → `Err(e)`
  unchanged; `mapErr` symmetric; `andThen` chains `Ok`→`Result` and short-circuits on `Err`; `isOk`/`isErr`;
  `unwrapOr`; `toString`.
- **`unwrap` re-throw + bridges (PASS):** `Ok(v).unwrap` → `v`; `Err(myError).unwrap` re-`throw`s (catchable
  by an enclosing `on(Error)`); `Ok(v).ok()` → `Some(v)`, `Err(e).ok()` → `None`; `Some(v).okOr(err)` →
  `Ok(v)`, `None.okOr(err)` → `Err(err)` — round-trip `result.ok()`↔`option.okOr(_)`.
- **`attempt` bridge (PASS):** `{ 21 * 2 }.attempt()` → `Ok(42)`; `{ throw E.new("x") }.attempt()` →
  `Err(<E instance>)`; `.attempt().map { … }.unwrapOr(0)` composes (error-handling.md §5 example).
- **Pending fixtures graduated (if present):** flip `errors/errors_throw_try_catch_finally` and
  `errors/errors_result_bridge` (U-CORE-6 §4 named them) from `pending/` to PASS; re-bless expected output
  from the built binary. **Confirm their current paths/names on HEAD** before moving.

## 7. Decisions flagged (flag, don't pick)
| ID | Decision | Options | Architect recommendation |
|---|---|---|---|
| **DEC-ERR-A** ⚠ **ADR DRAFTED (Proposed)** | **The ADR-0019 amendment for `Block#on`/`ensure` (+2).** Not pre-cleared by ADR-0023. | Recommendation was (A) a fresh per-unit amendment ADR. | **Drafted as [ADR-0038](../../../adr/0038-amend-floor-admit-block-on-ensure.md) (Proposed, +2, census 80→82)**, citing ADR-0008 as design authority. **Awaiting user ratification** — the one gate that must clear before the step-2 diff merges. |
| **DEC-ERR-B** | **`Ok(v)`/`Err(v)` construction sugar.** Does the `Name(args)` construction form generalise beyond `Some`? | **(A)** generalise the `Some(x)` construction desugar to any class with a matching `construct`; **(B)** ship `Ok.new(v)`/`Err.new(v)` now, add `Ok(v)` sugar as a follow-on. | **Confirm on HEAD** how `Some(x)` lowers. Recommend **(A)** if it is a small generalisation (cleaner surface, matches `Some`), else **(B)** to keep this unit's write-set small and defer the sugar. Do not block on it — `Ok.new(v)` is always available. |
| **DEC-ERR-C** | **`on` selector encoding + the `blk.on(T){e=>}` surface form.** §13/U-CORE-6 name `on(_)(_)`; does the parser accept a **trailing block after a parenthesized arg**? | **(A)** register `on` as `SignatureKind::Method(2)` (positional class + handler) and confirm trailing-block-after-args parses; **(B)** if it doesn't parse, add minimal grammar support, or accept the all-in-parens form `on(T, { e => })`. | **(A)** register `Method(2)`. The **`try` lowering emits the 2-arg `Invoke` directly**, so it does **not** depend on the surface trailing-block grammar; making the bare `blk.on(T){e=>}` form parse is a secondary nicety — verify on HEAD (compare `ifTrue({},ifFalse:{})` vs `whileTrue{}`), flag if a small grammar add is needed. |
| **DEC-ERR-D** | **The `vm.rs` restore helper vs inline in `block.rs`.** | **(A)** add `pub(crate) fn unwind_to` to `vm.rs`; **(B)** inline via existing `pub(crate)` VM methods from `block.rs` to avoid the `vm.rs` edit while U-FIBER holds it. | **(A)** for locality beside the other unwind handlers, **unless** U-FIBER is mid-flight on `vm.rs` at dispatch — then **(B)** to keep write-sets disjoint. Either way it is plumbing, not a floor binding. |

## 8. Must-not-preclude check ([error-handling.md](../../../spec/current/error-handling.md), [forward-compat.md](../../../spec/current/core/forward-compat.md) §2/§7)
- **Fiber error capture (concurrency.md §1, forward-compat §7 D7):** actively *served*, not precluded — the
  snapshot/restore is len-relative, so an inner `on`/`ensure` stays fiber-local and a `throw` not caught by
  any inner handler reaches the fiber floor and is captured into the fiber's result slot (U-FIBER Phase 0
  D7). Never hardcode the main stack or `base_frames == 0`.
- **`ensure` on any unwind:** designed for normal / `Raise` / non-local `return` today; a future fiber
  `abort` payload is just another `Err` arm the `ensure` "run cleanup then re-propagate" path already covers
  — additive, no rework.
- **`Result` mirrors `Option` (forward-compat §2):** pure `.ph`, its own `match`; a future migration of
  `Option` from native to `.ph` (now that `construct` exists) is symmetric and does not touch `Result`.
- **`retry` (error-handling.md §3, deferred):** not precluded — the protected block stays live; `on` holds
  it as the receiver `Value` and never frees it, so a future `retry` re-runs it. Do not consume/mutate the
  block.
- **`?` auto-propagation (result.md §5, deferred):** not precluded — no syntax added that conflicts;
  `andThen`/`unwrap` cover the cases explicitly for now.
- **Reifying native `RuntimeError` variants to surface classes (later unit):** not precluded — each already
  flows through the same `PhResult`/`?` channel; wrapping `Arity`/`Type`/`RangeError` into surface `< Error`
  classes later only *adds* catchable types, and `on(Error)` (catch-all) already catches the `Raise`-carried
  ones today.

## 9. Return contract (report to `phalcom-reviewer`)
The `throw` token/AST/parser/lowering + the non-`Error`-literal compile error · the exact `block_on`/
`block_ensure` bodies with the **snapshot → (close-upvalues, double-truncate) → dispatch** restore and the
non-local-return (`Ok` + shrunk frames) vs `Raise` (`Err`) asymmetry handled · confirmation **first-match-wins
re-propagates non-matches verbatim** and **`ensure` fires on all three exits + cleanup-supersedes** · the
`try` **nested re-wrapping** desugar (not a flat chain) + contextual-keyword handling that keeps
`.on()`/`.ensure()`/`Fiber>>try` working · the pure-`.ph` `Result`/`Ok`/`Err` + `okOr` reopen + `attempt`
over `on` + `unwrap` re-throw · confirmation **net floor delta = +2** and the **new ADR-0019 amendment ADR +
census bump** landed in lockstep (DEC-ERR-A) · how DEC-ERR-B/C/D resolved · the `errors` corpus label +
MANIFEST bump + any graduated `pending/` fixtures · the catch-restore + isA invariants · worktree /
serialization notes vs U-ITER, U-FIBER, and the `phalcom-ast`/`core.ph` clusters · `verify.sh` + `cargo doc`
tails.
