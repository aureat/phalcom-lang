# Spec D.2 — List Mutation Commands and Literal Construction

Status: implementation specification with two narrowly documented semantic gates. Requires C.1 and C.3. D.1 is recommended first so new List-building code can consistently use `append`, but D.2's literal/compiler work is otherwise separable.

## 1. Mission

Install the ratified List mutation-result model without adding a native primitive per verb.

Target executable surface in this phase:

```text
append(value)              -> Unit
prepend(value)             -> Unit
clear                      -> Unit
insert(value, at:)         -> Result<Unit, IndexError>
remove(at:)                -> Result<T, IndexError>
popFirst                   -> Option<T>
popLast                    -> Option<T>
removeAll(where:)          -> Int
replace(range, with:)      -> Result<Unit, SliceError>   # already C.3
swap(first:, second:)      -> Result<Unit, IndexError>
```

Two ratified-but-underspecified selectors remain gated:

```text
remove(value)              -> Option<T>
move(from:, to:)           -> Result<Unit, IndexError>
```

See §§13–14. Do not guess their missing semantics.

D.2 also decouples `[ ... ]` literal construction from the chainable historical `List#add`, allowing canonical mutations to return semantically meaningful values rather than `self`.

## 2. Repository baseline and leverage from C.3

HEAD's List native substrate before Specs C/D is roughly:

```text
new()
length_
at_
set_
push_
```

C.3 adds the one structural operation the old floor cannot express while preserving List identity:

```text
replaceSlice_(start, endExclusive, replacementList)
```

It snapshots replacements and performs an in-place `Vec::splice`-equivalent operation.

That primitive can implement insertion, deletion, clearing, prepend, pops, and compaction without introducing:

```text
insert_
remove_
pop_
clear_
move_
```

Do not add those primitives.

## 3. Free List literals from public mutation chaining

### 3.1 Current defect

Current parser lowering builds a literal as a public send chain conceptually equivalent to:

```phalcom
List.new().add(a).add(b).add(c)
```

This hard-wires the public `add` return value to `self`.

The ratified collection design says successful no-payload mutations return Unit, not the receiver merely for fluent construction.

### 3.2 Add explicit List literal AST

Migrate parser representation to an explicit expression node, parallel to Spec A's product literals and B's Map literal AST.

Suggested shape:

```rust
pub struct ListLiteralExpr {
    pub elements: Vec<Expr>,
    pub range: SourceRange,
}

Expr::ListLiteral(Box<ListLiteralExpr>)
```

Names may follow actual post-A/B AST conventions.

Parsing rules stay unchanged in this phase:

```text
[]
[a]
[a,b,c]
trailing comma where already accepted
```

Do not add spread elements; `*` expansion remains F.

### 3.3 Evaluation order

Elements evaluate exactly once, left to right.

If element `i` fails, no later element evaluates and no completed List value is produced.

An empty List literal allocates a fresh mutable List. It never normalizes to Unit.

## 4. Add direct List build bytecode

Add a compact build instruction, for example:

```rust
BuildList(u16)
```

Operand is element count.

Compiler lowering:

```text
compile element 0
compile element 1
...
compile element N-1
BuildList(N)
```

VM handler:

1. copy/pop the top N Values preserving lexical order;
2. allocate the native List object directly from that vector;
3. push the resulting List Value.

No public selector dispatch occurs during literal construction.

Update all bytecode bookkeeping required by HEAD:

- enum;
- `BYTECODE_NAMES`;
- `VARIANTS`/`index` exhaustive matches;
- disassembler/debug formatting;
- VM execution match;
- compiler exhaustive Expr handling;
- AST range/exhaustive matches;
- tooling exhaustive matches if ListLiteral is visible to LSP/indexing code.

Do not add a native primitive binding for `BuildList`; bytecode construction does not affect primitive-floor census.

### 4.1 Expansion headroom

