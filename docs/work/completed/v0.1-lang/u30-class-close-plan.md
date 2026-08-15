# U-CLASSCLOSE — Work order: classes are closed after definition

_Unit **B** of two implementing [PDR-0001](../../../pdr/0001-classes-are-closed.md),
as amended by [PDR-0002](../../../pdr/0002-class-declarations-join-the-binding-namespace.md).
**Blocked on [`U-CLASSNS`](../U-CLASSNS/u31-classns-plan.md)** — the redefinition error below is
undecidable until class identity is `(module, name)`-keyed._

> **Re-grounded 2026-07-19 after U-BINDINGS landed** (`b843fe2`, `42aafce`; tree green).
> Three deltas, all from PDR-0002 — read it before this plan:
>
> 1. **Ruling 8's import half is already shipped.** `import "a" as P` twice now errors via
>    `declare_global` (`compiler/lib/scope.rs:181-182`). This unit inherits a **confirming
>    test**, not an implementation.
> 2. **A class declaration must register its name in `global_bindings`** (§3.6 below). Classes
>    are currently exempt (`compiler/lib/mod.rs:57-61`) *solely* so reopening and stub
>    completion survive L-5 — a reason this unit deletes. The exemption leaves a live
>    cross-kind hole, verified: `class Point {}` + `import "m1" as Point` silently clobber
>    either way round, the class-then-import direction surfacing as a **runtime** error
>    (`<module m1> does not understand 'new()'`) far from the offending line.
> 3. **The two-span diagnostic is new machinery, not a copy.** `CompilerError` has 14 variants;
>    five carry one `SourceRange`, **none carries two**, and `BindingRedeclared` carries zero.
>    Budget for it.

> Retires ADR-0026 **Axis 1** ("methods are open"). Axis 2 (superclass reparenting sealed) is
> kept and strengthened. ADR-0026 is already flipped to Retired in both trackers.

---

## 1. Mission (one sentence)

Make `class Foo { … }` mean exactly one thing — *define a class* — by removing class reopening,
reserving kernel names, gating kernel stub completion to the core module, and restricting class
declarations to module top level.

---

## 2. Preconditions (verify on actual HEAD — do not assume)

All verified 2026-07-19 against `main`. **U-CLASSNS must be landed and green first.**

1. **The kernel does not need reopening.** core.ph declares every class exactly once
   (`rg -o "^class (\w+)" core.ph | sort | uniq -d` → empty). `add_class!`
   (`vm/bootstrap.rs:180`) writes only `define_global` + `classes.insert`, **never**
   `field_layouts`. `field_layouts.insert` has one call site repo-wide
   (`compiler/lib/class_decl.rs:424`), reached only when a `.ph` class body compiles.

   So the compiler already discriminates: **stub completion** = `classes` hit +
   `field_layouts` miss (`class_decl.rs:332`); **true reopen** = `field_layouts` hit
   (`class_decl.rs:308`). core.ph rides the first and never the second. This unit does **not**
   restructure core.ph.

2. **Two seams to close.** Compile-time `class_decl.rs:277-292` (today rejects added fields and
   superclass change, reuses the layout otherwise) and runtime `dispatch.rs:768` (`classes` name
   hit → pushes the existing `ClassId`).

3. **Reopening is arbitrary runtime mutation**, not declaration-order layering. Class
   declarations nest inside method bodies and execute on call — live-confirmed:

   ```phalcom
   class Patcher { static patch { class List { size => 12345 } return "patched" } }
   System.print([1,2].size)    // 2
   System.print(Patcher.patch)
   System.print([1,2].size)    // 12345
   ```

4. **Reach is total** — live-confirmed reopening of `Object`, `Class`, `Metaclass`, `Behavior`,
   `Number`, `List`, and `@sealed Option`. Existing instances are affected (same `ClassId`).

5. **Override is last-writer-wins with no chain.** `add_method` is `IndexMap::insert`
   (`heap/class.rs:150`). A duplicate body *partially overwrites* — non-overlapping methods
   survive. Same for duplicate members inside one body: `bar => 1` then `bar => 2` yields `2`,
   silently.

