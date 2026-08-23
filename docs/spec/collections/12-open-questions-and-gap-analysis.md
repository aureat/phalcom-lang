# Open Questions and Gap-Analysis Checklist

## 1. Purpose

This document lists decisions that remain unratified, consequences that need implementation review, and adversarial questions another agent should systematically investigate.

## 2. Highest-priority open decisions

### 2.1 Distinct `ArgumentPackType`

**Recommendation:** Ratify.

Without it, these become reflectively contradictory:

```phalcom
type Literal = (*: Int)
const callback: (*: Int) -> R
```

The first is an exact Tuple Type with literal key `#*`; the second needs an open positional domain.

Questions:

1. Is `ArgumentPackType` public?
2. Can users construct it directly?
3. What conversion message interprets a Tuple Type as a pack schema?
4. Are source and normalized forms both reflected?

### 2.2 Selector-valued call labels

Structural labels already allow:

```phalcom
+(_): handler
```

Should calls permit:

```phalcom
register(+(_): handler)
```

Options:

A. Call labels remain Symbols only.
B. Call labels become `Symbol | Selector`.
C. Direct syntax stays Symbol-only, but dynamic expansion may carry Selector labels.

Recommendation: A initially. B requires auditing selector interning, dispatch caches, method declaration grammar, protocol conformance, `doesNotUnderstand`, serialization, and diagnostics.

### 2.3 Complete expansion placement

Proposed grammar:

```phalcom
target(
  fixedPositionals,
  ***pack,
  fixedLabels
)
```

Questions:

1. May explicit labels follow `***pack`?
2. May explicit labels precede it? They probably cannot because it may emit positionals.
3. Is at most one `***` expansion mandatory?
4. Can `***record` appear after labels because its positional lane is statically empty, or is placement syntax-only?

Recommendation: syntax-only rule; one `***`; positionals before; labels after.

### 2.4 Record exactness and width subtyping

Questions:

1. Are Record Types exact by default?
2. Do immutable Records support width subtyping?
3. How does width subtyping interact with exact `**config: RecordType` capture?
4. Is there an explicit open-record type?

### 2.5 Set design

The entire Set surface remains provisional:

1. literal syntax;
2. mutability;
3. iteration order;
4. covariance;
5. hashability;
6. immutable Set naming.

## 3. Parser audit

The parser review must test:

```phalcom
(*P,)
(**P,)
(***P,)
(*: T)
(**: T)
***arguments: T
+(_): value
[#*]: value
```

Questions:

1. Does longest-token lexing make `***` unambiguous?
2. Can `(*: T)` be parsed without creating a special AST node?
3. Is `+(_): value` distinguishable from operator method syntax in all contexts?
4. Does a trailing comma remain necessary for a single unpacked domain item?
5. Can `...` remain both Ellipsis and repeated-tail marker without ambiguity?

## 4. Static-checker audit

The checker must answer:

1. Does every rest binder own exactly the declared lanes?
2. Are mixed schemas rejected on one-lane binders?
3. Are complete binders terminal?
4. Are split and complete modes mutually exclusive?
5. Are duplicate labels detectable statically when sources are known?
6. Are dynamic duplicate checks inserted otherwise?
7. Can generic `P: Tuple` retain both lanes through `***P`?
8. Does `*P` discard labels by type projection rather than by ad hoc syntax?
9. Are callable subtyping checks based on accepted packs?
10. Do defaults create unions, optional slots, or separate accepted-domain metadata?

## 5. Runtime and VM audit

Questions:

1. Is a call internally represented as one pack or separate arrays/maps?
2. Can split binders be zero-copy views?
3. Can complete capture reuse the original call pack?
4. When must a Tuple allocation occur?
5. How are duplicate labels detected efficiently across multiple expansions?
6. How are labels ordered reflectively?
7. Can pack projections be optimized away when unobserved?
8. How are Record captures materialized without violating Record construction rules?
9. What is the runtime failure when an unchecked value violates a rest annotation?
10. Do checked and unchecked invocation APIs differ?

