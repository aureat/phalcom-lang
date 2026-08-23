# Phalcom Collection Literal Spread Completion — Part I
## Frontend, Shared Positional Expansion Architecture, and List `*` Support

**Status:** implementation specification / execution plan
**Repository baseline:** `aureat/phalcom-lang` at `9ca245f86a62e0b3064d88027f7992c27330f36d`
**Depends on:** F.1/F.2 outgoing-pack infrastructure, E.1 cursor protocol, E.3 boundedness, D.2 direct List literal construction
**Companion:** Part II — labeled expansion, Record/Map construction, diagnostics, traceback, and completion gates

---

## 1. Mission

Close the remaining collection-literal spread gap without regressing the already-landed outgoing-pack or Tuple behavior.

This part implements:

```phalcom
[a, *source, b]
```

with the same positional expansion semantics already used by outgoing argument packs and dynamic Tuple construction.

This part also performs the frontend and compiler refactoring needed so positional expansion has **one semantic definition** rather than separate implementations for:

- outgoing argument packs;
- Tuple literals;
- List literals.

It does **not** implement Record or Map `**` expansion. Those are Part II.

---

## 2. Ratified language decisions

These decisions are fixed for this implementation.

### 2.1 Literal/operator matrix

| Literal | `*` | `**` | `***` |
|---|---:|---:|---:|
| Tuple | yes | yes | yes |
| List | yes | no | no |
| Record | no | yes | no |
| Map | no | yes | no |

Tuple support is already live and is a regression target, not a missing feature.

### 2.2 List has only a positional lane

A List contains an ordered sequence of values and has no labeled lane.

Therefore:

```phalcom
[*source]
```

is legal.

These are syntax errors:

```phalcom
[**source]
[***source]
```

The parser should reject these forms immediately and explain the invalid lane/operator relationship.

### 2.3 Positional expansion source semantics

`*source` accepts exactly:

1. `Unit` — contributes zero values;
2. `Tuple` — contributes only its positional lane;
3. any value implementing the ordinary cursor protocol required by F.2:
   - `iterate(_)`
   - `iteratorValue(_)`

No other special cases are introduced.

### 2.4 Tuple positional expansion does not mean Tuple iteration

This is a required semantic invariant.

For:

```phalcom
const t = (1, 2, name: 3)
[*t]
```

the result is equivalent to:

```phalcom
[1, 2]
```

not:

```phalcom
[1, 2, 3]
```

Ordinary Tuple iteration and `*Tuple` are different operations. `*Tuple` is a lane projection.

### 2.5 Duplicate policy

Lists have no labels and therefore no spread-induced duplicate error.

Duplicate semantics for Tuple/Record/Map remain unchanged and are handled in Part II where relevant.

---

## 3. Current repository state

The implementation must build on the live architecture rather than replacing it.

### 3.1 Tuple spread is already implemented

Current compiler lowering for a Tuple with expansion or a computed label:

- allocates an `ArgumentPackBuilderObject`;
- roots it in a compiler scratch local;
- processes entries in lexical order;
- uses `compile_positional_pack_expansion` for `*`;
- uses the existing label-expansion bytecodes for `**` / `***`;
- finalizes with `FinishTuplePack`.

Do not replace this with a new Tuple-specific builder.

### 3.2 List AST has a reserved expansion shape

`phalcom-ast/src/ast.rs` already has:

```rust
pub enum ListLiteralElement {
    Element { expr: Expr, range: SourceRange },
    Expansion { expr: Expr, range: SourceRange },
}
```

The parser currently does not produce the `Expansion` arm.

### 3.3 List parser still contains stale deferred-spread logic

`parse_list_literal` currently routes through `parse_comma_exprs`, and that helper rejects a leading `*` with a “not yet supported” syntax error.

The parser comments are also stale: they still describe old List desugaring through `List.new().add(...)`, even though List literals now have an explicit AST and compile through `BuildList`.

Both the behavior and comments must be corrected.

### 3.4 Static List construction is already good

Ordinary List literals compile as:

```text
evaluate e0
evaluate e1
...
evaluate eN
BuildList(N)
```

`BuildList(u16)` allocates the native List directly from the collected values.

This remains the fast path.

### 3.5 F.2 already has the correct generic `*` loop

Current `compile_positional_pack_expansion` already:

- invokes E.3 boundedness checking;
- evaluates the spread source exactly once;
- stores the source in a scratch local;
- stores the cursor in a scratch local;
- performs a Unit/Tuple fast probe;
- otherwise uses `iterate(_)` / `iteratorValue(_)`;
- appends every produced value to an argument-pack builder;
- keeps source and cursor GC-rooted across arbitrary sends.

This is the implementation to generalize, not duplicate.

---

## 4. Design principle: share expansion semantics, not destination storage

Do **not** build a universal collection builder.

Instead, separate:

```text
How does *source produce positional values?
```

from:

```text
Where are those values appended?
```

The compiler should have one positional expansion engine and a small destination abstraction.

Conceptually:

```rust
enum PositionalExpansionTarget {
    ArgumentPack { builder_slot: u16 },
    ListLiteral { list_slot: u16 },
}
```

Exact names may follow repository conventions.

The shared engine owns:

- boundedness checking;
- source evaluation;
- source scratch rooting;
- cursor scratch rooting;
- Unit/Tuple probing;
- generic iteration loop;
- source spans;
- loop control and scratch release.

The target owns only:

- fast Tuple positional-lane consumption;
- append-one-value behavior.

This prevents semantic drift between calls, Tuples, and Lists.

---

## 5. Frontend changes

### 5.1 Replace List use of `parse_comma_exprs`

Do not extend the generic `parse_comma_exprs` helper with List-specific spread semantics.

Instead, give List literals their own entry parser, parallel to Tuple/Map parsing.

Recommended conceptual helper:

```rust
fn parse_list_literal_element(&mut self) -> ParserResult<ListLiteralElement>
```

Rules:

1. If the current token is `*`:
   - record the `*` token start;
   - consume `*`;
   - parse one ordinary expression;
   - create `ListLiteralElement::Expansion`;
   - entry range spans from `*` through the expression.
2. If the current token is `**`:
   - syntax error anchored on `**`;
   - message explains that Lists have only a positional lane and `*` is the valid expansion operator.
3. If the current token is `***`:
   - syntax error anchored on `***`;
   - message explains that complete two-lane expansion is not meaningful for a List.
4. Otherwise:
   - parse an ordinary expression;
   - create `Element`.

Preserve:

- comma separation;
- trailing commas;
- newlines where existing List grammar permits them;
- empty `[]`.

### 5.2 Required diagnostics

Prefer explicit messages such as:

```text
`**` is not valid in a List literal; List has only a positional lane, use `*`
```

and:

```text
`***` is not valid in a List literal; complete expansion requires positional and labeled lanes, but List has only a positional lane
```

Exact wording may be adapted to repository style, but the error must identify:

- the operator;
- the destination literal;
- the lane mismatch;
- the valid alternative where useful.

Use the operator token's source range, not the whole literal.

### 5.3 Remove stale parser comments

Update `parse_list_literal` documentation so it describes:

- explicit `Expr::ListLiteral`;
- direct compiler construction;
- `*` expansion support;
- no public mutation-chain desugaring.

Do not leave historical claims that `List#add` is used to construct literals.

---

## 6. Compiler shape selection

The compiler should branch on whether the List contains expansion.

### 6.1 Static List

For a List with no `Expansion` entries:

- keep current behavior;
- retain `BuildList(u16)`;
- retain current count check;
- do not allocate a dynamic builder;
- do not reserve compiler scratch locals.

This is the hot/common path.

### 6.2 Dynamic List

For a List containing at least one `Expansion` entry:

- use an incremental hidden native List;
- do not use `BuildList(u16)`;
- do not use `ArgumentPackBuilderObject`;
- do not impose the static `u16` entry-count limit on the runtime result.

Recommended bytecodes:

```rust
BeginListLiteral
ListLiteralAppend
FinishListLiteral
```

Exact names may follow repository conventions.

Alternative naming is acceptable if stack contracts remain clear and the distinction from public List mutation is explicit.

---

## 7. Dynamic List bytecode contracts

### 7.1 `BeginListLiteral`

Semantic contract:

```text
...
BeginListLiteral
... | list
```

It allocates a fresh ordinary native `Object::List`.

The List is not a user-observable intermediate value. It becomes observable only after successful literal completion.

An optional immediate capacity hint may be encoded later, but capacity is not semantic.

### 7.2 `ListLiteralAppend`

Recommended stack contract:

```text
... | list | value
ListLiteralAppend
... | list
```

The opcode:

- validates the hidden builder is a List as an internal invariant;
- appends `value` directly to `ListObject::push`;
- performs no public selector dispatch;
- returns no user-visible mutation result.