6. **`Statement::Class` is position-unrestricted** — dispatched from the generic statement
   handler (`compiler/lib/mod.rs:342`). The parser imposes nothing.

7. **`None`'s global is the singleton, not the class** (`vm/bootstrap.rs:210-212`). The `None`
   class row is in `classes` (:262-265) expressly so core.ph *could* complete that stub. No such
   body exists — U-LIST dropped it.

---

## 3. Design

Five changes. Each maps to a numbered ruling in PDR-0001.

### 3.1 Redefinition within a module is a compile error (rulings 2 + 7)

`field_layouts` hit under the *same module key* → error. Also: duplicate **members** inside one
body — a field or a method — no silent last-writer-wins at any granularity.

> **U-BINDINGS L-4 was withdrawn in favour of this unit** (`81c8dc2`, 2026-07-19). Both units
> had independently ruled duplicate-member-is-an-error on the same day with conflicting
> diagnostic names; U-BINDINGS struck its rule rather than delete it, and **does not implement
> a duplicate-field check**. This unit owns fields *and* methods. Their corpus scan found
> **zero** duplicate field declarations, so the deferral costs nothing.

Diagnostic is **`X is already defined`** carrying **both spans** (original declaration and
duplicate), house miette style. The first declaration's span is read from `ClassLayout`, where
U-CLASSNS §3.4 stores it (`DEC-CLASSNS-A`, ruled option (i)) — that field exists and is dead
until this unit consumes it. The current `Cannot reopen class X` wording is retired with the
feature it names — it would describe something that no longer exists.

**Error codes** (ruled 2026-07-19). The house convention is a `namespace.snake_case:` prefix —
`attr.sealed_violation`, `attr.unknown`, `contract.impure_predicate`. This unit opens the
`class.*` namespace that U-CTOR already anticipates (`class.duplicate_selector`):

| code | fires on |
|---|---|
| `class.already_defined` | a second `class X` in the same module |
| `class.duplicate_member` | a repeated field or method inside one body |
| `class.reserved_name` | a kernel name declared outside the core module (§3.2) |
| `class.nested_declaration` | a class declared below module top level (§3.4) |

`already_defined` and `duplicate_member` stay **separate** rather than unified: the fix differs
(delete a block vs delete a line) and the spans differ in kind.

### 3.2 Kernel class names are reserved (ruling 3)

A non-core module declaring `List`, `Object`, `Number`, … is a compile error.

**Where the set comes from.** *Not* `add_class!` — that is a `macro_rules!` declared inside
`install_core` (`vm/bootstrap.rs:180`) and is invisible to the compiler. *Not* a hand-copied
list either; a second list drifts. Derive it from **U-CLASSNS's own output**: once `classes` is
`(module, name)`-keyed, the reserved set is exactly the core-module keys present when core.ph
finishes running. One source, no duplication, and it stays correct automatically if a kernel
class is added or removed.

Note `CoreClasses` (`universe/core_classes.rs:225`) is a *different* and wrong set for this
purpose — it enumerates every bootstrapped `ClassId` including rows never bound as a core global
(e.g. the `None` class row, `vm/bootstrap.rs:262-265`, whose global name is the singleton). The
reserved names are the ones actually *bound* in the core module, which is what the re-keyed
`classes` map gives you.

Module scoping alone would make a user's `List` a distinct-but-harmless local class (literals
bind `universe.classes.list_class` by `ClassId`, not by name), but "`class List` is silently not
List" is a trap. Reserving makes the closed kernel a stateable rule.

### 3.3 Stub completion is gated on the core module (ruling 4)

core.ph keeps layering `.ph` protocol onto Rust-installed kernel classes; nothing else can. No
new syntax — the compiler already computes the discriminating predicate (precondition 1) and
needs only the module check.

**Encode the case in the instruction**, and **delete** the runtime name-lookup fallback at
`dispatch.rs:768-788` rather than gating it.

Today `Bytecode::Class(u16)` ("creates a new class", `bytecode.rs:142`) means two things, and
the runtime guesses which by probing `classes` by name — a hit that collapses stub completion,
user reopen, and cross-module collision into one arm. The compiler already discriminates
(precondition 1) *and* knows the module, which the runtime arm does not cheaply. So the compiler
emits which one it means:

