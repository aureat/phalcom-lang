# U7 — Work order: static instance fields + `construct` (dispatch-ready)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Replaces the dynamic
per-instance field map with a **fixed per-class slot layout**, adds the `construct` initializer
protocol, and (scope-permitting) class-side static state. **Reviewer OFF** for this unit — it is not in
the load-bearing set (STATE.md review policy); **self-verify on the green gate** (`./scripts/verify.sh`)
+ `cargo doc`. Grounded in **ADR-0011** (static per-class instance slot layout), spec = `docs/spec/classes.md`
§1–2 + `docs/spec/object-model.md` §5. STATE.md ADR mapping is authoritative._

---

## 0. Mission (one sentence)
Give instances a **fixed slot vector** (`Box<[Value]>`) indexed by a per-class field table computed at
class-definition time (ADR-0011), make **read-before-write a compile error**, and add the **`construct`**
initializer protocol (alloc + body + implicit `return self`) as a method on the **metaclass**.

## 1. Hard guardrails (read before writing any code)
- **This replaces the dynamic field map with a static layout — a representation + feature change, not a
  rewrite of dispatch.** Fields become `GetField(slot)` / `SetField(slot)` array indices, not symbol
  probes. Do not touch selector encoding or method lookup (U3 owns them).
- **Fields are private and NOT inheritance-visible (classes §2).** A subclass that writes `_name` gets
  its **own new slot**; it never renumbers or shares the superclass's slots. This is what keeps offsets
  permanently stable — preserve it exactly.
- **Read-before-write analysis is whole-class, not per-method-local.** A field assigned in method A and
  read in method B is legal; a field read in *no* assignment set anywhere in the class is a compile error.
- **An unassigned slot reads `None`, backed by the private `Value::Nil` sentinel — reuse U6's surfacing
  helper; never leak the sentinel** (Invariant 4).
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md). Self-verify only on green — do not expand scope.

## 2. Preconditions (verify first; do not assume)
- **U1 merged + green** — `InstanceObject`/`ClassObject` already migrated to the Heap + `Copy` handles +
  tagged `Value`. U7 changes the *field storage shape* (`IndexMap` → `Box<[Value]>`) on that substrate.
- **U2 (metaclass tower + `verify_invariants`) merged + green** — `construct` declares a method on the
  **metaclass**; static/class-side dispatch requires the corrected parallel tower. If U2 is not landed,
  `construct` cannot be placed correctly — **STOP**.
- **U6 (absence → Option) merged + green** — the "unassigned field reads `None`" surfacing helper and
  the `None` singleton come from U6. U7 reuses them; it does **not** re-implement absence.
- Confirm `./scripts/verify.sh` is green on the base before the first edit (baseline).
- Re-run `graphify affected "InstanceObject"`, `graphify affected "GetField"`, and
  `graphify explain "ClassDef"` on the actual HEAD to confirm nothing new sits outside §3.

