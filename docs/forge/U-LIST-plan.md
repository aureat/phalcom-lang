# U-LIST — Work order: minimal kernel `List` (native array + `.ph` protocol)

_Self-contained implementation plan for **one** implementer. Spine-tail unit, scheduled before Wave F.
**Reviewer OFF** — self-verify on the green gate (`./scripts/verify.sh` exits 0) + `cargo doc` clean.
Grounded in **ADR-0020** (kernel `List` is a native-array-backed protocol) + its dependency **ADR-0019**
(freeze the VM-blessed primitive floor), resolving **DEC-A**. Specs:
[`messages-and-selectors.md`](../spec/messages-and-selectors.md) (rest-param → `List`),
[`method-lookup.md`](../spec/method-lookup.md) §2 (`msg.args`/`msg.labels` → `List`),
[`functions.md`](../spec/functions.md) (`callWith(_)`/`invokeOn(_,_)` → `List` of args). Re-grounded
against `main` at HEAD (post-U5/U6, U7 in flight).

---

## 0. STOP — ratification gate

**ADR-0019 and ADR-0020 are `Status: Proposed`, not Accepted.** This plan may be written and reviewed now,
but **implementation must not start until the user ratifies both to Accepted** — the same discipline as
U7's DEC-D→ADR-0017 gate. Confirm status in `docs/adr/README.md` before dispatching step 1.

## 1. Mission (one sentence)
Give the runtime a minimal, native-array-backed kernel `class List` — six floor primitives (allocate,
length, indexed get, indexed set, push, grow) plus a thin `.ph` protocol (`at(_:)`, `size`, `add(_:)`,
`each(_:)`, `toString`) — just enough to unblock `Message.args`/`labels` (U8) and rest-param collection
(U9), with map/reduce/filter/literals deliberately deferred to U-STD.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **U1/U2 landed** (handle/arena heap, metaclass tower) — `List` is a heap object like `String`, needs the
  arena and kernel-class wiring machinery both provide.
- **U6 landed** — `List.at(_:)` on an out-of-range index surfaces absence via `None`, reusing U6's
  absence-surfacing helper (Invariant 4) rather than inventing a second one.
