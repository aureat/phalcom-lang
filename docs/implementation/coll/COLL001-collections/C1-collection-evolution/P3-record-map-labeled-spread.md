# Phalcom Collection Literal Spread Completion — Part II
## Shared Labeled Expansion, Record `**`, Map `**`, Diagnostics, Traceback, and Completion Gates

**Status:** implementation specification / execution plan
**Repository baseline:** `aureat/phalcom-lang` at `9ca245f86a62e0b3064d88027f7992c27330f36d`
**Depends on:** Part I frontend/shared positional work where applicable, A.2/A.3 product finalizers, B.3 atomic Map construction, F.2 outgoing pack assembly, GC tracing infrastructure
**Companion:** Part I — frontend/shared positional expansion/List `*`

---

## 1. Mission

Finish collection-literal spread support by implementing the labeled expansion lane for:

```phalcom
#{ field: value, **source }
{ key: value, **source }
```

while preserving the distinct construction semantics of immutable Records and mutable Maps.

This part also closes diagnostic debt discovered by the audit:

- runtime product duplicates must not surface as `RuntimeError::Internal`;
- invalid literal-lane operators should receive informational syntax errors;
- traceback spans must identify the contribution that triggered a failure;
- labeled-lane extraction must be centralized so outgoing packs, Records, and Maps cannot drift semantically.

Tuple `**` / `***` support is already live and serves as a regression target.

---

## 2. Ratified language decisions

### 2.1 Literal/operator matrix

| Literal | `*` | `**` | `***` |
|---|---:|---:|---:|
| Tuple | yes | yes | yes |
| List | yes | no | no |
| Record | no | yes | no |
| Map | no | yes | no |

### 2.2 `**` source capability matrix

`**source` accepts exactly:

1. `Unit` — contributes no labels;
2. `Tuple` — contributes the Tuple labeled lane only;
3. `Record` — contributes all fields in stored encounter order;
4. `Map` — contributes all entries in insertion order, but every key must be a `Symbol`.

No String-to-Symbol coercion is permitted.

### 2.3 `***`

`***` remains Tuple/Unit-only and is meaningful only where both positional and labeled lanes exist.

This part does not extend `***` to Record, Map, or List.

### 2.4 Duplicate behavior

Duplicates are errors.

However, preserve destination-specific timing:

- **Map:** duplicate detection is incremental and immediate; later literal expressions must not execute after a duplicate/key-admission failure.
- **Record:** preserve A.3's existing dynamic duplicate timing: dynamic duplicates may be detected at finalization after all literal contributions have evaluated.
- **Tuple/outgoing pack:** preserve existing behavior.

Do not silently normalize all destinations to one failure timing.

---

## 3. Current repository state

### 3.1 Map AST/parser already reserve expansion

`MapLiteralEntry` already has:

```rust
Expansion { expr: Expr, range: SourceRange }
```

and `parse_map_literal` already recognizes `**expr`.

The compiler currently rejects that AST arm with a pending-Spec-F message.

Map needs runtime/compiler wiring, not a new grammar concept.

### 3.2 Record AST cannot represent expansion

Current Record shape is field-only:

```rust
RecordLiteralExpr {
    fields: Vec<RecordLiteralField>,
    range: SourceRange,
}
```

Every field contains:

```rust
label: ProductLabel
value: Expr
range: SourceRange
```

The parser requires a label at every Record entry.

Record therefore needs an AST structural change.

### 3.3 Map already has the correct destination builder

Current Map literals use:

```text
BeginMapLiteral

key
value
MapLiteralInsertUnique

...

FinishMapLiteral
```

`MapLiteralInsertUnique` uses the ordinary Map hash/equality machinery and raises a catchable `DuplicateKeyError` on collision.

Keep this design.

### 3.4 Product finalizers already enforce duplicate invariants

`finish_tuple` and `finish_record` both call a shared uniqueness check.

`finish_record` normalizes empty output to `Unit`.

Keep `finish_record` as the invariant boundary for dynamic Record construction.

### 3.5 Existing product error conversion is wrong

`BuildTuple` and `BuildRecord` currently map any `ProductBuildError` to `RuntimeError::Internal`.

