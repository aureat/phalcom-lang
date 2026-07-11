# U8 — Work order: `doesNotUnderstand(_:)` / `perform` + `SEND_DYNAMIC`

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Wave F unit.
**Reviewer OFF** — no independent `phalcom-reviewer` gate; you self-verify on the green gate
(`./scripts/verify.sh` exits 0) + `cargo doc` clean. Grounded in **ADR-0012**
(label-encoded selectors + IC-ready dispatch; dNU is the lookup-miss fallback) and the specs
[`method-lookup.md`](../spec/method-lookup.md) §1–3 + [`messages-and-selectors.md`](../spec/messages-and-selectors.md) §5.
STATE.md ADR mapping is authoritative._

---

## 0. Mission (one sentence)
Turn a lookup miss from a hard VM error into a **reified `Message` re-sent as `doesNotUnderstand(_:)`**
up the class chain, and add the shared **`SEND_DYNAMIC`** runtime-send primitive that powers both
`Object.perform(_:…)` (reflective send with a runtime-chosen selector) and `doesNotUnderstand`
forwarding — without corrupting U3's IC-ready dispatch shape.

## 1. Hard guardrails (read before writing any code)
- **This runs on the post-U1 substrate.** Assume the `Heap` + `Copy` handles (`ObjRef`/`ClassId`)
  and the tagged `Value` enum (ADR-0009/0010) have landed; native methods take `&Heap`/`&mut Heap`,
  not `self.borrow()`. Do NOT reintroduce `Rc<RefCell>`/`PhRef`. If U1 has not merged when you start,
  **STOP and report** — this unit is dependency-blocked, not a rewrite of U1.
- **Do NOT touch the variadic path.** The variadic-table probe (method-lookup §1 step 3) belongs to
  **U9**, which lands *after* you (Wave F+1) and inserts its probe *between* the exact probe and the
  dNU fallback you install. Leave a clearly-commented seam at the miss site so U9 slots in without a
  rewrite; do not implement the variadic table.
- **Do NOT change selector encoding.** `encode_selector(name, labels, kind)` in `signature.rs` is the
  single source of truth (ADR-0012, closes F8). Reflective/forwarding selector construction MUST route
  through it — never hand-format a selector string in `vm.rs` or a primitive.
- **Do NOT populate the inline cache.** IC population is deferred (ADR-0012 Consequences). Your dNU
  slow path must *not* corrupt or bypass the monomorphic IC slot: a miss is a slow path that runs the
  chain walk, then forwards; if you add per-class dNU-handler caching (spec §2), key it on the stable
  `ClassId` and keep it separate from the call-site IC slot.
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md). Do not self-approve beyond the green gate.

## 2. Preconditions (verify first; do not assume)
- **U1 merged** (Heap + tagged `Value`). Re-run `graphify affected "lookup_method"` and
  `graphify affected "call_method"` on the actual HEAD to confirm the miss site is where §4 expects.
- **U3 present** (label-encoded selectors, `Invoke(arity, selector_idx)` → `lookup_method` → miss →
  `RuntimeError::MethodNotFound`). This `MethodNotFound` arm in `vm.rs` (currently ~`vm.rs:507`) is the
  exact hook you replace.
