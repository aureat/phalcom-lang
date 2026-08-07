# Spec B.3 — Brace-Literal AST and Atomic Map Construction

Status: implementation specification with a scoped grammar blocker. Requires B.1 and Spec A.3. B.3a (association Map literals) is dispatchable. B.3b (`{}` as empty Map and bare-brace Set literal completion) is blocked until Phalcom ratifies how those forms coexist with its existing brace block literals.

## 1. Mission

Replace the legacy parser desugaring of Map literals into mutation-send chains with an explicit literal representation and a duplicate-safe construction path that preserves lexical evaluation order and arbitrary computed keys.

Required Map forms once implemented:

```phalcom
{
    timeout: 10,
    retries: 3,
}

{
    ["timeout"]: 10,
    [42]: "answer",
    [user.id]: metadata,
}
```

Bare keys are Symbols. Computed keys are arbitrary evaluated Map keys and are never coerced to Symbol.

Map literal duplicate logical keys fail; they do not overwrite.

This phase must also leave the AST/parser ready for Spec F's later `**` association expansion without implementing expansion execution here.

## 2. Current parser behavior to retire

HEAD's `parse_primary` currently classifies a brace as Map only when the first two tokens are `Identifier` + `:`. `parse_map_literal` then:

- accepts only identifier keys;
- synthesizes `Symbol.new("name")`;
- parses values;
- lowers the whole literal to `Map.new().at(k, put: v)...`.

That architecture is incompatible with the new contract because:

1. `{[expr]: value}` cannot parse as Map;
2. duplicate keys overwrite silently through ordinary mutation;
3. no AST survives to distinguish literal insertion from post-construction insertion;
4. future `**mapping` literal expansion has no structural insertion point;
5. literal failure timing cannot be enforced at a single finalization boundary.

Remove `map_construction_chain` as the Map-literal execution architecture after B.3a is complete.

## 3. Explicit Map literal AST

Add a dedicated expression node in `phalcom-ast/src/ast.rs`, parallel in spirit to Spec A's explicit product AST rather than parser-desugared List syntax.

Recommended conceptual shape:

```rust
Expr::MapLiteral(Box<MapLiteralExpr>)

struct MapLiteralExpr {
    entries: Vec<MapLiteralEntry>,
    range: SourceRange,
}

enum MapLiteralEntry {
    Association {
        key: MapLiteralKey,
        value: Expr,
        range: SourceRange,
    },
    // Reserved now for Spec F; parser may recognize and reject as pending
    // until expansion implementation lands.
    Expansion {
        expr: Expr,
        range: SourceRange,
    },
}

enum MapLiteralKey {
    BareSymbol {
        name: String,
        range: SourceRange,
    },
    Computed {
        expr: Expr,
        range: SourceRange,
    },
}
```

Exact Rust names may follow A.1 conventions. Required properties are:

- bare Symbol-key spelling remains distinguishable from computed expression syntax;
- every source entry carries its own range;
- source entry order is retained;
- future expansion has an AST location without redoing the brace parser.

Do not encode bare Map keys as `Symbol.new(String)` calls. The compiler should intern the Symbol directly through the same Symbol canonicalization helper established by Spec A.

## 4. Map association grammar — B.3a

### 4.1 Bare Symbol keys

Parse:

```phalcom
{ name: expr }
```

as `MapLiteralKey::BareSymbol("name")`.

This means key `#name`, not String `"name"`.

### 4.2 Computed keys

Parse:

```phalcom
{ [expr]: value }
```

as a computed key.

The inner `expr` is an ordinary expression; the brackets are key-computation delimiters in this brace-entry position, not a one-element List key. Parenthesized or nested expressions inside them retain normal expression semantics.

Do not coerce the result. In particular:

```phalcom
{ timeout: 1 }
```

and:

```phalcom
{ ["timeout"]: 1 }
```

must contain distinct Symbol and String keys.

### 4.3 Entry separators

Allow comma-separated entries and a trailing comma consistent with the rest of collection literal grammar.

Within an association-mode brace literal, every ordinary entry must be an association. A bare element encountered after association mode begins is a syntax error, not a runtime Set/Map choice.

### 4.4 `**` reservation

The authoritative language permits:

```phalcom
{ **mapping, key: value }
```

but expansion mechanics belong to Spec F.

