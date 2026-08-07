# Spec C.2 — Range Syntax and Bound Representation

Status: implementation specification. Requires A.2 for canonical Unit conventions only indirectly; may be implemented after C.1 or in parallel if files do not conflict. C.3 requires this phase.

## 1. Mission

Replace Phalcom's obsolete inactive Range design with the ratified Range-as-bounds model:

```phalcom
a..b    // lower-inclusive, upper-exclusive
a..=b   // lower-inclusive, upper-inclusive
a..      // lower only
..b      // upper-exclusive only
..=b     // upper-inclusive only
..       // fully unbounded bound descriptor
```

`...` is not a Range operator.

Range syntax must construct a native Range value directly; it must not dispatch through overridable class construction.

This phase establishes bound representation and syntax only. It intentionally does not pretend the deferred Progression/descending/exhaustion semantics are settled.

## 2. Current repository conflict to remove

HEAD's `RangeObject` contains:

```rust
start: Value,
end: Value,
inclusive: bool,
```

and comments define:

```text
a..b   -> inclusive
a...b  -> exclusive
```

`Range.class::new(start,end,inclusive)` is a public native primitive, while `.ph` Range methods derive size/iteration/hash/equality from those three mandatory fields.

Every part of that semantic convention conflicts with the new specification:

- `..` switches to upper-exclusive;
- `..=` becomes inclusive;
- `...` leaves the Range grammar;
- either bound can be absent;
- a Range is first a bound structure, not intrinsically a finite numeric iteration triple.

Do not preserve the old convention as a compatibility alias.

## 3. Lexer changes

### 3.1 Token set

Keep:

```rust
Token::DotDot
Token::DotDotDot
```

Add:

```rust
Token::DotDotEqual
```

or repository-consistent equivalent spelling.

### 3.2 Scan order

When scanning punctuation beginning with `.` distinguish:

```text
...  -> DotDotDot
..=  -> DotDotEqual
..   -> DotDot
.    -> Dot
```

The exact branch order may differ as long as maximal munch produces those tokens.

Do not tokenize `...` as `..` + `.`.

### 3.3 Tests/tooling

Update:

- lexer punctuation snapshots;
- token display/debug expectations;
- LSP semantic-token operator classification;
- any exhaustive token matches.

Add fixtures proving:

```text
1..2
1..=2
1...2
..
..=
```

lex into the intended tokens, with bare `..=` later rejected by the parser for missing upper endpoint.

## 4. Explicit Range AST

Do not desugar Range syntax into `Range.new` method calls.

Add an explicit node, conceptually:

```rust
pub struct RangeExpr {
    pub lower: Option<Expr>,
    pub upper: Option<Expr>,
    pub upper_inclusive: bool,
    pub range: SourceRange,
}

Expr::Range(Box<RangeExpr>)
```

Use repository-consistent boxing/field conventions.

Invariants after parsing:

- `upper_inclusive == true` implies `upper.is_some()`;
- lower bound, when present, is always inclusive;
- fully unbounded `..` has both endpoints absent and `upper_inclusive == false`;
- `a..` has lower present, upper absent, `upper_inclusive == false`.

Do not encode absence as a source `None` expression in AST. `..None` means an explicitly present endpoint whose value is surface None and is distinct from omitted upper bound.

## 5. Parser

### 5.1 Supported forms

Parse all six ratified forms.

Endpoint evaluation order is lexical:

```text
lower first, then upper
```

One-sided forms evaluate only the present endpoint.

### 5.2 Non-associativity

Reject unparenthesized second Range operators:

```phalcom
1..2..3
1..=2..3
1..2..=3
```

with a specific parse diagnostic rather than leaving a confusing trailing-token error if practical.

Parentheses do not automatically make the result semantically meaningful; they only allow ordinary expressions where syntax permits.

### 5.3 Precedence placement

The ratified constraints are:

```text
arithmetic binds tighter than Range
Range binds tighter than assignment
```

The source specification intentionally leaves exact numeric precedence implementation-defined and does not fully order Range relative to every boolean/comparison/coalescing operator.

Use the smallest reversible parser layer that satisfies the ratified constraints on current HEAD: insert one `parse_range` tier above assignment and below the existing tighter expression tier used for ordinary non-assignment endpoint expressions.