A dynamically duplicated computed product label is a normal user-program error, not an implementation bug.

This must be fixed as part of the spread completion.

---

## 4. Design principle: share labeled source extraction, not destination conflict policy

The central runtime seam should answer:

```text
What ordered (Symbol, Value) associations does **source contribute?
```

It should **not** decide what to do with duplicates in the destination.

Conceptually:

```rust
fn snapshot_labeled_lane(
    vm: &mut VM,
    source: Value,
) -> Result<Vec<(Symbol, Value)>, RuntimeError>
```

Exact signature may use `PhResult`.

The helper performs the common source classification:

```text
Unit   -> []
Tuple  -> labeled lane
Record -> fields
Map    -> ordered entries, validating Symbol keys
other  -> InvalidStarStarOperand
```

Then destinations consume the returned associations according to their own semantics.

### 4.1 Why snapshot

The helper should return owned/copyable entry data rather than borrowed references.

This is especially important for Map sources:

- destination insertion may invoke user `hash` / `==`;
- those sends can re-enter the VM;
- user code may mutate unrelated state, including potentially the source Map;
- no `&Heap` or `&MapObject` borrow may survive such sends.

Snapshotting also defines deterministic “expand the source as observed after source evaluation” semantics.

`Value` is Copy; this is a shallow association snapshot, not recursive cloning.

---

## 5. Refactor existing outgoing-pack `**`

The current `PackExpandLabels` logic already implements the correct operand matrix.

Extract its source-classification work into the shared helper.

Then rewrite `PackExpandLabels` as:

```text
source
builder
  -> snapshot_labeled_lane(source)
  -> append each association to ArgumentPackBuilder
```

Preserve the pack builder's duplicate behavior and ordering.

Do not change public F.2 semantics.

This refactor is important because Record and Map must not grow parallel, independently maintained switches over Unit/Tuple/Record/Map.

---

## 6. Record AST change

Preserve the useful existing `RecordLiteralField` type.

Introduce an entry enum:

```rust
#[derive(Debug, Clone)]
pub enum RecordLiteralEntry {
    Field(RecordLiteralField),

    Expansion {
        expr: Expr,
        range: SourceRange,
    },
}
```

Change:

```rust
pub struct RecordLiteralExpr {
    pub fields: Vec<RecordLiteralField>,
    pub range: SourceRange,
}
```

to:

```rust
pub struct RecordLiteralExpr {
    pub entries: Vec<RecordLiteralEntry>,
    pub range: SourceRange,
}
```

Exact naming may follow repository style.

### 6.1 Why retain `RecordLiteralField`

Keeping the field struct:

- preserves ProductLabel code;
- minimizes churn in existing compiler helpers;
- preserves field-specific source ranges;
- keeps static-field lowering easy to share with the current implementation.

Do not flatten Record fields into a generic PackItem; Record syntax and semantics are products, not calls.

---

## 7. Record parser changes

`parse_record_literal` should parse one of two entry forms.

### 7.1 Field

Existing behavior remains:

```phalcom
#{ name: expr }
#{ [computedLabel]: expr }
```

Produce `RecordLiteralEntry::Field`.

### 7.2 Expansion

Recognize:

```phalcom
#{ **source }
```

Produce:

```rust
RecordLiteralEntry::Expansion {
    expr,
    range,
}
```

The range spans from `**` through the source expression.

### 7.3 Invalid operators

Reject at parse time:

```phalcom
#{ *source }
#{ ***source }
```

Messages should identify the lane mismatch.

Recommended shape:

```text
`*` is not valid in a Record literal; Record expansion uses the labeled lane with `**`
```

and:

```text
`***` is not valid in a Record literal; Record has no positional lane, use `**` for labeled expansion
```

Anchor errors on the operator token.

### 7.4 Separators

Preserve:

- comma separation;
- trailing comma;
- multiline form;
- empty `#{}` normalization behavior.

---

## 8. Map parser diagnostics

Map already recognizes `**`.

Make sure invalid operators are diagnosed intentionally:

```phalcom
{ *source }
{ ***source }
```

