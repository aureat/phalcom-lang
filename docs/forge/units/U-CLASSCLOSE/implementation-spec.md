# U-CLASSCLOSE — Implementation spec

Companion to [`plan.md`](plan.md). Governed by
[decision 0065](../../../decisions/0065-classes-are-closed.md) (**Accepted**), as amended by
[decision 0066](../../../decisions/0066-class-declarations-join-the-binding-namespace.md).

**Status: READY to dispatch, with one decision required** — see [§3](#3-the-diagnostic--one-decision-required)
and [§14](#14-decisions-required). Unit **B** of two. **Blocked on
[`U-CLASSNS`](../U-CLASSNS/implementation-spec.md)**: the redefinition error is undecidable until
class identity is `(module, name)`-keyed.

`plan.md`'s preconditions were re-verified 2026-07-19 against the tree. **Two of its load-bearing
mechanisms do not exist as described**, and both change the unit's shape rather than its line
count: the `Class(u16, bool)` operand is unnecessary (§1.1), and there is no diagnostic renderer
for compile errors to hang a second span on (§1.2). A third — the `@variant` hazard `plan.md`
calls "the single most likely way to break the build" — is **not a hazard at all** (§1.3).
**Read §1 before §2.**

> **Tree state.** Written against `77b7030` (`d14599b` + two docs-only commits). Main has **live
> concurrent sessions** — re-verify before starting, commit narrow paths on `main` itself, never
> `git checkout -b`, never `git add -A`.

---

## 0. Preconditions — re-verified 2026-07-19 at `77b7030`

| # | `plan.md` claim | Verdict | Evidence |
|---|---|---|---|
| P1 | The kernel does not need reopening; `add_class!` never writes `field_layouts`; one insert site | ✅ holds, **line moved** | sole `field_layouts.insert` is `class_decl.rs:436`, not `:424` (U-BINDINGS moved it) |
| P2 | Two seams: compile-time `class_decl.rs:277-292`, runtime `dispatch.rs:768` | ✅ holds | guards now `:288`; runtime probe `dispatch.rs:768` inside the `Bytecode::Class` arm `:740-793` |
| P3 | Reopening is arbitrary runtime mutation, nestable in a method body | ✅ holds | the `Patcher` repro |
| P4 | Reach is total; existing instances affected | ✅ holds | same `ClassId`, nothing migrates |
| P5 | Override is last-writer-wins, `IndexMap::insert` | ✅ holds | `heap/class.rs:150` |
| P6 | `Statement::Class` is position-unrestricted; "the parser imposes nothing" | ⚠️ **half wrong** | the parser has **two** call sites of `parse_class` and they *are* the two positions — see §1.4 |
| P7 | `None`'s global is the singleton; the class row exists for stub completion | ✅ holds | `bootstrap.rs:210-212`, `:262-265` |
| P8 | `Bytecode::Class(u16)` "means two things and the runtime guesses which" | ❌ **wrong of the compiler** | true of the runtime arm only; the compiler already emits `Constant` vs `Class` — §1.1 |
| P9 | The two-span diagnostic is new machinery (0066 §4) | ✅ holds, **and is newer than 0066 thought** | there is no compile-error renderer at all — §1.2 |

---

## 1. Corrections to `plan.md` — read this first

### 1.1 The bool operand is unnecessary: stub completion already emits a different instruction

`plan.md` §3.3 and decision 0065 ruling 4 both reason from "`Bytecode::Class(u16)` means two
things, and the runtime guesses which by probing `classes` by name." That is true of the
**runtime arm**. It is **false of the compiler**, which has discriminated the two cases since
before this decision:

```rust
// compiler/lib/class_decl.rs:440-459
if let Some(&existing_class) = self.vm.classes.get(&name_sym) {
    let class_idx = self.add_constant(Value::Obj(existing_class));
    self.emit(Bytecode::Constant(class_idx), range);          // ← stub completion
} else {
    // …push the superclass (GetGlobal :453 or Constant :457)…
    self.emit(Bytecode::Class(name_idx), range);              // ← allocate fresh
}
```

The runtime arm's own comment says so explicitly (`dispatch.rs:755-767`), describing the
bootstrap case as

> a no-op fallthrough, since that case already resolves at compile time and never reaches here.

So **`Bytecode::Class` is already reached only on the allocate-fresh path**, and the runtime probe
at `:768` exists for exactly one reason: a *same-unit* reopen (`class Foo {}` twice in one file),
where at compile time of the second block `Foo` is not yet in `classes` because the whole unit
lowers to one closure before any `Bytecode::Class` executes.

Delete that probe — which ruling 4 mandates — and `Bytecode::Class` means precisely one thing,
**with no new operand**. The same-unit reopen it existed to serve becomes `class.already_defined`
at compile time (§2), so nothing is left for it to do.

**Ruling 4 is therefore already satisfied on the emit side.** "The compiler emits a distinct
opcode for stub completion" is true today: that opcode is `Constant`. What remains of ruling 4 is
the *deletion* and the *core gate*, both of which this unit does.

**Recommended shape: no operand.** `Bytecode::Class(u16)` unchanged.

| | bool operand `Class(u16, bool)` | no operand (recommended) |
|---|---|---|
| Encodes the case in the instruction | yes, redundantly — `Constant` already does | already encoded, by opcode identity |
| Touches `dispatch.rs:570`'s exhaustive match | yes (the one match in the workspace that breaks) | no |
| Changes `bootstrap`'s emitted bytecode | yes — stub completion would have to move off `Constant` onto `Class(idx, true)`, a behavior change in the one path that must not break | no |
| Disassembler output | `Class(3)` → `Class(3, false)` | unchanged |
| Reviewer can see the two cases | in the operand | in the opcode |

The bool's real cost is the third row: to make the flag *mean* anything, stub completion would
have to stop emitting `Constant` and start emitting `Class(idx, true)` — rewriting the bootstrap
path that this whole decision is designed **not** to disturb, in exchange for information the
opcode already carries.

**This is a decision, not a fait accompli** — see §14.1. `plan.md` and the dispatch handoff both
record "a bool operand, not a second opcode" as ruled. That ruling chose between *two* shapes on
the premise that `Class` was ambiguous. The premise is false; the third option was never on the
table when it was made.

### 1.2 The two-span diagnostic has nowhere to render — compile-error spans are dead today

Decision 0066 §4 correctly says the two-span diagnostic is new machinery. It is newer than that.

- **Miette is not used anywhere in this repo.** It is declared in the root `Cargo.toml` and
  `CLAUDE.md` names it as the convention, but `use miette` / `miette::` appears in **zero** `.rs`
  files. `CompilerError` derives `thiserror::Error` only, with no `#[derive(Diagnostic)]`. So
  "rendered as two miette labels" (0065 ruling 2, 0066 decision 2) describes a renderer that does
  not exist.
- **The spans on the five existing single-span variants are never rendered either.** The run path
  is `cmd_run` → `vm.compile_closure(module, &source)?` (`bin/phalcom/cli.rs:160`). The `?`
  propagates the `CompilerError` up through `anyhow::Result` to `main`, which prints its `Display`
  text and nothing else. `ConstructStaticCollision`'s `SourceRange` is carried and dropped.
- **The hand-rolled renderer in `diagnostics.rs` is reachable only for *parse* errors**, via
  `compile_closure`'s `map_err` calling `print_parse` (`interpret.rs:145`). It is
  `color_print`-based (`print_line_information`: caret span plus one line of context either side),
  unrelated to miette, and `cmd_run` never reaches it for a compile error. This matches the
  standing "traceback exists but unwired" finding — the same shape of dead diagnostic
  infrastructure, in a different corner.

So "carry both spans" decomposes into two independent pieces of work, and only the first is
forced by ruling 2:

1. put both locations in the error **value** — a `CompilerError` variant with two `SourceRange`s;
2. **render** them — which requires wiring a compile-error renderer that does not exist.

§3 lays out the options and recommends one. This is the unit's largest cost surprise and the
reason 0066's own escape clause exists.

### 1.3 `@variant` is not a hazard — and the ban is a syntax rule, not an invariant

`plan.md` §3.4 and its rubric both call `@variant` "the single most likely way to break the
build in this unit." Verified: **it cannot trip the ban at all.** The synthesized siblings are
built as Rust struct literals and handed straight to the compiler, never to the parser:

```rust
// compiler/attributes.rs:1407
siblings.push(Statement::Class(ClassDef {
    name: v.name.clone(),
    superclass: Some(SuperclassRef { name: class.name.clone(), range: v.range }),
    /* … */
}));
```

fed to `compile_class` directly (`class_decl.rs:786-790`), bypassing `parse_class` entirely. A
parser-level ban is structurally blind to them.

That is good news for the build and **bad news for the invariant**. The ban is a *syntax*
restriction, not an enforced property: any present or future desugaring that synthesizes a
`Statement::Class` escapes it. Today that is a closed set — an exhaustive workspace search finds
exactly **two** `Statement::Class` *construction* sites (`parser.rs:931` and `attributes.rs:1407`);
every other mention is a match or destructure. State the limit in the spec and in the code rather
than discovering it later:

> The parser ban is the enforcement point for *source-level* nesting. Synthesized class
> statements are trusted by construction. If a third synthesizer is ever added, it inherits that
> trust silently.

`plan.md`'s "test `@variant` expansion explicitly, first" stands — not because the ban might trip
it, but because it is the cheapest proof that the ban did not accidentally reach the compiler-side
path.

### 1.4 The parser change is one three-line arm — and it should parse-then-reject

`plan.md` P6 says "`Statement::Class` is position-unrestricted — the parser imposes nothing." The
second half is wrong in a way that makes the unit easier. `parse_class` (`parser.rs:895`) has
**exactly two call sites, and they are exactly the two positions**:

| Call site | Position |
|---|---|
| `parser.rs:359-364`, in `parse_top_item` | module top level — **keep unchanged** |
| `parser.rs:1557-1559`, in `parse_block_statements` | nested block body — **this is the whole change** |

No parser state tracks depth: `struct Parser` (`:172-186`) carries only `source`, `offset`,
`tokens`, `pos`, `prev_end`, `errors`. The distinction is already structural — carried by which
function you are in. Nothing needs to be invented.

**Reject after parsing, not before.** The nested arm is:

```rust
// parser.rs:1557-1559
Token::Class | Token::At => {
    stmts.push(self.parse_class()?);
}
```

`Token::At` shares it — attributes on a nested class. Erroring at token level would either
mis-diagnose `@` or need the arm split. Instead: call `parse_class()` as today, then reject the
result with `class.nested_declaration` pointing at the `class` keyword's span. This gives the
error a real span (rather than the `@`), keeps `@`-prefixed nesting handled uniformly, and lets
the parser continue. `synchronize()` (`:394`) already stops on `Token::Class` (`:402`), so
recovery needs no change and `parse_class` needs no signature change.

**Two doc comments become lies** and must change in the same commit — `parser.rs:1542` ("nested
`class` declarations are permitted") and `parser.rs:1554` (the `Token::At` rationale, "same
rationale as the top-level `parse_top_item` dispatch").

**Corpus cost: zero.** `^[[:space:]]+class ` over every `.ph` in the tree returns 0 hits, and
`phalcom-ast/tests/parser.rs` has no nested-class test. The ban reddens nothing.

### 1.5 The IC-fixture ruling's precondition fails — `world_version` is `pub(crate)`

`plan.md` §7 and the 2026-07-19 ruling say the two IC fixtures become Rust-level tests that
"drive the install path directly: fill a call site's `InlineCache`, call `add_method`, bump
`world_version`, assert the cache misses and refills." From an **integration** test in
`phalcom-core/tests/`, that is not implementable:

```rust
// vm/mod.rs:122
pub(crate) world_version: u64,
```

No `pub fn world_version` accessor exists. Everything else the ruling needs *is* public —
`InlineCache` and its three fields (`chunk.rs:10-17`), `Chunk.caches` (`:49`), `Callable.chunk`,
`ClosureObject.callable`, `Heap::class_mut`, `ClassObject::add_method` (`heap/class.rs:150`),
`VM::compile_closure`, `VM::run_in_module`.

**And the naive fix is a trap.** `add_method` does **not** bump `world_version` itself — all six
call sites bump it as a separate adjacent statement (`primitive/mod.rs:116-133`,
`dispatch.rs:926-930`, `universe/primitives.rs:163-197`). A test that calls `add_method` without
the bump leaves the cache *valid* against the stale entry and **passes while proving the
opposite** of invalidation. This is the single largest correctness risk in the test migration.

**Resolution: write them as in-crate unit tests, not integration tests.** `phalcom-core/src/chunk.rs`
already has a `#[cfg(test)] mod tests` (`:157`) with a `chunk_of` helper — the convention exists,
and inside the crate `pub(crate) world_version` is reachable with no public API growth. §11 specs
them there.

Rejected: **making `world_version` `pub`** or adding a `VM::install_method` accessor. Either is a
public-API commitment that previews decision 0065 ruling 7's reflection layer — the exact thing
the ruling defers and bounds. A test seam is not a reason to open it. See §14.2; this is the
conditional trigger the dispatch handoff named, surfaced rather than silently resolved.

### 1.6 Negative-lane mechanics, exactly

Governs every fixture in §11. From `phalcom-core/tests/support/mod.rs`:

- **Discovery is a directory walk** (`collect_cases`, `:44-60`), sorted, `*.ph`. Not a manifest —
  `tests/lang/MANIFEST.md` is prose and is read by nothing. Adding a file to the right directory
  wires it.
- **The driver is `tests/lang.rs`**, one `#[test]` per label directory. (Do not confuse it with
  `tests/golden.rs`, a separate seven-test harness with inline expected strings and no sidecars.)
- **Subprocess**, running the real compiled CLI (`env!("CARGO_BIN_EXE_phalcom")`).
- **Positive**: stdout compared **byte-exact** against a `.expected` sidecar (same stem), modulo
  one trailing newline.
- **Negative**: three assertions — non-zero exit, no panic, and
  `format!("{stdout}\n{stderr}")` **contains** the trimmed `.expected` content as a **substring**.
- **There is no bless/update mechanism.** No `UPDATE`/`BLESS` env var, no script. Sidecars are
  hand-authored and hand-verified.
- **Every negative `.expected` in the corpus today is exactly one line** (~90 files checked). A
  multi-line sidecar would be a first for the format, not just for the compiler.

The substring semantics is why 0065's warning about the both-spans fixture is right, and why §3's
choice matters: a single-span regression passes a substring check silently unless the sidecar
asserts text that only the two-span form can produce.

### 1.7 Line-number drift

| What | `plan.md` / 0065 says | Actual |
|---|---|---|
| sole `field_layouts.insert` | `class_decl.rs:424` | `:436` |
| `ClassLayout` literal | — | `:427` |
| reopen guard (`contains_key`) | `:277-292` | `:288` |
| layout reuse (reopen branch) | `:308` | `:319` |
| stub-completion branch | `:332` | `:343` |
| `Constant`-vs-`Class` emit fork | — | `:440-459` |
| `DefineGlobal` after class body | `:738` | `:738` ✅ |
| `FinalizeClass` doc comment | `bytecode.rs:322-329` | `:316-330` |
| `Class` doc comment | — | `bytecode.rs:140-142` |
| `@variant` sibling construction | `attributes.rs:1407` | `:1407` ✅ |
| recursive sibling compile | `class_decl.rs:768` | `:786-790` |

**Anchor by symbol.** These moved once this week and will move again.

---

## 2. Redefinition and duplicate members (rulings 2 + 7)

### 2.1 The predicate, after U-CLASSNS

Unit A makes the three cases fall out of one lookup instead of needing a policy check:

| `classes[(self.module, name)]` | `field_layouts[(self.module, name)]` | Means | Action |
|---|---|---|---|
| miss | miss | fresh class | allocate — emit `Class` |
| **hit** | **miss** | Rust-installed stub, not yet completed | stub completion — emit `Constant` (§5) |
| **hit** | **hit** | this module already declared it | **`class.already_defined`** |

The second row is reachable **only** in the core module, and that is a property, not a
convention: a `classes` entry with no `field_layouts` entry can only have come from
`add_class!`/the `None` row, both of which key to the core module handle `m`
(`bootstrap.rs:187,266`), and any `.ph` declaration writes both. **Assert the core gate
explicitly anyway** (§5) — ruling 4 asks for the check, and an assertion that can never fire is
the cheapest possible documentation of an invariant.

Note what disappears: the current reopen machinery at `:288` (reject added fields), `:295-296`
(superclass-change check), and `:319` (layout reuse) is **deleted**, not gated. Their two negative
fixtures die with them (§11).

### 2.2 Duplicate members within one body

A repeated field **or** method name inside one class body is `class.duplicate_member`. No silent
last-writer-wins at any granularity (`add_method` is `IndexMap::insert`, `heap/class.rs:150`, so
today `bar => 1` then `bar => 2` silently yields `2`).

This is an iteration over `class_def.members` **before** any member compiles, not a check threaded
through the compile loop — U-BINDINGS made `ClassMember::Field(FieldDef)` a first-class AST
variant (`phalcom-ast/src/ast.rs:200`), so fields and methods are both plain members now. Compare
on the member's *name*, and note that getters, setters, methods, and fields share one namespace
for this purpose only insofar as the existing selector encoding says they do — **do not** invent a
new collision rule here; `class.duplicate_selector` is U-CTOR's, and
`ConstructStaticCollision` (`error.rs:121`) already owns the construct-vs-static case.

**Corpus cost: zero.** U-BINDINGS' scan found zero duplicate field declarations, which is why it
withdrew L-4 in favour of this unit (`81c8dc2`).

### 2.3 Error codes

Ruled 2026-07-19, unchanged: `class.already_defined`, `class.duplicate_member`,
`class.reserved_name`, `class.nested_declaration`. `already_defined` and `duplicate_member` stay
**separate** — the fix differs (delete a block vs delete a line) and the spans differ in kind.

The house convention is a `namespace.snake_case` prefix. Note that today these codes live in the
*message text*, not in a structured field — `CompilerError` has no code field, and the existing
`attr.sealed_violation` surfaces as a message prefix. Follow that; do not add a code field for
this unit.

---

## 3. The diagnostic — one decision required

Ruling 2 (0065) and decision 2 (0066) both specify: `X is already defined`, carrying **both**
spans. §1.2 establishes that nothing renders a compile-error span today. Three shapes, in
increasing cost:

**Option A — both spans in the value, both locations in the message.** Add
`ClassAlreadyDefined(String, SourceRange, SourceRange)` and format the message with the original
declaration's line/column resolved from its span:

```
class 'Point' is already defined in this module (first declared at 3:1).
```

Cost: the variant, plus a line/column resolver (`diagnostics.rs::print_line_information` already
computes line and column from a byte range — extract that arithmetic into a reusable
`line_col(source, offset)`). No renderer wiring. The negative sidecar can assert
`first declared at 3:1`, which **only the two-location form can produce** — satisfying 0065's
"a single-span regression must not pass a substring check silently."

**Option B — Option A plus a compile-error renderer.** Wire `cmd_run`'s compile-error path
through a new `diagnostics::print_compile(source, msg, primary, secondary)` that calls
`print_line_information` twice. Delivers literal two-label rendering.

Cost: the renderer (~40 lines, reusing existing pieces) **plus** wiring the compile-error path in
`cli.rs:160`, which currently `?`-propagates. That wiring is a change to how *every* compile error
prints — it would incidentally light up the five existing dead spans, which is a genuine
improvement and a genuine scope expansion, with a blast radius across every negative fixture whose
sidecar asserts compile-error text.

**Option C — single span on the duplicate.** Amend decision 0066. Cheapest; explicitly named by
0066 as the sanctioned fallback.

**Recommendation: A.** It satisfies ruling 2's *user-visible intent* — the user's question is
"where is the other one," and A answers it — without pulling a diagnostics-infrastructure change
into a namespace-semantics unit. It is also forward-compatible: the variant already carries both
`SourceRange`s, so B becomes a pure rendering change later, with no re-derivation. B's renderer is
worth doing and should be its own unit, where its blast radius across the negative corpus can be
gated properly.

Under A, 0066 needs a **one-line amendment**: "rendered as two miette labels" becomes "carries
both spans in the error value; both locations appear in the message text. Literal two-label
rendering awaits a compile-error renderer, which does not exist." That is a mechanism correction,
not a reversal of ruling 2 — both spans are still carried and both locations still reach the user.
**Do not proceed on A without recording that amendment** (§14.1).

The first declaration's span comes from `ClassLayout.declared_at`, which U-CLASSNS §7 adds and
this unit is the first to read.

---

## 4. Reserved kernel names (ruling 3)

A non-core module declaring `List`, `Object`, `Number`, … is `class.reserved_name`.

**The set derives from unit A's re-keyed `classes` map** — the core-module keys present once
`core.ph` has finished running. One source, no duplication, automatically correct if a kernel
class is added or removed.

Explicitly **not** from:

- `add_class!` — a `macro_rules!` declared *inside* `install_core` (`bootstrap.rs:180`), invisible
  to the compiler;
- `CoreClasses` (`universe/core_classes.rs:225`) — the wrong set. It enumerates every bootstrapped
  `ClassId`, including rows never bound as a core global (the `None` class row,
  `bootstrap.rs:262-265`, whose global name holds the singleton);
- a hand-copied list — it drifts.

The check fires when `self.module != core_module` and the name is a core-module key. Module
scoping alone would already make a user's `List` a distinct-and-harmless local class (literals
bind `universe.classes.list_class` by `ClassId`, not by name), but "`class List` is silently not
`List`" is a trap; reserving makes the closed kernel a stateable rule.

> **Fixture note:** use `List` or `Object`. Do **not** reach for `Option` — it is `@sealed` to
> core (`bootstrap.rs:221-225`), so anything built on it may pass for the unrelated
> `attr.sealed_violation` reason. (Same trap flagged in U-CLASSNS §14.3.)

---

## 5. Stub completion, and deleting the runtime fallback (ruling 4)

**Two changes, and — per §1.1 — no new opcode and no new operand.**

**5.1 Gate the stub-completion branch on the core module.** At `class_decl.rs:440`, the `classes`
hit that emits `Constant` becomes reachable only when `self.module` is the core module. Per §2.1
that is already implied by the table state; make it explicit so ruling 4's check exists in code,
and so a future bootstrap change cannot silently widen it.

**5.2 Delete `dispatch.rs:768-788`.** The `classes.get(&name_sym)` probe, the reuse of the existing
`ClassId`, and the superclass-mismatch rejection at `:778-783` all go. What remains of the
`Bytecode::Class` arm is: pop the superclass, validate it is a class, `create_class`, push.

**Deleting rather than gating is what closes the nested-runtime-patch hole structurally.** If
allocate-fresh never consults `classes`, a re-executing method body has no path to any existing
class, whatever policy check might have been bypassed. Precondition P3's `Patcher` repro stops
working by construction rather than by rule.

**Bootstrap green is the gate for this step** — and it is a real gate, not a formality: `core.ph`
rides the compile-time `Constant` branch and never reaches the deleted code (`dispatch.rs:755-767`
says so in its own comment), so if bootstrap breaks here, the premise was wrong and the step must
stop rather than be patched around.

**5.3 Two doc comments describe removed features** and change in this same commit:

- `bytecode.rs:316-330`, `FinalizeClass`: "so a reopened class is re-finalized (rebuilt from
  scratch, not accumulated) every time its body compiles again."
- `bytecode.rs:140-142`, `Class`: "Creates a new class" is now the *whole* truth rather than one
  of two meanings — say so, and say that the compiler emits `Constant` for stub completion, since
  that is the only remaining place a reader could learn it.

**Retained, deliberately:** ADR-0018's guard machinery. `world_version` bumping
(`dispatch.rs:926-930`) and the sacred-pristine flags (`Universe::note_method_installed`,
`universe/mod.rs:188`) are artifacts of
*any* method install, not of reopening. Bootstrap still installs; ruling 7's reflection layer
will. Do not remove them, and say in the return contract that they are intact.

---

## 6. Class declarations are module top-level only (ruling 5)

Per §1.4: delete the `Token::Class | Token::At` arm's unconditional acceptance at
`parser.rs:1557-1559`, replacing it with parse-then-reject carrying `class.nested_declaration` on
the `class` keyword's span. `parse_top_item:359` untouched. `synchronize()` untouched.
`parse_class`'s signature untouched.

Update `parser.rs:1542` and `:1554`'s doc comments in the same commit — both currently promise
nested classes work.

Per §1.3, add a sentence to the ban's doc comment recording that synthesized `Statement::Class`
nodes (`attributes.rs:1407`) bypass this check by construction and are trusted.

---

## 7. The `None` `DefineGlobal` guard (`DEFERRED` #17)

`Statement::Class` emits `DefineGlobal` unconditionally at the end of every class body
(`class_decl.rs:738`). Harmless where the global already points at the class object; for `None` it
rebinds the global from the **singleton** to the **class**, breaking every `x == None`.

**Land the guard: skip `DefineGlobal` when the current binding is not that same class object.**
This unit is already editing that lowering path, so the marginal cost is near zero.

⚠️ **It is easy to mis-test.** `class None { … }` then `x == None` reports `true` either way —
both sides read the same clobbered binding. The comparison must use a *genuinely produced* `None`:

```phalcom
Some.new(5).filter { x => false } == None    // true before the guard, false after
```

`isNone` keeps answering correctly throughout; only the *binding* moves, and that asymmetry is
what keeps the defect quiet. #17's own line numbers predate U-REOPEN-FIX (`e85f31a`) — do not
trust them.

**Ruled 2026-07-19 — stop there.** Do **not** give `None` a body in `core.ph`, and do **not**
attempt `DEFERRED` #35's sealing unification. Both are
[`class-sealing-followups.md`](../../../deferred/class-sealing-followups.md) item 3; the body drags
ADR-0044's bootstrap ordering (`Nil`→`None` surfacing runs during bootstrap, before `.ph`
decorators) into a unit that otherwise does not touch it. **DEFERRED #17 does not close here** —
the guard is its prerequisite, and the entry is rewritten, not struck.