Pin tests only for normative relationships:

```phalcom
a + 1 .. b * 2
// endpoints are (a + 1), (b * 2)

x = a..b
// assignment receives Range
```

Do not create a new language-design claim in docs about ambiguous comparison/boolean mixtures beyond whatever parser placement mechanically entails. If a later syntax-wide precedence table ratifies a finer ordering, this tier must be movable without AST/runtime changes.

### 5.4 Prefix one-sided parsing

The parser must permit a Range expression to begin with `DotDot`/`DotDotEqual`, unlike ordinary infix binary operators.

Rules:

```text
..=  MUST be followed by an upper expression
..   MAY be followed by an upper expression or stand alone
```

Use expression-start/terminator knowledge rather than whitespace heuristics.

## 6. Runtime representation without arena bloat

### 6.1 Do not use two Rust `Option<Value>` fields blindly

A native `Object` arena slot is sensitive to the largest enum arm. Two `Option<Value>` values may enlarge `RangeObject` depending on representation and in turn enlarge every heap slot.

Preserve roughly the existing compact shape using the private sentinel internally:

```rust
pub struct RangeObject {
    lower: Value,
    upper: Value,
    upper_inclusive: bool,
}
```

Interpret internal `Value::Nil` as “bound absent”. User code cannot produce that sentinel.

An explicitly supplied surface `None` endpoint is `Value::Obj(none_singleton)` and therefore remains distinguishable from omission.

Constructor/helper API should accept Rust options and canonicalize internally, e.g.:

```rust
RangeObject::new(lower: Option<Value>, upper: Option<Value>, upper_inclusive: bool)
```

with assertions:

```text
upper absent -> upper_inclusive false
lower/upper present values must never be private Value::Nil
```

### 6.2 Accessors

Provide internal Rust observations:

```text
lower() -> Option<Value>
upper() -> Option<Value>
upper_inclusive() -> bool
has_lower()/has_upper() optional convenience
```

GC trace must visit present endpoint objects and naturally ignore absent Nil sentinels.

Add `size_of::<Object>()` regression/measurement if prior A work already establishes one. Do not silently grow every heap slot.

## 7. Range bound primitive surface

The core `.ph` layer cannot inspect a native RangeObject. Keep the smallest native observations needed by C.3 and future E.

Recommended public-to-core raw observers:

```text
lower_             -> Option<Value>
upper_             -> Option<Value>
upperInclusive_    -> Bool
```

`lower_`/`upper_` must return explicit Option, not raw “value or None”, because an explicitly present endpoint may itself be the surface None value:

```phalcom
None..x
```

is structurally lower-present even if a later slicing consumer rejects that endpoint type.

Reuse the existing Option wrapper helper rather than creating Range-specific Some allocation code.

Retire/rename obsolete `start_` / `end_` / `inclusive_` names if keeping them would preserve the wrong semantic vocabulary. Update floor governance accordingly.

## 8. Retire the public three-argument constructor

Once direct Range bytecode works, remove:

```text
Range.class::new(start,end,inclusive)
```

unless an independent current specification explicitly requires it after HEAD inspection.

Reasons:

- cannot express omitted bounds without inventing sentinel surface arguments;
- exposes an obsolete boolean inclusion protocol;
- would make Range literal implementation temptingly dispatch through overridable construction;
- syntax is the canonical construction mechanism.

Search examples/tests/benchmarks and migrate legitimate callers to Range syntax. Historical fixtures demonstrating the old constructor should be retired/superseded.

## 9. Direct construction bytecode

Add one bytecode conceptually:

```rust
BuildRange {
    has_lower: bool,
    has_upper: bool,
    upper_inclusive: bool,
}
```

Exact encoding may use flags/u8 to fit repository style.

Compiler stack order:

```text
if lower present: compile/push lower
if upper present: compile/push upper
BuildRange flags
```

Thus lower evaluates before upper.

VM handler pops only the endpoints indicated by flags, restores source order, allocates `RangeObject`, pushes `Value::Obj(range)`.

Do not invoke user code during construction.

Update every bytecode bookkeeping site:

- enum;
- name table;
- variant count/index match;
- disassembler;
- VM dispatch;
- tests/snapshots.

