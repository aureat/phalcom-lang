# §05 — Stage 4: live oracle and structured selectors (§D8 + §D9)

**Phase B — branch-only.** `phalcom-repl/**` only. `phalcom-lsp` is consumed as a
library and **never modified**.

This stage builds no user-visible surface. It builds the data [§06](06-surface.md)
renders. Keeping them separate is deliberate: the snapshot is testable without a
terminal, the surface is not.

## 1. §S1 — the snapshot

Access to VM state is an **immutable snapshot**, rebuilt once per cell boundary — after
`unwind_cell()` (§D10), before the next prompt renders.

```rust
pub struct ReplSnapshot {
    /// Name → class of its current value.
    globals: HashMap<Symbol, ClassId>,
    /// Full inheritance chain per class, walked once, own-depth tagged.
    members: HashMap<ClassId, Vec<Member>>,
    /// Class-valued globals, for class-side receivers.
    classes: HashMap<Symbol, ClassId>,
}
```

**Rejected — live borrow of the `Universe`.** Exact, but a keystroke query borrows a VM
that is not guaranteed at rest once a cell can leave a fiber suspended (§D10 preserves
suspended fibers deliberately).

**Rejected — reflective self-hosting** (`x.class.methods` as real sends). Elegant and
self-describing, but executes user-reachable code on a keystroke. That is Node's
getter-side-effect trap; DevTools needed V8's `throwOnSideEffect` to make console
completion safe. A snapshot has no such surface. **Do not reintroduce this** — it will
look like a simplification.

Staleness is bounded by construction: nothing executes between cells, so a snapshot taken
at the boundary is exact for the entire editing session that follows it.

### 1.1 Own-depth is free

The chain walk that builds `members` tags depth as it goes. §S3's ranking needs it and
must not walk again.

## 2. §D8 — two sources, one interface

Static (`phalcom-lsp`) is the floor: the line being typed has not executed, so only a
parser can speak to it. Live (the snapshot) overrides wherever it has an answer.

Live provides what static structurally cannot:

- **receiver class for any bound name** — ask the value its class, walk the real method
  dictionary (the Smalltalk path). Static's `ConstructResolver` handles only
  `let x = Cls.new(…)`.
- **unbound-name detection** — a name absent from `name_to_slot`, locals, and globals.
  A pre-flight `doesNotUnderstand`. Static cannot know what has *run*.
- **runtime-added methods**, and immunity to `core-table.json` drift.

**Merge rule: live wins for names that exist at runtime; static covers the current,
not-yet-executed line.**

### 2.1 §S2 — static sees the current line only

`phalcom-lsp` is fed **only the line being typed**, never an accumulated buffer of prior
cells.

The reason is PDR-0001 ruling 6. Cells *shadow*: a later `class Foo` binds a new
class. Replaying all cells as one synthetic document would present two `class Foo`
declarations to the index — and under PDR-0002 that is `class.already_defined`, a
hard error. Reconstructing shadowing inside the static layer would reimplement cell
semantics in a second place, with a second chance to get it wrong.

Consequence worth stating plainly: **the §D8 merge rule never actually fires.** Static
answers syntax for text that has not run; live answers everything that has. They do not
overlap, so "live wins" reduces to "no conflict is reachable." Implement the merge as a
lookup order, not as a conflict resolver.

## 3. §D9 — structured selectors

Structured `{name, labels, kind}` on both sides. The VM side uses the existing **total,
non-panicking** decoder (precondition 11 — `encode_selector` emits comma-form at
`method/mod.rs:102`, and the `doesNotUnderstand` path already round-trips it). The LSP
side already has `MemberKind` + `ParameterDef`s.

The UI renders from structure — bare `size`, `name = value`, `at(_)`. Dedup on
`(name, kind, arity)`.

**Rejected:** canonicalising to an encoded string and re-parsing it to render. Lossy
round-trip for no gain.

Kinds to reconcile: **getter / setter / method / subscript / variadic**.
`Initializer` is **not** among them (precondition 12): a `construct` installs one method,
class-side, under its ordinary selector (`new(_)`), not an `init`-prefixed one; the kind
survives only as a `SuperSend` marker (`compiler/lib/class_decl.rs:602`). Verified:
`B.methods → [#x]`, `B.class.methods → [#new(_)]`. An implementer who offers
`Initializer` in completion is reading a retired concept.

Variadics work end-to-end (precondition 13: `static sum(*nums)` called as `sum(1,2,3)`
yields `[1,2,3]`), and **zero variadic definitions exist in `core.ph`** — so the variadic
rendering path has no live example to test against in the core library. Write a fixture
class for it.

## 4. Tests

All testable without a terminal — this is why the stage is separate from [§06](06-surface.md).

| Test | Asserts |
|---|---|
| `snapshot_reflects_globals_after_cell` | a global bound in cell 1 appears with the right `ClassId` |
| `snapshot_tags_own_depth` | `List`'s own members are depth 0, `Iterable`'s 1, `Object`'s deeper |
| `snapshot_is_stable_between_cells` | a snapshot taken at a boundary does not observe later mutation |
| `unbound_name_detected` | a never-declared name is absent from all three maps |
| `selector_conformance_live` | **see below** |
| `initializer_kind_never_offered` | a `construct`ed class offers `new(_)` class-side, no `Initializer` |
| `variadic_selector_renders` | a `static sum(*nums)` fixture renders correctly |

**`selector_conformance_live`** walks every core class's **live** method dictionary and
asserts each renders to correct comma-form under §D9. It is live-authoritative and
deliberately does **not** diff against `core-table.json` — that file is ~40% stale
(30→52 classes, 214→305 entries) and its regeneration is a separate, spun-off task. This
test carries no dependency on it, and must not acquire one.

## 5. Write-set

| Path | Change |
|---|---|
| `phalcom-repl/src/snapshot.rs` | **new** — `ReplSnapshot`, chain walk, depth tagging |
| `phalcom-repl/src/oracle.rs` | **new** — the two-source lookup order (§2) |
| `phalcom-repl/src/repl.rs` | rebuild the snapshot at each cell boundary |
| `phalcom-repl/src/main.rs` | `mod snapshot; mod oracle;` |

Two new modules rather than growing `completer.rs`: [§06](06-surface.md) rewrites that
file wholesale, and data that outlives the rewrite should not live inside it.

**Conflict risk vs class work: none.**

## 6. Gate

Workspace green, 28 suites + seven new tests, 0 failures.

No manual check — nothing user-visible ships in this stage. If something is visible,
scope has leaked from [§06](06-surface.md).