---

## 8. Class names register in `global_bindings` (decision 0066, ruling 1)

A class declaration registers its name in the map `declare_global` maintains
(`compiler/lib/scope.rs:179`), so a class and an `import … as Name` can no longer both claim one
name in silence.

**Registration only — the class keeps its own check and its own diagnostic.** Do *not* route
`Statement::Class` through `declare_global`: that inherits `BindingRedeclared`'s guidance,

> `'{0}' is already declared in this scope; use assignment, or declare it in a nested scope to
> shadow.`

(`error.rs:77-78`, `CompilerError::BindingRedeclared`), which misinstructs **twice** for a class — you cannot assign one, and §6 bans
nested declarations outright.

| Collision | Reported as |
|---|---|
| class then class | `class.already_defined`, both locations (§3) |
| import then class | `class.already_defined`, both locations |
| class then import | `binding.redeclared`, from the import side — **no work here** |

⚠️ **`compiler/lib/mod.rs:57-61`'s doc comment documents the exemption this removes.** It
currently states class declarations "never interact with this map," naming reopening and kernel
stub completion as the reason. Update it in the same commit — the standing two-way-sync habit.

**Confirming test, not new work:** `import "a" as P` twice already errors via U-BINDINGS'
`declare_global` (`scope.rs:181-182`). Ruling 8's import half shipped with U-BINDINGS. Add the
fixture so the behavior is pinned by this unit's lane rather than left implicit in another unit's.