## 3. Confirmed write-set (from `graphify affected` on instance/field/construct symbols + source read)
| File | Why it's in scope |
|---|---|
| `phalcom-ast/src/token.rs` | Add `Token::Construct`. |
| `phalcom-ast/src/lexer.rs` | Lex the `construct` keyword (`_field` names already lex as identifiers, `lexer.rs:159`). |
| `phalcom-ast/src/ast.rs` | Add `ClassMember::Construct(ConstructDef)` (selector name + params + body). Field decls stay **implicit by assignment** — no field node needed; `Expr::Field` already exists. |
| `phalcom-ast/src/parser.rs` | Parse `construct` members (`parse_class_member`, `parser.rs:545`). |
| `phalcom-core/src/instance.rs` | `InstanceObject { class, slots: Box<[Value]> }` — replace `fields: IndexMap<Symbol, Value>` (`instance.rs:11`); slot-indexed accessors. |
| `phalcom-core/src/class.rs` | Per-class **field table** (`Symbol → slot offset`) + `field_count`, computed once at class-definition time; stored on the class object. |
| `phalcom-core/src/compiler/lib.rs` | Whole-class field-collection pass → assign slot offsets; lower `Expr::Field` read/write to `GetField(slot)`/`SetField(slot)` (replaces symbol-keyed `lib.rs:412,430`); **read-before-write compile error**; `construct` lowering (alloc + body + implicit `self` + implicit `return self`), declared on the metaclass. |
| `phalcom-core/src/bytecode.rs` | Change `GetField`/`SetField` operand from constant-index(symbol) to **slot(u16)**; add an allocation opcode (`NewInstance(class)` / `Alloc`) for `construct` if not already present. |
| `phalcom-core/src/vm.rs` | Execute slot-indexed `GetField`/`SetField`; execute `construct`/alloc; surface an unassigned slot read as `None` (reuse U6 helper). |
| `phalcom-core/src/primitive/{object,class}.rs` | Allocation primitive backing `construct`; reflection touching the new slot shape. |
| `phalcom-core/bin/phalcom/disasm.rs` | Slot-aware disassembly of `GetField`/`SetField`/alloc. |
| `core/core.ph` | Any bootstrap class that now uses `construct`/fields. Shared file → **sequence after U6/U-STD's `core.ph` edits**, never parallel. |

## 4. Design decisions (ADR-0011 / classes.md / object-model §5 — realize, don't re-litigate)
- **Instance slot layout (ADR-0011).** `InstanceObject { class, slots: Box<[Value]> }`, indexed by a
  **compile-time slot offset** from a per-class field table computed once at class-definition time.
  Field reads/writes compile to `GetField(slot)` / `SetField(slot)` — a direct array index, no symbol
  lookup. The dynamic `IndexMap<Symbol, Value>` is removed.
- **Whole-class field collection.** The compiler collects the set of `_`-prefixed names **assigned
  anywhere** in the class body, fixes the slot layout, and stores it on the class. This is what makes
  **read-before-write a compile error**: a field read whose name is in no assignment set is rejected
  (catches the `_naem` typo class).
- **Private, non-inherited fields (classes §2).** A subclass's fields occupy **fresh** slots and never
  renumber the parent's — offsets are permanently stable. Cross-hierarchy field access goes through
  accessors, not shared slots.
- **Unassigned slot → `None`.** Backed by the private `Value::Nil` sentinel (ADR-0010), surfaced `None`
  via U6's helper. Never leaked.
- **`construct` (classes §1, object-model §5).** A keyword; `ClassMember::Construct`. Lowering: (1) emit
  allocation of a fresh instance of the class, (2) run the body with `self` bound to it, (3) implicit
  `return self`. Declared as a method **on the metaclass** (class-side send). Multiple constructors are
  distinguished by **selector** (`encode_selector`, U3) — `new(name:age:)` vs `new(name:)` are two
  selectors, no arity hacks. **No implicit zero-arg `new`, no user-visible allocator** (`let i =
  self.new(); i.init(...)` is never written).
- **Static *methods/getters*** (`static species => …`, classes §3) already flow through the existing
  `is_static` flag + metaclass dispatch (U2/U3) — verify they still work under the new layout; they need
  no stored slot.

### BLOCKED-ON-DECISION — BD-U7-1: class-side *stored* static fields
The unit title says "static fields," but **class-side stored fields are unspecified**:
- `classes.md` §3 shows only `static species => "Homo sapiens"` — a computed **getter**, no stored state.
- **ADR-0011's "static" means "compile-time-fixed slot layout for *instances*"**, *not* class-side
  (`static` keyword) state — a naming collision. It does **not** cover per-class stored fields.
- `object-model.md` §6 says class-side *methods* obey the tower rules; it is silent on class-side
  *stored* state.

So a stored, mutable per-class field (e.g. `static _count = 0`) has **no spec or ADR coverage**. Options:
- **(A) Uniform application of ADR-0011 to the class object.** A class is an instance of its metaclass;
  give the class object its own slot vector for static fields, indexed by a per-**metaclass** field
  table (the exact ADR-0011 mechanism, one level up the tower). Clean and uniform, but currently
  unspecified — needs a NEW ADR.
