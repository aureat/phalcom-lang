# U-REPL — cell-model REPL on a live VM

## Role

Turn `phalcom-repl` from a text-processing shell into a REPL backed by the running
VM: persistent bindings across cells, correct diagnostics, multi-line input, and
completion/highlighting driven by live runtime state rather than regexes.

Scope is the evaluation substrate and the input loop. The completion/highlighting
*surface* rides on it and is staged after.

## Naming

Written in the target keyword spelling: **`const`** = immutable, **`let`** = mutable.
The tree currently spells these `let`/`var`; the rename is pending and outside this unit.

Every mechanism here keys on **mutability**, never on a keyword — the existing
`immutable_globals` already does. **Rule: no new identifier in this unit encodes a
keyword name.** The rename must remain a pure find-and-replace that never touches
this work.

## Spec / ADR anchors

- ADR-0012 — selector signature encoding (comma-form)
- ADR-0014 — `let`/`var` bindings (immutability rule; D4 scopes, does not weaken it)
- ADR-0030 §6, D7 — fiber state ownership
- ADR-0056 §2 — `phalcom-lsp` stays VM-free (this unit does not violate it; see D8)
- ADR-0060 — `[]` as a real, directly-sent selector

**No new ADRs** (D5). Rationale lives in module-level `//!` docs and in named
regression tests. See "Tests" — several tests exist specifically to carry a
decision that has no ADR to live in.

## Preconditions (confirmed this session — do not re-derive)

Each verified against the tree at `301044e`, empirically where noted.

1. **Top-level bindings are already globals, not stack locals.**
   `compile_pattern_bind_top_of_stack(..., as_global: true)` emits `DefineGlobal`
   — `phalcom-core/src/compiler/lib/patterns.rs:44`, stated at
   `compiler/lib/mod.rs:46`. Persistence across cells needs no new mechanism.

2. **`ModuleObject` already holds the table.** `globals: Vec<Value>`,
   `name_to_slot`, `globals_version` (`heap/module.rs:49`). `declare()` is
   **idempotent** — redeclare returns the existing slot (`heap/module.rs:129`);
   `define()` overwrites. Rebinding leaks no slots.

3. **`immutable_globals` lives on `Compiler`** (`compiler/lib/mod.rs:51`) and is
   consulted at exactly one site, compile-time only (`compiler/lib/expr.rs:302`).
   A `Compiler` lives for one `compile_closure` call, so cross-cell rebinding
   already works — **by lifetime accident, untested and unstated**. D4 makes it
   deliberate.

4. **`run_in_module` clears `frames` and `stack` but NOT `open_upvalues`**
   (`interpret.rs:167`). `open_upvalues` is keyed by *absolute value-stack index*.
   Safe exactly once (outermost entry, map already empty); repeated per cell it
   aliases stale entries onto the next cell's slots. See D10.

5. **Fibers own private buffers.** `FiberObject` holds its own `stack`, `frames`,
   `open_upvalues`, empty while running and mirrored into the VM's
   (`heap/fiber.rs:62`). At a cell boundary only the *running* fiber's state is in
   `vm.*`; suspended fibers are untouched by clearing and remain resumable.

6. **`runtime_error` pairs a compile-time span with the module's *current* source**
   (`vm/dispatch.rs:131`) and `.unwrap()`s it. One module + many cells renders
   earlier-cell spans against later-cell text. Latent panic on the `None` path.

7. **The parser never produces `UnrecognizedEof`.** Every truncation surfaces as
   `UnrecognizedToken { token: "", … }` — EOF is modelled as a zero-length token,
   so the purpose-built variant is dead. Verified across 15 inputs.

8. **Strings span newlines.** `"abc\ndef"` parses clean; `UnterminatedString`
   fires on end-of-input, not newline. An unterminated string is therefore
   *incomplete*, not an error.

9. **reedline 0.41 exposes `Validator` / `ValidationResult`** — the continuation
   hook, invoked per submission, not per keystroke.

10. **`ReplSession::eval` is a stub** (`phalcom-repl/src/repl.rs:34`) — increments a
    counter, returns it. The REPL evaluates nothing today.

11. **`encode_selector` already emits comma-form** (`method/mod.rs:102`), the same
    ADR-0012 spelling `phalcom-lsp`'s `comma_form` builds independently. A
    **total, non-panicking** inverse exists (the `doesNotUnderstand` path uses it).
    No format translation is needed — only drift control.

12. **`SignatureKind::Initializer` is retired for definition.** A `construct`
    installs one method, class-side, under its ordinary selector (`new(_)`), not an
    `init `-prefixed one; the kind survives only as a `SuperSend` marker
    (`compiler/lib/class_decl.rs:602`). Verified: `B.methods → [#x]`,
    `B.class.methods → [#new(_)]`.