---

## 9. Write-set

| Path | Change |
|---|---|
| `phalcom-ast/src/parser.rs` | nested-class rejection at `:1557-1559`; doc comments `:1542`, `:1554` |
| `phalcom-core/src/compiler/lib/class_decl.rs` | delete the reopen machinery (`:288`, `:295-296`, `:319`); redefinition error; duplicate-member scan; reserved names; core gate at `:440`; `DefineGlobal` guard at `:738`; `global_bindings` registration |
| `phalcom-core/src/compiler/lib/error.rs` | four new variants / messages (§2.3, §3) |
| `phalcom-core/src/compiler/lib/mod.rs` | `global_bindings` doc comment `:57-61` |
| `phalcom-core/src/vm/dispatch.rs` | **delete** `:768-788` |
| `phalcom-core/src/bytecode.rs` | `Class` and `FinalizeClass` doc comments (§5.3). **No enum change** under §1.1's recommendation |
| `phalcom-core/src/diagnostics.rs` | `line_col` extraction only, under §3 option A |
| `phalcom-core/src/chunk.rs` | the two IC tests, in the existing `#[cfg(test)] mod tests` (§11) |
| `phalcom-core/tests/lang/classes/` + `classes/negative/` | delete 4 `class_reopen_*` fixtures + sidecars; add new negative fixtures |
| `phalcom-core/tests/lang/ic/` | delete 2 `.ph` + `.expected`, superseded by the in-crate tests |
| `docs/decisions/0066-…md` | the §3 mechanism amendment, if A is chosen |

