# 65. Classes are closed: remove class reopening

- Status: Accepted
- Date: 2026-07-19
- Supersedes: [ADR-0026](../adr/accepted/0026-class-hierarchy-mutability.md) — reverses its
  **Axis 1** ("methods are open"). Axis 2 (superclass reparenting is sealed) is kept
  unchanged and strengthened.
- Related: [ADR-0011](../adr/accepted/0011-static-instance-slot-layout.md) (frozen slot
  offsets), [ADR-0018](../adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)
  (override-epoch guard — **retained**, see §Consequences),
  [ADR-0045](../adr/accepted/0045-module-import-relative-path-whole-module-binding.md)
  (file = module), `docs/forge/DEFERRED.md` #17

## Context

`class Foo { ... }` currently means three unrelated things depending on VM state: *define a
class*, *complete a Rust-installed kernel stub*, or *mutate an existing class in place*. The
third is class reopening. It was ratified by ADR-0026 on the grounds that it is free —
ADR-0018's override-epoch guard already existed, so allowing method addition and replacement
cost no new machinery.

That reasoning measured the wrong cost. Reopening is cheap in *dispatch*; it is expensive in
*namespace integrity*, and the two were never separated.

### Verified mechanics (2026-07-19, release build at HEAD)

**Two seams.** A compile-time one ([`class_decl.rs:277`](../../phalcom-core/src/compiler/lib/class_decl.rs))
rejects added fields and superclass changes, reusing the existing layout otherwise. A runtime
one ([`dispatch.rs:768`](../../phalcom-core/src/vm/dispatch.rs), `Bytecode::Class`) pushes the
existing `ClassId` on any `classes` name hit.

**Override is last-writer-wins with no chain.** `add_method` is an `IndexMap::insert`
([`heap/class.rs:150`](../../phalcom-core/src/heap/class.rs)). The displaced method — including
a Rust native primitive — becomes unreachable. There is no `super`-to-previous and no restore.
A duplicate body therefore does not *replace* a class, it *partially overwrites* it: whichever
methods did not overlap survive.

**Reach is total.** Live-confirmed reopening of `Object`, `Class`, `Metaclass`, `Behavior`,
`Number`, `List`, and `@sealed Option`. Reopening `Object` installs on every value in the
program. Existing instances are affected — same `ClassId`, nothing migrates.

**It is arbitrary runtime mutation, not declaration-order layering.** Class declarations nest
inside method bodies and execute on call:

```phalcom
class Patcher {
  static patch {
    class List { size => 12345 }
    return "patched"
  }
}
System.print([1,2].size)    // 2
System.print(Patcher.patch)
System.print([1,2].size)    // 12345
```

**The namespace is the actual defect.** `define_global`
([`vm/api.rs:128`](../../phalcom-core/src/vm/api.rs)) writes into the *module's own* globals —
bindings are module-scoped. But `classes`, `field_layouts`, `class_parents`, and
`sealed_classes` are `HashMap<Symbol, _>` on the VM
([`vm/mod.rs:101`](../../phalcom-core/src/vm/mod.rs)) — keyed by name, VM-wide. Bindings are
scoped; class identity is not. So two modules that each declare their own `Point`, mutually
unaware, silently collapse into one class:

```phalcom
import "modp" as P    // class Point { who => "from modp" }
import "modq" as Q    // class Point { who => "from modq" }
P.Point.new().who     // "from modq"
Q.Point.new().who     // "from modq"
P.Point == Q.Point    // true
```

Import order picks the winner. Declaring a class in your own file corrupts a library that
happened to use the name.

### The bootstrap constraint is already separated

Reopening is widely believed to *be* the bootstrap mechanism — that 22 of core.ph's class
declarations reopen classes Rust installed via `add_class!`. That is false as stated, and the
distinction is what makes this decision cheap.

- core.ph declares **every class exactly once**. `rg -o "^class (\w+)" core.ph | sort | uniq -d`
  is empty.
- `add_class!` ([`bootstrap.rs:180`](../../phalcom-core/src/vm/bootstrap.rs)) writes only
  `define_global` + `classes.insert`. It **never** writes `field_layouts`.
- `field_layouts.insert` has exactly one call site repo-wide
  ([`class_decl.rs:424`](../../phalcom-core/src/compiler/lib/class_decl.rs)), reached only when
  a `.ph` class body compiles.

So the compiler already discriminates:

| case | predicate | branch |
|---|---|---|
| kernel **stub completion** | `classes` hit + `field_layouts` miss | `class_decl.rs:332` |
| true **reopen** | `field_layouts` hit | `class_decl.rs:308` |

core.ph rides the first branch and never the second. The kernel does not need reopening; it
needs stub completion, which is a distinct mechanism that already exists.

## Decision

**A class is defined exactly once, by exactly one module. Classes are closed after
definition.**

1. **Class identity is module-scoped.** `classes`, `field_layouts`, `class_parents`, and
   `sealed_classes` are re-keyed on `(module, name)`. Two modules may each own a `Point`; they
   are two unrelated classes. Since file = module (ADR-0045), "same module" and "same file"
   are the same check.