where the grammar unambiguously selects Map-literal context.

Messages should explain that Map expansion is labeled association expansion and therefore uses `**`.

Do not allow these forms to degrade into generic “expected key” diagnostics if the operator token clearly expresses the user's intent.

---

## 9. Record compiler shape selection

### 9.1 Static Record

If every entry is a field and there is no expansion:

- preserve current `BuildRecord { fields }` path;
- preserve static duplicate detection;
- preserve computed-label `GuardSymbol` timing;
- preserve `u16` static bytecode count check.

No builder allocation on ordinary Records.

### 9.2 Dynamic Record

If any `RecordLiteralEntry::Expansion` exists:

- allocate a dedicated private Record literal builder;
- append explicit fields and expanded associations in encounter order;
- finalize through `finish_record`.

Do not use `ArgumentPackBuilderObject`.

---

## 10. Record literal builder

Add a compiler/VM-private heap payload, conceptually:

```rust
pub struct RecordLiteralBuilderObject {
    labels: Vec<Symbol>,
    values: Vec<Value>,
}
```

or:

```rust
entries: Vec<(Symbol, Value)>
```

Choose layout based on arena slot size and tracing/access convenience.

### 10.1 Required properties

The builder:

- is not surface-visible;
- has no class;
- has no primitive bindings;
- stores only pending Record construction state;
- preserves encounter order;
- does not perform duplicate detection during append;
- is finalized exactly once in normal compiler-generated use.

### 10.2 No HashSet in the builder

Do not add a duplicate set.

`finish_record` already provides an O(n)-expected HashSet uniqueness pass.

Adding an incremental set would:

- duplicate invariant logic;
- change dynamic duplicate timing;
- consume extra memory;
- make Record construction semantics accidentally match outgoing packs rather than A.3.

### 10.3 Why not ArgumentPackBuilder

`ArgumentPackBuilderObject` has:

- positional storage;
- labeled storage;
- pending-label reservation state;
- early duplicate detection.

Record needs none of those call-specific semantics.

Use a smaller dedicated object.

---

## 11. Record builder bytecodes

Recommended bytecodes:

```rust
NewRecordLiteralBuilder
RecordLiteralAppend
FinishRecordLiteral
```

Exact names may follow existing bytecode style.

### 11.1 `NewRecordLiteralBuilder`

```text
...
NewRecordLiteralBuilder
... | builder
```

Allocates the private heap object.

### 11.2 `RecordLiteralAppend`

Recommended stack contract:

```text
... | builder | Symbol | value
RecordLiteralAppend
... | builder
```

The opcode:

- verifies the builder type as an internal invariant;
- verifies the label is `Symbol` defensively;
- appends label/value in encounter order;
- does not check duplicate labels.

The compiler should already have emitted `GuardSymbol` for computed labels before evaluating the associated value, preserving A.3 failure timing for non-Symbol labels.

### 11.3 `FinishRecordLiteral`

```text
... | builder
FinishRecordLiteral
... | Record-or-Unit
```

The handler:

1. obtains/takes the accumulated fields;
2. calls `finish_record`;
3. maps any product build error to a normal user-facing runtime product error;
4. pushes `Unit` if no fields resulted;
5. otherwise pushes the immutable Record.

The builder becomes unreachable after successful replacement.

---

## 12. Rooting and GC for Record builders

The dynamic Record builder must be stored in a compiler scratch local across field expressions and expansion processing.

Do not rely on:

- a Rust local outside tracing;
- a transient stack position that nested sends can disturb.

### 12.1 Object integration

Adding the builder requires updates to:

```text
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs   # or equivalent typed accessor module
phalcom-core/src/heap/trace.rs
```

plus any debug-kind/test surfaces.

### 12.2 GC tracing

The builder's stored `Value`s are outgoing edges.

`trace_object` must visit every value.

`Symbol`s are not heap edges.

The exhaustive `Object` match should remain exhaustive with no wildcard.

Also update the normative memory-management edge documentation if repository policy requires every new handle-bearing object payload to be recorded there.

---

## 13. Dynamic Record lowering

For a spread-containing Record:

```text
reserve builder scratch
NewRecordLiteralBuilder
SetLocal(builder)
Pop
```

Then process entries in source order.

### 13.1 Explicit static label field

```text
push Symbol constant
compile value
GetLocal(builder)
RecordLiteralAppend
```

Adjust stack order to match the final opcode contract.

No runtime Symbol guard needed for a compiler-known Symbol.

### 13.2 Explicit computed label field

Required observable order:

```text
compile label expression
GuardSymbol
compile value expression
append
```

If label evaluation produces a non-Symbol, the value expression must not run.

Preserve existing A.3 semantics.

### 13.3 `**source`

```text
compile source exactly once
invoke shared labeled-lane snapshot/extraction
append every association to builder in snapshot order
```

Because Record duplicate checking is deferred, every expanded association is appended even if it duplicates an earlier label.

Later literal fields/expansions continue to evaluate unless another error occurs.

### 13.4 Finalization

After all entries:

```text
GetLocal(builder)
FinishRecordLiteral
release scratch
```

---

## 14. Record dynamic duplicate timing

This behavior is deliberate and must be tested.

Example:

```phalcom
#{
    **(a: 1,),
    a: second(),
    b: later(),
}
```

If the duplicate `a` is not statically provable before execution, A.3 permits duplicate detection at finalization.

Therefore:

```text
second() runs
later() may run
FinishRecordLiteral detects duplicate `a`
construction fails
```

Do not introduce early duplicate scanning merely because expansion now exists.

Static duplicates that the compiler can prove should still fail compilation before execution.

---

## 15. Map `**` lowering

Map already has the proper hidden destination object and unique insertion operation.

For:

```phalcom
{
    a: first(),
    **source,
    b: later(),
}
```

lower in exact source order:

```text
BeginMapLiteral

#a
first()
MapLiteralInsertUnique

evaluate source once
snapshot_labeled_lane(source)
for each (label, value):
    push label
    push value
    MapLiteralInsertUnique

#b
later()
MapLiteralInsertUnique

FinishMapLiteral
```

Do not add a Map builder object.

---

## 16. Map immediate failure semantics

`MapLiteralInsertUnique` remains the semantic gate for every association, whether explicit or expanded.

If an expanded association fails because of:

- mutable key rejection;
- user `hash` error;
- user `==` error;
- duplicate key;

then:

- no later association from the snapshot is inserted;
- no later literal entry is evaluated;
- the partially built Map remains hidden/unobservable.

This preserves B.3's incremental failure model.

---

## 17. Map source key validation

When a Map is used as a `**` source, every source key must be a Symbol.

If any key is not a Symbol, use the existing expansion error:

```text
Map key in ** expansion must be Symbol; got {found}
```

Do not:

- stringify keys;
- intern Strings;
- wait for destination-specific code to fail differently.

The shared labeled-lane extractor owns this source validation.

---

## 18. Snapshot ordering

The shared labeled-lane extractor must preserve:

- Tuple labeled-lane order;
- Record encounter order;
- Map insertion order.

Unit contributes nothing.

This order is observable in:

- outgoing dynamic selector construction;
- Record encounter-order storage;
- Map insertion order;
- duplicate timing.

Do not sort labels.

---

## 19. Source evaluation

For every `**source`:

- evaluate `source` exactly once;
- snapshot after evaluation;
- do not re-read the source expression per destination association.

A source expression with side effects must run once.

For a Map source, the snapshot defines the associations copied even if later user callbacks mutate that source during destination insertion.

---

## 20. Product duplicate diagnostic debt

Introduce a normal runtime mapping for:

```rust
ProductBuildError::DuplicateLabel(Symbol)
```

Do not map it to `RuntimeError::Internal`.

### 20.1 Required user-facing distinction

The error should carry enough context to render either:

```text
duplicate Tuple label `#name`
```

or:

```text
duplicate Record field `#name`
```

Exact punctuation and Symbol rendering may follow project style.

Recommended structured variant:

```rust
RuntimeError::DuplicateProductLabel {
    product: ProductKind,
    label: Symbol-or-String,
}
```

or separate variants if that better matches current error conventions.

The important requirement is normal user error classification, not the exact Rust enum name.

### 20.2 Apply consistently

Use the same conversion in:

- `BuildTuple`;
- `BuildRecord`;
- `FinishTuplePack`;
- `FinishRecordLiteral`;
- any other internal product finalization route.

Do not leave one path producing “internal error” for the same semantic failure.

---

## 21. Surface catchability decision

Follow the repository's current error model.

If product duplicate errors are represented as native `RuntimeError` variants today, keep them normal runtime errors unless there is an already-established surface `DuplicateFieldError`/equivalent.

Do not invent a new generic error class hierarchy solely for this unit unless the core language spec already requires one.

Map duplicates remain the existing catchable `DuplicateKeyError`.

This distinction is acceptable because Map already has an established surface duplicate-key class.

---

## 22. Static duplicate diagnostics

Where the compiler can prove duplicate Symbol identity, reject before execution.

### 22.1 Record

Preserve existing static product-label canonicalization and duplicate checks.

The AST migration from `fields` to `entries` must not weaken them.

Static duplicates may span:

- explicit field vs explicit field;
- explicit field vs statically knowable expanded content only if the compiler already has such structural facts.

Do **not** execute or inspect arbitrary spread source code to precompute labels.

For the first implementation, it is acceptable that a duplicate introduced by `**source` is runtime-only even if source happens to be a literal expression, unless a clean literal-only proof is trivial and covered by tests.

Correctness is guaranteed by finalization.

### 22.2 Map

Current static bare-Symbol duplicate detection should be migrated from free-form `CompilerError::Message` to a structured literal duplicate error if practical.

Recommended data:

```text
key/field Symbol
current span
first span
literal kind
```

The compile renderer is currently span-light, but preserve the structured information for future rendering/LSP use.

Do not expand scope into a full compiler diagnostic renderer rewrite.

---

## 23. Invalid expansion-source errors

Reuse existing structured F.2 errors.

### `**` invalid operand

```text
** expansion requires Tuple, Unit, Record, or Map; got {found}
```

### non-Symbol source Map key

```text
Map key in ** expansion must be Symbol; got {found}
```

### `***` invalid operand

Existing Tuple behavior remains:

```text
*** expansion requires Tuple or Unit; got {found}
```

Do not produce destination-specific copies of these messages.

---

## 24. Traceback and source-span policy

Runtime traceback line attribution uses the executing bytecode's source span.

New bytecodes/compiler-generated operations must therefore be stamped intentionally.

### 24.1 Record

| Work | Span |
|---|---|
| `NewRecordLiteralBuilder` | full Record literal |
| explicit static field append | field entry |
| computed `GuardSymbol` | computed label |
| explicit computed field append | field entry |
| `**` source evaluation | source expression |
| labeled extraction opcode/helper call | expansion entry |
| each Record append from expansion | expansion entry |
| `FinishRecordLiteral` | full Record literal |

A dynamic duplicate detected at finalization should therefore blame the Record literal as the caller location while the error message identifies the duplicate Symbol.

Do not store source ranges inside the heap builder solely to produce a second-label caret; that would complicate runtime state to alter semantics A.3 intentionally leaves at finalization.

### 24.2 Map

| Work | Span |
|---|---|
| `BeginMapLiteral` | full Map literal |
| explicit insertion | association entry |
| `**` source evaluation | source expression |
| extraction | expansion entry |
| insertion of expanded associations | expansion entry |
| `FinishMapLiteral` | full Map literal |

If a key's `hash` or `==` throws, preserve the thrown error and user-method frame. The caller frame should resolve to the `**source` entry.

---

## 25. Do not wrap user callback errors

Never rewrite exceptions/errors raised by:

- expansion source evaluation;
- Map source access if future implementation invokes language code;
- destination Map key `hash`;
- destination Map key `==`;
- any user code reached during these operations.

Do not replace them with generic:

```text
Map expansion failed
Record expansion failed
```

Only source capability errors and destination invariant errors should be synthesized by this feature.

---

## 26. Failure atomicity

### 26.1 Record

The private builder is compiler-owned until finalization.

If any field or expansion raises:

- no Record value is produced;
- builder remains unreachable after unwind;
- GC reclaims it later.

### 26.2 Map

Existing builder-private stack discipline remains.

If insertion fails:

- the partial native Map is never exposed;
- normal unwind discards the construction expression.

No rollback or copy-on-write destination is required.

---

## 27. Runtime cardinality and static bytecode limits

### 27.1 Static Record

`BuildRecord { fields: u16 }` may retain its static encoding limit.

### 27.2 Dynamic Record

A spread-containing Record must not inherit `u16` as a semantic field-count maximum.

The private builder uses runtime vectors.

A `**` source may contribute enough fields to exceed 65,535 if memory permits.

`finish_record` must accept runtime vector cardinality independently of bytecode operand width.

### 27.3 Map

Map dynamic construction already has no terminal encoded entry count.

No new count limit is needed.

---

## 28. Empty result semantics

### 28.1 Record

Record finalization must continue canonical zero-product normalization:

```phalcom
#{ **() }
```

evaluates to canonical `Unit`.

Likewise, any dynamic Record construction that receives zero final fields returns Unit.

Do not allocate a heap `RecordObject` with zero fields.

### 28.2 Map

An empty Map literal, including:

```phalcom
{ **() }
```

remains a fresh mutable empty Map, not Unit.

This distinction is fundamental:

```text
empty product -> Unit
empty mutable Map -> Map
```

---

## 29. Tooling migration for Record AST

Changing `RecordLiteralExpr.fields` to `entries` requires a repository-wide exhaustive audit.

Confirmed consumer:

```text
phalcom-lsp/src/index.rs
```

currently walks `record.fields`.

Update all visitors so:

- `Field` walks computed label expression if present and then value;
- `Expansion` walks its source expression.

Search for:

```text
RecordLiteralExpr
RecordLiteralField
.fields
Expr::RecordLiteral
```

across:

- parser tests;
- compiler;
- boundedness;
- LSP/indexing;
- formatting/debug helpers;
- source-range walkers;
- pattern/analysis utilities;
- fixture builders.

Do not rely on compiler errors alone to find semantic visitors that use wildcard matches.

---

## 30. Bytecode and heap bookkeeping

For every new bytecode:

- enum;
- name table;
- variant count;
- exhaustive index;
- VM dispatch;
- disassembler/debug output;
- opcode histogram assumptions;
- tests.

For every new heap `Object` variant:

- enum;
- module exports;
- typed accessors;
- GC tracer;
- debug/test object-kind helpers;
- memory-management edge docs where required.

Keep exhaustive matches exhaustive.

---

## 31. Efficiency policy

### 31.1 Outgoing packs

After refactor, no semantic/performance regression.

The shared source snapshot may replace existing local extraction logic, but avoid unnecessary allocations for Unit/Tuple/Record if the current implementation can cheaply construct the association vector.

Correctness first; benchmark only if a measurable regression appears.

### 31.2 Record

Static Records remain one terminal `BuildRecord`.

Dynamic Records allocate exactly one private builder plus its growing vector storage.

Do not allocate an `ArgumentPackBuilderObject` and then copy into a Record.

### 31.3 Map

Keep using the destination Map directly.

A `**Map` source requires an association snapshot before destination insertion. That temporary vector is intentional for re-entrant borrow soundness.

### 31.4 No new public primitives

Expected primitive-floor delta:

```text
0
```

All new machinery is compiler/bytecode/heap-internal.

---

## 32. Required tests — shared labeled extraction

Build tests that exercise the same source matrix through at least:

- outgoing pack `**`;
- Record `**`;
- Map `**`.

For every destination, test sources:

```text
Unit
Tuple with positional + labeled lanes
Record
Map with Symbol keys
invalid scalar/object
Map with non-Symbol key
```

This is the strongest guard that the helper truly centralizes semantics.

---

## 33. Required tests — Record

### Parser/AST

- empty `#{}`
- explicit fields
- computed label
- `#{ **source }`
- mixed explicit/spread entries
- trailing commas/newlines
- `#{ *source }` precise error
- `#{ ***source }` precise error