B.3a should, if tokens already make `**` recognizable, parse it into the reserved `Expansion` entry and emit a precise compile/pending diagnostic until F lands. If HEAD does not yet tokenize `**` separately, do not add an incomplete operator solely to B.3; document the reserved grammar and let F add the longest-token lexer support.

Do not lower `**` as two multiplication tokens or silently accept it as a normal expression.

## 5. Incremental duplicate-safe construction bytecode

### 5.1 Do not use a terminal `BuildMap(N)`

A single terminal build instruction that receives every already-evaluated key/value pair is attractive but semantically wrong for dynamic duplicate/key failures: a duplicate in entry 2 would be discovered only after entry 3, 4, ... expressions had already executed.

The literal must validate each contributed association before evaluation advances to the next source entry.

Use an incremental VM builder protocol. Recommended bytecodes:

```rust
BeginMapLiteral
MapLiteralInsertUnique
FinishMapLiteral
```

Exact enum names may follow repository conventions. These are internal bytecodes, not surface primitives.

Update every bytecode exhaustiveness surface required by HEAD:

```text
Bytecode enum
VARIANTS/index/name table
VM execution match
CLI/disassembler
instruction/source-range bookkeeping
compiled-artifact tests
```

### 5.2 Concrete stack contract

`BeginMapLiteral` allocates a fresh ordinary native `Object::Map` and pushes its `Value::Obj` handle on the VM value stack. The handle is **builder-private by stack discipline**, not a special new object type: no user binding or expression result can observe it until the literal finishes.

Conceptual stack after begin:

```text
... | hiddenMap
```

For each source association, compile:

```text
evaluate key
  ... | hiddenMap | key

evaluate value
  ... | hiddenMap | key | value

MapLiteralInsertUnique
  ... | hiddenMap
```

`MapLiteralInsertUnique` pops `value` and `key`, locates the key through B.1's shared Phalcom `hash` + `==` helper, rejects inadmissible/duplicate keys, inserts only if unique, and leaves the hidden Map handle in place.

After the final entry:

```text
FinishMapLiteral
  ... | mapValue
```

`FinishMapLiteral` may be a semantic/no-op marker if no runtime work is needed, but retaining it is useful for bytecode clarity, future expansion finalization, and assertions that only completed literal builders become expression results. If HEAD's VM conventions strongly favor eliminating a literal no-op instruction, the compiler may instead treat successful completion after the final unique insert as finalization; document that choice explicitly.

### 5.3 Failure and unwind behavior

If key evaluation, value evaluation, hashing, equality, admission, or duplicate detection fails, normal VM unwind discards the hidden Map handle with the surrounding expression stack. The partially constructed object is unreachable GC garbage.

No Phalcom code receives the builder Map itself, so mutation of the partial result cannot escape through ordinary user references. User key `hash`/`==` callbacks may execute language code, but they have no reference to the hidden literal Map unless the program obtains one through some unrelated global state.

Do not add a dedicated `Object::MapBuilder` heap arm or a Rust-only side stack: the ordinary Map handle plus bytecode stack discipline is enough.

### 5.4 Reuse the Map insertion core

Do not call public `map.insert` repeatedly from bytecode; that adds selector dispatch and would couple literal duplicate behavior to ordinary overwrite semantics. Refactor/reuse B.1's internal lookup/storage helper so both ordinary insertion and literal unique insertion share:

- key admission;
- Phalcom hash/equality;
- reentrant borrow discipline;
- ordered append storage.

Literal insertion differs only in conflict policy: existing key => duplicate error instead of overwrite.

### 5.5 Expansion headroom

This protocol is intentionally the seam Spec F needs. `**mapping` can later evaluate its source and contribute associations one at a time through the same unique-insert operation while the hidden Map remains below them on the stack. B.3 does not implement that loop.

## 6. Duplicate rejection

### 6.1 Dynamic rule

During `MapLiteralInsertUnique`, before inserting the association, use Phalcom key `hash` + `==` to detect whether an equivalent key is already present in the hidden literal Map.

On duplicate, fail the literal immediately. Do not overwrite and do not evaluate later entries.

The exact error payload is deferred by the language specification. Introduce/reuse a catchable `DuplicateKeyError` surface class or a structured compiler/runtime diagnostic consistent with HEAD. Avoid encoding a future generic `<K>` error type in this non-typing unit.

Failure occurs before the Map value is finalized/exposed.

### 6.2 Static rule

The compiler SHOULD reject duplicates when provable. Implement a useful conservative subset rather than flow analysis.