## 10. Existing `.ph` Range methods: remove obsolete semantic assumptions

Audit `class Range` in `phalcom-core/core/core.ph`.

Current methods assume two mandatory numeric endpoints and the old inclusion flag. They cannot remain unchanged.

### 10.1 Must be corrected/removed in C.2

Any implementation that spells inclusive Range as `..` and exclusive as `...` must change immediately.

Remove or quarantine methods whose semantics are explicitly deferred and whose old implementation would lie for one-sided ranges, including old structural `==`/`hash` if they assert a now-unratified value identity contract.

Do not preserve old equality/hash merely because the object remains immutable; Range equality/hashability are explicitly deferred.

### 10.2 Iteration compatibility

Full Range/Progression iteration is not C.2's mission. Avoid inventing behavior for:

- reversed bounds;
- negative steps;
- non-integer endpoint domains;
- fully unbounded iteration.

If current finite ascending integer Range iteration can be mechanically corrected to half-open/closed upper semantics without settling those cases, it may remain as a compatibility subset. It must explicitly reject/defer forms outside that subset rather than accidentally using `None`/Nil in arithmetic.

Do not add `by(step)` here merely to produce a half-specified Progression.

## 11. Rendering audit

The final printing policy is deferred, but all diagnostic/debug/native render paths must stop emitting obsolete `...` Range spelling.

At minimum:

```text
lower present + exclusive upper -> a..b
lower present + inclusive upper -> a..=b
lower only -> a..
upper exclusive only -> ..b
upper inclusive only -> ..=b
no bounds -> ..
```

If user-level `toString` is intentionally left deferred, native diagnostic rendering must still be truthful and non-panicking.

## 12. Progression deliberately deferred

The authoritative Range spec ratifies the conceptual distinction:

```text
Range = bounds/inclusion
Progression = stepped traversal
```

and names:

```phalcom
range.by(step)
```

as the direction for progression construction, while explicitly deferring:

- exact result/runtime object model;
- zero-step validation;
- sign mismatch behavior;
- descending semantics;
- negative stepping.

Do not create a skeletal `Progression` whose behavior would later become compatibility baggage. Spec E or a dedicated progression unit should implement it once those semantics are fixed.

## 13. Tests

### 13.1 Lexer

Snapshot every punctuation family including `..=` and preserve `...` as distinct.

### 13.2 Parser/AST

Assert exact nodes for all six forms, endpoint absence, inclusion bit, ranges/spans, and non-association errors.

### 13.3 Evaluation order

Use side-effecting endpoint functions to prove lower runs before upper and omitted endpoints do not run anything.

### 13.4 Runtime distinction

Prove omitted versus explicit None remains distinguishable through raw bound observers:

```text
..x        lower_ -> None
None..x    lower_ -> Some(None)
```

### 13.5 Old spelling rejection

`1...4` must not produce a Range. It should lex with `DotDotDot` and be rejected/handled only by whichever spread grammar exists at that position.

### 13.6 Construction override immunity

If `Range.new` can be redefined before retirement or a user binds a different `Range` global, Range syntax must still build the native Range value through bytecode, not dispatch.

## 14. Floor/governance

Expected direction:

- remove old `Range.class::new(_,_,_)` primitive binding;
- retain three underivable native bound observations, with corrected names/return shapes;
- net binding count likely decreases by one if the same three getter slots are reused.

Do not hard-code the count in implementation before inspecting post-A/B HEAD.

Amend/supersede old ADR/floor text that says:

```text
Range fields = start/end/inclusive
a..b is inclusive
a...b is exclusive
```

Historical documents may remain as historical records with supersession notes.

## 15. Completion checklist

C.2 is complete only when:

- six Range forms parse;
- `...` is not a Range;
- Range operators are non-associative;
- arithmetic/assignment precedence constraints are tested;
- AST preserves endpoint omission rather than synthesizing None;
- bytecode construction bypasses user dispatch;
- runtime distinguishes omitted from explicit None endpoint;
- old inclusion convention and old constructor are retired;
- Range arena layout is measured/not accidentally bloated;
- deferred Progression/descending semantics are not invented;
- `./scripts/verify.sh --full` passes.
