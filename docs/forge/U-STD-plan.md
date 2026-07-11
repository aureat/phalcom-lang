# U-STD — Work order: `core.ph` bootstrap standard library (dispatch-ready)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. **Reviewer OFF** —
self-verify on the golden corpus + `lang` labels + green gate. This is a **parallelizable Wave F leaf**
in its own worktree, disjoint from U-LEX (`phalcom-ast`) and U8 (`phalcom-core/src/vm`+`primitive/object`).
Source of truth: the class specs — [`object-model.md`](../spec/object-model.md),
[`classes.md`](../spec/classes.md), [`values-and-absence.md`](../spec/values-and-absence.md),
[`system.md`](../spec/system.md) — realised on the corrected object model (U2 tower, ADR-0015 toString)._

---

## 0. Mission (one sentence)
Grow `phalcom-core/core/core.ph` from the near-empty class stubs into the base-class **method surface** the
spec defines — the universal `Object` protocol, `Number`/`String`/`Symbol` behaviour, and the `System`
service surface — each method backed by an existing primitive or written in Phalcom, and each pinned by a
`lang`/golden corpus case.

## 1. Hard guardrails (read before writing any code)
- **`core.ph` is Phalcom source, not Rust.** "Docs" here = clear spec-referencing `//` comments on each
  class/section + the golden corpus staying green. Any *Rust* primitive glue you add still needs full rustdoc.
- **Build on the CORRECTED tower (U2) and landed absence model (U6).** Assume `Behavior`/`Class`/`Metaclass`
  are wired per object-model §5 and `verify_invariants()` passes; assume `Option`/`Some`/`None` exist.
  **Do not re-declare or re-wire kernel classes** — U2 owns the tower, U6 owns `Option`.
- **Do NOT own the `Bool` subtree.** `Bool`/`True`/`False` and their control selectors are **U11**
  (Wave F+1, ADR-0004). Leave `class Bool {}` as-is or minimally; add **no** `ifTrue:`/`and:`/`not` here.
- **Do NOT own `construct`/field syntax or semantics** — that is U7. Use only what U7 has landed.
- **`toString` default is fixed by BD-2 / ADR-0015:** `Object>>toString` renders `"<ClassName>"`; a class's
  own `toString`/`name` returns its own bare name (this is the F4 fix). Do not invent `printString`.
- **Additive, non-clobbering edits.** `core.ph` is a **shared file** across U6/U7 (landed) and U11 (later).
  Add methods to existing class bodies; never rewrite a class another unit owns. Keep your additions in
  clearly-commented, spec-referenced blocks so U11's later `core.ph` edit merges cleanly.
- **Collections (`List`/`Map`/`Set`/`Tuple`/`Range`) are OUT of scope** (§6 Deferred): they need primitive
  storage backing + literal syntax (U-LEX-deferred) that do not exist yet.

## 2. Preconditions (verify first; do not assume)
- Runs in an **isolated worktree off `main`** seeded from the committed green base **with U2 + U6 (and U7)
  merged in**. If the base lacks the corrected tower or `Option`, **STOP** — U-STD cannot verify without them
  (that is why it is scheduled at Wave F, after the spine).
- **graphify-first:** `graphify explain "Universe"`, `graphify explain "primitive"`, and
  `graphify affected "core.ph"` on HEAD to see which primitives already exist (`primitive/{object,number,
  string,symbol,system}.rs`) vs which methods must be pure Phalcom. Confirm what U6/U7 already added to `core.ph`.
- Read `phalcom-core/tests/lang/MANIFEST.md` and the existing `classes`/`messages`/`dispatch` cases to match
  the corpus conventions before adding cases.

