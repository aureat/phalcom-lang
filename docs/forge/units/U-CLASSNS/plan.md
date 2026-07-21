# U-CLASSNS — Work order: class identity becomes module-scoped

_Unit **A** of two implementing [PDR-0001](../../../pdr/0001-classes-are-closed.md)
(ruling 1). Unit B is [`U-CLASSCLOSE`](../U-CLASSCLOSE/plan.md) and **must land after this
one** — its redefinition error is undecidable without module-scoped identity._

> **Why two units.** This one reaches a **dispatch path** (`SuperSend`); unit B does not. Two
> gates rather than one so a red tree names its own cause. Ruled by the user 2026-07-19.

---

## 1. Mission (one sentence)

Re-key class identity from **name** to **(module, name)** so two modules that each declare
their own `Point` are two unrelated classes, not one — in the VM and in the LSP.

---

## 2. Preconditions (verify on actual HEAD — do not assume)

All verified 2026-07-19 against `main`. Re-confirm before writing code.

1. **Four VM tables are name-keyed, VM-global** (`phalcom-core/src/vm/mod.rs`):
   `classes: HashMap<Symbol, ClassId>` (:101), `field_layouts: HashMap<Symbol, ClassLayout>`
   (:136), `class_parents: HashMap<Symbol, Symbol>` (:172),
   `sealed_classes: HashMap<Symbol, ObjRef>` (:194).

2. **Bindings are already module-scoped.** `define_global` (`vm/api.rs:128`) writes into the
   module object's own globals. The asymmetry — scoped bindings, unscoped identity — is the
   defect.

3. **The bug reproduces.** `modp.ph` and `modq.ph` each declaring `class Point { who => ... }`:

   ```phalcom
   import "modp" as P
   import "modq" as Q
   P.Point.new().who     // "from modq"
   Q.Point.new().who     // "from modq"
   P.Point == Q.Point    // true
   ```

4. **`file = module`** — ADR-0045 keeps ADR-0027's premise. There is one import form,
   `import "path" as Name` (`phalcom-ast/src/parser.rs:511` hard-requires `as`).

5. **Two runtime readers of `vm.classes`, not one.** `dispatch.rs:768` (the reopen fallback —
   unit B deletes it) and **`dispatch.rs:885` (`SuperSend`)**, which probes by bare name every
   super send. This is the only dispatch-path reader that survives.

6. **`sealed_classes` ownership carries no conflict.** `install_core` creates the core module as
   `m` (`vm/bootstrap.rs:175`); `run_core_module` fetches the same handle back via
   `get_module_from_str` (`vm/bootstrap.rs:167` → `vm/api.rs:117`, a plain
   `modules.get(&sym).copied()`). So the compiler's `self.module` while compiling core.ph **is**
   `m`. `DEFERRED` #35's predicted bootstrap-vs-core.ph ownership hazard is not real — both
   writers write the identical value. **Verified 2026-07-19; do not re-derive.**

7. **LSP keeps its own name-keyed index.** `ClassMap { by_class: DashMap<String,
   Vec<ClassEntry>> }` (`phalcom-lsp/src/index.rs:147-149`). `ClassEntry` already carries
   `uri: Url` (:136). The LSP does not resolve `import` at all (`Statement::Import` is a no-op
   in every walker: `index.rs:630`, `completion.rs:308`, `semantic_tokens.rs:397`), so `Url` is
   already its correct and only proxy for "module."

---

## 3. Design

### 3.1 VM — re-key the four tables

Key on `(ObjRef /* module */, Symbol /* name */)`. Prefer a named `ClassKey` newtype over a
bare tuple so the ~40 call sites read as intent, not as positional pairs.

`sealed_classes` is the odd one: it is name-keyed but **module-valued** today, and its value is
consumed by the `sealed_in_module != self.module` check (`compiler/lib/class_decl.rs:366`).
Re-keying does not change that check's meaning — per precondition 6 the kernel rows are written
identically by both writers — but the value's role must be preserved: the key says *which class*,
the value says *who may subclass it*. Do not collapse the two.

### 3.2 VM — `SuperSend` (the dispatch-path tentacle)

`dispatch.rs:885` becomes a `(module, defining_sym)` probe. The module is already available in
that loop — the `Import` arm reads `self.heap.closure(closure_id).module` the same way.

**Do not optimize this here.** The site's comment justifies the name lookup by a future
`superclass=` mutation that PDR-0001 makes impossible, so the probe could in principle be
replaced by a stamped `ClassId`. That is deliberately deferred —
[`docs/deferred/class-sealing-followups.md`](../../../deferred/class-sealing-followups.md) item
1 — because it is an unmeasured dispatch-path change and this is a correctness gate. Re-key
only.