- **`List` type exists.** `Message.args`, `msg.labels`, and `perform(selector, args)` all traffic in a
  `List`. Today **no `List` class exists** — `primitive/mod.rs` reserves the *name string* `"List"`
  but core.ph defines only Object/Class/Metaclass/Number/String/Bool/Nil/Symbol/System. This is a
  **hard shared precondition** (see BLOCKED-ON-DECISION #1). Confirm a `List` primitive is available
  (U-STD) before writing the Message/perform surface, or scope a minimal internal list per #1.
- Confirm `./scripts/verify.sh` is green on your worktree base before the first edit (baseline).
- Runs in its **own isolated worktree off `main`** (`git worktree add ../phalcom.worktrees/u8 feat/u8`);
  it is a Wave-F fan-out unit. Do not share a tree with U9/U-STD.

## 3. Confirmed write-set (validate with `graphify affected` on post-U1 HEAD before editing)
| File | Why it's in scope |
|---|---|
| `phalcom-core/src/vm.rs` | Replace the `MethodNotFound` miss arm with the dNU forward; add the `SEND_DYNAMIC` opcode handler + a `send_dynamic(receiver, selector, args)` helper reused by `perform` and dNU forwarding. Leave the U9 variadic-probe seam. |
| `phalcom-core/src/bytecode.rs` | New `SendDynamic` opcode (selector + args built at runtime). |
| `phalcom-core/src/message.rs` **(NEW)** *or* `core/core.ph` | The reified `Message` object (`selector`, `name`, `labels`, `args`) — see design decision §4. Prefer a kernel `class Message` in `core.ph` + a primitive constructor over a bespoke Rust struct, per ADR-0011 static slots. |
| `phalcom-core/src/primitive/object.rs` | `perform(_:_:)` / `perform(_:)` reflective sends; default `doesNotUnderstand(_:)`; `respondsTo(_:)`; `Message` accessors if primitive-backed. |
| `phalcom-core/core/core.ph` | `class Message { … }`; `Object.doesNotUnderstand(_:)` default = raise `MessageNotUnderstood` (spec method-lookup §2). Decide primitive-vs-`.ph` per §4. |
| `phalcom-core/src/error.rs` | `RuntimeError::MessageNotUnderstood { selector, receiver }` (the default-dNU raise target). Keep/repurpose `MethodNotFound` only if still reachable; otherwise fold it out. |
| `phalcom-core/src/compiler/lib.rs` | Emit `SendDynamic` for `perform`-style runtime sends if surfaced syntactically; otherwise perform is primitive-only and this file is untouched (keep the write-set tight — include only if needed). |
| `phalcom-core/src/universe.rs` | Register `Message` + `MessageNotUnderstood` kernel wiring if Rust-side registration is required. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | Acceptance: proxy/dNU + perform corpus (see §5 test strategy). |

**Disjointness note for the orchestrator:** U8 and U9 **share `vm.rs`, `signature.rs` reuse,
`compiler/lib.rs`, and `core.ph`** — they are **NOT** parallelizable. U8 (Wave F) lands first and
installs dNU as the terminal miss fallback; U9 (Wave F+1) inserts the variadic probe *ahead of* that
fallback. Sequence, do not fan out together.

## 4. Design decisions (ADR-0012 + specs — realize, don't re-litigate)
- **Miss path (method-lookup §1).** Order is: IC → exact selector probe (walk superchain) →
  `[U9 variadic-table probe — leave seam]` → `doesNotUnderstand(_:)`. On exact-probe miss, synthesize a
  `Message` and perform an **exact** lookup of `doesNotUnderstand(_:)` on the receiver's class chain
  (guaranteed to hit `Object`'s default). Never recurse infinitely: if `doesNotUnderstand(_:)` itself
  is missing, that is an internal invariant violation, not another dNU.
- **`Message` reification (method-lookup §2).** Fields: `selector` (interned `Symbol`), `name`
  (`String`, the bare name), `labels` (`List` of `String`), `args` (`List`). **Recommend** a kernel
  `class Message` defined in `core.ph` whose instances are ordinary `InstanceObject`s (ADR-0011 static
  slot layout) built by a primitive constructor in `object.rs` — this keeps reflection uniform and
  avoids a bespoke Rust type. Selector→name/labels decomposition MUST reuse the `signature.rs`
  encoder's inverse, not a second parser.
- **`SEND_DYNAMIC` (messages-and-selectors §5).** One runtime primitive: build the selector `Symbol`
  at runtime from `(name, labels, arity)` via `encode_selector`, then run the *normal* lookup+dispatch
  (same `call_method` path, so IC/dNU/variadic all apply). Expose it as (a) an opcode `SendDynamic` for
  any spread/reflective call site, and (b) a `vm.send_dynamic(...)` Rust helper. **Build it once, use it
  three ways** (perform, dNU forwarding, and — later — U9/spread). `Object.perform(selector, args)` is
  a thin primitive over `send_dynamic`; `perform(_:)` is the zero-arg case.
- **Default `doesNotUnderstand(_:)` (method-lookup §2).** `Object.doesNotUnderstand(_:)` raises
  `MessageNotUnderstood` (surfacing selector + receiver). This replaces today's `MethodNotFound` hard
  error as the *observable* miss behavior; the error now originates from a real send a subclass can
  override. **Recommend** defining the default as a **primitive** (so it can format the raise cleanly
  and can't be accidentally shadowed by an incomplete core.ph load), with the `class Message` shell in
  core.ph.
- **Handler caching (method-lookup §2, optional).** Spec permits caching the resolved dNU handler per
  receiver class so proxy-heavy code doesn't re-walk. If you add it, key on stable `ClassId`
  (ADR-0009), keep it **separate** from the call-site IC, and invalidate on class-hierarchy mutation
  (open-Q4 unresolved → conservative: no cache, or a cache you can drop wholesale). **Recommend
  deferring the cache** (mark in DEFERRED) — correctness first; the miss path is already slow-by-design.

### BLOCKED-ON-DECISION
1. **`List` availability / ownership.** `Message.args`, `msg.labels`, and `perform(_:List)` require a
   real `List` type; none exists today (only the reserved name in `primitive/mod.rs`). U-STD owns
   `core.ph` and is scheduled *in the same Wave F* as U8 — a scheduling collision on a hard dependency.
   **Options:** (a) sequence U-STD's `List` *before* U8 within Wave F; (b) U8 ships a **minimal internal
   list value** (build/index/iterate only) that U-STD later supersedes; (c) temporarily model `args`
   as a Rust `Vec<Value>` behind a primitive accessor until `List` lands. **Recommendation: (a)** —
   land `List` first; it is a prerequisite for U9 too, so it is on the critical path regardless. If
   scheduling forbids (a), fall back to (b) with a `DEFERRED` entry to migrate. **Needs the orchestrator
   to confirm before U8 starts.**
2. **`perform` surface syntax vs primitive-only.** Is `perform`/`perform:with:` reflective *only*
   (a primitive on `Object`, no new syntax), or does it also need a spread call-site lowering
   (`f(*args)`) in the compiler? messages-and-selectors §5 couples `SEND_DYNAMIC` to spread, but spread
   call sites are arguably U9/a follow-on. **Recommendation:** U8 delivers `SendDynamic` + primitive
   `perform` only (no `*args` call-site syntax); spread-at-call-site is deferred to the unit that owns
   the parser change, reusing this opcode. Confirm scope.

**New ADR needed?** No new *decision* ADR — dNU/perform/`SEND_DYNAMIC` are covered by ADR-0012 +
method-lookup §2–3 + messages-and-selectors §5. **Propose a short amendment note on ADR-0008**
(layered exceptions) recording that `MessageNotUnderstood` is the default-dNU raise, if that error
class is not already enumerated there — flag for the `documentation-and-adrs` skill.

## 5. Build order (land as one coherent, self-verifiable diff)
1. **`error.rs`** — add `MessageNotUnderstood { selector, receiver }`. Full rustdoc.
2. **`bytecode.rs`** — add `SendDynamic` opcode + `///` doc + disasm arm (`bin/phalcom/disasm.rs` if the
   disassembler enumerates opcodes — check `graphify affected "Bytecode"`; add to write-set if so).
3. **`vm.rs`** — `send_dynamic(receiver, selector, args)` helper (build selector via `encode_selector`,
   run normal lookup+`call_method`); `SendDynamic` handler over it; replace the `MethodNotFound` miss
   arm with: synthesize `Message` → forward `doesNotUnderstand(_:)`. **Leave the U9 variadic seam**
   (a comment + the ordering hook) immediately before the dNU forward.
4. **`Message`** — `class Message` in `core.ph` + primitive constructor/accessors in `object.rs`
   (or `message.rs` if a helper module is cleaner). Decompose selector via the encoder's inverse.
5. **`object.rs` + `core.ph`** — `Object.doesNotUnderstand(_:)` default (primitive, raises
   `MessageNotUnderstood`); `Object.perform(_:_:)` / `perform(_:)`; `Object.respondsTo(_:)`
   (a pure exact-probe, no send). Register in `universe.rs` as needed.
6. **`tests/lang.rs`** — acceptance corpus (see below).

## 6. Fold-in cleanup (only within the write-set)
- In `vm.rs`, remove the dead commented-out `match result { … }` block in `call_method` (~lines 185–193)
  superseded by the `.map(...)` form — you are already rewriting adjacent dispatch code. Confirm with
  `graphify affected "call_method"` that nothing references it.
- If `RuntimeError::MethodNotFound` becomes unreachable after the dNU rewrite, remove it (and its
  `Display`) rather than leaving a dead variant; if still reached elsewhere, leave it and note why.

## 7. Test strategy (the green gate must assert)
- **Golden/`lang.rs`:** a `Proxy` class overriding `doesNotUnderstand(_:)` that forwards via
  `perform` to a target — assert the forwarded result (the spec's headline motivation).
- **Default dNU:** sending an unknown selector to a plain object surfaces `MessageNotUnderstood`
  with the correct selector + receiver rendering (was previously a hard `MethodNotFound`) — a
  behavior-change golden; update any existing golden that asserted the old text.
- **`Message` shape:** inside a dNU override, assert `msg.selector` / `msg.name` / `msg.labels` /
  `msg.args` for a labelled call (`x.move(to: a, duration: b)`) — verifies encoder-inverse correctness.
- **`perform`:** `3.perform(#"+(_:)", [4])` → `7` (or the project's Symbol-literal form) — reflective
  send parity with a static send; and `perform` of an unknown selector re-enters dNU (no infinite loop).
- **`respondsTo`:** true for a defined selector, false for an unknown one, without triggering dNU.
- **IC non-corruption:** a warm call site that then misses (polymorphic receiver) still dispatches
  correctly on subsequent hits — guards against the dNU slow path trampling the IC slot.
- **Fuzz (opt-in):** random unknown selectors never panic/UB; always route to dNU.

## 8. Mandatory rules
- **Docs:** `//!` on any new module (`message.rs` if created); `///` on every new public item
  (`SendDynamic`, `send_dynamic`, `Message`, `MessageNotUnderstood`, every new primitive) with
  `# Panics`/`# Errors` where applicable, intra-doc links, and ADR-0012 / method-lookup citations.
  `cargo doc --workspace --no-deps` adds **no new warnings**.
- **Green gate = review:** `./scripts/verify.sh` exits 0 (build + test + clippy + golden + `lang.rs`
  + invariants). Reviewer is OFF for U8 — **the green gate + `cargo doc` clean is your sole sign-off.**
  Don't add clippy warnings; fix pre-existing ones in files you rewrite.
- **Selector discipline:** every runtime selector goes through `encode_selector` (ADR-0012 / F8).
- `rust-best-practices` skill; no `unsafe` expected — if any lands, add a `// SAFETY:` note and run
  `rust-sanitizers-miri`.

## 9. Return contract (self-report; no reviewer)
Report: the `Message` representation choice (kernel `.ph` class vs Rust struct) + rationale · the
`SEND_DYNAMIC` helper signature and its three consumers · files changed · confirmation the U9 variadic
seam is left intact (quote the seam comment) · how the IC slot is left uncorrupted · the resolution
taken for BLOCKED-ON-DECISION #1 (`List`) and #2 (perform scope) · `verify.sh` tail + `cargo doc` tail ·
any new `DEFERRED.md` entries (dNU handler cache; spread call sites if deferred).