## 3. Confirmed write-set (leaf lane; keep disjoint from U8's `primitive/object.rs`)
| File | Why it's in scope |
|---|---|
| `phalcom-core/core/core.ph` | The base-class method definitions (the unit's core deliverable). **Shared file — additive only.** |
| `phalcom-core/tests/lang/{classes,messages,dispatch,system}/*` | `.ph`/`.expected` corpus cases pinning the new surface (data only). |
| `phalcom-core/tests/lang.rs` | Un-ignore the `system` label once the `System` surface lands (single-line edit). |
| `phalcom-core/src/primitive/{number,string,symbol,system}.rs` | **Only if** a spec method needs new native backing. **Avoid `primitive/object.rs`** — U8 (dNU/`perform`) owns it in the same wave; if an `Object`-level primitive is unavoidable, **sequence after U8, do not edit in parallel** (report the conflict). |

## 4. Design decisions (grounded in the class specs)

### D1 — Universal `Object` protocol (object-model §8)
Define/confirm on `Object`: `class`, `isA(_:)`, `==(_:)`, `!=(_:)`, `hash`, `toString`, `respondsTo(_:)`,
`perform(_:_:)`, `doesNotUnderstand(_:)`. Most are primitive-backed (`primitive/object.rs`).
- **`toString` = `"<ClassName>"`** per ADR-0015 (BD-2). `==`/`!=` default to **identity**; value types
  (`Number`, `String`) override with value equality; `hash` stays consistent with `==`.
- **`doesNotUnderstand(_:)` / `perform`** are **U8's** semantics — reference, do not implement. If U8 hasn't
  landed the primitive yet, leave the Phalcom-level shape and note the dependency.

### D2 — `Number` (object-model §4; values-and-absence §1; ADR-0005 flat `f64`)
Arithmetic, comparison, and `toString` on `Number`, backed by `primitive/number.rs`. One flat `f64` type
(ADR-0005); the Int/Float split is **open-Q2 — do not pre-split**. Keep operator methods consistent with the
label-encoded selectors from U3 (operators-as-sends, ADR-0012).

### D3 — `String` (object-model §4; lexical-structure §5)
Immutable UTF-8 behaviour + `toString` (returns self) + `+` concatenation (the target of interpolation
desugaring from U-LEX). Backed by `primitive/string.rs`. Interpolation *lowering* is U-LEX's job; U-STD only
guarantees `String>>+` and `Object>>toString` exist so the desugar has a target.

### D4 — `Symbol` (object-model §4)
Interned identifier/selector behaviour; `==` by identity (interning), `toString`. Backed by `primitive/symbol.rs`.

### D5 — `System` service surface (system.md, object-model §4)
Class-side (`static`) methods on `System`: `print(_:)` (fix the current stub), plus `clock` and any other
surface [`system.md`](../spec/system.md) mandates. Backed by `primitive/system.rs`. Un-ignore the `system`
`lang` label once these pass.

## 5. Build order (each step verifies green before the next)
1. **`Object` protocol (D1)** — confirm primitive wiring, add the Phalcom-level methods; add `classes`/
   `messages` corpus cases asserting `x.class`, `isA(_:)`, `==`, and `toString == "<ClassName>"`. Verify green.
2. **`Number` (D2)** — methods + `arithmetic`/`messages` cases. Verify green.
3. **`String` (D3)** — methods + `+`/`toString` cases. Verify green.
4. **`Symbol` (D4)** — methods + cases. Verify green.
5. **`System` (D5)** — surface + un-ignore `system` label + cases. Verify green.
6. **Golden sweep** — confirm `core_new.ph` / `person2.ph` / `hello.ph` / `arithmetic.ph` stay byte-identical.

## 6. Deferred (append to `DEFERRED.md`, do not build here)
- **Collections** `List`/`Map`/`Set`/`Tuple`/`Range` — need primitive storage backing + literal syntax
  (both absent). A dedicated collections unit (with its own ADR for storage repr) should own them.
- **`Function`/`Block`/`Method` reflective surface** (functions.md) — depends on U4/U10 closure work; not a
  bootstrap-library concern yet.
- **`Fiber`/`Future`, `Error` hierarchy** — concurrency + error-handling units own these.

## 7. BLOCKED-ON-DECISION
- **None hard-blocking.** All in-scope surface (Object/Number/String/Symbol/System) is fully spec'd.
- **Watch (not a blocker):** open-Q2 (Int/Float split) must **not** be foreclosed — keep `Number` methods
  written against the abstract numeric protocol so a future `Integer`/`Float` split (object-model §4 note)
  can slot in without rewriting `core.ph`.

## 8. Mandatory rules
- **`core.ph`:** every added class/section carries a spec-referencing `//` comment (which spec § it realises).
  Methods stay consistent with U3 label-encoded selectors and the corrected tower.
- **Any Rust primitive glue:** full rustdoc (`///` + `# Errors`/`# Panics`), `cargo doc --workspace
  --no-deps` clean.
- **Green gate:** `./scripts/verify.sh` exits 0 — golden byte-identical, new `lang` cases pass, `system`
  label un-ignored and green, no new clippy warnings.
- **graphify update** `.` `--no-cluster` after edits.

## 9. Return contract (self-verify; reviewer OFF)
Report: classes/methods added to `core.ph` (grouped by spec §) · which were primitive-backed vs pure
Phalcom · corpus labels added/un-ignored · proof goldens stayed byte-identical (`verify.sh` tail) · explicit
confirmation you touched **no kernel wiring (U2), no `Option` (U6), no `Bool` subtree (U11), and not
`primitive/object.rs` (U8)** · any DEFERRED entries filed.