**Not** in the write-set: `phalcom-core/core/core.ph` — zero true reopens, and stub completion is
untouched. No conflict with any `.ph`-editing unit in either order.

---

## 10. Build order

Each step an independently-green commit; verify each SHA **in a throwaway worktree**.

1. **Duplicate-member check** (§2.2). Self-contained, no reopen interaction, zero corpus cost.
2. **`@variant` regression test first, then the parser ban** (§6). The test proves the ban did not
   reach the compiler-side path; per §1.3 it cannot, so this step should be uneventful — which is
   exactly why it goes early and cheap.
3. **Redefinition error + the diagnostic** (§2.1, §3). Delete the 4 `class_reopen_*` fixtures in
   this step — they assert the behavior being removed.
4. **Reserved kernel names** (§4).
5. **Core gate + delete `dispatch.rs:768-788`** (§5). **Bootstrap green is the gate.**
6. **`None` `DefineGlobal` guard** (§7).
7. **`global_bindings` registration + `mod.rs:57-61` doc** (§8).
8. **The two IC tests** (§11), and delete their `.ph` fixtures.

`graphify update . --no-cluster` after the last code commit.

---

## 11. Tests

### 11.1 Migration is two tests, not a corpus sweep

| Fixture | Fate |
|---|---|
| `classes/class_reopen_appends_methods.ph` | **delete** — asserts the removed feature |
| `classes/class_reopen_field_bearing_appends_methods.ph` | **delete** |
| `classes/negative/class_reopen_add_field_rejected.ph` | **delete** — subsumed by `class.already_defined` |
| `classes/negative/class_reopen_superclass_conflict_rejected.ph` | **delete** — same |
| `ic/ic_add_method_invalidates.ph` | **rewrite** as an in-crate test (§11.2) |
| `ic/ic_override_after_caching.ph` | **rewrite** as an in-crate test (§11.2) |

