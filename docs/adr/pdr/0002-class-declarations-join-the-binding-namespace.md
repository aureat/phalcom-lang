# PDR-0002 — Class declarations join the binding namespace; the duplicate diagnostic carries both spans

- Status: Accepted
- Date: 2026-07-19
- Amends: [PDR-0001](0001-classes-are-closed.md) — narrows ruling 8 (its import half is
  already shipped) and fixes the mechanism and diagnostic shape behind ruling 2. Does not
  reverse anything in 0065.
- Self-amended 2026-07-19 (in place, status unchanged): **Decision 2's rendering mechanism.**
  "Two miette labels" named a renderer that does not exist — compile-error spans are not rendered
  at all today. Both spans still live in the error value and both locations still reach the user,
  now via the message text. See the amendment note under Decision 2.
- Related: `../forge/units/U-BINDINGS/u30-bindings-spec.md` (L-3/L-5, §12C),
  [ADR-0064](../adr/accepted/0064-let-const-bindings-and-field-mutability.md),
  [`U-CLASSCLOSE`](../forge/units/U-CLASSCLOSE/plan.md)

## Context

0065 was ruled against a tree where no same-scope redeclaration check existed. U-BINDINGS has
since landed (`b843fe2` step 1, `42aafce` step 2+3; tree green), and it changed three of that
decision's premises.

### 1. `import … as Name` is already covered

`declare_global` (`compiler/lib/scope.rs:179`) is documented as "the shared entry point behind
every source-level `let`/`const` binding at module scope **and `import … as Name`**", and
rejects a duplicate at `:181-182` with `CompilerError::BindingRedeclared`. Verified live:

```phalcom
import "m1" as P
import "m2" as P
// Error: 'P' is already declared in this scope; use assignment, or declare it in a
//        nested scope to shadow.
```

**0065 ruling 8's reachable half is therefore shipped.** U-CLASSCLOSE inherits a confirming
test, not an implementation.

### 2. Class declarations are exempt — for a reason 0065 deletes

`Statement::Class` emits `Bytecode::DefineGlobal` directly (`compiler/lib/class_decl.rs:738`),
never calling `declare_global`. The exemption is deliberate and documented
(`compiler/lib/mod.rs:57-61`):

> **Class names are not tracked here.** `Statement::Class` emits its own `DefineGlobal` directly
> (`class_decl.rs`) without going through `declare_global`, so class (re)declaration —
> **including a kernel stub completion reopen, `core.ph`'s bootstrap path** — never interacts
> with this map (§12C of the U-BINDINGS spec).

The exemption exists *only* to let reopening and stub completion survive L-5. 0065 removes
reopening and gives stub completion its own opcode, so the reason expires with this unit.

### 3. The exemption leaves a cross-kind collision hole

Classes are now the only module-level name-introducing form outside the check. Both orderings
pass silently — verified live:

```phalcom
import "m1" as Point        // then
class Point { who => "local" }
// → prints "local". The class won; the import binding is gone. No error.
```

```phalcom
class Point { who => "local" }   // then
import "m1" as Point
// → <module m1> does not understand 'new()'
```

The second is the worse failure: a **runtime** error naming a module, arbitrarily far from the
`import` line that actually caused it. `class X {}` twice still reopens silently, as 0065
already documented.

### 4. There is no two-span diagnostic to copy

`CompilerError` has 14 variants. Five carry a *single* `SourceRange`
(`DestructuringWithoutInitializer`, `ConstructStaticCollision`, `BreakOutsideLoop`,
`ContinueOutsideLoop`, `ThrowNonError`). **None carries two.** `BindingRedeclared(String)` —
the closest analogue, landed this week — carries **zero**.

0065 ruling 2 specifies both spans. That is new machinery, not a copy. This was mispriced when
ruled.

## Decision

**1. A class declaration registers its name in `global_bindings`, but keeps its own check and
its own diagnostic.**

Class-vs-class and import-then-class both report `class.already_defined` with both spans.
Class-then-import reports `BindingRedeclared` from the import side, which is accurate if
slightly generic. The cross-kind hole closes in both directions.

Rejected — **routing classes fully through `declare_global`**: it would inherit
`BindingRedeclared`'s guidance, *"use assignment, or declare it in a nested scope to shadow,"*
which misinstructs twice over for a class. You cannot assign a class, and 0065 ruling 5 bans
nested class declarations outright. Also rejected — **leaving the hole**: it is the same
silent-clobber class of defect 0065 exists to eliminate, and `a3` above shows it surfacing as an
unrelated runtime error.

**2. Build the two-span diagnostic.** `class.already_defined` carries the original declaration's
span and the duplicate's. First of its kind in the codebase; the span comes from `ClassLayout`,
where U-CLASSNS §3.4 stores it.