At minimum detect repeated canonical bare Symbol keys:

```phalcom
{ a: 1, a: 2 }
```

and equivalent static Symbol spellings if B.3's accepted grammar can express them through computed Symbol literals:

```phalcom
{ a: 1, [#a]: 2 }
```

when the compiler can canonicalize the computed key without executing user code.

It is optional in this phase to statically fold arbitrary Number/String computed-key equality. Dynamic construction remains the correctness backstop.

Never reject two general computed expressions merely because their source text is identical; user code may return different keys per evaluation.

## 7. Evaluation and failure order

For:

```phalcom
{
    [key1()]: value1(),
    [key2()]: value2(),
    [key3()]: value3(),
}
```

observable processing is:

```text
BeginMapLiteral
key1
value1
unique/admission check for entry 1
key2
value2
unique/admission check for entry 2
key3
value3
unique/admission check for entry 3
FinishMapLiteral
```

Thus a duplicate or invalid key in entry 2 prevents entry 3's key and value from running. This is a required semantic test, not merely an optimization.

Bare Symbol keys have no source-level evaluation side effect, so their constant push precedes the corresponding value exactly as the conceptual key/value order requires.

## 8. Key validity and failure timing

Each literal association uses the ordinary Map key-admission rules from B.1.

If a key is invalid/unhashable, fail after that key and its value have been evaluated but before evaluating the next entry, matching the per-entry builder sequence above.

If a key's user `hash` / `==` callback raises, propagate that failure normally. Do not wrap callback failures as DuplicateKeyError.

The hidden builder must honor the same reentrant borrow discipline and structural-mutation lock as ordinary Map insertion.

## 9. Empty Map and Set literal grammar — B.3b BLOCKER

### 9.1 Normative collection requirement

The collections specification ratifies:

```text
{}        -> empty Map
{a}       -> Set
{a,b}     -> Set
{a: b}    -> Map
Set()     -> empty mutable Set
```

and says classification is syntactic rather than type-contextual.

### 9.2 Existing Phalcom conflict

HEAD also ratifies/uses braced blocks:

```phalcom
{}
{ expr }
{x => expr}
{x, y => ...}
```

The first two collide exactly with empty Map and singleton Set. This cannot be solved by more lookahead: the token sequences are identical.

### 9.3 Mandatory gate

Before B.3b implementation, obtain a ratified language decision that changes one side of the grammar or otherwise defines an unambiguous syntactic distinction.

Valid decision-space examples for language design may include a changed zero-parameter block spelling, a changed Set spelling, or another explicit delimiter rule, but **this implementation spec chooses none of them**.

The implementer MUST NOT:

- use expected type to choose Block vs collection;
- make `{x}` Block in one position and Set in another without a ratified grammar rule;
- silently change `{}` from Block to Map and mass-rewrite the repository without language approval;
- invent `{x,}` singleton Set as a special rule;
- infer Block because an expression "looks callable";
- inspect closure arity/types at runtime.

B.3a's unambiguous association Maps can land before this gate because `{label: value}` and `{[expr]: value}` do not overlap existing valid block grammar in the same way.

### 9.4 Set scope after the gate

Even after the grammar decision, Spec B implements only the literal-family boundary necessary for collection classification. Full Set/ImmutableSet API/equality/order/hash remains deferred.

If the ratified syntax keeps `{elements}` as Set, construction may use the existing Set runtime/add path initially, but do not invent semantics beyond the eventual Set specification. Expansion `*values` remains Spec F.

## 10. Mixed brace entry rejection

Once a brace is syntactically classified as a collection and entry mode is known:

```phalcom
{ a, b: c }
{ *values, key: value }
```

must be syntax errors rather than runtime classification.

B.3a can already reject association-to-element mixing for forms whose first association makes Map classification unambiguous. Full symmetric Set-to-association rejection belongs to B.3b after the block collision is resolved.

## 11. Compiler integration

Add compiler handling for explicit `Expr::MapLiteral`. Do not route it through generic MethodCall selector construction.

Responsibilities:

- canonicalize bare Symbol keys through A's shared Symbol compiler helper;
- perform conservative static duplicate detection;
- emit per-entry builder operations preserving lexical timing;
- leave exactly one Map value on the stack on success;
- emit no Map value on failure;
- keep source ranges for diagnostics attached to the offending key/entry.