F may later replace the fixed element-count representation with incremental building for `*source`. Do not complicate D.2 for that future feature. The explicit List AST is the important seam; F may extend its entry enum later.

## 5. Canonical `append`

Public:

```phalcom
list.append(value)
// Unit
```

Implementation can directly call existing raw `push_(value)` and return canonical Unit.

After A, `push_` itself should already use Unit for successful no-payload operation. Still make the public return explicit and do not depend accidentally on raw return shape.

## 6. Canonical `prepend`

Public:

```phalcom
list.prepend(value)
// Unit
```

Build a temporary one-element replacement List using raw internal construction, then:

```text
replaceSlice_(0, 0, oneElementList)
```

Return Unit.

Do not implement prepend by repeatedly shifting slots in `.ph`; C.3 already admitted the bulk splice for exactly this structural need.

## 7. `clear`

Public getter-style command:

```phalcom
list.clear
// Unit
```

Use:

```text
replaceSlice_(0, size, emptyList)
```

Return Unit.

A clear on an already-empty List is successful Unit.

## 8. Insertion positions

Public:

```phalcom
list.insert(value, at: index)
// Result<Unit, IndexError>
```

### 8.1 Position domain

For current size `n`, valid normalized insertion positions are:

```text
0 <= p <= n
```

Unlike element indexing, `p == n` is valid and appends.

### 8.2 Negative positions

Use the same end-relative principle as C element indexes:

```text
if i >= 0: p = i
if i < 0 : p = n + i
```

Then require `0 <= p <= n`.

Do not clamp invalid insertion positions.

Examples at size 5:

```text
0   -> 0
5   -> 5
-1  -> 4
-5  -> 0
6   -> invalid
-6  -> invalid
```

D inherits C's temporary legacy representation rule: finite integral `Number` accepted until distinct Int/Float runtime values land. Keep the TODO/replacement seam centralized; do not create a second incompatible numeric-validation rule.

### 8.3 Result

On invalid normalized position:

```text
Err(IndexError(...))
```

On success:

1. build a one-element replacement List;
2. `replaceSlice_(p,p,replacement)`;
3. return `Ok(())`.

Malformed non-index representation may follow C's ordinary type/domain failure rather than being coerced into IndexError if that is how C landed. Preserve one consistent boundary.

## 9. Indexed removal

Public:

```phalcom
list.remove(at: index)
// Result<T, IndexError>
```

Use C.1 element-index normalization, not insertion-position normalization.

On valid index:

1. capture the element before mutation;
2. splice `[p,p+1)` with empty List;
3. return `Ok(capturedValue)`.

A captured surface `None` is a real payload and must become `Ok(None)`.

On invalid normalized coordinate:

```text
Err(IndexError(...))
```

Do not raise for ordinary out-of-range failure; this method is the recoverable form.

## 10. Pop operations

### 10.1 `popFirst`

```phalcom
list.popFirst
// Option<T>
```

Empty -> `None`.

Nonempty -> remove index 0 and return `Some(removed)`.

### 10.2 `popLast`

```phalcom
list.popLast
// Option<T>
```

Empty -> `None`.

Nonempty -> remove final element and return `Some(removed)`.

Do not call a strict `[]` on an empty List and catch IndexError; branch on size first.

Stored `None` must produce `Some(None)`.

## 11. `removeAll where:`

Public:

```phalcom
list.removeAll(where: predicate)
// Int
```

The source specification spells this as trailing `where:` syntax; use the actual current labeled-call spelling if trailing labeled closures are not yet landed.

Semantics:

1. traverse source encounter order;
2. invoke predicate exactly once for each encountered value under the current mutation-during-iteration policy (which remains deferred — tests MUST NOT mutate the same List from the callback);
3. values whose predicate result is `true` are removed;
4. values whose result is `false` remain in original relative order;
5. return number removed.

Recommended first implementation:

- create a `retained` List;
- append every value whose predicate is false;
- count removed values;
- after traversal, splice entire receiver `[0,size)` with `retained`;
- return count.