> **Amended 2026-07-19, mechanism only** — while writing
> [`U-CLASSCLOSE`'s implementation spec](../forge/units/U-CLASSCLOSE/implementation-spec.md) §1.2.
> This decision said "rendered as two miette labels." **That renderer does not exist, and neither
> does any other.** `use miette` / `miette::` appears in **zero** `.rs` files in the repo, despite
> miette being a declared workspace dependency and named in `CLAUDE.md`'s conventions;
> `CompilerError` derives `thiserror::Error` only. Worse, compile-error spans are not rendered
> *at all* today: `cmd_run` (`bin/phalcom/cli.rs:160`) `?`-propagates the error through
> `anyhow::Result` to `main`, which prints its `Display` text and nothing else — so the
> `SourceRange` on the five existing single-span variants is carried and dropped. The hand-rolled
> `color_print` renderer in `diagnostics.rs` is reachable only for *parse* errors, via
> `compile_closure`'s `map_err` (`interpret.rs:145`). Same shape as the standing
> "traceback exists but unwired" finding, in a different corner.
>
> **Ruled: both spans live in the error value; both locations appear in the message text.**
> `ClassAlreadyDefined(String, SourceRange, SourceRange)`, with the first declaration's
> line/column resolved from its span (e.g. *"class 'Point' is already defined in this module
> (first declared at 3:1)"*). §4's cost finding stands — this **is** new machinery — but the
> machinery is a variant plus a line/column helper, not a diagnostics-infrastructure change.
>
> **This is not the single-span fallback below.** Ruling 2's intent is fully delivered: both spans
> are carried, both locations reach the user, and the negative fixture asserts text only the
> two-location form can produce. Because the variant already carries both `SourceRange`s, literal
> two-label rendering later is a pure rendering change with no re-derivation. Wiring a
> compile-error renderer is **out of scope for U-CLASSCLOSE and wants its own unit** — it would
> change how every compile error prints, with blast radius across the negative corpus, and would
> incidentally revive the five dead spans.

Rejected — **matching `BindingRedeclared`'s span-less style**: that regresses below the file's
own existing standard, where five variants already carry one span. Rejected — **single span on
the duplicate**: cheaper and consistent with those five, but for a duplicate *class* the user's
entire question is "where is the other one," often far away in a long file. That is exactly what
a second label answers and a message cannot. The machinery is reusable by the next duplicate-ish
diagnostic.

If the cost proves higher than expected during implementation, **fall back to a single span and
amend this decision** — do not under-deliver ruling 2 silently.

**3. Keep the `class.*` error codes** ruled on 2026-07-19: `class.already_defined`,
`class.duplicate_member`, `class.reserved_name`, `class.nested_declaration`.

`binding.redeclared` now covers the same *concept* for let/const/import, but the class fix
differs (delete a block, not a line) and the guidance differs (no shadowing escape hatch exists
for classes). Note that decision 1 above makes the split fall out naturally: the cross-kind case
is reported by the import side as `binding.redeclared` with no work from us, and
`class.already_defined` covers class-vs-class.

## Consequences

- **U-CLASSCLOSE's scope shifts**, net roughly neutral. Loses: ruling 8's import
  implementation (now a test). Gains: the `global_bindings` registration, and two-span
  diagnostic machinery that was assumed to exist.
- **The `global_bindings` doc comment at `mod.rs:57-61` becomes wrong** when this lands — it
  documents an exemption this decision removes. Update it in the same pass, per the standing
  two-way-sync habit.
- **Duplicate-member detection got easier, independently.** `ClassMember::Field(FieldDef)` is now
  a first-class AST variant (`phalcom-ast/src/ast.rs:200`), so a duplicate-field check is an
  iteration over members rather than untangling the old `_x = e` getter-shaped production.
  U-BINDINGS withdrew its own L-4 in favour of this unit (`81c8dc2`) and found zero duplicate
  field declarations in the corpus, so the check ships with no migration cost.
- **`ClassLayout` already accepts per-class compile metadata** — it grew a const-field set in
  `42aafce` (`vm/mod.rs:34+`). U-CLASSNS §3.4's span field stacks onto that cleanly, which
  strengthens the `DEC-CLASSNS-A` option-(i) ruling rather than complicating it.
- **Nothing here reopens 0065.** All eight of its rulings stand; this decision changes only how
  two of them are mechanized.

## Alternatives considered

- **Amend 0065 in place rather than write this.** Rejected: 0065 is committed and was ruled
  against a materially different tree. Editing it silently would erase the fact that these three
  points were decided later, on new evidence, and would leave no record of why the mechanism
  changed.
- **Defer all three until U-CLASSCLOSE is implemented.** Rejected for decision 2 specifically:
  the two-span finding changes the unit's size, and cost discoveries belong in the record before
  dispatch, not in an implementer's surprise.