13. **Variadics work end-to-end.** `static sum(*nums)` called as `sum(1,2,3)`
    yields `[1, 2, 3]`. Zero variadic definitions exist in `core.ph`.

**Explicitly out of scope, tracked separately:** `core-table.json` is ~40% stale
(30→52 classes, 214→305 entries) and `examples/simple.ph` no longer compiles
(`@construct` rejected). Both spun off. This unit must not depend on either.

## Design

### D1 — One session module; each cell is its own compilation unit

The session owns a `VM` and a single `ModuleObject`. Each cell:
`compile_closure(module, src, Repl)` → run → print. Globals accumulate in the
shared table (precondition 1–2), so bindings persist with no new machinery.

Rejected: replaying the accumulated buffer per cell (side effects re-fire, O(n²));
cell-as-module with a parent-lookup chain (adds a fallback walk to the hot
`GetGlobal` path to serve a REPL-only feature).

### D2 — Source binds to the artifact, not to the module

```
Chunk        + source_id: u32              // beside `spans`; debug data, not hot
ModuleObject + sources: Vec<Arc<String>>   // cell-indexed
Compiler     stamps source_id into every Chunk it builds
dispatch.rs  chunk.source_id → module.sources[id]
```

Fixes precondition 6 for the REPL and removes the `.unwrap()` panic generally.

**Ownership vs U-TRACE (ruled: split).** `docs/deferred/tracing.md` step 2 targets the
same function for the same defect — *"store module_id and resolve source lazily so
defect 1's unwrap has nowhere to live."* These are different changes that happen to
meet in one place: U-REPL owns **where source lives** (`source_id` on the artifact);
U-TRACE owns **when it is resolved** (lazy resolution, compact capture at raise).
`source_id` is a precondition for U-TRACE's compact-capture record, so this ordering
serves both. U-REPL must not implement lazy resolution; U-TRACE must not re-litigate
where source is stored.

Prior art: CPython names REPL input `"<stdin>"` and cannot show source in tracebacks
through REPL-defined functions; IPython registers each cell in `linecache` and can.
This is IPython's fix applied at the compiled artifact instead of bolted on, and it
is the item that is hardest to retrofit once spans are baked.

### D3 — `CompileMode::Repl`

> **Note (Implementation Delta):** `CompileMode` governs contract-weaving (Debug/Release/Unchecked).
> As specced in `impl/README.md` delta 2 and `impl/02-session-and-cells.md §2.2`, the REPL compile mode is represented by a separate orthogonal `UnitKind` enum (`UnitKind::File` vs `UnitKind::Repl`), not a `CompileMode` variant.

A Repl-mode unit suppresses the trailing `Pop` on a **final expression statement**
and leaves the value for the loop to print; statements echo nothing. `_` binds the
last value as an ordinary global.

Mirrors CPython's `"single"` compile mode. Rejected: Lua-style `return `-prepending
with retry (double-compiles, misreports spans on the retry path).

### D4 — Immutability stays strong; the REPL gets a named exemption

> **Rewritten 2026-07-19 against U-BINDINGS.** This section originally specced
> `Compiler::immutable_globals`, a `HashSet<Symbol>`. `42aafce` replaced it with
> `Compiler::global_bindings: HashMap<Symbol, bool>` (name → is_mutable,
> `compiler/lib/mod.rs:62`) and added a **same-scope redeclaration ban**
> (`scope.rs:179-185`). The design below survives that change; the field name and
> shape did not.

```
Compiler.global_bindings      // bindings from THIS unit
                              //   → assignment to a `false` (const) entry errors (expr.rs:303)
                              //   → redeclaring any entry errors     (scope.rs:181)

ModuleObject.global_bindings  // bindings from PRIOR units, same shape
                              //   → both checks above apply, UNLESS CompileMode::Repl
```

The REPL exemption must relax **both** checks for prior-unit entries, not just the
const one: without lifting the redeclaration ban, `const x = 1` in cell 1 followed
by `const x = 2` in cell 2 errors, which is precisely Wren's behavior this unit
exists to avoid.

**One lifetime accident now carries three rulings.** `Compiler` being constructed
per-cell is currently what makes cross-cell rebinding work — and since U-BINDINGS
and PDR-0001 landed, it is *also* what makes the redeclaration ban not fire
across cells, and what lets 0065 ruling 6's class shadowing work (class
declarations register in `global_bindings` per PDR-0002, and that registry is
per-`Compiler` too). Three independent rulings, one undocumented lifetime. The
regression test below is the only thing that would catch a refactor breaking any of
them.

