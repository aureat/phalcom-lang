# Forge — Phase 2 (Planning) Index

_All remaining Phalcom units now have dispatch-ready work orders. This is the master map:
plan roster, dependency graph, write-set collision matrix, open-decision register, and the
new-ADR / spec-edit backlog. Landed: U-FE (ADR-0016), U0, U3 (ADR-0012), U1 (ADR-0009/0010),
**U2 (ADR-0002/0003, 2026-07-11 — reviewer gate skipped this pass, see U2-progress.md)**,
**U4 (ADR-0013/0006, 2026-07-11 — reviewer gate ON, caught a stubbed runtime on the first cut,
closed in a follow-up pass, see STATE.md)**, **U5 (ADR-0018, 2026-07-11 — reviewer OFF per policy)**,
**U6 (ADR-0007/0014/0021, 2026-07-11 — reviewer gate ON, BLOCKed once on inlined≠non-inlined body
result, fixed in `51f56e4`, PASSED, see STATE.md)**,
**U7 (ADR-0011/0017, 2026-07-11 — reviewer OFF per policy, see STATE.md)**,
**U-LIST (ADR-0019/0020, 2026-07-11 — ratified then landed same session, reviewer OFF per policy,
see STATE.md; also fixed a pre-existing bug that made `core.ph` inert)**,
**U8 (ADR-0012, 2026-07-12 — `b99ad22`/`806c9ea`, reviewer OFF per policy, see STATE.md;
`doesNotUnderstand(_:)`/`perform`/`respondsTo` + `Message` reification + `send_dynamic`)**,
**U9 (ADR-0012amd, 2026-07-12 — in-tree, no worktree, reviewer OFF per policy, see STATE.md;
rest params `*name` + `SignatureKind::Variadic`/`(*)` selector + call-prologue rest-arg collapse
+ derived-selector miss probe)**.
**U10 ✅ LANDED (2026-07-12) — non-local return. Stopped at the U10 hard boundary — U-LEX/U-STD/U11 not dispatched yet.**
Planning completed 2026-07-11 by 6 parallel `phalcom-architect` agents._