### 3.3 LSP — collapse, don't just re-key

`by_class`'s `Vec<ClassEntry>` exists to model **one class reopened across several files**. Under
0065 that case cannot occur: a class belongs to exactly one module. So the target is
`DashMap<(Url, String), ClassEntry>` (or `DashMap<Url, DashMap<String, ClassEntry>>`) — one
entry, not a vector.

This fixes two **live** wrong-answer bugs, independent of anything else in this unit:

- `ClassMap::members` (`index.rs:166-179`) unions members across every file declaring the name,
  de-duping first-seen-wins. `p.<cursor>` on file A's `Point` can offer file B's members.
- `ClassMap::parent` (`index.rs:181-186`) is `.find_map(|e| e.parent.clone())` — returns the
  first entry with an `extends`, so file B's superclass answers queries about file A's `Point`.

`ClassMap::remove_uri` (`index.rs:156-162`) already filters by `entry.uri != uri`, so the
invalidation path is not a blocker — it tolerates the multi-entry shape today and simplifies
under the new one.

`WorkspaceIndex::class_members` / `class_parent` / `has_class` (`index.rs:355-373`) grow a
uri/module parameter; `collect_class_members` (`completion.rs:449-474`) threads it from
`Backend::completion` (`backend.rs:626-640`), which already has the request `uri` in scope.

**Out of scope:** `core_table.rs`'s `classes: HashMap<String, Vec<CoreMember>>`
(`core_table.rs:92`). Kernel classes only, process-global, no per-module identity in the VM
either. Legitimately name-keyed — leave it.

### 3.4 `ClassLayout` gains a declaration span (DEC-CLASSNS-A — ruled 2026-07-19, option (i))

Unit B's diagnostic is `X is already defined` carrying **both** spans (0065 ruling 2). Today
`ClassLayout` (`vm/mod.rs:28`) is `{ name, field_slots, field_count, static_field_slots,
static_field_count }` — no span anywhere, and no sibling map records one, so the *first*
declaration's location is unrecoverable.

**Add a `SourceRange` field to `ClassLayout`, in this unit.** Rejected alternatives: a separate
`(module, name) → SourceRange` map owned by unit B (the same key twice, with no separation of
concerns to justify it, and unit B would be modifying a struct this unit just rewrote); and
degrading to a single-span diagnostic (contradicts a ruling).

Populate it at the one `field_layouts.insert` site (`class_decl.rs:424`) from `class_def.range`,
already in scope there. Unit B reads it; this unit only stores it — **no diagnostic work here**,
and the field is dead until B lands. That is intentional: it keeps the struct rewrite in one
unit.

### Rubric — hazards & preclusion (mandatory)

| Hazard | Mitigation |
|---|---|
| `class_parents` feeds the ctor-inherit guard chain-walk (`class_decl.rs:899,935`), already flagged fragile, and `DEC-CTOR-H` schedules that guard for deletion in U-CTOR-4 | Re-key mechanically; **do not harden the guard**. If the walk needs more than a key change, stop and flag rather than redesign it here. |
| Kernel rows are inserted from Rust (`api.rs:76-77`, `bootstrap.rs:186,265`) with no ambient "current module" | They belong to the core module handle `m`; thread it explicitly at those sites rather than defaulting. |
| Re-key silently changes who may subclass a sealed class | Precondition 6 proves the kernel rows are unaffected. Add an invariant test asserting `sealed_classes` for `Option`/`Some`/`None` still resolves to the core module after the change. |
| A bare tuple key makes ~40 sites positionally ambiguous | Use a `ClassKey` newtype. |
| Scope creep into unit B | This unit does **not** error on redefinition. Two modules' `Point` become distinct; a *same-module* duplicate keeps today's behavior until unit B. Resist fixing it here — the diagnostic wants both spans and that is B's design. |

---

## 4. Confirmed write-set (re-validate with `rg` / `graphify affected` on HEAD)

| Path | Why |
|---|---|
| `phalcom-core/src/vm/mod.rs` | the four table declarations + docs |
| `phalcom-core/src/vm/api.rs` | `create_class` inserts (:76-77); `define_global` untouched |
| `phalcom-core/src/vm/bootstrap.rs` | `add_class!` (:186), `None` row (:265), sealed rows (:223,228,269) — all keyed to `m` |
| `phalcom-core/src/compiler/lib/class_decl.rs` | every `classes`/`field_layouts`/`class_parents`/`sealed_classes` access (:277-292, :308-428, :755, :899, :935) |
| `phalcom-core/src/vm/dispatch.rs` | `SuperSend` probe (:885); `Bytecode::Class` (:768) re-keyed **in place**, deleted by unit B |
| `phalcom-lsp/src/index.rs` | `ClassMap`/`ClassEntry` + 5 inherent methods + 3 pub wrappers + insert/remove sites (~12 production, ~10 test) |
| `phalcom-lsp/src/completion.rs` | `collect_class_members` signature + 3 call sites (:449-474) |