Update every exhaustive AST consumer outside `phalcom-core` that must compile after adding the node, including LSP walkers/indexing/semantic-token code if they match `Expr` exhaustively. Follow the repository's normal AST-extension audit rather than adding wildcard arms that hide future omissions.

## 12. Tests — B.3a

### 12.1 Bare Symbol key

```phalcom
const m = { timeout: 10 }
```

asserts `m.get(#timeout)` present and `m.get("timeout")` absent.

### 12.2 Computed keys

Test String, Number, Tuple, Record and runtime-returned keys:

```phalcom
{
  ["timeout"]: 1,
  [42]: 2,
  [(1, 2)]: 3,
  [#{ x: 1 }]: 4,
}
```

subject to A.3 product key support.

### 12.3 Lexical evaluation order

Use side-effecting helpers that append markers and prove:

```text
key1, value1, key2, value2, ...
```

### 12.4 Duplicate timing

Use:

```phalcom
{
  [keyA()]: valueA(),
  [sameAsA()]: valueB(),
  [mustNotRun()]: valueC(),
}
```

and assert `mustNotRun` is never evaluated once the second association is found duplicate. This test forces the per-entry builder architecture from §7.

Also test static `{a: 1, a: 2}` compile rejection.

### 12.5 Post-construction overwrite remains legal

A literal duplicate fails, but:

```phalcom
const m = { a: 1 }
m[#a] = 2
```

is ordinary update and succeeds without moving encounter position.

### 12.6 Equal-but-nonidentical duplicate

Construct two distinct keys that compare/hash equal and ensure the literal rejects them as duplicate logical keys. Do not assert which object would have been retained because construction fails and the general retention policy is deferred.

### 12.7 Error propagation

Key `hash` / `==` callback failure propagates as that failure, not as duplicate. Invalid mutable collection key fails before later entries execute.

## 13. Tests — B.3b after grammar ratification

Only after the gate is resolved, add the final literal taxonomy tests required by the chosen syntax. If the ratified collections spelling remains unchanged, conformance must include:

```text
{}            Map
{a: 1}        Map
{[k]: 1}      Map
{a, b}        Set
Set()         distinct empty mutable Set
#{...}        Record/Unit from Spec A
[]            List
(...)         Tuple/Unit from Spec A
```

Also pin all surviving block-literal forms so the migration does not silently reclassify closures.

## 14. Repository migration audit

Search:

```text
parse_map_literal
map_construction_chain
Token::LBrace parse_primary
block literals `{}` / `{ expr }`
pending map literal tests
Symbol.new synthesized by Map parser
collection literal parser snapshots
```

Remove obsolete comments claiming `{}` is necessarily an empty block or that Map has no empty literal only after the grammar gate has actually changed that rule.

Do not edit historical ADRs to pretend the collision never existed; supersede/status them according to repository process where necessary.

## 15. Files expected to change

B.3a likely:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/... execution handler
phalcom-core/bin/phalcom/disasm.rs or equivalent
phalcom-core/src/primitive/map.rs or a shared internal Map helper module
LSP/exhaustive AST consumers
phalcom-ast/tests/...
phalcom-core/tests/lang/collections/...
compiled-artifact/bytecode tests as applicable
repository docs/status files
```

B.3b additionally changes the brace/block grammar and therefore may have a much larger migration write set. Recompute it only after the language decision; do not pre-author a mass rewrite against an unknown syntax.

Expected primitive-floor delta for B.3a: **0**.

## 16. Explicit non-goals

Do not implement:

- `**Map` execution or other association expansion — F;
- `*Set` expansion — F;
- full Set semantics;
- contextual literal typing;
- generic Map inference;
- `Map.from(record:)` — B.2;
- generic `toMap` / merging conversions — D;
- final block-vs-collection syntax without ratification;
- mutation-during-iteration policy;
- equal-key object-retention policy;
- printing/serialization final format.

## 17. Completion gates

For B.3a:

```sh
./scripts/verify.sh --full
```

The implementation report must include:

- explicit AST shape;
- concrete builder bytecode/state design;
- proof of per-entry duplicate/failure timing;
- bare vs computed key lowering;
- static duplicate subset;
- dynamic equal-key duplicate tests;
- list of exhaustive AST/bytecode consumers updated;
- primitive-floor delta, expected `0`;
- final verification tail.

B.3 as a whole must **not** be marked complete while B.3b's block-vs-collection grammar gate remains unresolved. The report should say `B.3a LANDED / B.3b BLOCKED` rather than claiming the complete literal taxonomy.