Delete each fixture's `.expected` sidecar with it. Zero production `.ph`, zero examples, zero
`core.ph`.

### 11.2 The two IC tests, in-crate

Per §1.5 these go in `phalcom-core/src/chunk.rs`'s existing `#[cfg(test)] mod tests`, where
`pub(crate) world_version` is reachable. Both today are *positive* golden fixtures asserting a
printed `2` — a black-box proxy. In-crate they can assert the mechanism directly, which is
strictly better coverage:

- **`ic_add_method_invalidates`** — today: three `a.val` sends warm a monomorphic cache on
  `class A { val => 1 }`, then a reopen installs `val => 2`, and the printed `2` (not stale `1`)
  is the only evidence. Rewritten: compile and run a snippet via `VM::compile_closure` +
  `VM::run_in_module`, locate the `Bytecode::Invoke` index for the selector in
  `closure.callable.chunk.code`, read `chunk.caches[i].get()` to confirm it is **populated**, then
  install and assert the cache entry's `method: ObjRef` **changed** and `version` no longer matches.
- **`ic_override_after_caching`** — same shape with a heavily warmed site (10 sends), proving a
  hot cache is busted too, not just a lightly-touched one.

⚠️ **The trap that makes this test lie.** `ClassObject::add_method` does **not** bump
`world_version`; all six call sites bump it as a separate adjacent statement. A rewrite that calls
`add_method` alone leaves the cache valid against the stale entry and **passes while proving the
opposite of invalidation**. Bump explicitly, and add an assertion that the version actually moved
before asserting anything about the cache.