### Value/order

- `**Tuple` uses labeled lane only;
- `**Record` preserves encounter order;
- `**Map` preserves insertion order;
- `**Unit` contributes nothing;
- mixed contributions preserve literal encounter order.

### Symbol validation

- source Map non-Symbol key fails;
- computed explicit label still fails before its value expression.

### Duplicate behavior

Test:

- explicit vs explicit static duplicate compile error;
- earlier explicit vs spread duplicate runtime error;
- spread vs later explicit duplicate;
- spread vs spread duplicate;
- dynamic duplicate is not `Internal`;
- later Record contribution timing matches A.3 finalization behavior.

### Empty normalization

```phalcom
#{ **() }
```

must be Unit.

### Large dynamic Record

Construct more fields dynamically than the `u16` static encoding boundary and assert the dynamic path is not capped by bytecode operand width.

If generating that many unique Symbols is too expensive for a regular test, use a lower-level VM/builder test to prove no `u16` truncation and one integration test near a practical boundary.

### GC

Force collection while:

- builder contains heap object values;
- source snapshot contains heap object values;
- nested sends execute.

Verify all survive.

---

## 34. Required tests — Map

### Basic expansion

- `{ **() }`
- `**Tuple`
- `**Record`
- `**Map`
- ordering across explicit and expanded entries.