- **U7 is NOT a technical precondition.** `List` is modeled exactly like `String`/`Closure`/`Block`: a
  dedicated `Object::List(ListObject)` heap variant reached through `Value::Obj(ObjRef)`
  ([`value.rs:31–44`](../../phalcom-core/src/value.rs#L31),
  [`heap.rs:66–80`](../../phalcom-core/src/heap.rs#L66)) — **not** an `InstanceObject` with U7 slots. It
  needs no field table and no `construct` lowering. **The only reason it's sequenced after U7 in the spine
  is the `core.ph` single-editor collision rule** (PHASE2-INDEX §2/§3: never co-schedule two `core.ph`
  editors) — land it once U7's `core.ph` edits are committed, not because U-LIST *depends* on U7's design.
- **ADR-0019 + ADR-0020 both `Status: Accepted`** (see §0). If either is still Proposed, **STOP**.
- Baseline `./scripts/verify.sh` green before the first edit.
- Re-run `graphify explain "Heap"`, `graphify explain "ObjRef"`, `graphify affected "core.ph"` on real
  HEAD — `core.ph` drifts with every spine unit; re-locate the insertion point post-U7.

## 3. Design (ADR-0019 §floor item 1 analog + ADR-0020 — realize, don't re-litigate)
Applying the language-design rubric (stdlib: *primitive/library boundary* axis; bootstrapping:
*kernel load order* axis):

- **Representation — new heap variant, not an instance.** `ListObject { elements: Vec<Value> }` in a new
  `phalcom-core/src/list.rs`, mirroring [`string.rs`](../../phalcom-core/src/string.rs)'s
  `StringObject` shape. Add `Object::List(ListObject)` to the [`Object` enum](../../phalcom-core/src/heap.rs#L66)
  (alongside `Module`/`Closure`/`Str`/`Block`). `Heap::alloc_list(&mut self, elements: Vec<Value>) -> ObjRef`
  plus `.list(id)`/`.list_mut(id)` accessors, mirroring `alloc_string`/`.as_string()`
  ([`heap.rs:113–115`](../../phalcom-core/src/heap.rs#L113)). No `Rc`/`RefCell` — the array is a heap
  handle like any other object (ADR-0009), GC-ready, no borrow-panic surface.
- **Six floor primitives (ADR-0020, native, in `phalcom-core/src/primitive/list.rs`):** allocate (backs
  `List.new()` — zero-arg, empty), length, indexed get (raw, may be out-of-range), indexed set (raw;
  **implemented but not surfaced at the `.ph` layer this unit** — no `at(_:put:)` selector yet, keeps the
  write-set/test-surface minimal; note as available-but-unexposed, candidate for U-STD to surface), push
  (append one element, grows capacity), raw-capacity grow (amortized growth, called by push — may be
  internal-only, not a separate primitive method if `Vec::push` already amortizes; implementer's call
  whether to expose a distinct grow op or fold it into push).
- **`.ph` protocol (ADR-0020 "hybrid: native primitives, self-defined control"), in `core.ph`:**
  ```phalcom
  class List {
    at(i) { /* raw indexed get; out-of-range -> None via U6 helper, not a Rust panic */ }
    size => /* raw length primitive */
    add(v) { /* raw push primitive; returns self for chaining, matching Smalltalk OrderedCollection */ }
    each(f) { /* iterate 0..size, f.call(self.at(i)) each step */ }
    toString => /* "[e1, e2, e3]" rendering via each + String concatenation */
  }
  ```
  `each(_:)` may be written either as `.ph` using index + `whileTrue` (U5 sacred-selector inliner is
  landed, so this is available) **or** as a native primitive if that is simpler given existing
  block-calling machinery from Rust (`call_method`/closures already exist post-U4) — implementer's call;
  document the choice in the return contract. **No** `map`/`collect`/`inject`/`filter`/slicing/literal
  syntax `[a, b, c]` — those are U-STD's job, layered over this minimal surface later.
- **Kernel registration — `List` is VM-blessed, like `Option`.** Add `let list_class = make_core_class(heap,
  "List", object_class, metaclass_class);` to
  [`universe.rs::create_core_classes`](../../phalcom-core/src/universe.rs#L89), positioned in the load
  order **immediately after `Option`/`Bool`, before anything that will depend on it** (ADR-0020's stated
  order: `Bool, Option, Number, Symbol, String → List → Map/Set/Iteration/Message.args/rest-params`).
  `core.ph` then reopens `class List { … }` with the `.ph` protocol bodies, same pattern as the `Option`
  skeleton reopen ([`core.ph:28`](../../phalcom-core/core/core.ph#L28)). This is *why* `List` doesn't need
  U7: it is created the same way `Option`/`Bool`/`String` are — natively, before `core.ph` loads — not
  through user-facing `construct`.
- **Absence at the boundary.** An out-of-range `at(_:)` read must **not** panic and must **not** leak the
  raw `Value::Nil` sentinel — surface `None` via the same VM helper U6 established
  (`VM::none_value()`/the Invariant-4 boundary), exactly as an unassigned instance field will once U7
  lands. This is the one place U-LIST and U6 genuinely interact.
- **New error variant, if needed.** No `RuntimeError` variant exists for a non-Number/non-List index
  argument to `at(_:)` (only `Type`/`Arity`/`MethodNotFound`/etc. — see
  [`error.rs:63–99`](../../phalcom-core/src/error.rs#L63)). A **non-integer or negative index is a type
  error** (reuse `RuntimeError::Type`), not a new variant; **out-of-range is absence** (`None`), not an
  error — do not conflate the two.

### Rubric — hazards & preclusion (mandatory)
- **Soundness:** `at(_:)` on an out-of-range index must surface `None`, never a Rust panic and never the
  raw `Value::Nil` sentinel (Invariant 4 applies here exactly as it does to unassigned instance fields).
  A negative or non-Number index is a hard type error, not a silent wrap or truncation.
- **Primitive/library boundary ⊗ bootstrap order (catalog hazard):** the six array primitives are floor
  (ADR-0019/0020); the surfaced protocol is `.ph`. Do not let convenience push `at`/`add`/`each` down into
  Rust "for speed" — that is exactly the boundary-creep ADR-0019 was written to forbid. If a future unit
  wants to move one down, it needs a superseding ADR (ADR-0019's ratchet rule), not a quiet primitive add.
- **Representation impact:** `Vec<Value>` behind a heap handle — no allocation on non-mutating reads;
  `add`/push may reallocate (standard amortized-growth `Vec` behavior). No change to `Value`'s tag layout;
  `Obj(ObjRef)` already covers this exactly as it does `String`/`Closure`/`Block`.
- **Dispatch impact:** none — `List` uses ordinary single dispatch via the existing `Invoke` path, same as
  every other kernel class. No selector-encoding change.
- **Preclusion (mandatory step-5):** shipping `List` with only six raw primitives + a thin `.ph` surface
  **forecloses nothing** for U-STD (map/reduce/filter/literals can be added additively as more `.ph`
  methods over the same native storage) but **does** commit to array storage over a cons-list — reversing
  that later would be a breaking representation change for anything holding a `List` `ObjRef`. Accept
  deliberately (ADR-0020's rejected alternative: `Some`/`None` cons-list, rejected for the `Option`-on-hot-
  path cost).
- **Precedent:** Smalltalk `OrderedCollection` (native storage, dispatched protocol) is the direct model;
  Python's `list`/JS `Array` are fully native (both rejected alternatives in ADR-0020 — they freeze the
  sequence API against introspection, the opposite of Phalcom's dogfooding goal).

## 4. Confirmed write-set (re-validate with `graphify affected` on post-U7 HEAD)
| File | Why it's in scope |
|---|---|
| `phalcom-core/src/list.rs` (**new**) | `ListObject { elements: Vec<Value> }` — mirrors `string.rs`. Full rustdoc. |
| `phalcom-core/src/heap.rs` | Add `Object::List(ListObject)` variant; `alloc_list`/`.list()`/`.list_mut()` accessors, mirroring the `Str`/`alloc_string` pattern. |
| `phalcom-core/src/value.rs` | `.class()`/`.to_string()`/`CallContext` match arms gain a `List` case (mirrors the existing `Str`/`Closure` arms — see [`value.rs:94,98,145,169`](../../phalcom-core/src/value.rs#L94)). No new `Value` variant — `Obj(ObjRef)` already covers it. |
| `phalcom-core/src/primitive/list.rs` (**new**) | The six floor primitives (allocate/length/get/set/push/grow — or push-with-amortized-grow folded together, implementer's call). Full rustdoc, `# Errors`/`# Panics`. |
| `phalcom-core/src/primitive/mod.rs` | Wire the new module in; the reserved name `List` already exists at [`primitive/mod.rs:66`](../../phalcom-core/src/primitive/mod.rs#L66) — no rename needed. |
| `phalcom-core/src/universe.rs` | `make_core_class(heap, "List", object_class, metaclass_class)` in `create_core_classes` (§89), positioned per the ADR-0020 load order; `install_primitives` wiring for the six native methods. |
| `phalcom-core/core/core.ph` | `class List { at(_:) …; size => …; add(_:) …; each(_:) …; toString => … }` reopen, **sequenced after U7's `core.ph` edits are committed** (collision rule). |
| `phalcom-core/src/error.rs` | Only if a new variant is genuinely needed for a malformed index — prefer reusing `RuntimeError::Type` (see §3) over adding one. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | Acceptance corpus (§7). No `list` label exists in the corpus yet — create the directory per MANIFEST.md's "adding a case" note. |

**Collision note (unchanged from U8-plan §3):** `List` edits `core.ph` + `primitive/mod.rs` (+ `list.rs`
heap variant). **Never co-schedule with another `core.ph` editor** — land after U7's `core.ph` commits,
before U8/U9/U-STD start theirs.

## 5. Build order (land as one coherent, self-verifiable diff)
1. **`list.rs`** — `ListObject` struct. Full rustdoc.
2. **`heap.rs`** — `Object::List` variant + `alloc_list`/accessors.
3. **`value.rs`** — class/`to_string`/`CallContext` match arms for the new variant.
4. **`primitive/list.rs`** — six floor primitives; wire into `primitive/mod.rs`.
5. **`universe.rs`** — `make_core_class` for `List` at the correct load-order position; `install_primitives`
   wiring.
6. **`core.ph`** — `.ph` protocol reopen (`at`/`size`/`add`/`each`/`toString`), sequenced post-U7.
7. **Tests** — goldens + negatives (§7); create the `list` corpus label.

## 6. Mandatory rules
- **Docs:** `//!` on every new module (`list.rs`, `primitive/list.rs`); `///` on every new public item
  (`ListObject`, `alloc_list`, each of the six primitives, the `Object::List` variant) with ADR-0019/0020
  citations. `cargo doc --workspace --no-deps` adds no new warnings.
- **Green gate:** `./scripts/verify.sh` exits 0. No new clippy warnings.
- `rust-best-practices`; no `unsafe` expected.

## 7. Test strategy (the green gate must assert)
- **Construction + basic ops:** `List.new()`, `add(_:)` three times, `size` → `3`, `at(0)`/`at(2)` round-trip
  the pushed values.
- **Absence at the boundary:** `List.new().at(0)` (empty list, index 0) → `None`, **not** a panic, not the
  raw sentinel.
- **`each(_:)`:** sums/collects over a 3-element list via a block, asserts the accumulated result — proves
  block-calling into `List` iteration works.
- **`toString`:** a small list renders as `"[1, 2, 3]"` (exact format — pin whatever the implementer
  chooses, consistently).
- **Type error:** `at("x")` (non-Number index) → `RuntimeError::Type`, not a panic or silent coercion.
- **Fuzz (opt-in):** random indices (including negative, huge, non-Number) never panic/UB.

## 8. Return contract (self-report; no reviewer)
Report: the `ListObject` representation + why it's a heap variant, not an `InstanceObject` (confirm no U7
dependency) · the six primitives + which are surfaced at the `.ph` layer vs internal-only · the
`each(_:)` implementation choice (`.ph` loop vs native primitive) and why · confirmation the absence
boundary reuses U6's helper (no raw sentinel leak) · kernel load-order position confirmed against
ADR-0020 · goldens/negatives + `verify.sh` tail · `cargo doc` tail · any new `DEFERRED.md` entries
(indexed-set surfacing, `map`/`collect`/`inject`, literal syntax — all explicitly deferred to U-STD).