2. **Redefinition within a module is a compile error.** Both a duplicate class and a duplicate
   member within one body — a field or a method. No silent last-writer-wins at any granularity.
   The diagnostic is `X is already defined`, carrying **both** spans (original and duplicate).
   The current `Cannot reopen class X` wording is retired with the feature it names.

3. **Kernel class names are reserved.** A non-core module declaring `List`, `Object`, `Number`,
   etc. is a compile error. The name set is the one enumerated in `add_class!`. Module scoping
   alone would make a user's `List` a harmless-but-distinct local class — literals bind
   `universe.classes.list_class` by `ClassId`, not by name — but "`class List` is silently not
   List" is a trap, and reserving makes the closed kernel a stateable rule rather than an
   emergent consequence of two other mechanisms.

4. **Stub completion is gated on the core module.** core.ph keeps layering `.ph` protocol onto
   Rust-installed kernel classes; nothing else can. No new syntax: the compiler already
   computes the discriminating predicate and only needs the module check. The compiler emits a
   **distinct opcode** for stub completion, and the runtime name-lookup fallback in
   `Bytecode::Class` is **deleted** rather than gated — the compiler knows which case it is,
   and deleting it closes the nested-runtime-patch hole structurally.

5. **Class declarations are module top-level only.** Nested declarations are rejected at the
   parser, so the grammar states the invariant and the error is positional. Implementer note:
   `@variant` synthesizes sibling `Statement::Class` nodes
   ([`attributes.rs:1407`](../../phalcom-core/src/compiler/attributes.rs)) compiled recursively
   from inside `compile_class` — the ban must not trip on those.

6. **REPL cells shadow; they do not reopen.** Redefining a class in a later cell binds a new
   class. Instances made under the old definition keep it (they hold a `ClassId`; nothing
   migrates); the old class becomes unreachable by name. No live object is ever silently
   patched.

   **This closes `DEC-REPL-A`.** The U-REPL cell-model plan raised exactly this question and
   scoped it out: *"`class Foo` emits `DefineGlobal`, so a cell can rebind a class name while
   `vm.field_layouts` still keys a layout to that symbol and live instances still point at the
   old class. […] Open — scoped out of this unit."* This ruling answers it with that plan's own
   diagnosis of the mechanism, and needs no machinery beyond the `DefineGlobal` substrate
   §D1/§D2 of that plan already establishes.

   No conflict with the plan's **two-set immutability** (§D4): those
   `Compiler::immutable_globals` / `ModuleObject::immutable_globals` sets are consulted only
   for `let`/`const` pattern bindings (`compiler/lib/patterns.rs:47`,
   `compiler/lib/expr.rs:302`). Class declarations never insert into them — they emit
   `Bytecode::DefineGlobal` directly (`compiler/lib/class_decl.rs:719`). The two rulings govern
   disjoint code paths.

   > **Coordination.** As of 2026-07-19 the U-REPL plan is **not on `main`** — it lives on the
   > unmerged worktree branch `worktree-repl-cell-model` (tip `7ac0b5a`, built on `301044e`),
   > verified by `git show main:docs/forge/units/U-REPL/plan.md` returning nothing. Whoever
   > merges that branch should mark `DEC-REPL-A` resolved by this ruling rather than deciding
   > it again.

7. **Runtime method installation is deferred to a reflection layer, with its bounds fixed
   now.** Such an API may add or replace methods on **user** classes only. It may not touch
   kernel classes, superclass links, or metaclasses. Recording the bounds here is what prevents
   reopening from returning under another name.

8. **A name may be bound once per module — including by `import`.** Two bindings of the same
   name in one module is a compile error, with the same `X is already defined` diagnostic and
   both spans.

   Today this reaches exactly one shape. There is a single import form,
   `import "path" as Name` ([`parser.rs:511`](../../phalcom-ast/src/parser.rs) hard-requires
   `as`; ADR-0045 replaced ADR-0027's qualified/selective/aliased trio with whole-module
   binding alone), so **alias collision** is the reachable case and it is silent today:

   ```phalcom
   import "modp" as P
   import "modq" as P
   P.Point.new().who    // "from modq" — second binding wins, no error
   ```

   Two same-named *classes* cannot collide in an importer's namespace, because whole-module
   binding means they are always reached as `P.Point` / `Q.Point` — the alias already
   qualifies them. Combined with (1), that is the intended end state for the current surface,
   reached without new machinery.

   **Forward constraint, binding on any future import work.** If selective import
   (`import Point from "modp"`) is ever added, importing two same-named classes into one
   module is a compile error — it does not shadow, alias-by-arrival-order, or last-write-win.
   The diagnostic should offer renaming or a qualified import. Recorded here rather than
   deferred to that unit's own design, because every collision case audited in this decision
   defaulted to silent last-writer-wins, and a new import form is the most likely place for
   that default to reappear.

## Consequences