- **allocate fresh** — never consults `classes`
- **complete a bootstrapped stub** — binds the pre-existing `ClassId`; emitted **only** when
  compiling the core module

**Shape: a bool operand, not a second opcode.** `Bytecode::Class(u16, bool)`, following the
house precedent of `Bytecode::Method(u16, bool)`'s `is_static` flag (`bytecode.rs:147`). Avoids
a second name in the dispatch table for what is one instruction with two modes.

Deleting the lookup — rather than gating it — is what closes the nested-runtime-patch hole
(precondition 3) *structurally*: if allocate-fresh never consults `classes`, a re-executing
method body has no path to the kernel class regardless of any policy check.

⚠️ **`FinalizeClass`'s doc comment goes stale.** `bytecode.rs:322-329` currently reads "so a
reopened class is re-finalized (rebuilt from scratch, not accumulated) every time its body
compiles again." Reopening no longer exists — update the comment in this unit rather than
leaving it describing a removed feature.

### 3.4 Class declarations are module top-level only (ruling 5)

Enforced at the **parser**, so the grammar states the invariant and the error is positional.

⚠️ `@variant` synthesizes sibling `Statement::Class` nodes (`compiler/attributes.rs:1407`)
compiled recursively from inside `compile_class` (`class_decl.rs:768`). Those are synthesized
top-level siblings — **the ban must not trip on them.** This is the single most likely way to
break the build in this unit.

### 3.5 The `None` `DefineGlobal` guard — `DEFERRED` #17 (ruled scope)

`Statement::Class` unconditionally emits `DefineGlobal` at the end of every class body. Harmless
where the global already points at the class object; for `None` it rebinds the global from the
singleton to the class, breaking every `x == None`.

Land the guard: **skip `DefineGlobal` when the current binding is not that same class object.**
This unit is already editing that lowering path, so the marginal cost is near zero.

⚠️ **It is easy to mis-test.** `class None { … }` then `x == None` reports `true` either way —
both sides read the same clobbered binding. Use a *genuinely produced* `None`:

```phalcom
Some.new(5).filter { x => false } == None    // true before, false after
```

`isNone` keeps answering correctly throughout; only the *binding* moves. #17's own line numbers
predate U-REOPEN-FIX (`e85f31a`) and should not be trusted. (Repro from U-BINDINGS §12C.)

**Ruled 2026-07-19 — stop there.** Do **not** add a `class None` body to core.ph and do **not**
attempt `DEFERRED` #35's sealing unification. Both are
[`class-sealing-followups.md`](../../../deferred/class-sealing-followups.md) item 3; the body
drags ADR-0044's bootstrap ordering (`Nil`→`None` surfacing runs during bootstrap, before `.ph`
decorators) into a unit that otherwise does not touch it.

### 3.6 Class names register in `global_bindings` (PDR-0002, ruling 1)

A class declaration calls into the same map `declare_global` maintains
(`compiler/lib/scope.rs:179`), so a class and an `import … as Name` can no longer both claim one
name in silence.

**Registration only — the class keeps its own check and its own diagnostic.** Do *not* route
`Statement::Class` through `declare_global` itself: that would inherit `BindingRedeclared`'s
guidance, *"use assignment, or declare it in a nested scope to shadow,"* which misinstructs
twice for a class — you cannot assign one, and §3.4 bans nested declarations outright.

Resulting behavior:

| collision | reported as |
|---|---|
| class then class | `class.already_defined`, both spans |
| import then class | `class.already_defined`, both spans |
| class then import | `binding.redeclared` (from the import side, no work here) |

⚠️ **`compiler/lib/mod.rs:57-61`'s doc comment documents the exemption this removes.** Update it
in the same pass — it currently states class declarations "never interact with this map."

### Rubric — hazards & preclusion (mandatory)