## 1. Unit plan roster (each is a self-contained work order)
| Unit | Plan | Mission (1-line) | Spec / ADR | Reviewer |
|---|---|---|---|:--:|
| U1 | [U1-plan.md](U1-plan.md) | handle/arena heap + tagged `Value` (behavior-preserving migration) | 0009/0010 | ✅ |
| U2 | [U2-plan.md](U2-plan.md) · **✅ LANDED** [U2-progress.md](U2-progress.md) | metaclass tower parallel rule + `Behavior` kernel + `verify_invariants()` | 0002/0003 · object-model §5–6 | **skipped this pass** |
| U4 | [U4-plan.md](U4-plan.md) · **✅ LANDED** (see STATE.md) | first-class blocks/closures, Lua open/closed upvalues, frame-token infra | 0013/0006 · blocks.md | **✅ ran, caught stubbed runtime, fixed** |
| U5 | [U5-plan.md](U5-plan.md) · **✅ LANDED** (see STATE.md) | control-flow-as-message + sacred-selector inliner w/ deopt guard | control-flow.md · **0018** | — (reviewer OFF per policy) |
| U6 | [U6-plan.md](U6-plan.md) · **✅ LANDED** (see STATE.md) | absence → `Option`, `let`/`var`, no surface `nil`, `if(opt)` rejected | 0007/0014 · **0021** · values-and-absence.md | **✅ ran, BLOCKed on inlined≠non-inlined, fixed, PASSED** |
| U7 | [U7-plan.md](U7-plan.md) · **✅ LANDED** (see STATE.md) | fixed instance slot layout + `construct` initializer + class-side stored static fields | 0011/**0017** · classes.md | — (reviewer OFF per policy) |
| U-LIST | [U-LIST-plan.md](U-LIST-plan.md) · **✅ LANDED** (see STATE.md) | minimal kernel `List` — native array floor + thin `.ph` protocol | 0019/0020 (**Accepted**) · messages/method-lookup | — (reviewer OFF per policy) |
| U8 | [U8-plan.md](U8-plan.md) · **✅ LANDED** (see STATE.md) | `doesNotUnderstand(_:)` / `perform` + `send_dynamic` (opcode deferred to U9) | 0012 · method-lookup.md | — (reviewer OFF per policy) |
| U9 | [U9-plan.md](U9-plan.md) / [U9-implementation-spec.md](U9-implementation-spec.md) · **✅ LANDED** (see STATE.md) | variadics — rest params `*xs`, `SignatureKind::Variadic`/`(*)` selector, call-prologue rest-arg collapse, derived-selector miss probe (reused `ClassObject.methods`, no new table) | 0012amd · messages-and-selectors.md §4 | — (reviewer OFF per policy) |
| U10 | [U10-plan.md](U10-plan.md) / [U10-implementation-spec.md](U10-implementation-spec.md) · **✅ LANDED** (see STATE.md) | non-local return — `return` inside a block unwinds to the home method via frame token (`Bytecode::ReturnNonLocal` + eager unwind + `DeadFrameError`) | 0013 · blocks.md §5 | — (reviewer OFF per policy) |
| U11 | [U11-plan.md](U11-plan.md) | Bool tower: abstract `Bool` + singleton `True`/`False` | 0004 | — |
| U-LEX | [U-LEX-plan.md](U-LEX-plan.md) / [U-LEX-implementation-spec.md](U-LEX-implementation-spec.md) · **✅ LANDED (2026-07-12)** (see STATE.md) | surface-syntax delta: block comments `/* … */`, digit separators `1_000_000`, lexer-level newline suppression, `?.`/`??` coverage (U6-landed, fixture only), `\(expr)` string interpolation | 0016 · **0022** · lexical-structure.md | — (reviewer OFF per policy) |
| U-STD | ✅ **LANDED (2026-07-12)** [U-STD-implementation-spec.md](U-STD-implementation-spec.md) | **Built Option (B) per user ratification** — the `Option`+`List` combinator layer (map/flatMap/filter/ifSome/unwrapOr; map/reduce/filter/includes/isEmpty/at(_:put:)), pure `.ph`, zero new primitives. The plan's Object/Number/String/Symbol/System scope was ~90% already-landed or re-carved to future `U-CORE-N` (see DEFERRED #29). | catalog-delta §2.2/§2.4 | — |

Reviewer ON = load-bearing → independent `phalcom-reviewer` gate (STATE.md policy): **U1, U2, U4, U6**.

## 2. Dependency graph (build order)
```
U1 (heap) ──┬─> U2 (tower) ──┬─> U7 (fields/construct)
            │                └─> U11 (Bool tower)
            ├─> U4 (blocks) ──┬─> U5 (control-flow+inliner)
            │                 ├─> U10 (non-local return)
            │                 └─> U9 (variadics)
            ├─> U6 (Option/let-var)
            └─> U3 (landed) ──> U8 (dNU/perform)