| case | result |
|---|---|
| file: `const x = 1` … `x = 2` | error — ADR-0014 unchanged |
| cell 1 `const x = 1`; cell 2 `x = 2` | allowed |
| both in **one** cell | error — a real mistake, still caught |
| future non-REPL multi-unit path | error — no accidental pre-authorization |
| `xx = 2`, never declared | runtime error — unchanged (`SetGlobal` has no core fallback) |

`const` remains a promise about the *binding*, now enforced across units — strictly
stronger than today, where it holds only by the accident in precondition 3. The
exemption is confined to a named mode rather than weakening the rule globally.

Prior art: Wren errors on redefinition (`wrenDefineVariable` returns `-1`,
`resources/wren/src/vm/wren_vm.c:1575`) and its REPL is unusable for it. V8 built a
dedicated REPL mode to make `let`/`const` redeclarable. Erlang needed `f(X)` to
forget a binding. Rebinding is not optional in a REPL.

### D7 — Multi-line continuation

**Route EOF to `UnrecognizedEof`.** Fix the parser to emit the variant that already
exists for this and is currently dead (precondition 7). Detection quality is
identical to sniffing `token == ""`, but the signal becomes named, documented and
testable rather than a load-bearing implementation detail — the same failure shape
as precondition 3, which this unit exists partly to remove.

Pays for itself independently: `class Foo {` currently reports `Unexpected input: ""`
— an error message about an empty token. Routed, it reads
`Unexpected end of file. Expected "}"`. The LSP gets that too.

Drives a reedline `Validator` (precondition 9). Classification:

| input | verdict |
|---|---|
| `class Foo {`, `let x = 1 +`, `foo(1,`, `[1, 2,` | incomplete |
| `let s = "abc` (strings span lines, precondition 8) | incomplete |
| `let x = )`, `1 +* 2` | error — non-empty offending token |
| `let x = 1`, `` (empty) | complete |