**Say in the return contract what coverage these assert**, so the swap from `.ph` to Rust is
auditable rather than taken on faith.

### 11.3 New negative-lane fixtures

All go in a `negative/` subdir or the suite reddens. Sidecar = one line, hand-authored, asserted
as a **substring** of `stdout\nstderr` (§1.6).

- duplicate class in one module → `class.already_defined`
- duplicate **method** in one body → `class.duplicate_member`
- duplicate **field** in one body → `class.duplicate_member`
- `class List` in a user module → `class.reserved_name`
- `class None` in a user module → `class.reserved_name`
- class declaration inside a method body → `class.nested_declaration`
- `import "m1" as Point` then `class Point` → `class.already_defined`
- `class Point` then `import "m1" as Point` → `binding.redeclared` (the reverse ordering; no
  implementation work, but it pins §8's table)
- `import "a" as P` twice → `binding.redeclared` (**confirming test**, shipped by U-BINDINGS)

**The both-locations fixture is the one to get right.** Under §3 option A its sidecar must assert
text only the two-location form can produce — e.g. `first declared at 3:1` — **not** merely
`is already defined`, which a single-span regression would satisfy silently. This is the entire
reason 0065 flagged it.

### 11.4 Positive lane

- `@variant` still expands (step 2's regression test, kept);
- bootstrap runs — implicitly gated by every test, but assert it explicitly at step 5;
- **the `None` binding survives**, using the genuinely-produced form from §7:
  `Some.new(5).filter { x => false } == None`. The obvious `x == None` form does **not** detect the
  defect.

---

## 12. Gates

- `./scripts/verify.sh` exits 0 (build + full `cargo test --workspace` + clippy).
- `cargo doc --workspace --no-deps` clean — every touched public item keeps full rustdoc, including
  the rewritten `Class`/`FinalizeClass` docs and `mod.rs:57-61`.
- clippy: no **new** warnings against the pre-existing baseline.
- Golden stdout unchanged except the six deleted fixtures. **No sidecar is re-blessed** — there is
  no bless mechanism (§1.6), so any change to an existing `.expected` is a hand edit and must be
  justified in the return contract.
- `graphify update . --no-cluster`.

---

## 13. What must this not preclude

- **The reflection layer** (ruling 7) — user classes only, never kernel, superclass links, or
  metaclasses. Do not delete the guard machinery it will need (§5), and do not open a public
  method-install API for a test's convenience (§1.5).
- **`None`'s body + `DEFERRED` #35** — §7's guard is their prerequisite. Leave them reachable;
  neither closes here.
- **Post-bootstrap freeze / H17** — this unit delivers the precondition (once core.ph has loaded,
  no method can be installed on a kernel class from any source). Do **not** enforce the freeze
  point here and do **not** claim the perf win; it is unmeasured
  ([`class-sealing-followups.md`](../../../deferred/class-sealing-followups.md) item 2).
- **A compile-error renderer** — §3 option A leaves both spans in the error value precisely so
  that B becomes a pure rendering change with no re-derivation.
- **U-REPL** — that branch's `DEC-REPL-A` is answered by ruling 6 (cells shadow, never reopen).
  This unit removes the seam `DEC-REPL-A` reasons about, so whichever lands second rebases onto a
  changed premise. Flag at integration.
- **ADR-0026 Axis 2** — reparenting stays sealed. Nothing here reopens that question.
- **U-BINDINGS L-5's exemption collapses into §5.1.** L-5 rejects same-scope redeclaration at the
  `DefineGlobal` site and exempts reopens, keyed on the same `field_layouts` miss/hit predicate
  this unit uses (independently derived). Once this unit lands, that exemption **becomes** §5.1's
  core gate and should collapse into it rather than persisting as a second special case.

---

## 14. Decisions required

Two, both surfaced rather than silently resolved. Neither blocks the other steps.

**14.1 The `Class` opcode shape, and the 0066 mechanism amendment.**

- §1.1 shows the ruled `Class(u16, bool)` operand is redundant — the compiler already emits
  `Constant` for stub completion, and the bool would only mean something if bootstrap were
  rewritten to stop doing that. **Recommendation: no operand.** The ruling chose between two
  shapes on a premise (`Class` is ambiguous to the compiler) that is false.
- §3 shows the ruled two-span *rendering* has no renderer to hang on. **Recommendation: option A**
  — both spans in the value, both locations in the message — plus a one-line amendment to decision
  0066 recording the mechanism change. 0066's own escape clause anticipates this
  ("if the cost proves higher than expected… amend this decision — do not under-deliver ruling 2
  silently"). Under A, ruling 2's intent is fully delivered; only "miette labels" changes.

**14.2 The IC-fixture home.** §1.5: `world_version` is `pub(crate)`, so the ruled "Rust-level
tests driving `add_method` + `world_version`" cannot be integration tests.
**Recommendation: in-crate `#[cfg(test)]` tests in `chunk.rs`**, which needs no API change. The
alternative — making `world_version` `pub` or adding a `VM::install_method` — previews ruling 7's
reflection layer for a test seam's convenience, and should not be done casually. This is the
conditional trigger the dispatch handoff named.

---

## 15. Return contract

Per-step SHAs with `git show --stat`. Confirmation that `core.ph` is untouched. The `@variant`
expansion proof (§10 step 2). Bootstrap green at step 5, called out separately. **What the two
in-crate IC tests assert**, explicitly, so the `.ph`→Rust swap is auditable and the
`world_version` trap (§11.2) is demonstrably avoided. Confirmation that `world_version` bumping
and the sacred-pristine machinery are intact. The `None` before/after using the
genuinely-produced form. Whether any existing `.expected` was hand-edited, and why.
`./scripts/verify.sh` + `cargo doc --workspace --no-deps` clean at each SHA, **verified in a
throwaway worktree**.
