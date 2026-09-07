# Spec C.1 — Strict/Safe Indexed Access and Subscript Assignment Semantics

Status: implementation specification. Requires Spec A.3 and B.1 landed. Independent of Range syntax and slicing; do not wait for C.2.

## 1. Mission

Replace the old total-`at` sequence contract with the ratified distinction between strict subscript access and safe lookup while preserving Phalcom's selector-based, user-overridable subscript dispatch.

The end state for finite indexed sequences is:

```text
seq[index]
    -> element
    -> or raises IndexError

seq.get(index)
    -> Some(element)
    -> or None

seq[index, default: fallback]
    -> element if present
    -> otherwise already-evaluated fallback

seq.get(index, orElse: block)
    -> element if present
    -> otherwise block.call(originalIndex)
```

List and Bytes also support strict element assignment through `[index, put:]` / source `seq[index] = rhs`. Tuple remains immutable.

Tuple additionally supports:

```text
tuple[Symbol]
    -> labeled component
    -> or KeyError

tuple.get(Symbol)
    -> Option
```

Map keeps B.1 key semantics. A negative integer Map key is never an index.

This phase also corrects the language-wide expression rule:

```phalcom
const r = obj[index] = rhs
```

must bind `r` to the original `rhs`, not to the setter method's return value.

## 2. Preserve U-INDEX architecture

Do not revert bracket syntax to `at` desugaring.

HEAD's U-INDEX architecture is correct:

- parser creates `Expr::Index` / `Expr::SetIndex`;
- index argument lists can contain positional and labeled arguments;
- compiler derives `SignatureKind::Subscript` selectors;
- a setter appends ordinary label `put` after source subscript arguments;
- user-defined bracket selectors remain overridable methods.

C.1 changes the core collection methods and the SetIndex result-preservation lowering, not selector identity.

## 3. Shared raw index parsing/normalization

Create one Rust helper module or one clearly shared helper location used by List, Tuple, and Bytes primitives. Recommended location:

```text
phalcom-core/src/primitive/index.rs
```

or the closest existing primitive utility module if HEAD has gained one.

Do not keep three independent copies of index arithmetic.

### 3.1 Compatibility representation

Until the Int/Float split lands, accept only finite integral `Value::Number` values:

```text
finite && fract() == 0
```

Reject fractional, NaN, and infinities as a type/domain error.

Unlike HEAD's `expect_index`, do **not** reject negativity at the parsing stage.

Return a signed intermediate representation. Avoid casting a negative or out-of-range `f64` directly to `usize` and relying on Rust cast saturation/truncation behavior.

A suitable first-cut helper shape is:

```rust
pub(crate) enum NormalizedIndex {
    Valid(usize),
    OutOfRange,
}

pub(crate) fn normalize_element_index(value: &Value, len: usize)
    -> PhResult<NormalizedIndex>
```

Exact names are implementation-local.

### 3.2 Normalization

For length `n` and integer `i`:

```text
if i >= 0:
    normalized = i
else:
    normalized = n + i
```

valid iff:

```text
0 <= normalized < n
```

Examples at size 5:

```text
0   -> 0
4   -> 4
-1  -> 4
-5  -> 0
5   -> invalid
-6  -> invalid
```

Use comparisons before conversion to `usize` so enormous legacy `Number` values do not silently wrap.

### 3.3 Future numeric-tower replacement seam

Put a conspicuous comment and test marker at the legacy `Value::Number` acceptance branch:

```text
TODO(NUMERIC-TOWER): require Int; Float 1.0 must be rejected.
```

Do not scatter this TODO through List/Tuple/Bytes.

## 4. Correct existing raw sequence primitives without adding floor

C.1 should keep the existing raw `at_` / `set_` floor rather than adding `get_`/`hasIndex_` primitives.

This matters because the primitive floor is intentionally narrow and the public Option/strict distinction is derivable once the raw operation:

1. validates index representation;
2. understands negative coordinates;
3. exposes length through the existing size primitive.