| Hazard | Mitigation |
|---|---|
| `@variant` sibling classes trip the top-level ban | Precondition 6 + §3.4. Test `@variant` expansion explicitly, first. |
| A hand-copied reserved-name list drifts from `add_class!` | Derive it from the macro's own set. |
| Deleting `dispatch.rs:768` breaks bootstrap | core.ph rides the *compile-time* branch (precondition 1); the runtime fallback is only reached by same-unit reopens. Bootstrap green is the gate for this step. |
| ADR-0018 guard machinery looks newly dead | It is **not**. `world_version` bumping (`dispatch.rs:927,930`) and sacred-pristine flags (`universe/mod.rs:191-194`) are artifacts of *any* method install — bootstrap still installs, and ruling 7's reflection layer will. Do not remove them. |
| Perf claims about H17 | Out of scope, unmeasured. See `class-sealing-followups.md` item 2. |
| Scope creep into the `None` body / #35 | §3.5. Guard only. |

---

## 4. Confirmed write-set (re-validate on HEAD)

| Path | Why |
|---|---|
| `phalcom-ast/src/parser.rs` | top-level-only class declarations (§3.4) |
| `phalcom-core/src/compiler/lib/class_decl.rs` | redefinition error + both spans, duplicate-member check, reserved names, core-gated stub completion, distinct opcode emit, `DefineGlobal` guard |
| `phalcom-core/src/compiler/lib/mod.rs` | `Statement::Class` dispatch (:342) if the ban needs a compiler-side assist |
| `phalcom-core/src/bytecode.rs` | the new stub-completion opcode |
| `phalcom-core/src/vm/dispatch.rs` | new opcode arm; **delete** the `Bytecode::Class` name-lookup fallback (:768-788) |
| `phalcom-core/src/vm/bootstrap.rs` | reserved-name set derivation (read-only if the macro can expose it) |
| `phalcom-core/tests/lang/classes/` | delete 4 `class_reopen_*` fixtures |
| `phalcom-core/tests/lang/ic/` | rewrite 2 IC fixtures (§7) |

**Not** in the write-set: `core.ph`. No conflict with U-BINDINGS in either order.

---

## 5. Build order (small, independently-green diffs)

1. Duplicate-member check inside one body (§3.1 second half) — self-contained, no reopen
   interaction.
2. `@variant` regression test **first**, then the parser top-level ban (§3.4).
3. Redefinition error + both-spans diagnostic (§3.1). Delete the 4 `class_reopen_*` fixtures in
   this step — they assert the behavior being removed.
4. Reserved kernel names (§3.2).
5. Distinct stub-completion opcode + core gate; **delete** `dispatch.rs:768` fallback (§3.3).
   Bootstrap green is the gate.
6. `None` `DefineGlobal` guard (§3.5).
7. Rewrite the 2 IC fixtures against the new method-install path (§7).

Commit per green step; verify each SHA in a throwaway worktree.

---

## 6. Mandatory rules

Same as U-CLASSNS §6 — full rustdoc, `cargo doc` clean, `./scripts/verify.sh` green,
`graphify update .`, narrow-path commits on `main`, never `git checkout -b`, never `git add -a`.

---

## 7. Test strategy (the green gate must assert)

**Migration is two tests, not a corpus sweep.** A census of every `.ph` in the tree found six
files with same-file duplicate class declarations — all tests, zero production code, zero
examples, zero core.ph:

| Fixture | Fate |
|---|---|
| `classes/class_reopen_appends_methods.ph` | delete — asserts the removed feature |
| `classes/class_reopen_field_bearing_appends_methods.ph` | delete |
| `classes/negative/class_reopen_add_field_rejected.ph` | delete — subsumed by the broader already-defined error |
| `classes/negative/class_reopen_superclass_conflict_rejected.ph` | delete — same |
| `ic/ic_add_method_invalidates.ph` | **rewrite** — uses reopening as a *vehicle* to test IC invalidation, a mechanism that must survive |
| `ic/ic_override_after_caching.ph` | **rewrite** — same |

**The two IC fixtures become Rust-level tests** (ruled 2026-07-19). They need a vehicle for
installing a method post-cache-fill, and once reopening is gone the only user-reachable
installer is gone with it — ruling 7's reflection layer does not exist yet and has no unit, no
plan, no date. So drive the install path directly: fill a call site's `InlineCache`, call
`add_method` (`heap/class.rs:150`), bump `world_version` (`dispatch.rs:927,930`), assert the
cache misses and refills.