This is an internal construction opcode, not a primitive binding.

### 7.3 `FinishListLiteral`

Recommended stack contract:

```text
... | list
FinishListLiteral
... | list
```

It may be a semantic/no-op finalization marker if no runtime transformation is necessary.

Retaining an explicit finish opcode is preferred because it:

- documents the builder lifetime;
- mirrors Map literal construction;
- gives tests a stable boundary;
- leaves room for future construction assertions/optimizations.

If implementation conventions strongly favor eliminating a no-op finish opcode, the spec may be satisfied by compiler-owned finalization with equivalent invariants, but that choice must be documented.

---

## 8. Rooting strategy for dynamic List construction

A List literal may contain nested sends and generic spread iteration. The destination List must remain rooted across all of them.

Recommended lowering:

1. reserve a compiler scratch local for the hidden List;
2. emit `BeginListLiteral`;
3. store the resulting List handle in that local;
4. pop the transient copy;
5. process every literal contribution;
6. reload the List;
7. finish;
8. release the scratch local.

Do not rely on an untraced Rust local inside VM code across arbitrary language execution.

This follows the same discipline as F.2's pack builder and source/cursor locals.

---

## 9. Refactor `compile_positional_pack_expansion`

Rename/generalize the routine so it no longer hardcodes an argument-pack destination.

Recommended conceptual API:

```rust
fn compile_positional_expansion(
    &mut self,
    target: PositionalExpansionTarget,
    expr: Expr,
    range: SourceRange,
) -> Result<(), CompilerError>
```

or equivalent decomposition.

The implementation must preserve the current F.2 evaluation order exactly.

### 9.1 Required shared sequence

For `*expr`:

```text
1. E.3 check
2. evaluate expr exactly once
3. store source in hidden local
4. reserve cursor local
5. try Unit/Tuple positional projection
6. if handled -> done
7. otherwise validate/use generic cursor protocol
8. call iterate(source, None)
9. loop while cursor != None:
      value = iteratorValue(source, cursor)
      append value to target
      cursor = iterate(source, cursor)
10. release scratch locals
```

No later literal entry may begin evaluating until the generic source is fully exhausted.

### 9.2 Tuple/Unit fast path

The existing `PackTryExpandTuplePositionals` opcode is pack-destination-specific.

Do not blindly reuse it for Lists.

Choose one of these designs:

**Preferred:** extract the Unit/Tuple source classification/projection into a destination-neutral runtime helper and expose target-specific bytecodes that call it.

For example:

```rust
TryAppendTuplePositionalsToPack
TryAppendTuplePositionalsToList
```

Both delegate to the same runtime function that answers whether the source was Unit/Tuple and exposes the positional lane.

**Acceptable:** add a destination-neutral “snapshot positional lane if Unit/Tuple” helper and have the two VM opcodes consume its result.

Do not implement two independent type-switch definitions.

### 9.3 Generic Iterable validation

Preserve the F.2 error:

```text
* expansion requires Tuple, Unit, or an iterable value; got {found}
```

Do not invent a List-specific type error.

The semantics belong to `*`, not to its destination.

---

## 10. E.3 boundedness integration

Every List `*` expansion is a full-source exhaustor.

It must call the same E.3 entry point as outgoing positional spread:

```rust
boundedness::require_exhaustible(...)
```

or the existing compiler wrapper.

### 10.1 Required compile-time behavior

Reject:

```phalcom
[*(0..)]
```

Reject through immutable fact propagation:

```phalcom
const xs = (0..)
[*xs]
```

Allow:

```phalcom
[*(0..10)]
```

Allow:

```phalcom
[*(0..).iter.take(10)]
```

Allow when static boundedness is Unknown:

```phalcom
[*someIterable]
```

The checker must continue following its soundness rule:

```text
reject only proven Unbounded
Unknown is legal
```

### 10.2 Diagnostic operation name

Prefer an operation description that identifies literal spread, for example:

```text
cannot exhaust a provably unbounded source with `expansion`
```

Reusing the current F.2 `expansion` wording is acceptable and keeps behavior consistent.

---

## 11. Evaluation order

For:

```phalcom
[first(), *source(), later()]
```

required order is:

```text
first()
source() exactly once
fully exhaust source
later()
```

If:

- `source()` throws;
- iteration setup throws;
- `iteratorValue` throws;
- iteration advancement throws;

then `later()` must not run.

Within a generic source, each cursor operation follows the existing F.2 cursor protocol order.