**Not** in the write-set: `core.ph` (this unit does not touch it — so there is **no conflict
with U-BINDINGS** in either order, despite U-BINDINGS codemodding 111 sites there),
`core_table.rs`, `hover.rs`, `backend.rs` beyond threading one `uri`.

---

## 5. Build order (small, independently-green diffs)

1. `ClassKey` newtype + re-key `field_layouts` alone (compile-time only, no runtime reader).
2. Re-key `classes`, including the Rust-side kernel inserts. Green here means bootstrap survives.
3. Re-key `class_parents` + `sealed_classes`; add the sealed-kernel invariant test.
4. `SuperSend` probe (`dispatch.rs:885`). **The only dispatch-path diff — commit alone.**
5. LSP `ClassMap` collapse + API threading.

Commit per green step ([`commit-frequently`]); never commit a non-compiling tree. Verify each
SHA in a throwaway worktree, not in-tree — an in-tree gate hides partial-stage commits.

---

## 6. Mandatory rules

- Full rustdoc on every touched public item ([`docs/rust-documentation-guidelines.md`](../../../rust-documentation-guidelines.md)); `cargo doc --workspace --no-deps` clean.
- `./scripts/verify.sh` exits 0.
- `graphify update . --no-cluster` after the diff.
- Narrow-path commits on `main` itself. **Never** `git checkout -b` and never `git add -a` —
  this repo has live concurrent sessions.

---

## 7. Test strategy (the green gate must assert)

**The unit's reason to exist**, positive lane — two modules, same class name, no interaction:

```phalcom
import "modp" as P     // class Point { who => "from modp" }
import "modq" as Q     // class Point { who => "from modq" }
System.print(P.Point.new().who)     // from modp
System.print(Q.Point.new().who)     // from modq
System.print(P.Point == Q.Point)    // false
```

Plus: the importer declaring its own `Point` leaves `P.Point` intact; `SuperSend` resolves the
right parent when two modules each have a `Base`/`Derived` pair with identical names; kernel
sealing still resolves to the core module (invariant test, per the rubric); LSP unit tests for
`class_members`/`class_parent` with two same-named classes in two `uri`s — the case
`index.rs`'s current test module never exercises.

Error fixtures go in the **negative** subdir or the suite reddens.

---

## 8. Decisions flagged (flag, don't pick)

**None open.** All eight rulings are settled in PDR-0001, and `DEC-CLASSNS-A` was ruled
2026-07-19 — see §3.4, the span lives on `ClassLayout` and this unit adds it.

Two things are ruled *out* of this unit and must not be quietly pulled in: `SuperSend`
`ClassId` stamping and the post-bootstrap freeze/H17 measurement (both
[`class-sealing-followups.md`](../../../deferred/class-sealing-followups.md) items 1-2). Two things are ruled *out* of this
unit and must not be quietly pulled in: `SuperSend` `ClassId` stamping and the post-bootstrap
freeze/H17 measurement (both
[`class-sealing-followups.md`](../../../deferred/class-sealing-followups.md) items 1-2).

---

## 9. Must-not-preclude check

- **Unit B** needs `(module, name)` to make its redefinition error decidable. Delivered.
- **Reflection layer** (0065 ruling 7) — user classes only. Module-scoped identity is what makes
  "user class" a checkable predicate rather than a naming convention.
- **`SuperSend` stamping** — a later stamped-`ClassId` optimization must stay open. Re-keying the
  probe does not close it.
- **`DEFERRED` #35** — precondition 6 resolves its ownership unknown; the sealing-representation
  unification stays reachable.
- **U-CTOR-4** — `class_parents` is scheduled for guard deletion. Re-key without hardening.

---

## 10. Return contract (report to `phalcom-reviewer`)

Per-step SHAs with `git show --stat`; the two-module reproduction before/after; confirmation
that `./scripts/verify.sh` and `cargo doc` are clean at each SHA verified **in a throwaway
worktree**; the LSP test added for the two-same-named-classes case; explicit confirmation that
`core.ph` is untouched and that `SuperSend` was re-keyed only, not optimized.