Serial spine:  U1 → U2 → U4 → U5 → U6 → U7 → U-LIST
Wave F (parallel, disjoint):   U8 ‖ U-LEX ‖ U-STD  → then U10
Wave F+1 (parallel):           U9 ‖ U11
```
**U-LIST (NEW, ratified 2026-07-11 · DEC-A = "land minimal List first"):** a minimal kernel `class List`
(construct/`at`/`size`/`add`/`each`/`toString`; no map/reduce/literals) — a **hard prerequisite of both
U8** (`Message.args`/`labels`, `perform(_:List)`) **and U9** (rest-params collect into a `List`). Schedule
it at the **spine tail, before Wave F**. It edits `core.ph` + `primitive/mod.rs` (+ maybe `list.rs`) →
**never co-schedule with another `core.ph` editor.**

**U-LIST storage design gate — CLEARED (2026-07-11):** ADR-0019 (freeze the VM-blessed primitive
floor) and [ADR-0020](../adr/0020-kernel-list-native-array-protocol.md) (native `Vec<Value>` behind
the handle/arena `Heap`, five floor primitives — not six, "grow" folds into `push` — protocol
authored in `.ph`) were both ratified **Accepted** by the user, then U-LIST landed the same
session. See STATE.md's "U-LIST — LANDED" section for the as-built design and a codebase-wide
`core.ph`-execution bug it found and fixed along the way.
`core.ph` is a single shared file edited additively along U6 → U-STD → U11 — the wave order
already serializes those touches; **never co-schedule two `core.ph` editors**.

## 3. Write-set collision matrix (the real parallelism constraint)
**`phalcom-ast` (parser/AST) is contended by FIVE units** — they cannot share a wave:
| Unit | Why it touches `phalcom-ast` |
|---|---|
| U4 | block literals + trailing-block sugar (blocks were NOT in the parser — brief was wrong) |
| U5 | `if`/`while` desugaring to block sends (**iff decision DEC-E = "U5 owns"**) |
| U6 | `var`, `??`, `?.`, drop surface `nil`, `let` mutability |
| U7 | `construct` keyword + `ClassMember::Construct` |
| U-LEX | block comments, digit separators, string interpolation, `?.`/`??` tokens |

The serial spine (U1→U2→U4→U5→U6→U7) already serializes the spine's `phalcom-ast` edits.
**U-LEX must run alone in `phalcom-ast`** — schedule it in Wave F only if no spine `phalcom-ast`
work is concurrent, or give it its own serialized slot.

Other latent hazards:
- **`primitive/object.rs` — U-STD ✕ U8** both want it in Wave F. U-STD is instructed to avoid it
  and sequence-after-U8 if an `Object`-level primitive proves unavoidable.
- **`core.ph` — U6 / U-STD / U11** serialized by wave order; keep edits additive.
- **`vm.rs` / `compiler/lib.rs` / `bytecode.rs`** shared by nearly every spine unit → spine stays serial.

## 4. OPEN DECISIONS — need the user before the affected sub-feature can be built
_Each unit's bulk proceeds regardless; only the named sub-feature waits._
| ID | Unit | Decision | Architect recommendation |
|---|---|---|---|
| **DEC-A** ✅ **RESOLVED + IMPLEMENTED (user, 2026-07-11)** — storage design sub-gate ✅ **ADR-0019/0020 ratified, U-LIST landed same session** | U8, U9, U-STD | **Kernel `List` unscheduled but a hard dep** of dNU (`Message.args`) and rest-params. | **U-LIST landed** (see STATE.md), unblocking U8/U9/U-STD's `List` dependency. |
| **DEC-B** ✅ **RESOLVED + IMPLEMENTED (2026-07-12, landed with U9)** | U9 | **Variadic dispatch table key** — messages §4 (`key by (name, min_arity)`) isn't implementable as written (a call of arity K needs `min ≤ K`, an exact-tuple hash can't answer). | Key by **bare name**, via the canonical `<name>(*)` selector spelling in the *existing* `ClassObject.methods` map — **no new table** (U9-implementation-spec.md §0 point 2/3 refines "reject a 2nd same-name variadic" to "silently overwins", same as any duplicate-selector redefinition today; DEFERRED #24). |
| **DEC-C** ✅ **RESOLVED (user, 2026-07-11 → Option A, landed with U6)** | U6 | **How is `if(opt)` a compile error?** No static/flow analysis exists; general static detection impossible. | **(A)** runtime no-coercion floor (branch opcode requires `Bool`; Option never implements branch protocol) **+** compile-time rejection of *syntactically-literal* Option conditions. **→ shipped as [ADR-0021](../adr/0021-no-truthiness-enforcement.md).** |
| **DEC-D** ✅ **RESOLVED + IMPLEMENTED (user, 2026-07-11; landed with U7, `f38e591`)** | U7 | **Class-side _stored_ static fields** were unspecified (ADR-0011 "static" = instance layout, not class-side state — naming collision). | **→ INCLUDE in U7 (option A): apply ADR-0011 up the tower** — class object gets its own `static_slots` indexed by a per-*metaclass* field table. [ADR-0017](../adr/0017-class-side-stored-static-fields.md) **Accepted** and landed: `static_slots` + metaclass field table wired, offset-stability-up-the-tower proven (`subclass_static_field_offset_stability`). See U7-plan §3, STATE.md "U7 — LANDED". |
| **DEC-E** | U5 / U-LEX | **Who owns `if`/`while`/`for` surface parsing?** No control-flow AST node exists today. Sets the U5↔U-LEX write-set boundary in `phalcom-ast`. | **U5 owns** tightly-scoped parse-time desugaring to block sends (adds `phalcom-ast` to U5's write-set). |
| **DEC-F** ✅ **RESOLVED (user, 2026-07-12 → `\(expr)`, overriding the architect's `{expr}` recommendation)** | U-LEX | **String-interpolation sigil** (open-Q5): `{expr}` / `${expr}` / `\(expr)`. | User ratified **`\(expr)`** (Swift-style). Requires a new ADR + rewriting `pending/lexical_string_interpolation.ph` and spec §5's `{expr}` examples off the old sigil before U-LEX's D4 slice builds. |

Soft flags (architect can proceed on the recommendation; confirm if you disagree):
- **U8:** ✅ **RESOLVED (landed 2026-07-12).** `perform` primitive-only; spread call sites `f(*args)` deferred to U9. The `Bytecode::SendDynamic` opcode was **also** deferred to U9 (nothing emits/decodes it this unit) — U8 shipped the `VM::send_dynamic` helper only (DEFERRED #21).
- **U9:** ✅ **RESOLVED (landed 2026-07-12).** Block variadics `{ *xs => }` deferred — no such grammar exists (block-literal params use a separate scanner never reached by `parse_param_list`), so nothing silently mis-parses; DEFERRED #9 stays open for a future unit.
- **U5 (BD-U5-2):** `repeat(_:)` semantics + unary-operator selector names unpinned → implement unambiguous sacred selectors first, defer `repeat`.
- **U-STD:** open-Q2 Int/Float split unresolved → write `Number` against the abstract numeric protocol so the split isn't foreclosed.

## 5. New ADRs / spec edits to draft (via `documentation-and-adrs`)
| Item | Kind | Owner unit | When |
|---|---|---|---|
| **ADR-0018** — sacred-selector inliner + override-epoch deopt guard | ✅ landed | U5 | landed with U5 (0017 was taken by class-side static fields) |
| **ADR-0012 amendment** — variadic dispatch-table key + `_...` selector spelling | amendment | U9 | before U9 (DEC-B) |
| **ADR-0021** — no-truthiness enforcement (typed branch floor + literal-only compile check) | ✅ landed | U6 | landed with U6 (DEC-C = Option A) |
| **Class-side field storage on the metaclass instance** (ADR-0011 up the tower) | new ADR (**REQUIRED**, DEC-D=A) | U7 | **before** U7's static-stored-field slice |
| **ADR-0022** — string-interpolation `\(expr)` sigil (open-Q5, ratified `\(expr)` 2026-07-12) | ✅ landed | U-LEX | landed with U-LEX D4 (DEC-F resolved) |
| Collection-literal lowering `(a,b)`/`[…]`/`{a:1}` | new ADR | (deferred; needs List) | with collections unit |
| **ADR-0008 amendment note** — `MessageNotUnderstood` = default-dNU raise | note | U8 | with U8 |
| **ADR-0002 pointer note** — `Rc::new_cyclic` superseded by 0009 handle-patching | fold-in edit | U2 | with U2 |
| **ADR-0003 status** — flip "Open question pending" → Accepted | fold-in edit | U2 | with U2 |
| **object-model.md §3/§4 reconciliation** — contradicts ADR-0004 (True/False visibility) | spec edit (doc-only) | U11 | with U11 |

## 6. Cross-unit corrections the architects made to the original briefs
- **Blocks were NOT already in the parser/AST** (U4 brief assumed they were) → U4 write-set includes `phalcom-ast`.
- **Block invocation protocol is `call`/`arity`** (functions.md §1–2), **not** Smalltalk `value`/`value:` → all block-send phrasing corrected to spec.
- **U4/U10 boundary** made crisp: U4 = closures/upvalues/`call` + frame-token *infrastructure* (ships no non-local-return); U10 = `ReturnNonLocal` opcode + unwind + `DeadFrameError`.
- **U2 rewrites 3 currently-green invariant tests** (they encode the collapsed F6 apex) — not just "un-ignore 2".
- **F4 (`object_name`/instance `toString`, ADR-0015)** scoped OUT of U2 → needs a home unit (see DEFERRED).