## 6. Reflection audit

Verify that reflection distinguishes:

```text
source annotation
normalized ArgumentPackType
local binding Type
callable domain Type
rest mode
```

Adversarial examples:

```phalcom
method(*args: Int)
method(*args: (*: Int))
method(***args: Int)
method(***args: (*: Int, **: Int))
```

The first pair should normalize equally for the positional lane. The second pair should normalize equally for both lanes.

## 7. Equality and hashing audit

Questions:

1. Are `#+`, `#+()`, and `#+(_)` distinct keys?
2. Do `*:` and `[#*]:` collide?
3. Are normalized equivalent Types equal?
4. Is Type identity interned but non-normative?
5. Are ArgumentPackType hashes independent of source spelling?
6. Does labeled insertion order affect Type equality or only reflection?

Recommendation: structural Type equality should ignore source order where language semantics treat labels as keyed slots, while reflection may preserve declaration order. This needs ratification.

## 8. Call assembly audit

Stress cases:

```phalcom
target(*a, *b, fixed: value, **c, **d)
target(prefix, ***pack, fixed: value)
target(**tupleWithIgnoredPositionals)
target(***record)
```

Check:

1. left-to-right evaluation;
2. exact-once evaluation;
3. lane order;
4. duplicate errors;
5. partial side effects before failure;
6. static versus dynamic operand validation;
7. Selector-valued structural keys.

## 9. Generic forwarding audit

Canonical target:

```phalcom
forward<P: Tuple, R>(
  callable: (***P,) -> R,
  ***arguments: P
) -> R {
  return callable(***arguments)
}
```

Questions:

1. Is `P: Tuple` sufficient, or should the bound be `ArgumentPack`?
2. Does a broad `Tuple` bound include Selector-keyed tuples that are not call-compatible?
3. Is a dedicated `Pack` protocol/type needed?
4. Can a Record specialize `P`?
5. Can partially open pack types be generic arguments?
6. How are variance and substitution handled inside `***P`?

## 10. Tuple/Record composition audit

Because literal spread is call-only, explicit APIs are required.

Questions:

1. How are positional lanes concatenated?
2. How are labeled collisions handled?
3. Are override APIs separately named?
4. Does Tuple concatenation normalize lanes into positional-then-labeled order?
5. Can Records merge Selector keys?
6. Is composition lazy or eager?

## 11. Optionality and defaults audit

Cases:

```phalcom
method(timeout: Duration = second)
method(*args: Int, format: Format = defaultFormat)
```

Questions:

1. Is the callable domain a union of pack shapes?
2. How is optionality reflected?
3. Does omission differ from passing `None`?
4. Can open labeled capture receive a label also declared with a default? It should not; fixed binding should consume it first.

## 12. Error-recovery audit

The parser and checker should recover after malformed forms:

```phalcom
method(****args)
method(* *args)
method(**: T)
target(***, value)
```

Diagnostics should identify intended pack syntax without cascading into unrelated parse failures.

## 13. Security and robustness audit

1. Can malicious label hashing cause pathological duplicate checks?
2. Can recursive Type satisfaction overflow?
3. Can enormous unpacked packs exhaust memory before arity checks?
4. Are source locations preserved through expansion for tracebacks?
5. Are dynamic expansions exception-safe and cleanup-safe?

## 14. Recommended ratification sequence

1. `ArgumentPackType` as a distinct Type.
2. Call-label domain (`Symbol` versus `Symbol | Selector`).
3. Complete-expansion placement and one-per-call rule.
4. Default/optional callable-domain representation.
5. Record exactness and width subtyping.
6. Tuple/Record explicit composition APIs.
7. Set literal, mutability, and ordering.
8. Public reflection constructors and conversion APIs.

## 15. Final consistency criterion

The feature set is considered closed when every legal source form has:

1. one parse;
2. one contextual interpretation;
3. one normalized semantic object;
4. one reflection shape;
5. one satisfaction relation;
6. one failure rule for invalid use;
7. conformance tests covering static and dynamic paths.