### Invalid sources

- scalar/unsupported object;
- Map with non-Symbol key.

### Duplicate matrix

Test collision:

- explicit before spread;
- duplicate within one spread source;
- spread vs spread;
- spread before explicit;
- logically equal distinct key objects using Phalcom `hash`/`==`.

### Failure timing

Use stateful probes proving that once a duplicate/key failure occurs:

- later expanded associations are not inserted;
- later literal expressions do not execute.

### Callback propagation

Create keys whose:

- `hash` throws;
- `==` throws.

Verify the original error is preserved.

### Source mutation/re-entrancy robustness

Exercise a destination key callback that mutates or otherwise touches the source Map after the source snapshot was taken.

Verify:

- no borrow panic;
- no invalid iteration state;
- expansion consumes the original snapshot order.

### Concurrent-mutation lock

Keep existing Map re-entrant structural mutation protections intact.

---

## 35. Required tests — diagnostics and traceback

### Product duplicate

A runtime duplicate Record field must render as a normal duplicate-field/product error, never:

```text
internal error (this is a Phalcom bug...)
```

Run the equivalent Tuple dynamic duplicate path as a regression.

### Spread source error

An unsupported `**` source must identify the operator capability problem.

### Map non-Symbol key

Message must identify Map source key requirement.

### Traceback

For errors thrown in:

- destination Map `hash`;
- destination Map `==`;
- nested code called by an expansion source;

verify the traceback contains:

1. the user callback/method frame at the actual throwing line;
2. the caller frame corresponding to the `**source` literal contribution.

Test both human-readable and structured/JSON traceback output if the repository maintains both.

---

## 36. Required tests — bytecode shape and fast paths

Assert:

- ordinary Record with no expansion still uses `BuildRecord`;
- expansion Record uses builder bytecodes and not one terminal count-based build;
- ordinary Map keeps existing begin/insert/finish protocol;
- Map `**` adds extraction/contribution work without replacing the unique insertion operation;
- Tuple existing dynamic path remains `ArgumentPackBuilder`-based;
- outgoing `**` remains semantically unchanged after helper extraction.

---

## 37. Recommended write-set

Expected primary files:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/error.rs

phalcom-core/src/error.rs
phalcom-core/src/product.rs

phalcom-core/src/vm/dispatch.rs

phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/heap/record_literal_builder.rs   # recommended new module

phalcom-core/src/primitive/map.rs                 # reuse; minimal/no semantic change expected

