# U9 — Work order: variadics (rest parameters + variadic dispatch)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Wave F+1 unit,
sequenced **after U8**. **Reviewer OFF** — no independent `phalcom-reviewer` gate; you self-verify on
the green gate (`./scripts/verify.sh` exits 0) + `cargo doc` clean. Grounded in **ADR-0012**
(selector arity encoding; the `Signature.variadic` flag was reserved here) and **ADR-0006**
(`Function` as abstract callable root), plus specs [`functions.md`](../spec/functions.md) §1–4 and
[`messages-and-selectors.md`](../spec/messages-and-selectors.md) §4. STATE.md ADR mapping is
authoritative._

---

## 0. Mission (one sentence)
Let a method or block declare a **rest parameter** (`sum(*numbers)`) that collects trailing positional
arguments into a `List`, encode it as the variadic selector `sum(_...)`, and make dispatch resolve a
variadic call via the **variadic-table probe** that sits *between* U3's exact selector probe and U8's
`doesNotUnderstand` fallback — all without changing selector identity for fixed-arity methods.

## 1. Hard guardrails (read before writing any code)
- **Runs on the post-U1 + post-U8 substrate.** Assume `Heap`/`Copy` handles + tagged `Value`
  (ADR-0009/0010), U4 blocks/closures (ADR-0013), U3 selectors, **and U8's dNU miss path already
  installed** (the terminal fallback + the `SendDynamic` primitive). If any of U1/U4/U8 has not merged
  when you start, **STOP and report** — U9 is dependency-blocked.
- **Insert, don't rewrite, the miss path.** U8 left a commented **variadic seam** immediately before
  its `doesNotUnderstand(_:)` forward. Your variadic-table probe slots into that seam (method-lookup §1:
  exact probe → **variadic probe** → dNU). Do NOT move the dNU fallback or the exact probe; do NOT
  touch the IC slot semantics.
- **One encoder.** Add the variadic branch to `encode_selector` in `signature.rs` (the single source of
  truth, ADR-0012 / F8). A variadic method interns as `sum(_...)`; do not hand-format this anywhere else.
- **Variadics are positional-only** (messages-and-selectors §4): a labelled parameter cannot be
  variadic, and the rest parameter must be **last**. Enforce both at compile time with a clean
  diagnostic — do not silently accept `foo(to:, *rest)` or `foo(*rest, x)`.
- **No `**kwargs`.** Do not add keyword-rest; labels are selector identity. Out of scope.
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md).

## 2. Preconditions (verify first; do not assume)
- **U1, U4, U8 merged.** Re-run `graphify affected "encode_selector"`, `graphify affected "Signature"`,
  and `graphify affected "call_method"` on the actual HEAD to confirm the seam and structs are where §4
  expects.
- **`Signature { selector, kind, positional_arity, variadic }` already exists** (`signature.rs`) — U3
  reserved `variadic: bool` and `new_with_arity(...)`. `encode_selector` currently has **no** variadic
  branch; that is core U9 work, not cleanup.
- **`List` type exists** (rest params collect into a `List`; example `numbers.reduce(0){…}`). Today no
  `List` class exists (only the reserved name in `primitive/mod.rs`). **Hard shared precondition** —
  same blocker as U8 #1; U-STD must have landed `List` (with at least construction + iteration; `reduce`
  for the spec example). See BLOCKED-ON-DECISION #1.
- **AST/parser lack rest syntax.** `ParameterDef { name, label }` has no rest marker; the
  recursive-descent parser (ADR-0016, hand-written) does not parse a `*name` parameter. You extend both.
- Confirm `./scripts/verify.sh` is green on your worktree base before the first edit.
- Runs in its **own isolated worktree off `main`** (`git worktree add ../phalcom.worktrees/u9 feat/u9`).

## 3. Confirmed write-set (validate with `graphify affected` on post-U8 HEAD before editing)
| File | Why it's in scope |
|---|---|
| `phalcom-ast/src/ast.rs` | Add a rest marker to `ParameterDef` (e.g. `is_rest: bool`). |
| `phalcom-ast/src/parser.rs` | Parse a leading `*` on the final parameter → rest param; reject non-last / labelled rest. |
| `phalcom-ast/src/lexer.rs` | Only if `*` is not already tokenizable in parameter position (it exists as the multiply operator — likely no change; confirm). |
| `phalcom-core/src/signature.rs` | Variadic branch in `encode_selector` (`sum(_...)`); `min_positional_arity` accounting on `Signature`. |
| `phalcom-core/src/method.rs` | Thread the variadic flag through `MethodObject`/constructors (field already exists on `Signature`). |
| `phalcom-core/src/callable.rs` | Mark the compiled `Callable` variadic (so the VM prologue reshapes the stack); distinguish declared fixed arity from rest slot. |
| `phalcom-core/src/closure.rs` | Blocks may be variadic too (functions.md §1–2, `Function` protocol) — carry the flag if a block can declare `*rest`. |
| `phalcom-core/src/class.rs` | Per-class **variadic table** built at class-definition time (method-lookup §1 step 3), keyed per BLOCKED-ON-DECISION #2. |
| `phalcom-core/src/compiler/lib.rs` | Emit variadic signature; register the method in the variadic table; compile the rest-param slot; enforce last-position + no-label rules. |
| `phalcom-core/src/vm.rs` | Fill U8's variadic seam with the variadic-table probe; add the **call prologue** that collects trailing stack args into a `List` bound to the rest slot before the frame runs. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | Variadic acceptance corpus (§7). |