Rejected: deferring the fixtures until reflection lands (leaves a coverage hole for an unbounded
period), and dropping the coverage (IC invalidation is exactly the silent-wrong-answer class of
bug that survives for months unnoticed).

This is arguably the more honest home anyway — the invariant under test is VM-internal, and the
`.ph` fixture only ever used reopening because it was the sole user-reachable installer.
`phalcom-core/tests/invariants.rs` is the existing home for VM-level assertions of this kind.
**Say in the return contract what coverage the Rust tests assert, so the swap is auditable.**

New negative-lane fixtures: duplicate class in one module; duplicate method in one body;
duplicate field in one body; `class List` in a user module; class declaration inside a method
body; `class None` in a user module; **and both cross-kind orderings** (`import … as Point` then
`class Point`, and the reverse — §3.6). Positive lane: `@variant` still expands; bootstrap runs;
`x == None` holds after a `None`-adjacent compile.

**Confirming test, not new work** (PDR-0002): `import "a" as P` twice already errors via
U-BINDINGS' `declare_global`. Add the fixture so the behavior is pinned by this unit's lane
rather than left implicit in another unit's.

**The both-spans assertion is the fixture to get right.** Since this is the codebase's first
two-span diagnostic, the negative fixture must assert that *both* labels render — a single-span
regression would otherwise pass a substring check silently.

Error fixtures go in the **negative** subdir or the suite reddens.

---

## 8. Decisions flagged (flag, don't pick)

None open. Settled: PDR-0001's eight rulings, the (b) scope ruling on #17,
`DEC-CLASSNS-A` (span on `ClassLayout`, unit A), the IC-fixture swap to Rust-level tests (§7),
and the four `class.*` error codes (§3.1) — all ruled 2026-07-19.

Explicitly ruled **out**: the `None` body, `DEFERRED` #35 sealing unification, `SuperSend`
`ClassId` stamping, post-bootstrap freeze + H17.

---

## 9. Must-not-preclude check

- **Reflection layer** (ruling 7) — user classes only, never kernel/superclass/metaclass. Do not
  delete the guard machinery it will need.
- **`None` body + #35** — the `DefineGlobal` guard here is their *prerequisite*; leave them
  reachable.
- **Post-bootstrap freeze / H17** — this unit delivers the precondition. Do not enforce the
  freeze point here, and do not claim the perf win.
- **U-REPL** — that branch's `DEC-REPL-A` is answered by ruling 6. This unit removes the reopen
  seam `DEC-REPL-A` reasons about, so whichever lands second rebases onto a changed premise.
  Flag at integration; see 0065 ruling 6's coordination note.
- **ADR-0026 Axis 2** — reparenting stays sealed. Nothing here reopens that question.
- **U-BINDINGS L-5** — a **live coupling, flagged by U-BINDINGS §12C** (committed `20c26e0`).
  L-5 rejects same-scope redeclaration at the `DefineGlobal` site, and *until this unit lands*
  a reopen must **not** be treated as a redeclaration — core.ph completes kernel stubs through
  that path and bootstrap fails immediately otherwise. Their exemption keys on the same precise
  predicate this plan uses (`field_layouts` miss = stub completion at `class_decl.rs:332`, hit =
  true reopen at `:308`), independently derived. **Once this unit lands, that exemption becomes
  the core-module gate of §3.3 and should collapse into it rather than persisting as a second
  special case.** Whichever unit lands second must reconcile the two.

---

## 10. Return contract (report to `phalcom-reviewer`)

Per-step SHAs with `git show --stat`; confirmation that core.ph is untouched; the explicit
decision made about the two IC fixtures and why coverage is not reduced; proof that `@variant`
expansion survives the top-level ban; `./scripts/verify.sh` + `cargo doc` clean at each SHA
verified **in a throwaway worktree**; explicit confirmation that `world_version` /
sacred-pristine machinery was left intact.
