# U7 — Work order: fixed instance slot layout + `construct` (RE-GROUNDED, post-U4)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. **Reviewer OFF**
(STATE.md review policy) — self-verify on the green gate (`./scripts/verify.sh` exits 0) + `cargo doc`
clean. Grounded in **ADR-0011** (static per-class instance slot layout), spec = `docs/spec/classes.md`
§1–2 + `docs/spec/object-model.md` §5. **This revision supersedes the phase-2 draft** and re-grounds
every file/line reference against `main` at HEAD (post-U1/U2/U3/U4), assuming **U5 + U6 have landed**._

---

## 0. Re-grounding delta (what changed since the phase-2 draft — READ FIRST)
The original draft was written before U1–U4 landed and referenced a pre-heap tree. Verified against
HEAD on 2026-07-11:

| Draft claim | Actual HEAD | Consequence for U7 |
|---|---|---|
| Fields in `IndexMap<Symbol, Value>` | ✅ still true — [`instance.rs:19`](../../phalcom-core/src/instance.rs#L19) `pub fields: IndexMap<Symbol, Value>` | The representation change is fully needed; nothing pre-empted it. |
| Add slot(u16) operand to `GetField`/`SetField` | Opcodes **already `GetField(u16)` / `SetField(u16)`** ([`bytecode.rs:36,38`](../../phalcom-core/src/bytecode.rs#L36)) but the u16 is a **constant-index of the field `Symbol`**, emitted at [`compiler/lib.rs:561,582`](../../phalcom-core/src/compiler/lib.rs#L561). | **Opcode shape does not change** — only the *meaning* of the operand (const-idx → slot). Smaller diff than the draft implied; `disasm.rs` label text changes, not the arity. |
| Selector construction via `signature.rs` | **Wrong file.** The encoder is `encode_selector` / `make_signature` in [`method.rs:82,134`](../../phalcom-core/src/method.rs#L82). `signature.rs` is a thin module; the live encoder is `method.rs`. | All selector work routes through `crate::method::{encode_selector, make_signature}` (already imported at `compiler/lib.rs:18`). |
| `construct` needs a brand-new selector kind | **`SignatureKind::Initializer(u8)` already exists** ([`method.rs:25`](../../phalcom-core/src/method.rs#L25)) and `encode_selector` already renders it as `init name(labels:)` ([`method.rs:84`](../../phalcom-core/src/method.rs#L84)). | **`construct` maps onto `SignatureKind::Initializer` — do not invent a kind.** The signature machinery already anticipates constructors; only the parser + lowering are missing. |
| Unassigned slot surfaces via "U6 helper" | U6 (Option/`None` + surfacing helper + private `Value::Nil` sentinel) is **assumed landed** this revision. `core.ph` today has only `class Nil {}` ([`core.ph:13`](../../phalcom-core/core/core.ph#L13)); U6 adds `Option`/`Some`/`None`. | **Hard precondition: confirm U6 merged** (see §2). Reuse its helper; do not re-derive absence. |
| `construct` on the metaclass via new machinery | Class-side install already works: `parse_class_member` handles `is_static` ([`parser.rs:548`](../../phalcom-ast/src/parser.rs#L548)); the `Method(selector_idx, is_static)` opcode installs on the metaclass; `create_class` wires the tower (U2, [`vm.rs:152`](../../phalcom-core/src/vm.rs#L152)). | `construct` is a **class-side initializer**: parse it, encode with `Initializer(n)`, install with `is_static = true`. No new install path. |

**Net:** U7 is a **representation change (IndexMap → slot vector) + one parser keyword + one lowering
pass**, reusing the existing `Initializer` selector kind and the existing static-install path. It does
**not** touch selector encoding, method lookup (U3), or the tower (U2).

## 1. Mission (one sentence)
Replace the per-instance `IndexMap<Symbol, Value>` with a **fixed `Box<[Value]>` slot vector** indexed by
a **per-class field table** computed once at class-definition time (ADR-0011), make **read-before-write a
compile error**, add the **`construct`** initializer (parse → `SignatureKind::Initializer` selector →
class-side lowering: alloc fresh instance + run body with `self` bound + implicit `return self`), **and
extend the same slot mechanism one level up the tower to give class objects their own stored static
fields** (per user decision on DEC-D — see §3, requires a new ADR authored first).

## 2. Preconditions (verify on actual HEAD — do not assume)
- **U1/U2/U3/U4 landed** (they are, per STATE.md). Native code takes `&Heap`/`&mut Heap`; no `Rc<RefCell>`.
- **U6 landed** — `Option`/`Some`/`None` classes in `core.ph`, the private `Value::Nil` sentinel, and the
  **absence-surfacing helper** that maps the sentinel → `None`. U7 reuses that helper verbatim for
  unassigned-slot reads. **If U6 is not merged, STOP** — the "unassigned field reads `None`" behavior has
  no backing and the golden will be wrong.
- **U5 landed** — U5 owns `if`/`while`/`for` parse-time desugaring and adds those keywords to
  `phalcom-ast`. U7 does not depend on control-flow *semantically*, but U7 **adds another `phalcom-ast`
  keyword (`construct`) on top of U5's + U6's parser edits** — rebase and re-locate the insertion points
  in `parser.rs`/`token.rs`/`lexer.rs` against post-U5/U6 HEAD (they will have moved).
- Baseline: `./scripts/verify.sh` green **before** the first edit.
- Re-run `graphify affected "InstanceObject"`, `graphify affected "GetField"`,
  `graphify explain "ClassObject"` on real HEAD to confirm nothing new sits outside §4.

## 3. Design (ADR-0011 / classes.md §1–2 / object-model.md §5 — realize, don't re-litigate)
Applying the language-design rubric (object-model: *instance layout* axis; dispatch: *selector identity*
via `Initializer`):

- **Slot layout.** `InstanceObject { class: ClassId, slots: Box<[Value]> }`. Reads/writes compile to
  `GetField(slot)` / `SetField(slot)` — a direct array index, no `Symbol` probe. The `IndexMap` is removed.
- **Per-class field table.** `ClassObject` gains `field_slots: IndexMap<Symbol, u16>` (name → offset) +
  `field_count: u16`, computed **once** at class-definition time and stored on the class. Offsets are
  assigned in first-assignment order over the whole class body.
- **Whole-class field collection = the read-before-write mechanism.** The compiler collects every
  `_`-prefixed name **assigned anywhere** in the class body (all methods/getters/setters/`construct`),
  fixes the layout, then lowers reads. **A field *read* whose name is in no assignment set anywhere in the
  class is a compile error** (catches the `_naem` typo). A field assigned in method A and read in method B
  is legal (whole-class, not per-method-local).
- **Private, non-inherited fields (classes §2) — the soundness keystone.** A subclass that writes `_name`
  gets its **own fresh slot** in *its* table; it never renumbers or aliases the superclass's slots.
  Cross-hierarchy access goes through accessors, not shared offsets. This is *why* offsets are permanently
  stable — preserve it exactly. (Instance layout = superclass slots `[0..k)` then own slots `[k..k+m)`;
  a subclass appends, never rewrites.)
- **Unassigned slot → `None`.** `InstanceObject::new` fills `slots` with the private `Value::Nil` sentinel
  (ADR-0010). A `GetField` on an unwritten slot surfaces `None` **via U6's helper** — the sentinel is
  never leaked (Invariant 4). **Do not default slots to a `None` *instance*** — that would reintroduce the
  bootstrap-absence cycle U6 solved; the raw sentinel is the whole point.
- **`construct` (classes §1, object-model §5).** Keyword → `ClassMember::Construct(ConstructDef)` with a
  selector name + labelled params + body. Encode the selector with **`SignatureKind::Initializer(n)`**
  (already renders `init new(name:age:)` in `method.rs`). Lowering:
  1. emit allocation of a fresh instance of the class (`slots` = `field_count` sentinels),
  2. bind `self` to it, run the body,
  3. **implicit `return self`**.
  Install **class-side** (`is_static = true` path → metaclass). Multiple constructors are distinguished by
  **selector** — `new(name:age:)` and `new(name:)` are two `Initializer` selectors, **no arity hacks, no
  default args** (see hazard below). No implicit zero-arg `new`, no user-visible allocator.
- **Static methods/getters** (`static species => …`) already flow through `is_static` + metaclass dispatch
  (U2/U3) and need no slot — verify they still work under the new layout.

### Rubric — hazards & preclusion (mandatory)
- **Soundness (bootstrap-absence cycle):** unassigned slots MUST hold the raw `Value::Nil` sentinel, not a
  constructed `None` object. Constructing `None` requires a class whose fields default to absence → cycle.
  U6 already blessed the sentinel; U7 must not undo that by eagerly materializing `None` into slots.
- **Dispatch impact:** `construct` reuses `Initializer` selectors; **no change to selector encoding or
  lookup.** Confirmed `encode_selector` already handles `Initializer`. Do not touch U3's path.
- **Representation impact:** `IndexMap` → `Box<[Value]>` is the point — removes a per-field hash probe and
  makes fields IC-/slot-friendly for later. One allocation per instance (`field_count` wide).
- **Identity-dispatch ⊗ optional arity (catalog hazard):** because constructor identity is the
  `Initializer` selector, **any future default/optional arg on `construct` would change the selector and
  miss** — exactly the default-args⊗selector-identity trap. U7 ships **no default args**; flag that adding
  them later is a cross-cutting decision, not a local one.
- **Preclusion (mandatory step-5):** fixed slot layout + private-non-inherited fields **forecloses**
  (a) adding a field to a *live* class / `become:`-style reshape (offsets are frozen at definition), and
  (b) shared *protected* inherited fields (subclasses must use accessors). Both are acceptable per ADR-0011
  and *good* for a future inline cache (stable offsets). Record in DEFERRED if either is ever wanted.
- **Precedent:** Smalltalk/Ruby use fixed instance-variable slots with subclass-appended layout (same
  stable-offset discipline); Self/JS prototypes pay dynamic-shape cost to avoid it. We take the fixed side
  deliberately (ADR-0011).

### DECISION — DEC-D: class-side *stored* static fields → **INCLUDE (option A), user-ratified 2026-07-11**
`static _count = 0` (mutable per-class state) had **no spec/ADR coverage** — ADR-0011's "static" means
*compile-time-fixed instance layout*, not `static`-keyword class state (naming collision); classes.md §3
shows only computed getters. **The user chose (A): apply ADR-0011 uniformly one level up the tower.**

- **Uniform mechanism.** A class *is* an instance of its metaclass. So a **class object gets its own slot
  vector** (`ClassObject.static_slots: Box<[Value]>`) for stored static fields, indexed by a
  **per-*metaclass* field table** — the *exact* ADR-0011 mechanism, shifted up one tower level. Instance
  fields index by the class's `field_slots`; static fields index by the metaclass's `field_slots`.
- **`static _count = 0` lowering.** A `static`-marked `_`-prefixed assignment collects into the
  **metaclass** field table (not the class's). `static _count` reads/writes compile to slot ops against the
  **class object's** `static_slots`, not `self`'s. Same whole-class collection + read-before-write rule,
  keyed to the static assignment set. Unassigned static slot → `None` (same sentinel + U6 helper).
- **Offset stability up the tower.** A subclass's metaclass appends its own static slots after its
  super-metaclass's; static offsets are as permanently stable as instance offsets (same non-inherited
  private-field discipline, one level up).

**PREREQUISITE — a NEW ADR must be authored *before* the static-stored-field code** (via
`documentation-and-adrs`): *"Class-side field storage on the metaclass instance (ADR-0011 up the tower)."*
It records: static fields live on the class object's own slot vector; indexed by the metaclass field table;
`static _count` is class-state, not instance-state; offset-stability + `None`-default carry up unchanged;
and the `static` keyword's two meanings (compile-time layout vs class-side storage) are reconciled. **Do
not write the static-stored-field code until this ADR is Accepted.** Instance fields + `construct` +
static methods/getters proceed *regardless* and can land first; the static-stored-field slice lands behind
the ADR as the final step of U7 (or a fast follow-up if the ADR is still in review when the rest is green).

## 4. Confirmed write-set (re-validate with `graphify affected` on post-U5/U6 HEAD)
| File | Why it's in scope |
|---|---|
| `phalcom-ast/src/token.rs` | Add `Token::Construct`. |
| `phalcom-ast/src/lexer.rs` | Lex the `construct` keyword (`_field` names already lex as identifiers). |
| `phalcom-ast/src/ast.rs` | Add `ClassMember::Construct(ConstructDef)` (name + labelled params + body). No field node — fields stay implicit by assignment; `Expr::Field` already exists. |
| `phalcom-ast/src/parser.rs` | Parse `construct` in `parse_class_member` (~L546; **re-locate post-U5/U6**). |
| `phalcom-core/src/instance.rs` | `InstanceObject { class, slots: Box<[Value]> }` — replace `fields: IndexMap` (L19); slot-indexed accessors; `new(class, field_count)` fills sentinels. |
| `phalcom-core/src/class.rs` | `ClassObject` gains `field_slots: IndexMap<Symbol, u16>` + `field_count: u16` (instance layout) **and `static_slots: Box<[Value]>` for class-side stored fields** (DEC-D); helper to resolve a name → slot (own table only, non-inherited), for both the class's `field_slots` and the metaclass's field table. |
| `phalcom-core/src/compiler/lib.rs` | Whole-class field-collection pass → assign instance offsets **and a parallel `static`-field collection → metaclass field table + class-object static offsets** (DEC-D); lower `Expr::Field` read (L557) + write (L577) to slot ops; **`static _field` reads/writes lower to the class object's `static_slots`, not `self`**; **read-before-write compile error** (both sets); `construct` lowering (alloc + body + implicit `return self`), installed class-side via `Initializer` selector. |
| `phalcom-core/src/bytecode.rs` | `GetField`/`SetField` operand meaning: const-idx → **slot(u16)** (arity unchanged); add an alloc opcode (`NewInstance(class)`) if none exists for `construct`. |
| `phalcom-core/src/vm.rs` | Execute slot-indexed `GetField`/`SetField`; execute `construct`/alloc; surface unassigned slot as `None` via U6 helper. |
| `phalcom-core/src/primitive/{object,class}.rs` | Allocation primitive backing `construct`; reflection touching the new slot shape. |
| `phalcom-core/bin/phalcom/disasm.rs` | Slot-aware label text for `GetField`/`SetField`/alloc (operand is now a slot, not a symbol constant). |
| `core/core.ph` | Any bootstrap class that now uses `construct`/fields. **Shared file — sequence after U5/U6's `core.ph` edits, never parallel.** |
| `CERTAIN: error.rs` | `CompilerError` variant for read-before-write (unassigned field read) with a span, if not reusing an existing one. |

## 5. Build order (land as one coherent diff)
_Steps 1–7 are the fully-spec'd core and may land first; step 8 (static stored fields) gates on the new
ADR (DEC-D) and lands last within U7 — or as an immediate follow-up if the ADR is still in review._
1. **`phalcom-ast`** — `Token::Construct`; `ClassMember::Construct(ConstructDef)`; parse it. Full rustdoc.
2. **Field table + layout** — `class.rs` `field_slots`+`field_count`; `instance.rs` `slots: Box<[Value]>`.
3. **Bytecode** — `GetField(slot)`/`SetField(slot)` semantics + alloc opcode; `disasm.rs` slot labels.
4. **Compiler** — whole-class field-collection pass → offsets; lower `Expr::Field` r/w to slot ops;
   read-before-write compile error; `construct` lowering via `Initializer` selector, class-side install.
5. **VM** — slot-indexed field exec; `construct`/alloc exec; unassigned slot → `None` (U6 helper).
6. **Primitives + `core.ph`** — allocation primitive; migrate any bootstrap class using fields/`construct`.
7. **Tests** — goldens + negatives (§7) for instance fields + `construct`.
8. **[gated on new ADR] Class-side stored static fields (DEC-D)** — author the ADR first
   (`documentation-and-adrs`); then `ClassObject.static_slots`; the parallel `static`-field collection →
   metaclass field table; lower `static _field` r/w to the class object's `static_slots`; VM exec;
   static-field goldens + negatives + offset-stability-up-the-tower test.

## 6. Mandatory rules
- **Docs:** `//!` on every touched module; `///` on every new public item (`Token::Construct`,
  `ConstructDef`, `field_slots`/`field_count`, `slots`, the alloc opcode, the read-before-write
  diagnostic) with ADR-0011 citations + intra-doc links. `cargo doc --workspace --no-deps` adds no new
  warnings.
- **Green gate (self-verify — no reviewer):** `./scripts/verify.sh` exits 0. Golden output byte-identical
  where unchanged. No new clippy warnings; fix pre-existing ones in files you rewrite. `rust-best-practices`.
- **Tests the harness must assert:**
  - **Positive golden:** a `Person` with `construct new(name:age:)`, getters/setters (`name`, `name=`),
    round-tripped; a declared-but-unassigned field reads `None`; two constructors (`new(name:age:)` vs
    `new(name:)`) dispatch by selector; `construct` returns `self`.
  - **Negative golden (must fail to *compile*):** reading `_naem` (assigned nowhere) → compile error; a
    user-visible zero-arg `new`/allocator where none exists → error.
  - **Layout invariant:** a subclass writing `_name` gets a fresh slot; the superclass's offsets are
    unchanged (offset stability under inheritance). Add to `tests/invariants.rs` if it fits the harness.
  - **[DEC-D] Static-field golden:** `static _count = 0` incremented across two instances reads shared
    class state (not per-instance); a `static`-declared-but-unassigned static field reads `None`; a
    subclass's static field does not disturb the super's static offsets (offset stability up the tower).

## 7. Return contract (self-report; no reviewer)
Report: slot-layout repr (`Box<[Value]>` + per-class `field_slots`) · the whole-class field-collection +
read-before-write algorithm · `construct` lowering + **confirmation it reuses `SignatureKind::Initializer`
and the class-side install path** (no new selector kind) · confirmation the `Value::Nil` sentinel never
leaks and unassigned reads surface `None` via U6's helper · **DEC-D status** (static *stored* fields
INCLUDED per user: was the new ADR authored + Accepted? is `static_slots` + metaclass field table wired?
offset-stability-up-the-tower proven?) · goldens/negatives + `verify.sh` tail · `cargo doc` tail ·
offset-stability evidence · any new `DEFERRED.md` entries.