This gives O(n) bulk replacement and stable retained order without adding a retain primitive.

If callback/source failure occurs before final splice, propagate the failure and leave the receiver structurally unchanged by D.2's own algorithm. Do not promise transactional behavior if the callback itself mutates the receiver; that case is deferred.

## 12. `swap(first:second:)`

Public:

```phalcom
list.swap(first: a, second: b)
// Result<Unit, IndexError>
```

Normalize both as C.1 element indexes against the same pre-mutation size.

Validation order is source argument order:

1. validate/normalize `first`;
2. validate/normalize `second`;
3. if either invalid, return Err(IndexError) identifying the failing role when current error substrate allows;
4. if normalized positions equal, return `Ok(())` without writes;
5. otherwise capture both values and use existing `set_` to exchange them;
6. return `Ok(())`.

Negative indices are supported.

Do not allocate a replacement List or use splice for simple equal-length swap unless current code organization makes it materially simpler; `set_` is already the right floor.

## 13. Gate: `remove(value)` occurrence policy

The ratified high-level signature is:

```phalcom
list.remove(value)
// Option<T>
```

The source specification explicitly says first-match sequence behavior is expected but MUST be confirmed in the final List specification if not otherwise stated.

That confirmation has not occurred in the supplied ratified material.

Therefore D.2 MUST NOT silently choose between:

```text
remove first equal value
remove last equal value
remove an unspecified equal value
```

`removeAll where:` already provides the explicit all-matches operation, but that is not enough to turn an expectation into a normative first-match rule.

Leave `remove(value)` unshipped/pending and create a tiny follow-up design decision. Once ratified, implementation is straightforward using equality sends plus indexed removal/splice and needs no new primitive.

Do not preserve an older Set/Map-style `remove` behavior as evidence; List sequence occurrence semantics are a separate question.

## 14. Gate: `move(from:to:)` coordinate semantics

The ratified signature is:

```phalcom
list.move(from: source, to: destination)
// Result<Unit, IndexError>
```

Both coordinates use negative-index normalization and invalid coordinates use IndexError. What is not pinned is how `destination` is interpreted when `source < destination`:

```text
A. destination names the element-coordinate in the original/pre-removal List
B. destination names the insertion position in the post-removal List
```

These differ by one for forward moves and affect user-observable behavior.

Do not choose one during implementation.

Once ratified, use C.3 splice/removal infrastructure rather than adding a move primitive. The operation can capture the value, remove it, compute the ratified destination rule, and insert it.

## 15. `replace(range,with:)`

C.3 already implements the ratified recoverable slice replacement:

```phalcom
list.replace(range, with: replacements)
// Result<Unit, SliceError>
```

D.2 must not duplicate or rename it. Include it in List API docs/tests as part of the complete mutation-result philosophy and ensure no later `add`/chainability migration regresses its Unit-in-Ok result.

## 16. Retire chainable `add`

After direct List literal construction is green and repository callers are migrated:

1. replace ordinary List-building uses of `add(v)` with `append(v)`;
2. change internal core code to raw `push_` where it deliberately needs a low-level builder without public semantics;
3. remove `List#add` from the public core surface.

Do not touch `Set#add`: full Set semantics are outside D and its selector is a different collection's API.

Do not keep `List#add` as a receiver-returning compatibility alias. That would preserve the exact mutation-design rule D.2 is intended to retire.

If external compatibility policy later requires a deprecation window, it needs an explicit language compatibility decision rather than an undocumented alias in the implementation spec.

## 17. Existing setter mutation result

C.1 makes source-level subscript assignment evaluate to the original RHS independently of setter method return.

Therefore List's internal `[_,put:]` method may return Unit after successful mutation without changing:

```phalcom
result = list[i] = rhs
```

which still evaluates to `rhs` via compiler machinery.