phalcom-lsp/src/index.rs
```

Potential secondary files:

```text
phalcom-core/src/chunk.rs
phalcom-core/src/diagnostics/...
docs/spec/current/memory-management.md
bytecode/disassembly tooling
test fixture builders
```

---

## 38. Suggested implementation sequence

### Phase 1 — source helper extraction

1. Identify current `PackExpandLabels` source classification.
2. Extract destination-neutral labeled-lane snapshot helper.
3. Rewrite outgoing-pack expansion to use it.
4. Run all F.2 completion tests before touching Record/Map.

### Phase 2 — Record AST/parser migration

1. Add `RecordLiteralEntry`.
2. Change `RecordLiteralExpr.fields -> entries`.
3. Update parser.
4. Add invalid operator diagnostics.
5. Update all AST walkers/LSP.
6. Restore full existing static Record test coverage before runtime builder work.

### Phase 3 — Record builder/runtime

1. Add builder heap payload.
2. Add typed accessors.
3. Add GC tracing.
4. Add bytecodes.
5. Add compiler dynamic path.
6. Finalize through `finish_record`.
7. Preserve static `BuildRecord` path.

### Phase 4 — Map `**`

1. Replace pending compiler rejection.
2. Evaluate source once.
3. snapshot labeled lane.
4. feed each association through `MapLiteralInsertUnique`.
5. test immediate failure timing and callback propagation.

### Phase 5 — product diagnostics

1. Add normal duplicate-product runtime mapping.
2. update `BuildTuple`;
3. update `BuildRecord`;
4. update `FinishTuplePack`;
5. update `FinishRecordLiteral`;
6. strengthen static duplicate CompilerError shapes where practical.

### Phase 6 — traceback/hardening

1. verify source spans;
2. GC stress;
3. Map re-entrancy/source-snapshot tests;
4. large dynamic Record;
5. bytecode fast-path assertions;
6. full repository test suite.

---

## 39. Completion gate

The collection-spread gap is fully closed only when all of these are true:

- Tuple `*` / `**` / `***` remain working;
- List `*` works per Part I;
- List `**` / `***` are parse-time lane errors;
- Record `**` parses and executes;
- Record `*` / `***` are parse-time lane errors;
- Map `**` executes through the existing unique-insert protocol;
- Map `*` / `***` are parse-time lane errors;
- `**` has one shared source capability definition;
- Map source keys are required to be Symbols;
- source evaluation occurs exactly once;
- association order is preserved;
- Map duplicates short-circuit immediately;
- Record dynamic duplicates preserve A.3 finalization timing;
- runtime product duplicates are no longer reported as internal compiler/runtime bugs;
- Record dynamic construction remains GC-safe;
- Map expansion is re-entrant-borrow safe through snapshotting;
- static Record fast path remains `BuildRecord`;
- dynamic Record results are not semantically capped by `u16`;
- empty dynamic Record normalizes to Unit;
- user callback errors propagate unchanged;
- traceback caller spans identify the actual spread contribution;
- outgoing-pack behavior remains green after helper extraction;
- no public primitive bindings are added.

---

## 40. Non-goals

Do not use this unit to redesign:

- Record duplicate timing;
- Record equality/hash;
- Map equality/hash;
- collection iteration protocols;
- Map overwrite behavior outside literal construction;
- public Record merge/update APIs;
- public dynamic Tuple/Record builders;
- generic collection-builder abstractions;
- compiler diagnostic rendering architecture;
- the type system.

If Record duplicate short-circuiting is desired later, amend A.3 explicitly in a separate semantic change.

---

## 41. Final architectural target

The completed implementation should have this shape:

```text
                         source
                           │
              ┌────────────┴────────────┐
              │                         │
        positional lane            labeled lane
              │                         │
 Unit / Tuple / Iterable      Unit / Tuple / Record / Map
              │                         │
     shared compiler loop       shared snapshot helper
              │                         │
      ┌───────┴───────┐       ┌────────┼────────┐
      │               │       │        │        │
 ArgumentPack       List     Pack    Record     Map
   builder        literal   builder   builder   literal
      │               │       │        │        │
     Tuple           List   dynamic   finish_   unique
                             sends    record    insert
```

The invariant is:

> **Share source-lane semantics. Keep destination construction and failure policy specific to the destination.**

That closes the feature without coupling Lists/Records/Maps to call semantics and without allowing the definitions of `*` / `**` to drift across language features.