Do not interleave later literal expressions with source exhaustion.

---

## 12. Failure atomicity

A dynamic List literal is observationally atomic.

If construction fails:

- the partially populated native List must not become visible to source code;
- normal unwind removes the only compiler-owned root;
- the GC may reclaim the abandoned List later.

No rollback is needed because no user code was handed the destination List.

This is the same construction principle already used for Map literals.

---

## 13. Runtime cardinality and limits

### 13.1 Static List limit

`BuildList(u16)` may continue limiting the number of statically encoded List entries because that is an instruction operand constraint.

### 13.2 Dynamic List result

A spread-containing List literal must **not** inherit the `u16` static count limit.

For example:

```phalcom
[*largeIterable]
```

may produce more than 65,535 values if memory permits.

The dynamic path uses `Vec<Value>` growth and therefore naturally supports runtime cardinality.

Do not add a semantic List-size cap solely to mirror the static bytecode operand.

---

## 14. Allocation and performance policy

### 14.1 Ordinary Lists

No change:

- no builder object;
- no scratch locals;
- no loop machinery;
- one direct `BuildList`.

### 14.2 Dynamic Lists

Use an ordinary native List as the builder.

Do not add:

- `Object::ListBuilder`;
- a Rust-only side stack;
- a temporary argument pack;
- public List mutation sends.

### 14.3 Capacity hint

Optional first implementation:

```text
capacity >= number of explicit literal elements
```

This is useful and cheap.

Do not perform complicated compile-time size theorem proving solely for capacity reservation.

A later optimization may incorporate known finite spread cardinalities.

---

## 15. Bytecode bookkeeping

If new bytecodes are added, update every synchronized surface required by the repository:

- `phalcom-core/src/bytecode.rs`
  - enum variants;
  - `BYTECODE_NAMES`;
  - `Bytecode::VARIANTS`;
  - exhaustive `Bytecode::index`;
- VM dispatch exhaustive match;
- disassembly/debug output;
- opcode histogram/index assumptions;
- tests asserting bytecode shape.

Do not leave bytecode-name/index tables partially updated.

---

## 16. Source-span policy

Stamp instructions deliberately.

| Instruction/work | Span |
|---|---|
| `BeginListLiteral` | full List literal |
| explicit element expression | expression's own span |
| explicit `ListLiteralAppend` | element entry span |
| `*` source evaluation | source expression span |
| Unit/Tuple spread probe | spread entry span |
| compiler-generated `iterate` send | spread entry span |
| compiler-generated `iteratorValue` send | spread entry span |
| appending yielded value | spread entry span |
| `FinishListLiteral` | full List literal |

This ensures a runtime error inside user iteration code preserves the user frame while the calling frame points to the actual `*source` contribution.

---

## 17. Error propagation

Do not catch and rewrite errors raised by:

- `source` evaluation;
- `iterate(_)`;
- `iteratorValue(_)`;
- nested user code called by those methods.

A thrown user error must remain the original error.

Do not emit wrappers such as:

```text
List spread failed
```

unless the failure is genuinely a compiler/runtime invariant violation.

Expansion capability errors should use the existing structured `*` error.

---

## 18. Tuple regression requirements

The positional refactor must not alter Tuple semantics.

Required regression cases:

```phalcom
(*(1, 2),)
```

or equivalent syntax must continue to expand Tuple positionals.

A mixed Tuple:

```phalcom
const t = (1, 2, name: 3)
```

must contribute only `1, 2` under `*`.

Unit remains an empty positional contribution.

Generic iterable fallback remains unchanged for outgoing packs and Tuple construction.

Existing F.2 nested dynamic-send operand-window regression tests must stay green.

---

## 19. Parser/AST/LSP audit

List itself already has an expansion AST arm, so the AST type need not change.

Nevertheless, inspect all exhaustive consumers of `ListLiteralElement` and verify they already handle `Expansion`.

At minimum audit:

- parser;
- compiler;
- LSP/index traversal;
- source-range visitors;
- format/debug AST tests;
- boundedness inference.

Any walker that currently assumes every List entry is an ordinary expression must be updated.

---

## 20. Required tests

### 20.1 Parser

- `[]`
- `[1]`
- `[1, 2,]`
- `[*xs]`
- `[1, *xs, 2]`
- multiline spread literal
- `[**xs]` precise syntax error
- `[***xs]` precise syntax error

### 20.2 Static fast path

Compile an ordinary List and assert `BuildList` remains present.