Plus:
- **Trailing `\`** as explicit continuation. Must be **stripped and joined before
  lexing** (`\` is not in the grammar and would lex as `InvalidToken`).
  *Consequence:* compiled text ≠ typed text, so byte offsets shift. `sources[]`
  stores the **compiled** text so spans stay valid; history/echo therefore shows
  joined lines. Accepted.
- **Escape from a stuck buffer** — blank line submits as-is, Ctrl-C discards. A
  mis-detected `Incomplete` must never trap the user.
- **`...` continuation prompt** so the state is visible.

Rejected: delimiter counting (blind to `let x = 1 +`); Python's `codeop`
compile-and-retry (exists only because CPython exposes no clean signal — Phalcom
will have one).

### D8 — Live oracle from the start

Two sources behind one interface. Static is the floor (the line being typed has not
executed, so only a parser can speak to it); live overrides wherever it has an answer.

Live provides what static structurally cannot:
- **receiver class for any bound name** — ask the value its class, walk the real
  method dictionary (the Smalltalk path). Static's `ConstructResolver` handles only
  `let x = Cls.new(…)`.
- **unbound-name detection** — a name absent from `name_to_slot`, locals, and
  globals. A pre-flight `doesNotUnderstand`. Static cannot know what has *run*.
- **runtime-added methods**, and immunity to `core-table.json` drift.

Merge rule: **live wins for names that exist at runtime; static covers the current,
not-yet-executed line.** They disagree only when a name was rebound since the static
view was built — there live is simply correct.

`phalcom-repl` depends on both `phalcom-lsp` and `phalcom-core`. The arrow never
reverses, so ADR-0056 §2's VM-free constraint on the LSP is untouched.

### D9 — Selector representation

Structured `{name, labels, kind}` on both sides. The VM side uses the existing total
decoder (precondition 11); the LSP side already has `MemberKind` + `ParameterDef`s.
The UI renders from structure — bare `size`, `name = value`, `at(_)` — and snippet
slots read off `labels`. Dedup on `(name, kind, arity)`.

Rejected: canonicalising to an encoded string, then re-parsing it to render. Lossy
round-trip for no gain.

Kinds to reconcile: **getter / setter / method / subscript / variadic**.
`Initializer` is not among them (precondition 12).

### D10 — Cell boundaries unwind; they do not raw-clear

Use `unwind_to(0, 0)`, which closes open upvalues before truncating, **not**
`run_in_module`'s raw `frames.clear(); stack.clear()`.

Precondition 4 is the reason: the map is keyed by absolute stack index, so clearing
the stack beneath it aliases cell N's captured slots onto cell N+1's values — silent
corruption, not a crash. Same family as the F1 fiber-floor defect and E001–E003.

Suspended fibers are unaffected (precondition 5) and stay resumable from a later cell.

**Consequence for the loop:** `runtime_error` prints a trace but does **not** unwind.
The cell loop owns unwinding after a failed cell and must not assume the VM cleaned up.

## Stage boundaries (each independently verifiable green)

1. **D2** — cell source map. Standalone; fixes a live `.unwrap()` panic with no
   REPL present. No dependencies.
2. **D7 parser half** — route EOF to `UnrecognizedEof`. Standalone; improves
   diagnostics with no REPL present.
3. **D1 + D10 + D3** — session module, unwinding cell loop, echo mode. First point
   at which the REPL evaluates anything.
4. **D4** — two-set immutability + cross-cell rebind test.
5. **D7 REPL half** — `Validator`, trailing `\`, `...` prompt, escape hatches.
6. **D8 + D9** — live oracle, structured selectors; delete the regex highlighter
   and `guess_type_from_name`.

Stages 1–2 land value even if the unit stalls.

## Write-set (STOP-and-report if outside)

- `phalcom-core/src/chunk.rs`, `heap/module.rs`, `interpret.rs`, `vm/dispatch.rs`
- `phalcom-core/src/compiler/lib/{mod,expr,patterns}.rs`
- `phalcom-ast/src/{parser,error}.rs` (D7 parser half only)
- `phalcom-repl/**`
- `docs/forge/units/U-REPL/**`

**Not in the write-set:** `tools/vsphalcom/src/generated/core-table.json`,
`examples/**`, `phalcom-lsp/**` (consumed as a library; not modified).

## Tests / verification

Carrying weight that would otherwise sit in an ADR (D5):

- **cross-cell rebind** — `const x` in unit 1, assign in unit 2 on one module,
  under `CompileMode::Repl`, succeeds; under file mode, errors; within one unit,
  errors. This is the test that stops a future refactor from silently restoring
  Wren's behavior.
- **cell source map** — a method defined in cell 1 raising in cell 3 renders cell
  1's source, not cell 3's.
- **open-upvalue hygiene** — a cell that errors with a live captured local, followed
  by a cell that pushes new values, does not alias. Regression for D10.
- **continuation classification** — promote `phalcom-ast/tests/probe_continuation.rs`
  (currently a `println` probe, untracked scratch) into assertions over the table in
  D7: complete / incomplete / error per input.
- **selector conformance** — walk every core class's **live** method dict and assert
  each renders to correct comma-form under D9. Live-authoritative and standalone;
  deliberately does **not** diff against `core-table.json`, so it carries no
  dependency on the spun-off regeneration task.
- **echo mode** — expression cells yield a value, statement cells do not, `_` tracks.

## Decisions to flag (DEC-REPL)

- **DEC-REPL-A — class redefinition. CLOSED** by
  [PDR-0001](../../../pdr/0001-classes-are-closed.md) ruling 6: *REPL
  cells shadow; they do not reopen.* A later cell's `class Foo` binds a **new**
  class; instances made under the old definition keep it (they hold a `ClassId`,
  nothing migrates); the old class becomes unreachable by name. No live object is
  ever silently patched. Ruling 6 answers this with the diagnosis recorded above
  and confirms it needs no machinery beyond §D1/§D2. **Do not re-decide.**
- **DEC-REPL-B — snippet insertion. CLOSED** by [`surface.md`](surface.md) §S7:
  insert `name(` with the cursor inside; arity-0 selectors insert the bare name. A
  tab-stop engine is its own unit — it needs a placeholder state machine and a rule
  for Tab, which §S4 has already bound to ghost text. **Do not re-decide.**
- **DEC-REPL-C — dead editor stack. CLOSED** by [`surface.md`](surface.md) §S8:
  delete `phalcom-repl/src/rustyline/` and drop the `rustyline` dependency as the
  **first** implementation step, before §S4/§S5 rewrite the hinter, highlighter, and
  completer — so those rewrites have one target instead of two. **Do not re-decide.**

Nothing under DEC-REPL is open. The surface's command namespace
([`surface.md`](surface.md) §S9 — `:reload`, `:reset`, `:help`, only `:reload` built)
is ruled there and carries no §D-series dependency.

## What must this not preclude (P4)

- **Fibers.** D10 leaves suspended fibers resumable across cells (precondition 5).
  Nothing here assumes single-fiber execution.
- **The `const`/`let` rename.** No identifier introduced here encodes a keyword.
- **A REPL-as-server model.** Nothing in D8 assumes the oracle is in-process; the
  merge rule would survive the REPL becoming a server the editor queries (the
  Clojure/nREPL shape), should that ever be wanted.
- **`core-table.json` regeneration.** The conformance test is live-authoritative
  precisely so the two units never block each other.