### 4.1 `List::at_`

Change `list_raw_at` to use shared normalization.

Contract remains an internal total read:

```text
valid -> raw stored Value
invalid normalized coordinate -> surface None singleton
malformed index representation -> runtime type/domain failure
```

A valid List element may itself equal surface `None`; therefore public code MUST NOT infer absence from the raw returned value alone.

### 4.2 `Tuple::at_`

After A.3, `Tuple::at_` addresses total product order:

```text
all positionals
then all labeled values
```

Apply the same negative normalization over total `size`.

Do not index only the positional lane.

### 4.3 `Bytes::at_`

Apply identical coordinate normalization. A hit returns the octet Number; miss returns raw boundary `None` as today.

### 4.4 `List::set_` and `Bytes::set_`

Use the same negative index helper.

The raw primitive may retain an internal runtime failure on normalized miss because the public `.ph` setter wrapper will preflight and raise the surface `IndexError`. The raw path must no longer reject all negative inputs merely for being negative.

On success, return canonical Unit after Spec A rather than the historical `None` no-payload convention.

Do not add Tuple mutation.

## 5. Surface `IndexError`

Add `IndexError < Error` using the same kernel-error construction mechanism B.1 uses for `KeyError` on implementation HEAD.

Do not create a parallel error-allocation subsystem.

The exact structured payload is deferred by the language spec, but error creation SHOULD retain enough information to later expose:

- original supplied index;
- normalized index if meaningful;
- sequence size;
- operation (`read`, `write`, etc.).

For this phase a stable message plus original index/size, if easily expressible by the landed Error substrate, is sufficient.

Do not bake normalized-index details into selector identity or public API.

## 6. Public List safe/strict access

Rewrite the List access portion of `phalcom-core/core/core.ph`.

### 6.1 Internal validity check

Because raw `at_` returns the same surface `None` both for an invalid coordinate and for a valid slot containing `None`, public code must determine validity from the original index and `size`.

A safe pattern is:

1. call `at_(index)` first — this validates that the current legacy representation is an integral Number and performs the raw lookup;
2. compute the normalized coordinate in `.ph` from the now-validated numeric value and `size`;
3. determine whether the coordinate is in `0 <= j < size`;
4. interpret the raw value according to that validity bit.

Do not compare the raw returned value to `None` to decide presence.

Factor the repeated `.ph` arithmetic into the smallest class-local helper if useful; do not introduce a new public collection hierarchy merely for this helper.

### 6.2 Safe `get`

```phalcom
list.get(index)
```

returns:

```text
valid coordinate -> Some(rawValue)
invalid coordinate -> None
```

If the stored element is `None`, result is `Some(None)`.

### 6.3 Strict bracket read

```phalcom
list[index]
```

returns the stored value on valid coordinate and raises `IndexError` on invalid coordinate.

No `Option` leaks through strict syntax.

### 6.4 Eager default

Define the distinct selector represented by:

```phalcom
list[index, default: fallback]
```

The compiler already evaluates bracket arguments in lexical order before dispatch. Therefore `fallback` is evaluated even on a hit. Do not lazify it inside the method.

Return stored value on hit, fallback on invalid coordinate.

### 6.5 Lazy `orElse:`

Install the selector equivalent to:

```text
get(_,orElse:)
```

so an ordinary in-parentheses form works even if labeled trailing-closure syntax has not landed yet:

```phalcom
list.get(index, orElse: { missing => ... })
```

When labeled trailing closures become executable, their syntax must derive the same selector identity.

On miss call the block exactly once with the **original supplied index**. Do not pass the normalized internal coordinate.

On hit do not call the block.

## 7. Tuple integer and Symbol domains

Tuple's single `[_]` family must dynamically distinguish the argument domain; dispatch remains selector-only.

### 7.1 Integer domain

If argument is an index representation, use the total product order and the same List strict/safe rules.

### 7.2 Symbol domain

If argument is a Symbol:

- scan A.3's labeled-lane observations by Symbol identity;
- strict `tuple[symbol]` returns its value or raises `KeyError`;
- `tuple.get(symbol)` returns `Some(value)` or `None`;
- a stored value of surface `None` yields `Some(None)` from safe get.

Do not String-coerce labels.

### 7.3 Other domains

An argument that is neither a valid index representation nor Symbol is a type/domain error. Once the numeric tower lands this becomes conceptually “expected Int or Symbol”.

### 7.4 Default and `orElse:`

Both numeric and Symbol domains participate in:

```phalcom
tuple[key, default: fallback]
tuple.get(key, orElse: block)
```

A label miss supplies the original Symbol to `orElse:`. An index miss supplies the original index.

## 8. Bytes access

Mirror List for:

```phalcom
bytes[index]
bytes.get(index)
bytes[index, default: fallback]
bytes.get(index, orElse: block)
bytes[index] = octet
```

Retain octet validation on writes.

Strict out-of-range reads/writes raise `IndexError` at the public boundary. Safe read miss returns `None`.

## 9. Map lookup additions on top of B.1

Do not run sequence normalization for Map.

```phalcom
map[-1]
```

means lookup of key `-1`.

B.1 already provides:

```text
map.get(key) -> Option
map[key] -> value or KeyError
```

C.1 adds the shared lookup vocabulary:

### 9.1 Eager default

```phalcom
map[key, default: fallback]
```

The fallback expression is eagerly evaluated by normal argument evaluation.

Perform one logical Map lookup; do not hash/compare twice just to decide hit/miss.

### 9.2 Lazy fallback

```text
get(_,orElse:)
```

On miss call the block with the original key exactly once.

### 9.3 Lazy lookup-and-insert

Mutable Map supports selector:

```text
get(_,orPut:)
```

Semantics:

```text
hit  -> return existing value, do not call block
miss -> call block(originalKey) once
        insert produced value using ordinary Map insertion semantics
        return produced value
```

Use B.1's single-lookup/internal locate seam where possible so the implementation does not perform one miss lookup then a second full hash/== lookup to insert. Preserve reentrant-hash borrow discipline.

Do not alter equal-but-nonidentical stored-key retention policy.

## 10. Reject duplicate setter `put:` label before dispatch

The source:

```phalcom
obj[put: address] = value
```

is invalid because setter lowering would append a second `put:` label.

HEAD currently appends `put` in the compiler. Add an earlier parser/compiler diagnostic at the `Expr::Index` -> `Expr::SetIndex` transition or the earliest existing duplicate-label validation seam.

Requirements:

- reject if the source bracket argument list already contains label `put`;
- use the ordinary label identity/canonicalization rules current at implementation HEAD;
- do not globally reserve the word `put`; `obj[put: address]` as a getter remains legal;
- do not defer the problem to method lookup with a bizarre duplicate-label selector.

Spec F will generalize duplicate labels for expanded packs; C.1 needs only the statically explicit setter collision.

## 11. Fix subscript-assignment expression value

### 11.1 Current defect

HEAD compiles SetIndex as:

```text
receiver
arguments in lexical order
RHS
Invoke setter
```

so the value left by the expression is whatever `[]=` returns.

Ratified semantics require:

```text
receiver
arguments
RHS
setter dispatch/execution
expression result = original RHS
```

The RHS must be evaluated exactly once and object identity preserved.

### 11.2 Do not change setters to return RHS

The compiler must enforce assignment-expression semantics independently of method implementation.

A user may define:

```phalcom
[i, put: v] { return #anything }
```

and:

```phalcom
const x = obj[i] = rhs
```

must still yield `rhs`.

Do not require or document setter methods to return their assigned value.

### 11.3 Recommended lowering: hidden preservation stack slot using existing bytecodes

Prefer avoiding a special post-invoke opcode because an `Invoke` may enter a bytecode method frame and complete later; a simple single-step opcode cannot discard the eventual result unless call-frame continuation state is extended.

Use an unnameable compiler-owned temporary stack/local slot:

```text
1. reserve hidden slot (push placeholder)
2. compile receiver
3. compile subscript arguments in lexical order
4. compile RHS
5. copy RHS into hidden slot using existing SetLocal-like semantics
6. Invoke setter with the ordinary receiver/arguments/RHS segment
7. Pop setter result
8. treat preserved hidden slot as this expression's one result
9. remove compiler-local metadata WITHOUT emitting a Pop for that slot
```

The hidden local name, if the compiler requires one, must be impossible to resolve from source and must never be capturable by user code.

If HEAD's local bookkeeping makes this unsafe, an equivalent dedicated stack instruction is acceptable, but document why it is needed. Do not change evaluation order or evaluate RHS twice.

### 11.4 Failure behavior

If receiver/argument/RHS evaluation, dispatch, or setter execution fails, the assignment expression does not successfully yield the preserved RHS. Ordinary VM unwind handles the hidden slot.

### 11.5 Regression fixture

Define a user setter that deliberately returns a distinguishable value and assert:

```phalcom
let rhs = SomeObject.new()
let result = receiver[index] = rhs
result === rhs
```

Also count side effects in receiver, index, and RHS producers to prove exactly-once order:

```text
receiver -> index arguments left-to-right -> RHS -> setter body
```

## 12. Tests

### 12.1 Rust helper tests

Table-test normalization at lengths 0, 1, 5:

```text
0, last positive, -1, -len, len, -(len+1)
fractional
NaN
+/- infinity
very large finite integral Number
```

### 12.2 Language tests — List

Cover:

- `[0]`, `[-1]`, `[-size]` hits;
- positive and negative OOB strict `IndexError`;
- safe misses;
- a List slot storing `None` -> `get` returns Some(None);
- strict valid slot storing `None` returns surface None, not an error;
- eager default side effect executes on both hit and miss;
- lazy orElse side effect only on miss and receives original `-100`.

### 12.3 Tuple

Cover:

- total order across positional/labeled boundary;
- negative indexing into labeled suffix;
- Symbol hit/miss;
- Symbol present-None;
- String is not a Symbol label;
- integer/label default and orElse forms.

### 12.4 Bytes

Cover negative read/write, OOB IndexError, safe get, octet validation unchanged.

### 12.5 Map

Prove key `-1` remains key `-1`, and `orElse`/`orPut` do not sequence-normalize it.

### 12.6 Assignment

Test custom setters returning Unit, receiver, Number, and None while source assignment always evaluates to original RHS.

Test duplicate `put:` rejection on setter syntax but legal getter `obj[put: x]` parsing.

### 12.7 Numeric pending regression

Add a clearly marked ignored/pending test:

```phalcom
list[1.0]
```

expected eventually to fail once numeric values preserve Int/Float identity.

Do not make this phase fail indefinitely on a distinction the runtime cannot represent.

## 13. Floor and governance

Expected native binding delta: **0**.

C.1 changes semantics of existing List/Tuple/Bytes raw primitives but should not add a new primitive selector merely to implement safe/strict wrapping.

Update current docs/ADRs that still state:

```text
at(_) is the public total collection contract
negative indices invalid
```

only where they are normative/current. Historical as-built documents should be annotated/superseded rather than rewritten as though they never shipped.

If the post-A/B floor census names return contracts for `at_`/`set_`, update those descriptions but do not change binding counts.

## 14. Completion checklist

C.1 is complete only when:

- all three finite indexed native families use one normalization helper;
- negative indexing works consistently;
- strict `[]` raises IndexError and safe `get` returns Option;
- stored None is never confused with coordinate absence;
- Tuple supports both integer and Symbol lookup domains;
- Map negative integer keys are untouched;
- `default:` and `orElse:` behavior is tested;
- Map `orPut:` is one-call/one-insertion correct;
- setter `put:` collisions reject early;
- subscript assignment result is original RHS independent of setter return;
- the numeric-tower compatibility seam is isolated/documented;
- `./scripts/verify.sh --full` passes.