**Disjointness note:** U9 shares `vm.rs`, `signature.rs`, `compiler/lib.rs`, and `method.rs` with U8 —
**not parallelizable with U8**; it is correctly sequenced to Wave F+1 after U8 merges. It may run in
parallel with **U11 (Bool tower)** only if their write-sets are confirmed disjoint at schedule time.

## 4. Design decisions (ADR-0012 / ADR-0006 + specs — realize, don't re-litigate)
- **Selector encoding (messages-and-selectors §4).** A variadic declaration `sum(*numbers)` interns as
  `sum(_...)`. A *call* `sum(1, 2, 3)` interns (as usual, via the compiler) as `sum(_:_:_:)` — it will
  **not** match `sum(_...)` on the exact probe, by design. Add exactly one variadic arm to
  `encode_selector`; the `_...` marker denotes "zero-or-more trailing positional". A variadic with a
  fixed prefix (`format(fmt, *args)`) encodes the fixed labels then `_...` (e.g. `format(_:_...)` or the
  encoder's canonical form — fix and document the exact spelling).
- **Variadic table (method-lookup §1 step 3).** Built once per class at definition time. On an exact
  probe miss, probe the variadic table by the **bare name** and check the call arity against the
  method's minimum positional arity; on hit, resolve + (later) cache in the IC. Step 2 of the spec says
  this "never runs on a warm call site" — so it lives strictly in the slow path, in U8's seam.
- **Call prologue / stack reshape.** `call_method` receives the *call-site* arity from
  `Invoke(arity, …)`; for a variadic target this exceeds (or equals) the declared fixed arity. Before
  the closure frame runs, the VM must pop the `call_arity - fixed_arity` trailing values, build a
  `List`, and place it in the rest slot so the body sees exactly `fixed_arity + 1` locals. This is a
  VM-side prologue keyed off the `Callable` variadic flag — **the single subtle correctness point of
  this unit** (stack-offset math must match `CallFrame::new`'s `stack_offset`).
- **`Function` protocol (ADR-0006).** `arity` on a variadic reports the declared *fixed* count; the rest
  parameter is not counted in `arity`. `callWith(_:List)` (functions.md §1) naturally applies a List to
  any callable and is the reflective analogue — but `callWith` is Function-surface owned elsewhere; U9
  only guarantees its variadic-binding semantics are consistent (a `List` of N spreads to N positional
  slots). Do not implement `callWith` here unless it already exists; note the interaction.
- **Blocks (functions.md §2).** If the surface grammar allows `{ *xs => … }`, a block Callable carries
  the same variadic flag and prologue. If block variadics are out of the current grammar scope, restrict
  U9 to method rest params and record the block case in DEFERRED — **decide explicitly** (see #3 below).

### BLOCKED-ON-DECISION
1. **`List` availability.** Identical hard dependency to U8 #1: rest params collect into a `List`, which
   does not exist yet. **Recommendation:** `List` must land (U-STD) before U9; it is on the critical
   path for both Wave-F+ units. Confirm `List` (with construction + iteration + `reduce` for the spec
   example) is merged before U9 starts. **Needs orchestrator confirmation.**
2. **Variadic-table key + min-arity probe shape (genuinely underspecified).**
   messages-and-selectors §4 says the table is "keyed by `(name, min_positional_arity)`", but a call
   site with arity K must find a variadic entry whose `min_positional_arity <= K` — an exact tuple hash
   key cannot answer a `<=` query. **Options:** (a) key the table by **bare name → variadic method**
   (at most one variadic per name per class) and check `K >= min` at probe time — simplest, matches the
   "few variadics per name" reality; (b) key by name → a small sorted list of variadic overloads and
   pick the greatest `min <= K`; (c) literal `(name, min)` hash + descend `K, K-1, …, 0` probing. **
   Recommendation: (a)** — one variadic overload per name per class, `check K >= min`; reject a second
   variadic of the same name at class-definition time with a diagnostic. This needs either an
   **ADR-0012 amendment** or an explicit spec clarification of §4 — **flag to the orchestrator; do not
   silently pick.**
3. **Block variadics in scope?** functions.md §2 gives blocks the full `Function` protocol, which
   *could* include `*rest`. **Recommendation:** if the hand-written parser already has (or trivially
   extends to) `{ *xs => … }`, include it — the prologue is shared; otherwise restrict U9 to method rest
   params and DEFER block variadics. Confirm scope with the orchestrator.

**New ADR needed?** Likely a **short ADR-0012 amendment** (or a dedicated micro-ADR) to pin the
variadic-table key + `min <= K` probe semantics (BLOCKED #2) and the canonical `_...` selector spelling,
since messages-and-selectors §4's phrasing is not implementable as literally written. Flag for the
`documentation-and-adrs` skill. No new *decision* is otherwise required — encoding, positional-only,
last-position, and no-`**kwargs` are all already ratified.

## 5. Build order (land as one coherent, self-verifiable diff)
1. **`ast.rs`** — `ParameterDef.is_rest`. Full rustdoc.
2. **`parser.rs`** (+ `lexer.rs` only if needed) — parse `*name` as the final param; syntactic errors
   for non-last and labelled rest.
3. **`signature.rs`** — variadic branch in `encode_selector`; `min_positional_arity` on `Signature`.
4. **`method.rs` / `callable.rs` / `closure.rs`** — thread the variadic flag from signature to the
   compiled `Callable`.
5. **`compiler/lib.rs`** — emit the variadic signature; enforce last-position + no-label; register the
   method into the class variadic table; allocate the rest slot.
6. **`class.rs`** — variadic-table storage + build-at-definition; the probe accessor.
7. **`vm.rs`** — fill U8's variadic seam with the probe; add the call prologue (collect trailing args →
   `List` → rest slot) gated on the `Callable` variadic flag.
8. **`tests/lang.rs`** — variadic corpus.

## 6. Fold-in cleanup (only within the write-set)
- `signature.rs::make_signature` carries a Setter-arity workaround comment (line ~130) — if you touch
  the arity accounting there for min-positional, tidy it; otherwise leave it (out of scope).
- No other fold-ins; keep the diff focused.

## 7. Test strategy (the green gate must assert)
- **Zero-prefix variadic:** `sum(*numbers)` with `sum(1,2,3)` → `6`; `sum()` → `0` (empty `List`).
- **Fixed-prefix variadic:** `format(fmt, *args)` with 1 and with 3 trailing args — prologue math with a
  non-zero fixed arity (the subtle case).
- **Coexistence:** a class defining both `sum(_:_:)` (fixed) and `sum(*)` (variadic) — the exact probe
  wins for arity 2, the variadic table for other arities. Asserts the two selectors stay distinct.
- **Dispatch ordering:** an unknown variadic-shaped call that has no variadic entry falls through to
  **U8's dNU** (not a hard error) — proves the probe sits before the fallback and doesn't swallow it.
- **Compile-time rejections:** `foo(*rest, x)` (rest not last) and `foo(to:, *rest)` (labelled+variadic)
  each produce a clean parser/compiler diagnostic, not a panic.
- **Rest binds a real `List`:** inside the body, `numbers.reduce(0){acc,n => acc+n}` works — proves the
  collected value is a first-class `List`, not a Rust `Vec` leak.
- **Fuzz (opt-in):** random arities against a variadic method never desync the stack (assert stack depth
  invariant after return) — guards the prologue's `stack_offset` math.

## 8. Mandatory rules
- **Docs:** `///` on every new/changed public item (`is_rest`, the variadic `encode_selector` arm,
  `min_positional_arity`, the variadic-table type + accessors, the prologue helper) with intra-doc links
  and ADR-0012/0006 citations; `//!` updated on touched modules. `cargo doc --workspace --no-deps` adds
  **no new warnings**.
- **Green gate = review:** `./scripts/verify.sh` exits 0. Reviewer is OFF for U9 — **the green gate +
  `cargo doc` clean is your sole sign-off.** Don't add clippy warnings; fix pre-existing ones in files
  you rewrite.
- **Selector discipline:** the `_...` encoding goes through `encode_selector` only (ADR-0012 / F8).
- `rust-best-practices` skill; the stack-reshape prologue is the fragile spot — assert invariants in
  tests rather than trusting the offset math by inspection. No `unsafe` expected; if any lands, add a
  `// SAFETY:` note and run `rust-sanitizers-miri`.

## 9. Return contract (self-report; no reviewer)
Report: the variadic-table key/probe shape actually implemented (and the BLOCKED #2 resolution) · the
canonical `_...` selector spelling chosen · the prologue's stack-offset reasoning + which test guards it ·
the resolution of BLOCKED #1 (`List`) and #3 (block variadics) · confirmation the exact probe and U8's
dNU fallback are unchanged (quote the seam you filled) · files changed · `verify.sh` tail + `cargo doc`
tail · any new `DEFERRED.md` entries (block variadics if deferred; `callWith` interaction; spread call
sites `f(*args)` if not owned here).
