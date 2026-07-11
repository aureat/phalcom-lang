# U8 — Work order: `doesNotUnderstand(_:)` / `perform` + `SendDynamic` (RE-GROUNDED, post-U4)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Wave-F unit. **Reviewer OFF**
— self-verify on the green gate (`./scripts/verify.sh` exits 0) + `cargo doc` clean. Grounded in
**ADR-0012** (label-encoded selectors + IC-ready dispatch; dNU is the lookup-miss fallback) + specs
[`method-lookup.md`](../spec/method-lookup.md) §1–3 and
[`messages-and-selectors.md`](../spec/messages-and-selectors.md) §5. **This revision supersedes the
phase-2 draft** and re-grounds every reference against `main` at HEAD (post-U1/U2/U3/U4). U8 is
semantically independent of U5/U6, but rebases onto their + U7's `vm.rs`/`core.ph` edits (see §2)._

---

## 0. Re-grounding delta (what changed since the phase-2 draft — READ FIRST)
| Draft claim | Actual HEAD | Consequence for U8 |
|---|---|---|
| Miss arm at ~`vm.rs:507` | Actual miss site is **[`vm.rs:698–708`](../../phalcom-core/src/vm.rs#L698)**: `if let Some(method) = receiver.lookup_method(self, selector_sym) { call_method(...) } else { return Err(RuntimeError::MethodNotFound { selector, value }) }`. | This `else` arm is the **exact hook** you replace. Re-locate again post-U7 (it drifts every spine unit). |
| Encoder lives in `signature.rs` | **Wrong file.** `encode_selector(name, labels, kind)` is in [`method.rs:82`](../../phalcom-core/src/method.rs#L82) (with `make_signature` at L134, `SignatureKind` at L20). `signature.rs` is a thin module. | **All runtime selector construction routes through `crate::method::encode_selector`** — never hand-format. Its inverse (name/labels/kind decomposition) is what `Message` needs. |
| `RuntimeError::MessageNotUnderstood` to add | Today only `RuntimeError::MethodNotFound { selector: String, value: String }` exists ([`error.rs:71`](../../phalcom-core/src/error.rs#L71)). | Add `MessageNotUnderstood { selector, receiver }`; **repurpose/retire `MethodNotFound`** once the dNU rewrite makes it unreachable (§6). |
| `List` available via U-STD in the same wave | **No `List` exists** — only the reserved name string `List = "List"` in [`primitive/mod.rs:66`](../../phalcom-core/src/primitive/mod.rs#L66). `core.ph` defines no collection. U5/U6 do **not** add `List`. | **Resolved (DEC-A, user 2026-07-11): a minimal `List` unit (U-LIST) lands *before* U8** — it's also a U9 dep. `Message.args`/`labels`/`perform(_:List)` build on it. See §3. |
| dNU `Message` needs a bespoke Rust struct | Instances are already fixed-slot `InstanceObject`s after **U7** (ADR-0011). | **Recommend a kernel `class Message` in `core.ph`** built by a primitive constructor — no bespoke Rust type; reflection stays uniform. Requires U7 landed (it is, on the Wave-F base). |

**Net:** U8 = replace one `else` arm with a dNU forward + add one `SendDynamic` opcode + one
`send_dynamic` helper (built once, used three ways) + a kernel `Message` + `Object` reflection primitives.
No change to selector encoding or the IC slot.

## 1. Mission (one sentence)
Turn a lookup miss from a hard VM error into a **reified `Message` re-sent as `doesNotUnderstand(_:)`** up
the class chain, and add the shared **`SendDynamic`** runtime-send primitive that powers `Object.perform`,
dNU forwarding, and (later) U9/spread — without corrupting U3's IC-ready dispatch shape.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **U1/U2/U3/U4 landed** (they are). Native methods take `&Heap`/`&mut Heap`; no `Rc<RefCell>`.
- **U7 landed** — `Message` is an ordinary fixed-slot `InstanceObject`; the kernel `class Message` uses
  fields + `construct`. **If U8 is scheduled before U7, model `Message` differently or STOP** — a heap
  `InstanceObject` with the old `IndexMap` still works, but the plan targets the U7 slot layout.
- **U5/U6 landed** — U8 doesn't depend on control-flow/absence semantically, but it **shares `vm.rs`,
  `compiler/lib.rs`, `core.ph`, `error.rs`** with the spine. Start U8's worktree from **post-U7 HEAD**;
  re-locate the miss site (§0) and the `core.ph` insertion point after their edits.
- **U-LIST merged + green** (DEC-A resolved: a minimal kernel `List` lands before U8 — full work order
  [U-LIST-plan.md](U-LIST-plan.md)). `List` is a hard dep of `Message.args`/`labels` and
  `perform(_:List)`. **If U-LIST is not merged, STOP.**
- Baseline `./scripts/verify.sh` green on the worktree base. Runs in its **own worktree** off the Wave-F
  base (`git worktree add ../phalcom.worktrees/u8 feat/u8`) — do **not** share a tree with U9/U-STD.
- Re-run `graphify affected "lookup_method"`, `graphify affected "call_method"`,
  `graphify explain "RuntimeError"` on real HEAD before editing.

## 3. Design (ADR-0012 + method-lookup §1–3 + messages §5 — realize, don't re-litigate)
Applying the language-design rubric (dispatch: *missing-method hooks* + *reflective send* axes):

- **Miss path order (method-lookup §1):** IC → exact-selector probe (walk superchain via
  `lookup_method`) → **`[U9 variadic-table probe — LEAVE SEAM]`** → `doesNotUnderstand(_:)`. On exact-probe
  miss: synthesize a `Message`, then do an **exact** lookup of the `doesNotUnderstand(_:)` selector on the
  receiver's chain (guaranteed to hit `Object`'s default). **Recursion guard:** if `doesNotUnderstand(_:)`
  itself is missing, that is an internal invariant violation (`RuntimeError::Internal`), **not** another
  dNU — never loop.
- **`SendDynamic` (messages §5) — build once, use three ways.** One runtime primitive: build the selector
  `Symbol` at runtime from `(name, labels, arity)` via **`encode_selector`**, then run the *normal*
  `lookup_method` + `call_method` path (so IC/dNU/variadic all apply uniformly). Expose as
  (a) opcode `Bytecode::SendDynamic` for reflective/spread call sites, and
  (b) a Rust helper `vm.send_dynamic(receiver, selector, args)`.
  Consumers: `Object.perform`, dNU forwarding, and — deferred — U9/spread. **`perform(selector, args)`** is
  a thin primitive over `send_dynamic`; `perform(_:)` is the zero-arg case.
- **`Message` reification (method-lookup §2).** Fields: `selector` (interned `Symbol`), `name` (`String`,
  bare name), `labels` (`List` of `String`), `args` (`List`). **Kernel `class Message` in `core.ph`**
  (ordinary `InstanceObject`, U7 slots) built by a primitive constructor in `object.rs`. Selector →
  name/labels/kind decomposition **MUST reuse the `method.rs` encoder's inverse**, not a second parser.
- **Default `doesNotUnderstand(_:)` (method-lookup §2).** `Object.doesNotUnderstand(_:)` raises
  `MessageNotUnderstood { selector, receiver }` — this replaces today's `MethodNotFound` as the
  *observable* miss behavior, now originating from a real send a subclass can override. **Recommend a
  primitive** (clean raise formatting; can't be shadowed by a partial `core.ph` load), with the `class
  Message` shell in `core.ph`.
- **`respondsTo(_:)`** — a **pure exact-probe** (`lookup_method` only), **never triggers dNU**.
- **Handler caching (method-lookup §2, optional) — DEFER.** Correctness first; the miss path is
  slow-by-design. If ever added: key on stable `ClassId`, keep **separate** from the call-site IC,
  invalidate on hierarchy mutation (open-Q4). Mark in DEFERRED.

### Rubric — hazards & preclusion (mandatory)
- **Inline cache ⊗ mutable hierarchy (catalog hazard):** the dNU slow path must **not** populate or corrupt
  the monomorphic IC slot. A miss runs the chain walk then forwards; if you ever cache the dNU handler, key
  it on `ClassId` and keep it out of the call-site IC. Add an **IC-non-corruption golden** (warm site →
  polymorphic miss → subsequent hits still dispatch correctly).
- **Dispatch impact:** adds a *terminal* fallback and a runtime-selector send; **does not change selector
  encoding or the exact-probe.** The U9 variadic probe slots **between** exact-probe and dNU — leave a
  clearly-commented seam (quote it in the return report) so U9 needs no rewrite.
- **Representation impact:** `Message` is a normal slot instance; `SendDynamic` builds a `Symbol` +
  a `List` of args per reflective call — allocation on the *slow/reflective* path only, never the hot path.
- **Soundness:** the recursion guard (dNU-of-missing-dNU → `Internal`) and "`send_dynamic` re-enters
  `call_method` and is reentrant" are the two invariants to prove. A `perform` of an unknown selector must
  re-enter dNU exactly once, not loop.
- **Preclusion (mandatory step-5):** installing dNU as the terminal miss fallback **forecloses** silently
  swallowing misses — every miss now allocates a `Message` + sends. That is the spec's intent (proxies),
  but it means the *fast* path must keep the miss check branch-predictable; do not move dNU synthesis ahead
  of the exact-probe. Also: making the observable error a *sendable* `MessageNotUnderstood` precludes
  treating "method not found" as a pure Rust panic anywhere downstream — audit for that assumption.
- **Precedent:** Smalltalk `doesNotUnderstand:` + `perform:` (the canonical proxy mechanism); Ruby
  `method_missing` + `send`; Objective-C forwarding. All pay the reflective-send allocation cost on miss —
  accepted here; the win is uniform proxy/forwarding.

### DECISION — DEC-A: `List` availability → **LAND A MINIMAL List UNIT FIRST (option a), user-ratified 2026-07-11**
`Message.args`, `msg.labels`, and `perform(_:List)` require a real `List`; none exists (only the reserved
name string in `primitive/mod.rs:66`). U5/U6 do **not** provide it. **The user chose (a): a minimal kernel
`List` unit lands before U8.** This is the right call — `List` is *also* a hard dependency of **U9**
(rest-params `*xs` collect into a `List`), so it is on the critical path regardless; doing it once, first,
serves both.

**New unit — U-LIST (prerequisite, schedule before U8; blocks U8 and U9). Full work order:
[U-LIST-plan.md](U-LIST-plan.md).** Summary:
- **STOP — gated on ADR ratification.** [ADR-0020](../adr/0020-kernel-list-native-array-protocol.md)
  (kernel `List` is a native-array-backed protocol) pins the storage design and
  [ADR-0019](../adr/0019-freeze-vm-blessed-primitive-floor.md) (freeze the VM-blessed primitive floor) is
  its dependency — both landed **2026-07-11** but are **Status: Proposed**, not Accepted. Mirrors U7's
  DEC-D→ADR-0017 gate: do not dispatch U-LIST implementation until the user ratifies both to Accepted.
- **`List` is NOT technically gated on U7.** It is a native heap variant (`Object::List(ListObject)`,
  reached via `Value::Obj(ObjRef)`) exactly like `String`/`Closure`/`Block` — not an `InstanceObject` built
  on U7's slot layout, and needs no `construct`. It is created VM-blessed in `create_core_classes`, the
  same way `Option`/`Bool` are (see U-LIST-plan §2/§3). **Only `core.ph`'s single-editor collision rule**
  sequences it after U7 in the spine — not a genuine technical dependency. (`Message`, below, is the one
  piece of U8 that *does* need U7's slots.)
- **Scope (deliberately minimal, per ADR-0020):** six floor primitives (allocate, length, indexed get,
  indexed set, push, grow) + a thin `.ph` protocol (`at(_:)`, `size`, `add(_:)`, `each(_:)`, `toString`).
  **No** map/reduce/filter/slicing/literals — those belong to U-STD. Just enough for `Message` + `perform`
  + U9 rest-params. Full design, write-set, build order, and test strategy: **[U-LIST-plan.md](U-LIST-plan.md)**.
- **Collision note:** U-LIST edits `core.ph` + `primitive/mod.rs` (+ new `list.rs` heap variant). It must
  **not** co-schedule with another `core.ph` editor — land after U7's `core.ph` commits, before U8/U9/U-STD.

**U8 precondition (added):** confirm **U-LIST merged + green** before writing the `Message`/`perform`
surface. If it is not, **STOP** — U8 is dependency-blocked, not a place to inline a list.
- **BD-U8-2 (soft): `perform` surface.** Deliver `SendDynamic` + primitive `perform`/`perform:with:` only
  (no `f(*args)` call-site syntax); spread-at-call-site defers to the parser-owning unit (U9), reusing this
  opcode. Confirm scope.

**New ADR?** No new decision — covered by ADR-0012 + method-lookup §2–3 + messages §5. **Propose a short
amendment note on ADR-0008** recording that `MessageNotUnderstood` is the default-dNU raise, if not already
enumerated (flag for `documentation-and-adrs`).

## 4. Confirmed write-set (re-validate with `graphify affected` on post-U7 HEAD)
| File | Why it's in scope |
|---|---|
| `phalcom-core/src/vm.rs` | Replace the `MethodNotFound` `else` arm (L698–708) with the dNU forward; add the `SendDynamic` handler + `send_dynamic(receiver, selector, args)` helper. **Leave the U9 variadic seam** just before the dNU forward. |
| `phalcom-core/src/bytecode.rs` | New `SendDynamic` opcode (selector + args built at runtime). |
| `phalcom-core/src/method.rs` | **Reuse** `encode_selector`; add the **inverse** (selector `Symbol` → name/labels/kind) if not already present, for `Message` decomposition. (Read-mostly; add only the decoder.) |
| `phalcom-core/src/primitive/object.rs` | `perform(_:_:)` / `perform(_:)`; default `doesNotUnderstand(_:)` (primitive raise); `respondsTo(_:)`; `Message` accessors if primitive-backed. |
| `phalcom-core/core/core.ph` | `class Message { … }` (U7 fields + `construct`); `Object.doesNotUnderstand(_:)` wiring. **Shared file — sequence after U7's `core.ph` edits.** |
| `phalcom-core/src/error.rs` | Add `RuntimeError::MessageNotUnderstood { selector, receiver }`; retire `MethodNotFound` if unreachable (§6). |
| `phalcom-core/src/universe.rs` | Register `Message`/`MessageNotUnderstood` kernel wiring if Rust-side registration is needed. |
| `phalcom-core/bin/phalcom/disasm.rs` | `SendDynamic` disasm arm. |
| `phalcom-core/src/compiler/lib.rs` | **Only if** `perform`/spread is surfaced syntactically (BD-U8-2 = no) — otherwise untouched. Keep the write-set tight. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | Acceptance corpus (§7). The `dispatch()`/`messages()` groups already exist ([`lang.rs`](../../phalcom-core/tests/lang.rs)). |

**Disjointness note:** U8 and U9 share `vm.rs`, the `method.rs` encoder, `compiler/lib.rs`, and `core.ph`
— **NOT parallelizable.** U8 (Wave F) lands first as the terminal fallback; U9 (Wave F+1) inserts the
variadic probe ahead of it. Sequence.

## 5. Build order (land as one coherent, self-verifiable diff)
1. **`error.rs`** — `MessageNotUnderstood { selector, receiver }`. Full rustdoc.
2. **`bytecode.rs` + `disasm.rs`** — `SendDynamic` opcode + doc + disasm arm.
3. **`method.rs`** — selector-decoder inverse (if absent), rustdoc'd; reused by `Message`.
4. **`vm.rs`** — `send_dynamic` helper (build selector via `encode_selector`, normal lookup+`call_method`);
   `SendDynamic` handler over it; replace the L698 `else` arm with: synthesize `Message` → forward
   `doesNotUnderstand(_:)`. **Leave the U9 variadic seam** (comment + ordering hook) immediately before the
   dNU forward.
5. **`Message` + reflection** — `class Message` in `core.ph`; primitive constructor/accessors in
   `object.rs`; `Object.doesNotUnderstand(_:)` (primitive raise); `Object.perform(_:_:)`/`perform(_:)`;
   `Object.respondsTo(_:)` (pure probe). Register in `universe.rs` as needed.
6. **`tests/lang.rs`** — acceptance corpus (§7).

## 6. Fold-in cleanup (only within the write-set)
- If `RuntimeError::MethodNotFound` is unreachable after the rewrite, remove it + its `Display`; if still
  reached elsewhere, leave it and note why (`graphify affected "MethodNotFound"` first).
- Remove any dead commented dispatch block in `call_method` you are already rewriting (confirm with
  `graphify affected "call_method"`).

## 7. Test strategy (the green gate must assert)
- **Proxy/dNU (headline):** a `Proxy` overriding `doesNotUnderstand(_:)` that forwards via `perform` to a
  target — assert the forwarded result.
- **Default dNU:** unknown selector on a plain object surfaces `MessageNotUnderstood` with the right
  selector + receiver rendering (was `MethodNotFound`) — a **behavior-change golden**; update any golden
  asserting the old text.
- **`Message` shape:** inside a dNU override, assert `msg.selector`/`msg.name`/`msg.labels`/`msg.args` for
  a labelled call (`x.move(to: a, duration: b)`) — verifies encoder-inverse correctness.
- **`perform`:** `3.perform(#"+(_:)", [4])` → `7` (project Symbol-literal form) — reflective parity with a
  static send; `perform` of an unknown selector re-enters dNU **once** (no infinite loop).
- **`respondsTo`:** true for a defined selector, false for unknown — **without** triggering dNU.
- **IC non-corruption:** a warm call site that then misses (polymorphic receiver) still dispatches
  correctly on subsequent hits — guards the dNU slow path against trampling the IC slot.
- **Fuzz (opt-in):** random unknown selectors never panic/UB; always route to dNU.

## 8. Mandatory rules
- **Docs:** `//!` on any new module; `///` on every new public item (`SendDynamic`, `send_dynamic`,
  `Message`, `MessageNotUnderstood`, the selector decoder, every new primitive) with `# Panics`/`# Errors`,
  intra-doc links, and ADR-0012 / method-lookup citations. `cargo doc --workspace --no-deps` adds no new
  warnings.
- **Green gate = sign-off:** `./scripts/verify.sh` exits 0 (build + test + clippy + golden + `lang.rs` +
  invariants). Reviewer OFF — green + `cargo doc` clean is the sole sign-off. No new clippy warnings.
- **Selector discipline:** every runtime selector goes through `encode_selector` (ADR-0012 / F8).
- `rust-best-practices`; no `unsafe` expected — if any lands, `// SAFETY:` note + `rust-sanitizers-miri`.

## 9. Return contract (self-report; no reviewer)
Report: the `Message` representation (kernel `.ph` class vs Rust struct) + rationale · the `send_dynamic`
signature and its three consumers · files changed · **the exact U9 variadic seam comment (quoted)** ·
how the IC slot is left uncorrupted · confirmation **U-LIST** landed and `Message.args`/`labels`/`perform`
traffic in the real kernel `List` (DEC-A) + BD-U8-2 (perform scope resolution) ·
`verify.sh` tail + `cargo doc` tail · any new `DEFERRED.md` entries (dNU handler cache; spread call sites).