- **Requirement met: users cannot alter core classes.** Reserved names (3) plus core-gated
  stub completion (4) plus the deleted runtime fallback (4) close every path found in the
  audit above, including the nested-method-body patch.

- **Two silent bug classes disappear.** Cross-module class collision, and partial overwrite by
  duplicate body. Neither was ever a designed behavior.

- **`docs/forge/DEFERRED.md` #17 changes character, and unit B fixes it.** Reopening
  `class None` rebinds its global from the singleton to the class object, breaking `x == None`.
  It **dissolves as a user-reachable bug** — ruling 3 means nobody outside core can write
  `class None` at all. It **survives as a bootstrap task**: bootstrap inserts the `None` class
  row into `self.classes` (`vm/bootstrap.rs:262-265`) expressly so core.ph *can* complete that
  stub, and ruling 4 now sanctions exactly that. So unit B lands the guard — skip
  `DefineGlobal` when the current binding is not that same class object — because it is already
  editing that lowering path. Pull #17 out of U-BINDINGS and rewrite the entry rather than
  closing it. Reproduced live 2026-07-19, discharging its "unverified since 2026-07-11" caveat.

  **It is easy to mis-test.** `class None { … }` then `x == None` reports `true` either way —
  both sides read the same clobbered binding. The comparison must use a *genuinely produced*
  `None`:

  ```phalcom
  Some.new(5).filter { x => false } == None    // true before, false after
  ```

  `isNone` keeps answering correctly throughout; only the *binding* moves, and that asymmetry
  is what keeps the defect quiet. (Sharper repro courtesy of U-BINDINGS §12C; #17's own line
  numbers predate U-REOPEN-FIX `e85f31a` and should not be trusted.)

- **Giving `None` a body, and `DEFERRED` #35's sealing unification, are explicitly out of
  scope.** Unit B fixes the blocker and stops. The body would drag ADR-0044's bootstrap
  ordering (`Nil`→`None` surfacing runs during bootstrap, before `.ph` decorators) into a unit
  that otherwise does not touch it, and #35 flags that as an open unknown rather than
  known-safe. Ruled 2026-07-19; the full thread is
  [`docs/deferred/class-sealing-followups.md`](../deferred/class-sealing-followups.md) item 3.
  One thing unit A must answer in passing: re-keying `sealed_classes` on `(module, name)` bears
  directly on #35's ownership unknown — whether bootstrap's module handle and core.ph's are the
  same is currently unverified.

- **ADR-0018's guard machinery is retained, not retired.** `world_version` bumping and the
  sacred-pristine flags are artifacts of *any* runtime method install, not of reopening
  specifically. Bootstrap still installs methods, and the deferred reflection layer (7) will
  too. What ADR-0018 loses is only its role as ADR-0026's justification.

- **What this actually unlocks is a post-bootstrap freeze**, not guard deletion. Once core.ph
  has loaded, no method can be installed on a kernel class from any source, so kernel class
  shape is statically known. That is the property perf item H17's core.ph cursor probe was
  gated on. **Unmeasured** — scope the freeze, measure H17 after, promise nothing before.

- **Migration cost is two tests.** A census of every `.ph` in the tree found six files with
  same-file duplicate class declarations, all tests. Four are `class_reopen_*` tests of the
  feature and die with it (two are negative-lane tests whose assertions the broader
  already-defined error subsumes). Two — `ic_add_method_invalidates` and
  `ic_override_after_caching` — use reopening as a *vehicle* to test inline-cache invalidation,
  a mechanism that must survive; they need rewriting against the new method-install path. Zero
  production code, zero examples, zero core.ph.

- **The REPL is where this costs something.** Iterative "patch a method and retry" was ADR-0026's
  strongest argument. Cell shadowing (6) preserves the workflow; it does not preserve patching a
  live object graph in place.

## Alternatives considered

- **Keep reopening, restrict it to non-kernel classes.** Addresses the stated requirement and
  nothing else. Cross-module collision and partial overwrite both survive, since neither is
  about the kernel. Rejected: it treats the symptom.

- **Module-scope the class registry and stop there.** Fixes collision, and makes a user's
  `List` a distinct local class rather than a kernel patch. But intra-module redefinition stays
  silent, and "`class List` is not List" is its own trap. Rejected as incomplete — though note
  this is decision (1), which ships either way; the question was only whether to stop there.

- **An explicit `extend Foo { ... }` form.** Self-documenting, and it would make the three
  meanings of `class` into two named ones. Rejected: it is new grammar serving exactly one
  privileged caller (core.ph), and it keeps a monkey-patch primitive in the language under a
  clearer name — which is the outcome this decision exists to avoid.

- **Move the 22 kernel protocols into Rust.** Removes the stub-completion case entirely.
  Rejected: it deletes the core library's authored-in-Phalcom property, which is a design goal,
  to solve a problem that decision (4) solves with a module check.

- **ADR-0026's own rejection of "fully sealed (Wren)"** was: *"Axis 1 is free here, so there is
  no reason to forbid it."* Free in dispatch cost, which is what ADR-0018 had already paid for.
  The namespace cost was never priced, and it is the larger one.
