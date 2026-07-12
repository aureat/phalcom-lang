# U-CORE-6 — Error root + wire dNU → `MessageNotUnderstood` (Implementation Spec)

> **Status:** Normative work order. Authored for a `phalcom-implementer` to
> execute end-to-end. Where a fact was verified against source, the `file:line`
> is cited; verify line numbers before editing (concurrent forge sessions shift
> them).
>
> **Scope in one line:** the *minimal reification slice* of [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md) —
> reify `Error` (root) + `MessageNotUnderstood` (`< Error`) as surface classes with
> `message`/`raise`, and rewire the existing native miss path (U8's
> `object_does_not_understand` + `Message` reification) to **raise a surface
> `MessageNotUnderstood`** carrying the reified `Message` through the **unified
> unwind** — *not* the native `RuntimeError::MessageNotUnderstood`.

> **Baseline: re-grounded at HEAD `9a6fb81`** (previous grounding: `4e2ec73`,
> pre-U-CORE-1/2/3). Recommended-order position (per `implementation-status.md`'s
> spine, unchanged by this refresh): **U-CORE-1, U-CORE-2, U-CORE-3 are
> landed**; **U-CORE-4 is in flight right now** — a concurrent session has
> uncommitted edits to `value.rs`, `universe.rs`, `primitive/{number,object}.rs`,
> `core.ph`, `interner.rs`, `primitive/symbol.rs`, plus `tests/{invariants,golden,lang}.rs`
> and a run of `tests/lang/*` fixtures (adding the `Object#toString`
> class-receiver fix + `Number#toString`); **U-CORE-5 is pre-grounded** (its
> spec was refreshed to this same HEAD but it has **not** been implemented yet —
> its own header states it "runs after U-CORE-4 lands"). **U-CORE-6 is LAST in
> the U-CORE track** — dispatch only after both U-CORE-4 and U-CORE-5 have
> actually landed (committed), since both touch files this unit also touches
> (`universe.rs`, `core.ph`, `tests/invariants.rs`) and this unit's own
> `verify_invariants`/floor-census edits must land on their final state, not a
> moving target.
>
> **Floor math (express as delta, do not hardcode a literal).** Stable today,
> confirmed unaffected by the in-flight U-CORE-4 edits (`vm.rs`, `error.rs`,
> `interpret.rs`, `compiler/lib.rs` are **not** in the concurrent session's
> touched-file set): **85 installed `(class, selector)` bindings / 69 distinct
> native fns** (post-U-CORE-3). U-CORE-4 is expected to land at **86 bindings**
> — unambiguous, both `floor-census.md`'s own U-CORE-4 amendment note and
> [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)'s
> cumulative ledger agree (85 + 1 = 86). The **distinct-fn** count is
> convention-sensitive: `floor-census.md` §1.1's own U-CORE-4 note computes
> **71** (it counts the `object_to_string` rehome off `object_name` as a
> *second* new distinct fn, even though the `(Object, toString)` *binding*
> count doesn't move), while U-CORE-5's sibling pre-ground used the simpler
> shorthand **70** (+1 only) — **re-read `floor-census.md` §1.1 live at
> dispatch, do not pick one**. U-CORE-5 adds **+0/+0** (confirmed: "adds zero
> floor primitives — no ADR-0019 amendment", its own as-built line 11).
> **This unit's own delta is unambiguous at the binding level: +2**
> (`Error#message`, `Error#raise`) **on top of whatever U-CORE-4 lands —
> i.e. 88 installed bindings once all four omnibus-cleared units
> (U-CORE-1/3/4/6) have landed**, which is exactly the cumulative ceiling
> [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)
> already authorizes ("Floor count. Cumulative, if all four units land as
> specified: **73 → 88** (+7 +5 +1 +2)"). This unit's own **distinct-fn** delta
> is also +2 (`error_message`, `error_raise` are wholly new fns, no rehome
> subtlety) — so **73** on the 71-convention, **72** on the 70-convention;
> again, confirm live rather than hardcoding either. Any anchor below in
> `value.rs`/`universe.rs`/`primitive/object.rs`/`core.ph` is tagged **⚠ will
> shift once U-CORE-4/5 land — re-confirm at dispatch**; anchors in `vm.rs`,
> `error.rs`, `interpret.rs`, and `compiler/lib.rs` were re-verified live this
> pass and are stable (those files carry no concurrent edits).

---

## §0. Prerequisites + scope gate

### Already landed (do not rebuild)

| Dep | What it gives U-CORE-6 | Ground truth |
|---|---|---|
| **U8** | Overridable `doesNotUnderstand(_)` hook + `Message` reification (`VM::new_message`) + `forward_does_not_understand` miss forward. | `primitive/object.rs:219` ⚠ (`object_does_not_understand`; was `:140` at the prior grounding — `object.rs` is mid-edit by the concurrent U-CORE-4 session, re-confirm before editing), `vm.rs:487` (`new_message`, stable), `vm.rs:530` (`forward_does_not_understand`, stable) |
| **U10** | The **unified unwind** substrate: `Bytecode::ReturnNonLocal` + frame-token eager unwind + `RuntimeError::DeadFrameError`. This is the *Return-token* payload of ADR-0008's one unwind primitive; U-CORE-6 adds the sibling *Raise* payload. | `vm.rs:1126` (`ReturnNonLocal` handler, was `:1068`, stable file), `error.rs:138-139` (`DeadFrameError`, unchanged) |
| **U-CORE-0** | Q2 ruling (confirm ADR-0008, do not redesign), the census, the invariant ledger, the forward-compat gate. | [`decisions.md`](../../../spec/v0.2/core/decisions.md) §Q2, [`floor-census.md`](../../../spec/v0.2/core/floor-census.md), [`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md) §U-CORE-6, [`forward-compat.md`](../../../spec/v0.2/core/forward-compat.md) §2 |
| **U-CORE-3** *(landed since the prior grounding — new dependency, confirmed)* | Landed the `Method`/`Object#methodFor(_)` reflection primitives via the same `primitive!`/`primitive_static!` registration idiom this unit reuses for `Error#message`/`Error#raise`. More importantly, U-CORE-3 **explicitly deferred** the reification of `RuntimeError::Arity`/`RuntimeError::Type` into surface `ArgumentError`/`TypeError` to *this* unit — its own as-built says "Reifying those is U-CORE-6; R-INV-3.4's 'ArgumentError' today means the native `RuntimeError::Arity`." **U-CORE-6 does not pick this up**: per §0 "Explicitly OUT of scope" below, the native `RuntimeError::Arity`/`Type` paths stay native *through* this unit too — only the dNU→`MessageNotUnderstood` slice lands here. The hand-off is one-directional (U-CORE-3 → future unit); U-CORE-6 is a way-station, not the destination, for `ArgumentError`/`TypeError`. | [`U-CORE-3/as-built.md`](../U-CORE-3/as-built.md) §0.2 (lines 62-65: "The full error hierarchy... Reifying those is U-CORE-6"), §4.3 R-INV-3.4 (line 543: "the surface `ArgumentError` is U-CORE-6"), §3.2/589-591 (`invoke_method_object`'s `RuntimeError::Arity` "is exactly the native path that will be re-pointed") |
| **Class-tower machinery** | `make_core_class` (create a kernel row + parallel-rule metaclass), Phase-E field stamping (`Some`/`Message`), `add_class!` globals, the reopen path. | `universe.rs:622` ⚠ (`make_core_class`, was `:492` — `universe.rs` is mid-edit; re-confirm), `vm.rs:142-157` (Phase E stamping, stable, unchanged from prior grounding), `vm.rs:336-372` (`add_class!` macro + calls, was `:323-352`, stable file) |

### Explicitly OUT of scope (RESERVE, do not implement)

Per [`decisions.md`](../../../spec/v0.2/core/decisions.md) §Q2 and [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md),
the following are **later units** — U-CORE-6 must *reserve* their shapes (keep them
layerable) but ship **none** of them:

- **`Result` / `Ok` / `Err`** and the bridges `{…}.attempt()`, `result.unwrap()`,
  `option.okOr(_)`, `result.ok()` ([values-and-absence.md](../../../spec/v0.2/values-and-absence.md) §4).
  Later **Result unit**; must mirror `Option`/`Some`/`None` (abstract root + two
  concrete subclasses), ADR-0008/[ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md).
- **The full handling protocol** — `blk.on(ErrorClass){…}`, `blk.ensure{…}`, and the
  `try`/`catch`/`finally` sugar over it ([error-handling.md](../../../spec/v0.2/error-handling.md) §2).
  **This is no longer a vague "later unit": the keyword spelling is now ratified
  by [ADR-0031](../../../adr/0031-error-handling-surface-syntax.md)** (Accepted,
  2026-07-12 — landed *during* this refresh pass): `throw`/`try`/`on`/`catch`/`ensure`
  as pure 1:1 sugar over `.on(_)(_)`/`.ensure(_)`, with `throw expr ≡ expr.raise()`.
  ADR-0031 explicitly confirms it targets **this unit's mechanism** ("U-CORE-6's
  non-minimal slice implements against this spelling," decisions.md §Q2). U-CORE-6
  still ships **none** of the syntax (no parser/compiler work here) — but the
  later error-syntax unit now has a concrete, ratified target instead of an open
  question, which de-risks this unit's design (§5 confirms no rework needed).
- **Surface `DeadFrameError` / `TypeError` / `ArgumentError` / `RangeError` classes.**
  The runtime *already raises these natively today* as `RuntimeError` variants —
  they stay native this unit (see §1, "native errors that remain unreified"). Only
  the dNU miss is reified. (Note: [catalog-delta.md](../../../spec/v0.2/core/catalog-delta.md)
  §2.7 tags all six error rows, including these four, with owner "U-CORE-6" — that
  column is coarse and does not distinguish this unit's *minimal* slice from the
  reification of native `RuntimeError` variants, which is explicitly reserved
  past this unit. Do not let that table's ownership column expand this unit's
  scope; §0 here and U-CORE-3's own hand-off are the authoritative scope fence.)
- **`throw`-as-compile-error** (`throw "oops"` rejected at compile time,
  error-handling.md §1). That is a parser/compiler check owned by the error-syntax
  unit (now [ADR-0031](../../../adr/0031-error-handling-surface-syntax.md)). This
  unit delivers the *mechanism* (`raise` lives only on `Error`), not the syntactic
  rejection (§4, R-INV-6.3).

### Non-negotiables carried in

- **ADR-0019 floor is frozen.** This unit adds **two** native primitives
  (`Error#message`, `Error#raise`) — that is an **ADR-0019 amendment**, already
  pre-cleared "in principle" by the omnibus
  [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)
  (Accepted; its Decision §4 names `Error#message`/`raise` by owner "U-CORE-6",
  **+2 bindings**, verbatim). This unit still needs its own **per-unit landing-record
  ADR** to formally move the census 86→88, mirroring the pattern
  [ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md) set for
  U-CORE-3 and the (currently "Proposed", not yet landed)
  [ADR-0036](../../../adr/0036-amend-floor-admit-number-tostring.md) is setting
  for U-CORE-4. **Claim the next available ADR number at dispatch** — as of this
  pass the highest assigned is 0036 (0034 was drafted-then-dropped per
  `docs/adr/README.md`, so 0035/0036 are the two most recent; the next free slot
  is **0037**, but re-run `ls docs/adr/` before claiming, since sibling units may
  claim numbers first) (§6).
- **No truthiness ([ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)).** Nothing here reintroduces surface `nil`.
- **No sacred-selector contact.** `message`/`raise` are not sacred (floor-census §5);
  the inliner is untouched.

---

## §1. What exists vs what is missing (grounded)

### The miss path today (verified)

1. `Bytecode::Invoke` misses the exact-selector probe (and the variadic probe) →
   `forward_does_not_understand(receiver_idx, selector, …)` (`vm.rs:1080`, was `:1023`).
2. `forward_does_not_understand` (`vm.rs:530`, was `:510`) truncates the args, synthesizes a
   4-slot `Message` via `new_message` (`vm.rs:487`, was `:467`), pushes it, looks up
   `doesNotUnderstand(_:)` and dispatches it via `call_method`. A user override is
   invoked here; otherwise the default resolves on `Object`.
3. The default `object_does_not_understand` (`primitive/object.rs:219` ⚠, was `:140` —
   this file is mid-edit by the concurrent U-CORE-4 session, re-confirm at dispatch)
   **returns `Err(RuntimeError::MessageNotUnderstood { selector, receiver })`**
   (`error.rs:82`, unchanged).
4. That `PhError` propagates via `?` up `call_method` → `run_until` → `run_in_module`
   → `interpret_source`'s `inspect_err`, which calls `runtime_error` (`vm.rs:690`, was `:632`)
   to print `err.to_string()` + a source-mapped trace; `run_file` maps
   `PhError::Runtime(_)` → exit **70** (`interpret.rs:125`, unchanged).

**What is missing:** steps 3–4 surface a *native* `RuntimeError` string, not a
*surface* `Error` object. There is **no `Error` / `MessageNotUnderstood` class**, no
`message`/`raise` protocol, and the unwind carries no catchable `Value`. `Error`,
`MessageNotUnderstood`, `DeadFrameError`, `TypeError`, `ArgumentError`, `RangeError`
are catalogued ([object-model.md](../../../spec/v0.2/object-model.md) §4 "Errors",
now lines 170-179, was cited "160-165" — the table drifted down as the "Concurrency"
subsection was inserted above it) and **absent from the tower** (`universe.rs`
`create_core_classes` has no `Error` row, re-confirmed this pass at `universe.rs:94-218`
⚠; [catalog-delta.md](../../../spec/v0.2/core/catalog-delta.md) §2.7 marks all six ❌/❌).

### The unified-unwind gap (the load-bearing design point)

ADR-0008: "the VM's unwind carries either a `Return` (frame-token) or a
`Raise(error)` payload." U10 built the **Return** payload
(`Bytecode::ReturnNonLocal` + eager frame-token unwind). The **Raise** payload does
not exist yet as a value-carrying channel: today an error is a `PhError` string that
propagates via Rust `?` and, uncaught, renders + exits. U-CORE-6 introduces the
Raise payload as a **surface-`Error`-carrying** `PhError` that propagates through
the *same* `PhResult`/`?` channel an uncaught error already uses — so that later
`on`/`ensure` (block protocol, now spelled by [ADR-0031](../../../adr/0031-error-handling-surface-syntax.md))
and a fiber's result-slot capture (forward-compat §1) intercept a real `Value`, not
a Rust string.

### Native errors that remain unreified (reserved, note only)

These `RuntimeError` variants (`error.rs:63`, unchanged) still raise natively this unit; the
later error unit reifies each to its surface `< Error` class. **Do not touch them:**
`Arity` (→ `ArgumentError`), `Type` (→ `TypeError`), `ZeroDivision`, `DeadFrameError`
(→ surface `DeadFrameError`), `InvalidSetClass`/`InvalidSetSuper`, `UndefinedVar`,
list-index type errors, etc. Their corpus fixtures in `tests/lang/runtime-errors/`
must stay **byte-identical** (re-confirmed present this pass: `runtime_unknown_method`,
`runtime_perform_unknown_selector`, `runtime_comparison_unsupported`,
`runtime_non_local_return_dead_frame`, etc. — no `pending/` prefix, all currently green).

> **Cross-unit hand-off (U-CORE-3 → U-CORE-6 → later).** U-CORE-3 already wired
> arity/type mismatches on `Method#invokeOn(_,_)`/`bound.call` to the native
> `RuntimeError::Arity`/`Type` and was explicit that it was **not** reifying them:
> "Arity failures raise a plain `RuntimeError` today; when U-CORE-6 reifies
> `ArgumentError` over the unified unwind, `invoke_method_object`'s
> `RuntimeError::Arity` is exactly the native path that will be re-pointed at the
> [surface class]" (`U-CORE-3/as-built.md` lines 589-591). **This unit does not
> perform that re-pointing** — it was scoped out in the paragraph above and
> stays reserved past U-CORE-6 too. The hand-off chain is therefore: U-CORE-3
> (native `Arity`/`Type`, notes the future target) → U-CORE-6 (reifies only the
> dNU/`MessageNotUnderstood` slice, explicitly declines the `Arity`/`Type` slice)
> → a later error-reification unit (re-points `RuntimeError::Arity`/`Type` at
> surface `ArgumentError`/`TypeError`, now that `Error` exists as a root to hang
> them off). Do not conflate "U-CORE-6 reifies `Error`" with "U-CORE-6 reifies
> *every* `RuntimeError` variant" — only `MessageNotUnderstood` moves this unit.

---

## §2. The native/`.ph` split + exact insertion points

**Decision: mirror `Message` exactly — Rust-created rows, Phase-E stamped field
layout, native construction in the miss path, native accessors.** This is a
deliberate architect call (see §6 D2): the alternative — a `.ph` `message` getter
over the field — trips the compiler's **read-before-write** check (`compiler/lib.rs:84`,
re-verified unchanged this pass; a getter that *reads* `_message` without any in-class *assignment* is rejected), and
would couple the Rust miss-path to `.ph` field-declaration order. `Message` already
solves exactly this problem the native way (floor-census §2.14); `Error`/`MNU` follow it.

| Concern | Native (Rust) | `.ph` |
|---|---|---|
| `Error`, `MessageNotUnderstood` **class rows** | ✅ `create_core_classes` (`make_core_class`) | optional empty reopen for surface visibility (see below) |
| Field layout (`_message`; MNU adds `_reifiedMessage`) | ✅ stamped in `VM::new` Phase E (mirrors `Some`/`Message`) | — |
| `Error#message` (getter → slot 0) | ✅ `error_message` **primitive** *(ADR-0019 amendment, pre-cleared by ADR-0023)* | — |
| `Error#raise` (unwind primitive) | ✅ `error_raise` **primitive** *(ADR-0019 amendment, pre-cleared by ADR-0023)* | — |
| Building the `MessageNotUnderstood` on a miss | ✅ rewritten `object_does_not_understand` builds it directly | — |
| `Raise` unwind payload | ✅ new `RuntimeError::Raise { error, rendered }` (**plumbing, not a floor binding** — confirmed by ADR-0023 Decision §4: "producing a `RuntimeError::Raise` payload — plumbing, not itself a bound selector") | — |
| Globals `Error` / `MessageNotUnderstood` | ✅ `add_class!` in `install_core` | — |

> **Floor delta: +2 bindings on top of whichever base U-CORE-4 lands** (two new
> installed bindings: `message`, `raise` on `Error`) — see the header "Floor
> math" box for the full accounting (base 86 → 88 at the binding level,
> unambiguous per [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)'s
> "73 → 88" cumulative ledger). The `RuntimeError::Raise` variant is *plumbing
> the primitive returns*, not an installed `(class, selector)` binding, so it
> does **not** count. R-INV-0.1 census + R-INV-6.5 update in lockstep (§4).

### Insertion points (exact)

1. **`error.rs`** — add the unwind payload and retire the native miss variant.
   - `use crate::value::Value;` (verified no cycle; `Value` implements `Debug`
     manually and `Clone`+`Copy`, re-confirmed unchanged, so the
     `#[derive(Error, Debug, Clone)]` on `RuntimeError` still holds).
   - Add:
     ```rust
     /// The surface-`Error` unwind payload — the `Raise(error)` half of ADR-0008's
     /// single unwind primitive (the sibling of U10's `Return`/`ReturnNonLocal`).
     /// `error` is a surface `Error` subclass instance (catchable, `isA(Error)`);
     /// `rendered` is a snapshot of its `message` for the uncaught-render path.
     #[error("{rendered}")]
     Raise { error: Value, rendered: String },
     ```
   - **Remove** `RuntimeError::MessageNotUnderstood` (`error.rs:82`, unchanged).
     Grep first: its only constructor is `object_does_not_understand`; corpus
     fixtures match on the *stdout string*, not the variant name, so the `rendered`
     string (below) keeps them green. If any non-dNU site references it, STOP and
     report. (Re-grepped this pass: still only one constructor.)

2. **`universe.rs` `create_core_classes`** (`universe.rs:94` ⚠, was `:93`) — after the
   `message_class` row (`universe.rs:192` ⚠, was `:173`), add:
   ```rust
   let error_class = make_core_class(heap, "Error", object_class, metaclass_class);
   let message_not_understood_class =
       make_core_class(heap, "MessageNotUnderstood", error_class, metaclass_class);
   ```
   `MNU`'s superclass is `error_class`, so `error_class` **must** be created first
   (mirror the `Option → Some/None` ordering, `universe.rs:168-170` ⚠, was
   `:149-151`). Add both to the `CoreClasses { … }` literal
   (`universe.rs:194-217` ⚠, was `:194ish`) and to the `struct CoreClasses`
   (`universe.rs:642` ⚠, was `:512` — drifted +130 lines from added rustdoc on
   the reflection/`toString` fields; re-confirm exact line at dispatch) with
   rustdoc.

3. **`vm.rs` `VM::new`** — after the `Message` stamp (`vm.rs:151-157`, stable,
   was `:150-158`), stamp the two error layouts (same idiom as `Some`,
   `vm.rs:142-148`, stable):
   ```rust
   { // Error: one field `_message` at slot 0.
       let error_class = vm.universe.classes.error_class;
       let msg_sym = vm.interner.intern("_message");
       vm.heap.class_mut(error_class).field_slots.insert(msg_sym, 0);
       vm.heap.class_mut(error_class).field_count = 1;
   }
   { // MessageNotUnderstood < Error: inherits `_message` (slot 0), adds
     // `_reifiedMessage` (slot 1). Subclass fields append after superclass
     // (compiler/lib.rs:711-715 offset rule, was cited `:713`, stable) — keep
     // 0/1 consistent with that.
       let mnu = vm.universe.classes.message_not_understood_class;
       let msg_sym = vm.interner.intern("_message");
       let reified_sym = vm.interner.intern("_reifiedMessage");
       vm.heap.class_mut(mnu).field_slots.insert(msg_sym, 0);
       vm.heap.class_mut(mnu).field_slots.insert(reified_sym, 1);
       vm.heap.class_mut(mnu).field_count = 2;
   }
   ```

4. **`vm.rs` `install_core`** (`vm.rs:330`, was cited `:317`, stable file) — add two
   `add_class!` lines after `add_class!(message_class);` (`vm.rs:372`, was `:352`):
   ```rust
   add_class!(error_class);
   add_class!(message_not_understood_class);
   ```
   This binds the globals **and** inserts them into `self.classes`, which routes any
   `core.ph` reopen through the **existing-class** path (`compiler/lib.rs:734`,
   re-verified unchanged this pass, `Bytecode::Constant`) rather than `create_class` — so the reopen never re-applies
   a computed `ClassLayout` and never clobbers the Phase-E `field_count` (this is the
   same mechanism that keeps `Some`'s stamped `field_count = 1` alive across its
   empty `class Some {}` reopen). **Do not** reopen `Error`/`MNU` in `core.ph` with a
   *body that reads a field* — that re-introduces the read-before-write hazard. An
   empty `class Error {}` / `class MessageNotUnderstood {}` reopen is *optional and
   harmless* (surface-visibility only, like `class Some {}`); the `add_class!` global
   already makes the name resolvable, so **skipping the reopen entirely is preferred**
   (fewer moving parts; matches how `Message` ships with no `.ph` reopen). **Note:**
   `core.ph` is in the concurrent U-CORE-4 session's touched-file set (⚠) — re-diff
   before editing, this unit only needs to *not* add a field-reading reopen, it does
   not need to touch whatever U-CORE-4 added elsewhere in the file.

5. **`primitive/error.rs`** *(new module)* — `error_message`, `error_raise` (§3).
   Register it in `primitive/mod.rs` (`pub mod error;` — re-confirmed absent this
   pass, the module list is currently `boolean, block, class, list, method, module,
   nil, number, object, string, symbol, system`) and add human-form `Sig`
   display aliases if the file keeps them (floor-census §1.2 — display only, not
   lookup keys).

6. **`primitive/object.rs`** — rewrite `object_does_not_understand` body
   (`primitive/object.rs:219` ⚠, was `:140`; §3). The receiver's message-slot
   accessors it must stay consistent with are `message_selector`/`message_name`/
   `message_labels`/`message_args` at `object.rs:249/259/270/280` ⚠ respectively,
   and the shared `message_slot` helper at `object.rs:230` ⚠ (all in the
   concurrently-edited file — re-confirm exact lines at dispatch).

7. **`universe.rs` `install_primitives`** (`universe.rs:246` ⚠) — install the two
   `Error` primitives (place near the `Message` accessors block,
   `universe.rs:277-281` ⚠, was cited `:249-253`):
   ```rust
   let error_cls = vm.universe.classes.error_class;
   primitive!(vm, error_cls, "message", SignatureKind::Getter, error_message);
   primitive!(vm, error_cls, "raise",   SignatureKind::Method(0), error_raise);
   ```
   `raise` is `raise()` (0-arity method, interns `raise()`), matching
   object-model §4 and `throw expr === expr.raise()` (now also the literal
   desugaring ADR-0031 §1 specifies); `message` is a getter (interns
   `message`).

8. **`universe.rs` `verify_invariants`** (`universe.rs:461` ⚠) — R-INV-6.1 boot
   check (§4). The existing `Some`/`Message` field-count fence to mirror sits at
   `universe.rs:585-594` ⚠ (was cited `:589-593`, essentially unchanged position
   — `message_class.field_count != 4` is still literally at line **592**,
   coincidentally unchanged from the prior grounding despite the surrounding
   file drifting elsewhere).

---

## §3. Concrete bodies

### 3.1 Rewritten `object_does_not_understand` (`primitive/object.rs:219` ⚠, was `:140` — re-confirm at dispatch, file is mid-edit by the concurrent U-CORE-4 session)

Keep the *exact* rendered string (`"{receiver} does not understand '{selector}'"`) so
`runtime_unknown_method.expected` (`3 does not understand 'wibble'`) and
`runtime_perform_unknown_selector.expected` (`3 does not understand 'bogus'`) stay
byte-identical (both fixtures re-confirmed present and un-pending this pass).
Change only the terminal step: build a surface `MessageNotUnderstood`
carrying the reified `Message` (already handed in as `args[0]`), then unwind via the
Raise payload.

```rust
pub fn object_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    // Selector text from the reified Message (slot 0), exactly as today.
    let selector = match message_slot(vm, &args[0], 0) {
        Some(Value::Symbol(sym)) => vm.resolve_symbol(sym).to_string(),
        _ => "<unknown>".to_string(),
    };
    let receiver_name = receiver.to_string(vm);
    let rendered = format!("{receiver_name} does not understand '{selector}'");

    // Reify the surface MessageNotUnderstood: slot 0 = message string,
    // slot 1 = the reified Message (`args[0]`, census §2.14). Built directly in
    // Rust — the Message precedent (VM::new_message), no `.ph` construct.
    let mnu_class = vm.universe.classes.message_not_understood_class;
    let field_count = vm.heap.class(mnu_class).field_count; // == 2 (Phase E)
    let mut inst = crate::instance::InstanceObject::new(mnu_class, field_count);
    inst.slots[0] = vm.alloc_string_value(rendered.clone());
    inst.slots[1] = args[0]; // the reified Message
    let mnu = Value::Obj(vm.heap.alloc(Object::Instance(inst)));

    // Raise it through the unified unwind (NOT the native RuntimeError variant).
    Err(RuntimeError::Raise { error: mnu, rendered }.into())
}
```

> The overridable-hook contract is untouched: `forward_does_not_understand`
> (`vm.rs:530`, was `:510`) still looks up `doesNotUnderstand(_:)` and dispatches a user override
> *before* this default is reached (R-INV-6.4). A proxy that overrides and returns a
> value never enters this function.

### 3.2 `primitive/error.rs` (new)

```rust
//! Native primitives on `Error` — the raisable root (object-model.md §4,
//! ADR-0008). `raise` is the surface half of the unified unwind; `message`
//! reads the error's `_message` slot. Both are ADR-0019 floor additions,
//! pre-cleared in principle by ADR-0023 (decisions.md §Q2).

/// Signature: `Error::message` — the error's message string (slot 0), surfaced
/// (`None` if unset). Native slot accessor, mirroring `Message::selector`
/// (avoids the read-before-write hazard a `.ph` getter over this field trips).
pub fn error_message(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let slot0 = match receiver {
        Value::Obj(id) => vm.heap.as_instance(*id).map(|i| i.slots[0]),
        _ => None,
    }.ok_or_else(|| RuntimeError::Type { expected: "Error", found: receiver.type_name() })?;
    Ok(vm.surface_absence(slot0))
}

/// Signature: `Error::raise()` — unwind the stack with `self` as a surface
/// `Error` (ADR-0008; `throw expr` desugars here per ADR-0031 §1). Returns the
/// `Raise(error)` payload; a fiber boundary (later) captures `error` into its
/// result slot, an `on(_)`/`ensure` (later, now spelled by ADR-0031) intercepts
/// it — this unit only produces it.
pub fn error_raise(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    // Render via the receiver's own `message` protocol so a future computed
    // override is honored. Re-entrancy here is safe: the stack is healthy
    // (inside a live primitive, pre-unwind), same pattern as object_perform.
    let message_sym = vm.get_or_intern("message");
    let rendered = vm.send_dynamic(*receiver, message_sym, &[])?.to_string(vm);
    Err(RuntimeError::Raise { error: *receiver, rendered }.into())
}
```

> **Only `Error` subclasses respond to `raise`** — the primitive is installed on
> `Error` only. A non-`Error` receiver has no `raise`, so a future `throw 42`
> (`42.raise()`) misses → dNU → `MessageNotUnderstood`; this is the runtime half of
> R-INV-6.3 (the compile-time rejection of `throw 42` is error-syntax, now spelled
> by [ADR-0031](../../../adr/0031-error-handling-surface-syntax.md), still deferred
> from this unit).

### 3.3 Propagation & rendering (no changes needed, confirm)

- `RuntimeError::Raise` sits inside `PhError::Runtime`, so `run_file`
  (`interpret.rs:125`, unchanged) maps it to exit **70** with **no edit**.
- `runtime_error` (`vm.rs:690`, was `:632`) prints `err.to_string()` = `rendered` (via
  `#[error("{rendered}")]`) + the live-frame trace — byte-identical to the old
  `MessageNotUnderstood` render, **no edit**.
- The surface `error` `Value` rides along untouched for the later `on`/`ensure`/fiber
  consumers.

---

## §4. Test strategy

### `_pending` fixtures this unit relates to ([pending-retirement.md](../../../spec/v0.2/core/pending-retirement.md) §4)

Neither is a **direct** flip — both need surface syntax this unit does not add:

| Fixture | Category | This unit delivers | Flips when |
|---|---|---|---|
| `errors/errors_throw_try_catch_finally` | B+C | the **raise mechanism** (`Error`/`MNU`, `raise`, unified-unwind payload) | error-syntax — now [ADR-0031](../../../adr/0031-error-handling-surface-syntax.md) (ratified, not yet implemented) **+** the `on`/`ensure` block-protocol unit |
| `errors/errors_result_bridge` | B | **nothing** — `Result`/`Ok`/`Err` + `.attempt()`/`.unwrap()` are RESERVED (§0) | the later **Result unit** **+** error-syntax (ADR-0031) |

Set the acceptance bar on **new unit-local fixtures in already-supported syntax**,
not on these lexer/Result-gated ones.

### New unit-local fixtures (the acceptance bar)

1. **`tests/lang/runtime-errors/` — uncaught surface-MNU raise renders (NEGATIVE,
   plain syntax).** New `.ph` sending an unknown message to a *user* object plus
   `.expected` = the exact miss string. Proves the reified raise renders identically
   to the old native path (no user-visible regression). Also **keep**
   `runtime_unknown_method` / `runtime_perform_unknown_selector` byte-identical
   (regression guards on the `rendered` format; both re-confirmed present and green
   this pass, no `pending/` prefix).
   ```phalcom
   // status: NEGATIVE
   class Widget {}
   System.print(Widget.new().frobnicate())
   ```
   `.expected`: `<Widget instance> does not understand 'frobnicate'` (confirm the
   exact receiver rendering via `Value::to_string`; bless from the built binary).

2. **`tests/invariants.rs` — R-INV-6.2 surface-class assertion (corpus, Rust).**
   The `.ph`/stdout lane cannot observe the raised object without `catch`; assert it
   at the VM level (models the existing `VM::new()` + `send_dynamic` corpus style —
   `tests/invariants.rs` is currently mid-edit by the concurrent U-CORE-4 session,
   ⚠ re-diff its current line count/imports before adding this test):
   ```rust
   #[test]
   fn genuine_miss_raises_surface_message_not_understood() {
       let mut vm = VM::new();
       let bogus = vm.get_or_intern("frobnicate");
       let err = vm.send_dynamic(Value::Number(3.0), bogus, &[]).unwrap_err();
       // (a) It is the Raise payload, not the old native MessageNotUnderstood.
       let raised = match err {
           PhError::Runtime(RuntimeError::Raise { error, .. }) => error,
           other => panic!("expected Raise, got {other:?}"),
       };
       // (b) The raised object isA(Error) and is a MessageNotUnderstood.
       let cls = raised.class(&vm.heap); // or raised.class(&mut vm) per the API
       assert_eq!(cls, vm.universe.classes.message_not_understood_class);
       assert!(is_a(&vm, cls, vm.universe.classes.error_class),
               "raised object must be isA(Error)");
       // (c) It carries the reified Message in slot 1.
       // (assert slot 1 is an instance of Message)
   }
   ```
   (Use a small `is_a` helper walking the superclass chain, or reuse the runtime's
   lookup; the point is the three assertions.)

### Invariants this unit adds ([invariant-requirements.md](../../../spec/v0.2/core/invariant-requirements.md) §U-CORE-6)

| # | Invariant | Where | Notes |
|---|---|---|---|
| **6.1** | `MessageNotUnderstood < Error < Object`; parallel rule holds for both new rows (extends R-INV-0.2). | **H** (`verify_invariants`) + **C** | Boot: assert `error.superclass == Object`, `mnu.superclass == Error`, and `X.class.superclass == X.superclass.class` for both. Corpus: same via handle identity + a user subclass of `Error`. |
| **6.2** | A genuine miss (dNU not overridden) raises a **surface** `MessageNotUnderstood` carrying the `Message`, `isA(Error)`, **not** native `RuntimeError`. | **C** | The Rust corpus test above. |
| **6.3** | Only `Error` subclasses are raisable — `raise` lives on `Error` only. | **C** | Assert an `Error` (or subclass) instance responds to `raise` and `3` / a `String` does not (`respondsTo` via `Symbol.new("raise()")`, no `#…` literal needed). `throw 42` compile-rejection = deferred to the ADR-0031 error-syntax unit. |
| **6.4** | An overriding `doesNotUnderstand(_)` still intercepts **before** the default raise. | **C** | Guard `tests/lang/dispatch/dispatch_dnu_proxy_forwards.{ph,expected}` (Proxy `doesNotUnderstand` forwards via `perform`, `status: PASS`). **Fixture identity corrected this pass**: the prior grounding cited `dispatch/pending/dispatch_does_not_understand`; the actual, current, already-promoted fixture is `tests/lang/dispatch/dispatch_dnu_proxy_forwards.ph` (confirmed on disk, no `pending/` prefix, `// status: PASS`). [`pending-retirement.md`](../../../spec/v0.2/core/pending-retirement.md) line 93 still names the old `dispatch/pending/dispatch_does_not_understand` path — that doc has not caught up to the promotion (do not edit it; it is out of this unit's write-set). It is category-A green today; U-CORE-6 must keep it green. |
| **6.5** | Floor census (R-INV-0.1) updated in lockstep for the `message`/`raise` additions. | **C** | Bump the census audit's expected binding count to **+2 on top of whatever U-CORE-4 landed it at** (88 total per the header's floor-math box) and add the two `(Error, selector)` rows; amend [`floor-census.md`](../../../spec/v0.2/core/floor-census.md) §2 + §1.1 in the same change. Re-read the live count first — do not assume 86 as the pre-U-CORE-6 base without confirming U-CORE-4 has actually landed. |

**Boot vs corpus:** 6.1 → **H + C**; 6.2/6.3/6.4/6.5 → **C**.

---

## §5. Must-not-preclude ([forward-compat.md](../../../spec/v0.2/core/forward-compat.md) §2 — *the* section — + §1)

| Hazard (§2/§1) | How this design clears it |
|---|---|
| §2 — a *second, non-`Error`* error channel | The miss raises a `MessageNotUnderstood` **`< Error`**; `raise` is on `Error` only. Single channel, ADR-0008-conformant. |
| §2 — wiring dNU to a non-`Error` or to **host termination** | dNU raises a surface `Error` subclass **value** that propagates through the ordinary `PhResult`/`?` channel. Uncaught → the existing top-level render/exit (unchanged behavior), **not** a special `throw`-terminates-host path. |
| §2 — forking the unwind | The Raise payload is the **sibling** of U10's `Return` payload within the *one* unwind (ADR-0008), carried by the same `PhResult` the VM already threads. No second mechanism; `ensure`-on-any-unwind and `on(_)` (now spelled by [ADR-0031](../../../adr/0031-error-handling-surface-syntax.md)) layer over `RuntimeError::Raise { error, .. }` later. |
| §2 — `Result` shape incompatible with `Option` | `Result`/`Ok`/`Err` are **reserved, not built**; §0 pins them to the `Option` abstract-root + two-subclass shape so the later unit mirrors it. Nothing here shapes them. |
| §2 — `ensure` as exception-only | Not built here; but because Raise is a payload of the unified unwind (not a bespoke exception path), a later `ensure` that fires on *any* unwind (Return/Raise/abort) is additive. ADR-0031 §2's `ensure { … }` clause ("always runs — normal exit, throw, return, or abort") is exactly this — confirms this unit's design imposed no rework. |
| §1 — fiber captures a propagating `Error` into its result slot | The Raise payload **carries the surface `error` `Value`** (not a Rust string). A future `Fiber` boundary extracts that `Value` into its result slot; `throw` is never special-cased as host-process termination. |
| §1 — `Value` openness / frame-locality | No new `Value` arm (`Error`/`MNU` are ordinary `InstanceObject`s). The Raise propagates via `?`, touching no frame indices — it stays fiber-local when fibers arrive. |

**Reserved shapes to keep layerable (do not implement):** `Result`/`Ok`/`Err`
(abstract + two subclasses, `Option`-mirrored); `attempt`/`unwrap`/`okOr`/`ok`
bridges; `on(_)(_)`/`ensure(_)` block protocol and its now-ratified
[ADR-0031](../../../adr/0031-error-handling-surface-syntax.md) `try`/`on`/`catch`/`ensure`
keyword sugar; surface `DeadFrameError`/`TypeError`/`ArgumentError`/`RangeError`.

**PHASE2-INDEX ADR-0008 amendment note (folded in):** *"`MessageNotUnderstood` is the
default-dNU raise."* This unit realizes it: the default `doesNotUnderstand(_)` raises
a surface `MessageNotUnderstood` through the unified unwind.

**Staleness check, this pass:** ADR-0031 (error-handling surface syntax) was
ratified *during* this refresh (dated 2026-07-12, same day) — it retroactively
confirms this unit's `throw expr === expr.raise()` assumption and the
block-protocol shape exactly as designed. **No design change required**; this is
the "must-not-preclude" gate paying off, not a gap.

---

## §6. Open sub-decisions + traceability

### Sub-decisions (recommended, flag if deviating)

- **D1 — Raise payload placement.** *Recommended:* `RuntimeError::Raise { error:
  Value, rendered: String }` (inside `PhError::Runtime`). Rationale: `run_file`
  already maps `PhError::Runtime(_)` → exit 70, and `#[error("{rendered}")]` gives
  the uncaught-render path for free — **zero edits** to `runtime_error`/`run_file`.
  *Alternative:* a top-level `PhError::Raise(Value)` (purer payload) — cleaner for
  `on`/fiber but forces edits to `run_file`'s match and a special-case render in
  `runtime_error`. The **load-bearing** requirement either way: `error` is the raw
  surface `Value` (never let `rendered` become what `on`/fiber reads); `rendered` is
  a display cache only.
- **D2 — native accessors vs `.ph` getter.** *Recommended:* native `message`/`raise`
  (Message precedent). Rationale: a `.ph` `message { return _message }` trips the
  read-before-write check (`compiler/lib.rs:84`, re-verified unchanged) because the field is only *read*,
  never *assigned*, in the reopened body; and a `.ph` getter over a Rust-stamped slot
  couples the miss-path to `.ph` field order. This trades a strict-minimal floor
  (+1) for robustness (+2); both are ADR-0019 amendments regardless, and both are
  already named explicitly by ADR-0023's Decision §4 ("`Error#message`... and
  `Error#raise`... **+2 bindings**"), confirming D2 was the ruling this pass, not
  just a recommendation. *Migration:* once a real `.ph` `Error` construction path
  exists (user `throw`, later unit), `message` may move to `.ph` over a field the
  class also assigns.
- **D3 — `error_raise` render source.** *Recommended:* send `message` (honors a
  future computed override; re-entrancy is safe pre-unwind). *Alternative:* read
  slot 0 directly (fewer cycles, but ignores overrides). The default dNU builds
  `rendered` locally either way (no send).
- **D4 — retire `RuntimeError::MessageNotUnderstood`.** *Recommended:* remove it
  (single constructor). Grep-gate: if any non-dNU site references the variant, STOP
  and report before removing. (Re-grepped this pass: still exactly one constructor,
  `object_does_not_understand`.)

*No new design-level ADR is required* — ADR-0008 already governs the model, and
[`decisions.md`](../../../spec/v0.2/core/decisions.md) §Q2 confirms it. A
**floor-amendment landing-record ADR** *is* required (see below) — that is a
bookkeeping ADR (mirrors ADR-0028/ADR-0036), not a design ADR.

### ADR-0019 amendment — status update this pass

The omnibus [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)
(**Accepted**, 2026-07-12) already exists and already admits `Error#message`/`Error#raise`
to the floor **in principle**, verbatim: *"`Error#message`/`raise` (owner: U-CORE-6) —
`Error#message` (getter, native slot-0 accessor) and `Error#raise` (initiates the
unified unwind, producing a `RuntimeError::Raise` payload — plumbing, not itself a
bound selector). **+2 bindings.**"* This clears the ADR-0019 gate; it does **not**
itself land the binding (ADR-0023 is explicit: *"Each named primitive is still
*installed* only when its owning unit actually lands and bumps the census"*).

**What this unit must still do:** author its own **per-unit landing-record ADR**
— the same role [ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md)
played for U-CORE-3 and the currently-"Proposed" [ADR-0036](../../../adr/0036-amend-floor-admit-number-tostring.md)
is playing for U-CORE-4 — that:

1. Cites ADR-0023 as the in-principle clearance (no re-litigating derivability).
2. Records the actual landing: floor moves to *whatever U-CORE-4 left it at*, **+2**
   (the two `Error` bindings), and the concrete distinct-fn delta (+2:
   `error_message`, `error_raise`).
3. Updates [`floor-census.md`](../../../spec/v0.2/core/floor-census.md) §1.1/§2 and
   the R-INV-0.1 audit in the same change (mirrors how ADR-0036's Related-list
   cites "floor-census.md §1.1, §2.1, §2.4 (re-baselined in the same
   implementation change as this ADR)").
4. **Claims the next free ADR number at dispatch** — as of this pass, 0036 is the
   highest assigned (0034 was drafted then dropped, per `docs/adr/README.md`
   line 96: "An earlier draft ADR-0034... was dropped"), so **0037** is the
   expected next slot; re-run `ls docs/adr/` immediately before creating the file,
   since a concurrent session may claim it first.

### Traceability

| Claim / requirement | Source |
|---|---|
| Confirm ADR-0008; U-CORE-6 = minimal reification | [`decisions.md`](../../../spec/v0.2/core/decisions.md) §Q2; [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md) |
| `Error`/`MNU`/… catalog rows (`message`, `raise`) | [object-model.md](../../../spec/v0.2/object-model.md) §4 "Errors" (lines 170-179, was cited 160-165) |
| Raise / handling as block protocol; `throw expr === expr.raise()`; only-`Error` throwable | [error-handling.md](../../../spec/v0.2/error-handling.md) §1-2, §4 |
| Surface keyword spelling for `throw`/`try`/`on`/`catch`/`ensure` (ratified this pass) | [ADR-0031](../../../adr/0031-error-handling-surface-syntax.md) (Accepted, 2026-07-12) |
| One unwind primitive (Return vs Raise payloads) | [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md); U10 spec §2 |
| `Result`/`Ok`/`Err` reserved, `Option`-mirrored | [values-and-absence.md](../../../spec/v0.2/values-and-absence.md) §4; [ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md) |
| dNU/`Message` reification the raise wires to | [floor-census.md](../../../spec/v0.2/core/floor-census.md) §2.14; [catalog-delta.md](../../../spec/v0.2/core/catalog-delta.md) §2.7/§4.5 |
| Floor amendment pre-cleared in principle (+2 bindings) | [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md) Decision §4; sibling per-unit landing records [ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md) (U-CORE-3), [ADR-0036](../../../adr/0036-amend-floor-admit-number-tostring.md) (U-CORE-4, Proposed) |
| Invariants R-INV-6.1…6.5 | [invariant-requirements.md](../../../spec/v0.2/core/invariant-requirements.md) §U-CORE-6 (verbatim-matched this pass, no drift) |
| Must-not-preclude (errors + fibers) | [forward-compat.md](../../../spec/v0.2/core/forward-compat.md) §2, §1 |
| Fixtures `errors_throw_try_catch_finally` / `errors_result_bridge` | [pending-retirement.md](../../../spec/v0.2/core/pending-retirement.md) §4 |
| R-INV-6.4 guard fixture — corrected identity | `tests/lang/dispatch/dispatch_dnu_proxy_forwards.ph` (confirmed on disk, `status: PASS`, no `pending/` prefix — supersedes the stale `dispatch/pending/dispatch_does_not_understand` name still in `pending-retirement.md` line 93) |
| U-CORE-3 → U-CORE-6 error-reification hand-off | [`U-CORE-3/as-built.md`](../U-CORE-3/as-built.md) §0.2 (lines 62-65), R-INV-3.4 (line 543), §589-591 |
| open-Q9 resolved by ADR-0008 + ADR-0031 | [open-questions.md](../../../spec/v0.2/open-questions.md) §9 (`~~struck~~`, → ADR-0008; surface spelling → ADR-0031) |
| Current miss path (files) | `primitive/object.rs:219` ⚠ (was `:140`); `vm.rs:487,530,690` (was `:467,510,632`); `error.rs:63,82,138-139` (unchanged); `interpret.rs:125` (unchanged) |
| Reopen preserves stamped `field_count` | `compiler/lib.rs:734` (existing-class path, unchanged); `vm.rs:142-148` (`Some` stamp, unchanged) |
| Read-before-write hazard | `compiler/lib.rs:84` (unchanged), field collection `compiler/lib.rs:~660-730` (was cited `584-730`; drifted, re-verify at dispatch) |