Assert no dynamic List bytecodes appear when there is no spread.

### 20.3 Dynamic bytecode path

Compile `[1, *xs, 2]` and assert:

- incremental List construction is used;
- `BuildList` is not used for the whole literal;
- shared positional expansion lowering is present.

### 20.4 Value semantics

Test:

- Unit spread;
- Tuple spread;
- Tuple with labeled lane ignores labels under `*`;
- List spread;
- bounded Range;
- custom Iterable;
- lazy pipeline;
- empty source.

### 20.5 Invalid source

```phalcom
[*42]
```

must produce:

```text
* expansion requires Tuple, Unit, or an iterable value; got int
```

or the repository-equivalent type name.

### 20.6 Evaluation timing

Use a stateful probe proving:

```text
explicit-before
source creation
all source iteration
explicit-after
```

and proving source operand executes exactly once.

### 20.7 Failure short-circuit

Throw during source evaluation and iteration.

Verify later List elements do not execute.

### 20.8 Boundedness

Pin:

- direct open Range rejection;
- `const` propagation rejection;
- `take` legal;
- Unknown legal.

### 20.9 GC/rooting

Force enough allocation to trigger GC while:

- the source is stored in a scratch local;
- the cursor is live;
- the hidden List already contains heap object values.

Verify no stale handles and correct final contents.

### 20.10 Nested dynamic operand windows

Add a regression where the spread source expression itself performs a dynamic pack send, and the List literal is nested inside another call/expression.

This test is mandatory because current HEAD specifically fixed nested dynamic-send operand windows.

### 20.11 Large runtime result

Create a dynamic spread result above the static `u16` count boundary without spelling that many literal entries.

Assert successful construction.

---

## 21. Recommended write-set

Expected primary files:

```text
phalcom-ast/src/parser.rs

phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/heap/list.rs         # only if capacity/helper API changes

phalcom-lsp/src/index.rs              # audit/update if needed

phalcom-core/tests/...                # new literal-spread completion suite
```

Potential secondary files depending on bytecode tooling:

```text
phalcom-core/src/chunk.rs
phalcom-core/bin/phalcom/...
opcode/disassembly test files
```

Do not modify E.3 semantics unless a real integration defect is discovered.

---

## 22. Suggested implementation sequence

### Phase 1 — parser

1. Add List-specific element parsing.
2. Produce `ListLiteralElement::Expansion`.
3. Reject `**` / `***` with precise lane diagnostics.
4. Remove stale List-desugaring comments.
5. Add parser tests.

### Phase 2 — shared positional lowering

1. Refactor `compile_positional_pack_expansion` into destination-aware shared lowering.
2. Preserve outgoing-pack behavior byte-for-byte where practical.
3. Keep E.3 call centralized.
4. Add unit/compiler tests before wiring List.

### Phase 3 — dynamic List construction

1. Add List literal builder bytecodes.
2. Add VM handlers using ordinary `ListObject`.
3. Root destination in scratch local.
4. Route List `Expansion` through shared positional lowering.
5. Keep `BuildList` for static Lists.

### Phase 4 — hardening

1. GC stress.
2. nested dynamic operand-window tests;
3. traceback/source-span tests;
4. >`u16` dynamic-result test;
5. opcode bookkeeping verification.

---

## 23. Completion gate

Part I is complete only when all of the following are true:

- List `*` syntax parses and executes;
- List `**` / `***` fail at parse time with informational lane diagnostics;
- `*` semantics are shared with F.2 rather than duplicated;
- Tuple positional projection remains correct;
- generic Iterable expansion uses the existing cursor protocol;
- source and cursor remain GC-rooted;
- E.3 rejects only provably unbounded spread sources;
- ordinary List literals still use the static `BuildList` fast path;
- dynamic List cardinality is not capped by `u16`;
- user iteration errors propagate unchanged;
- source spans point runtime tracebacks at the spread contribution;
- nested dynamic operand-window and GC stress tests pass;
- no new public primitive bindings are introduced.

---

## 24. Non-goals

Do not use this work to add:

- `**` or `***` to Lists;
- new List public mutation APIs;
- a universal collection-builder abstraction;
- new iterable protocols;
- runtime iteration limits;
- type-system work;
- compile-time arbitrary cardinality inference;
- Record/Map labeled expansion beyond the shared seams needed by Part II.

Part II owns Record/Map `**`, labeled-lane extraction, Record builder state, duplicate diagnostics, and full completion testing.