Prefer Unit for the setter's own command result when called reflectively/directly, consistent with the mutation philosophy.

Do not reintroduce receiver chaining here.

## 18. Equality/hash behavior

List remains:

```text
ordered structural equality
mutable / unhashable as Map/Set key
```

D.2 mutation additions do not alter those rules.

Test that mutation changes subsequent structural equality as expected but do not add a List hash implementation.

## 19. Tests — literal construction

Add AST/compiler/runtime tests for:

```text
[]
[1]
[1,2,3]
[sideEffectA(), sideEffectB()]
trailing comma where accepted
nested Lists
List containing Unit and None
```

Assert:

- empty literal is List, not Unit;
- fresh identity for two empty literals;
- left-to-right exactly-once evaluation;
- bytecode/disassembly includes BuildList or actual chosen direct-build instruction;
- no public `add` send is emitted.

Keep spread pending for F.

## 20. Tests — mutations

### 20.1 Append/prepend/clear

Pin content, size, and exact Unit return.

### 20.2 Insert

At size 3 test positions:

```text
0
1
3
-1
-3
4 invalid
-4 invalid
```

Prove invalid insert returns Err and leaves List unchanged.

### 20.3 Indexed remove

Cover positive/negative valid indices, OOB Err, and stored None payload.

### 20.4 Pops

Cover empty and nonempty, including `Some(None)`.

### 20.5 removeAll

Cover remove none/all/some, stable retained order, callback count, and returned count.

### 20.6 Swap

Cover positive, negative, same-index no-op, each invalid argument position, and Unit-in-Ok.

### 20.7 Gated operations

Add pending/ignored tests or explicit design TODO fixtures for duplicate-value `remove(value)` and forward `move`, so future work cannot accidentally land an arbitrary rule unnoticed.

## 21. Repository migration audit

Search the entire repository for List-specific uses of:

```text
.add(
List.new()
.push_(
.at(_, put:)
[...]=
```

Distinguish Set `.add` from List `.add` before replacing anything.

Likely high-impact locations include:

- `phalcom-core/core/core.ph` result builders;
- parser literal lowering;
- collection tests;
- examples and benchmarks that construct Lists imperatively;
- old forge docs/tests asserting fluent List mutations.

Chained patterns such as:

```phalcom
List.new().add(a).add(b)
```

MUST be rewritten because `append` returns Unit.

Prefer:

```phalcom
[a, b]
```

where literal syntax is suitable, or explicit statements:

```phalcom
let xs = List.new()
xs.append(a)
xs.append(b)
```

## 22. Expected write set

Likely:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/**
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm.rs or vm execution module
phalcom-core/bin/phalcom/disasm.rs
phalcom-core/core/core.ph
phalcom-core/tests/lang/**
LSP/tooling files with exhaustive Expr matches
examples/** / benchmarks/** as migration requires
floor/current docs describing old add/list-literal lowering
```

No new heap Object arm and no new primitive method binding should be required.

## 23. Primitive-floor accounting

Expected D.2 primitive-floor delta against post-C HEAD:

```text
+0
```

D.2 reuses:

```text
List::push_             existing
List::set_              existing
List::replaceSlice_     C.3
```

`BuildList` is VM bytecode, not a primitive selector binding.

If an implementer believes another native List mutation primitive is necessary, stop and demonstrate why C.3's general splice cannot express the operation before amending the floor.

## 24. Completion gate

The executable part of D.2 is complete only when:

- List literals build directly and do not depend on `add`;
- `List#add` is retired after migration;
- canonical command mutations return Unit rather than self;
- insert/indexed-remove/pop/removeAll/swap match Result/Option/count semantics;
- all structural mutations reuse C.3 splice rather than new native verbs;
- `remove(value)` and `move(from:to:)` remain explicitly pending unless their missing semantics were separately ratified before implementation;
- primitive floor delta is zero;
- repository migration is complete;
- `./scripts/verify.sh --full` passes.