- **(B) Descope static stored fields from U7 (RECOMMENDED).** Ship instance fields + `construct` +
  static methods/getters — **all fully spec'd** — and defer static *stored* fields to a follow-up unit
  gated on a new ADR. Keeps U7 tight, fully spec-grounded, and green-verifiable on its own (per the
  "small, independently-verifiable units" guardrail).

**Recommendation: (B) — descope, and propose a NEW ADR** ("class-side field storage on the metaclass
instance," extending ADR-0011 up the tower) drafted via the `documentation-and-adrs` skill before any
class-side stored-field work. **Do not pick unilaterally.** Instance fields + `construct` proceed
regardless of BD-U7-1.

## 5. Build order (land as one coherent diff)
1. **`phalcom-ast`** — `Token::Construct` (`token.rs`, `lexer.rs`); `ClassMember::Construct` (`ast.rs`);
   parse it (`parser.rs`). Full rustdoc.
2. **Field table + layout** — `class.rs` per-class `Symbol→slot` table + `field_count`; `instance.rs`
   `slots: Box<[Value]>`. Cite ADR-0011.
3. **Bytecode** — `bytecode.rs` `GetField(slot)`/`SetField(slot)` + alloc opcode; `disasm.rs` slot-aware.
4. **Compiler** — whole-class field-collection pass → offsets; lower `Expr::Field` to slot ops;
   read-before-write compile error; `construct` lowering onto the metaclass.
5. **VM** — slot-indexed field exec; `construct`/alloc exec; unassigned slot → `None` (U6 helper).
6. **Primitives + `core.ph`** — allocation primitive; migrate any bootstrap class using fields/construct.
7. **Tests** — goldens + negatives (§7-tests).

## 6. Fold-in cleanup (only if fully inside this write-set)
U7 touches `phalcom-ast/src/parser.rs`; **DEFERRED #2/#3 are assigned to U6** (which lands first). Do
not double-fold. If U6 left either open and it is `parser.rs`-local, `graphify affected` first, then
fold only if trivial. Do not touch DEFERRED #1 (U1 owns it).

## 7. Mandatory rules
- **Docs** ([`docs/rust-documentation-guidelines.md`](../rust-documentation-guidelines.md)): `//!` on
  every touched module; `///` on every new public item (`Token::Construct`, `ConstructDef`, the class
  field table, `slots`, the new `GetField`/`SetField` operands, the alloc opcode, read-before-write
  diagnostic) with ADR-0011 citations + intra-doc links. `cargo doc --workspace --no-deps` adds **no new
  warnings**.
- **Green gate (self-verify — no reviewer):** `./scripts/verify.sh` exits 0 (build + test + clippy +
  golden + invariants). Golden output byte-identical where unchanged. Don't add clippy warnings; fix
  pre-existing ones in files you rewrite.
- **Tests the harness must assert:**
  - Positive golden: a `Person` with `construct new(name:age:)`, getters/setters (`name`, `name=`),
    round-tripped; a declared-but-unassigned field read yields `None`; multiple constructors
    (`new(name:age:)` vs `new(name:)`) dispatch by selector; `construct` returns `self`.
  - Negative golden (must fail to **compile**): reading `_naem` (a field assigned nowhere) → compile
    error; and a program that tries a user-visible zero-arg `new`/allocator where none exists.
  - Layout invariant: a subclass writing `_name` gets a fresh slot and does not disturb the superclass's
    slot offsets (offset stability under inheritance).

## 8. Return contract (self-report; no independent reviewer)
Report: the slot-layout representation (`Box<[Value]>` + per-class field table) · the whole-class
field-collection + read-before-write algorithm · `construct` lowering + metaclass placement · confirmation
the private sentinel never leaks and unassigned reads surface `None` via U6's helper · **BD-U7-1 status**
(was static *stored* fields descoped? was the new ADR drafted?) · goldens/negatives added with
`verify.sh` tail · `cargo doc` tail · offset-stability evidence · any new `DEFERRED.md` entries.
