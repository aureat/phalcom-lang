# Phalcom Updated Language Semantics — Implementation Specification

> **Status:** implementation work order / consolidated language change-set
> **Target:** current `aureat/phalcom-lang` `main` baseline, inspected 2026-08-21
> **Audience:** a Phalcom implementation agent working across `phalcom-ast`, `phalcom-core`, the universe package, tests, documentation, and `phalcom-lsp`
> **Scope:** ratified/favored iteration and destructuring syntax; bilateral cooperative operators; `Ordering` and `<=>`; non-overridable sameness `===`; chained relational expressions; `matches`; `Ellipsis` / `...`; `understands`; `if let` / `while let`; corresponding LSP and runtime semantics
>
> **Dispatch note:** repository line numbers will move. This document intentionally keys implementation work to **paths, concrete types, functions, selectors, and current code shapes**. Re-confirm exact line numbers against HEAD before editing.

---

# 0. Executive contract

This change-set must leave Phalcom with a small number of reusable semantic mechanisms rather than one-off syntax implementations.

The core rules are:

1. **Control-flow headers do not require wrapper parentheses.** Canonical syntax is `if condition {}`, `while condition {}`, `for item in items {}`. Parentheses remain meaningful as ordinary expression grouping and tuple/pattern syntax.
2. **`for` consumes patterns, not identifiers.** Its lane grammar is `pattern [at index] in iterable`.
3. **Option C is ratified for parallel iteration.** Complete lanes separated by a top-level comma advance in lockstep: `for x in xs, y in ys {}`. The explicit value-level equivalent is `for (x, y) in (xs, ys).zipped {}`.
4. **`at` binds the zero-based iteration ordinal of one lane.** It is not a collection key/index protocol and it does not expose an implementation cursor.
5. **`.indexed` and `.zipped` remain ordinary first-class iterable operations.** The compiler may optimize the `for` sugar directly, but observable semantics must agree with these value-level operations.
6. **Destructuring is one shared pattern system** used by `let`, `const`, `for`, `if let`, and `while let`. `if let` / `while let` use refutable matching; unconditional `let`/`const`/`for` use the same patterns in irrefutable mode and raise on mismatch.
7. **Bilateral operators are language-level cooperative dispatch over ordinary methods.** A protocol method may return the canonical `unsupported` singleton to decline one candidate. A raised error is a real failure and stops fallback.
8. **Arithmetic/bitwise bilateral protocols use a direct selector and a reflected selector.** `a + b` considers `a.+(b)` and `b.+(from: a)`. The reflected candidate always computes the original `lhs OP rhs`; it is not `rhs OP lhs`.
9. **Right-hand strict subtypes get reflected priority when they define a more-specific reflected implementation.** This prevents a broad base implementation from swallowing a subtype's mixed-type semantics.
10. **`<=>` is a special bilateral comparison operator.** It uses `compare(_)` on either operand; a successful right-side comparison has its `Ordering` reversed.
11. **`Ordering` is temporarily a manually singletonized sealed class**, not `@variant`, until unit-variant semantics are improved.
12. **`===` is exact runtime sameness and cannot be overridden.** It is not `==`; it performs no coercion and no user dispatch.
13. **Relational chains are Python-style.** `0 <= x < 10` evaluates `x` once, left-to-right, and short-circuits.
14. **`matches` is right-owned:** `candidate matches pattern` invokes `pattern.matches(candidate)`. The first implementation supports ordinary equality-pattern fallback and selector-pattern matching without pretending the full future pattern language has already landed.
15. **`...` is an ordinary expression value outside pattern syntax:** the unique `Ellipsis` instance. Inside selector/future structural patterns, `...` remains grammar metasyntax and must not evaluate to the Ellipsis object.
16. **`understands` is left-owned reflective capability:** `object understands selector` invokes `object.understands(selector)` and tests lookup without actually sending the message or entering `doesNotUnderstand`.
17. Existing landed `is`, `is!`, `is not`, `in`, `not in`, `is in`, `is not in`, and their already-ratified particle combinations are **dependencies, not reimplementation scope**. Integrate them into relation chaining without redesigning them here.
18. Do **not** add `!==`, `for ... else`, compact Cartesian syntax, header-global `at`, or arbitrary pattern syntax not specified below.

---

# 1. Repository baseline and high-risk seams

The implementation must begin by re-reading the following current files. They are the actual seams this change-set extends.

| Area | Current source of truth | Existing shape that matters |
|---|---|---|
| Token alphabet | `phalcom-ast/src/token.rs` | `Token::{EqualEqual,BangEqual,Less,LessEqual,Greater,GreaterEqual,DotDotDot,At,In,Is,...}`; `DotDotDot` already exists; `matches`/`understands` do not need global keyword tokens |
| Lexer | `phalcom-ast/src/lexer.rs` | longest-operator scanning must recognize `===` and `<=>` before their shorter prefixes |
| AST | `phalcom-ast/src/ast.rs` | `BinaryOp`; `Pattern::{Name,Tuple,List}`; `ForStatement { binding, binding_range, iter, body, range }`; selector-pattern AST already exists |
| Parser | `phalcom-ast/src/parser.rs` | `parse_binary`; `parse_pattern`; `parse_binding`; `parse_for`; existing `if`/`while` parse/desugar paths; selector-spec parser |
| Binary lowering | `phalcom-core/src/compiler/lib/expr.rs` | current `Expr::Binary` path states that ordinary binary operators lower to sends; this becomes a classification rather than a universal rule |
| Pattern lowering | `phalcom-core/src/compiler/lib/patterns.rs` | `compile_pattern_bind_top_of_stack`, `compile_pattern_bind_from_slot`, `emit_pattern_arity_check`; initializer is already evaluated once |
| Loop lowering | `phalcom-core/src/compiler/lib/loops.rs` | `compile_for` uses cursor protocol, synthetic locals, jump loop, per-iteration `CloseUpvalue`, and no body `block_call` |
| Loop state | `phalcom-core/src/compiler/lib/state.rs` | `LoopContext` / function nesting must remain valid after generalized loop patterns |
| Bytecode | `phalcom-core/src/bytecode.rs` | ordinary `Invoke(u8,u16)` plus jump opcodes; BYTECODE_NAMES/index tables must stay synchronized when adding opcodes |
| VM dispatch | `phalcom-core/src/vm/dispatch.rs` | exact lookup, call-frame entry, inline-cache path, compiler-internal invoke handling |
| Dynamic/rest send | `phalcom-core/src/vm/send.rs` | `lookup_rest_method`; useful for `understands`, but bilateral protocol candidates should remain exact fixed-arity selectors |
| Value/class lookup | `phalcom-core/src/value/mod.rs` | `Value::class`, `Value::lookup_method`, `Value::value_eq`; add exact sameness separately rather than changing equality |
| Numeric ops | `phalcom-core/src/primitive/number.rs` | current arithmetic/comparison functions use `promote_pair` and raise a type error for non-number operands; bilateral migration must turn operand-domain refusal into `unsupported` without swallowing real numeric failures |
| Selector model | `phalcom-common/src/selector.rs` | rich `Selector`/`SelectorPattern`; `SelectorPattern::matches` already implements structural matching |
| Runtime selector pattern | `phalcom-core/src/heap/selector_pattern.rs` | `SelectorPatternObject` already contains rich and compiled runtime pattern forms |
| Selector surface | `phalcom-core/core/universe/src/reflection/selector.ph` | exact selector surface class exists, while exact `#...` literals currently lower to `Symbol`; account for this in `matches`/`understands` |
| Object surface | `phalcom-core/core/universe/src/object/object.ph` | current `Object` defines `is` / `is!`; add derived comparison defaults and equality-pattern fallback here when possible |
| Iterables | `phalcom-core/core/universe/src/collections/iterable.ph` | cursor protocol is `iterate(_)` / `iteratorValue(_)`; generic transforms already operate directly on it |
| Universe organization | `phalcom-core/core/universe/src/package.ph` and subpackage `package.ph` files | new Ordering/Ellipsis/reflection pieces must be exposed through the package graph |
| `unsupported` | `phalcom-core/core/universe/src/errors/unsupported.ph` | existing manually singletonized `Unsupported`; do not replace it with an exception or native sentinel |
| Core class bootstrap | `phalcom-core/src/universe/core_classes.rs` | native heap variants require an actual class ID; `SelectorPatternObject` currently maps through the object model and may need a dedicated class |
| GC roots | `phalcom-core/src/vm/gc.rs` | any VM-retained late-bound core singleton must be traced/rooted |
| LSP dispatch | `phalcom-lsp/src/semantic/dispatch.rs`, `infer.rs`, `flow.rs`, `scope.rs`, `occurrence.rs`, `surface.rs`, `index.rs` | operator dispatch, pattern definitions/scopes, flow narrowing, selector references, and semantic tokens must be updated together |
| Language corpus | `phalcom-core/tests/lang/`, `phalcom-core/tests/lang/MANIFEST.md`; `phalcom-ast/tests/{lexer,parser}.rs` | add positive/negative/runtime/AST coverage instead of only unit tests |

Two existing implementation details are load-bearing and must survive:

- `compiler/lib/loops.rs::compile_for` deliberately emits an inlined cursor loop so a loop body can suspend/yield without crossing a native block call.
- `compiler/lib/patterns.rs` deliberately evaluates a destructuring source once and uses scratch locals. Generalizing patterns must retain single evaluation and must not regress scratch-local/upvalue safety.

---

# 2. Semantic classification of binary-looking syntax

Do not route every new relation through the same mechanism. Define a compiler-level classification.

| Surface | Ownership / lowering | Overridable? | Result |
|---|---|---:|---|
| `a + b`, `a - b`, `a * b`, `a / b`, `a ~/ b`, `a % b`, `a ** b` | bilateral direct/reflected protocol | yes | arbitrary value |
| `a << b`, `a >> b`, `a & b`, `a ^ b`, `a \| b` | bilateral direct/reflected protocol | yes | arbitrary value |
| `a <=> b` | bilateral `compare(_)`, reverse RHS result | yes | `Ordering` |
| `< <= > >=` | existing operator methods remain valid; add `Object` defaults derived from `<=>` so implementing `compare(_)` is sufficient for user types | yes | Bool |
| `== !=` | existing equality model; **not bilateralized in this unit** | yes/current rules | Bool |
| `a === b` | intrinsic exact sameness | **no** | Bool |
| existing `is` / `is!` families | already-landed semantic relation | existing guarantee | Bool |
| `x in c` families | already-landed right-owned containment | existing design | Bool |
| `candidate matches pattern` | `pattern.matches(candidate)` | yes | Bool |
| `object understands selector` | `object.understands(selector)` | yes/reflective | Bool |
| `and`, `or` | existing lazy control semantics | existing design | Bool |

This classification should exist in one helper/table rather than being duplicated between parser, compiler, LSP, and documentation.

---

# 3. Task Set A — exact sameness `===`

Implement `===` before `Ordering`, because the canonical Ordering implementation should use sameness internally.

## A.1 Syntax

Add a token such as `Token::TripleEqual`.

In `phalcom-ast/src/lexer.rs`, recognize `===` before `==` and `=`. Add lexer snapshots for:

```phalcom
x === y
#===(_)
```

The selector/symbol parser must accept `===` as a symbolic selector base wherever `==`, `+`, etc. are currently accepted.

Add `BinaryOp::Same` (or a better project-consistent name). Do not add `!==`.

## A.2 Runtime semantics

`===` asks whether two values are the **same runtime value/object**, not whether they are equal under `==`.

Implement a dedicated helper in `phalcom-core/src/value/mod.rs`, e.g.:

```rust
pub fn same_as(&self, other: &Value) -> bool
```

Required semantics:

- different runtime tags/classes of immediate scalar representation => false;
- `Int` immediates: identical integer payload => true;
- `Float` immediates: exact IEEE representation (`to_bits`) => true; therefore `0.0 === -0.0` is false and identical NaN bit-patterns may be same even though `==` is false;
- Bool/Unit/None/Symbol immediates: exact payload/interned identity;
- nested immediate `Some` representation: same wrapper representation and recursively same payload;
- heap objects: exact `ObjRef` identity only;
- equal-content strings stored at different handles are **not** same;
- equal-content heap `LargeInt` values stored at different handles are **not** same;
- no numeric cross-kind coercion (`1 === 1.0` is false);
- private `NIL` handling remains internal and must not surface.

Do not modify `Value::value_eq`; `==` and `===` are intentionally different.

## A.3 Method-shaped interface and override prohibition

Expose an `Object::===(_)` native method for reflection/direct sends, but make the selector semantic/sacred:

```phalcom
obj === other
obj.===(other)
```

must have the same semantics.

Reuse the existing mechanism that protects compiler/LSP-guaranteed `is` / `is!` selectors from semantic override. If current HEAD still implements the surface methods but lacks a shared override-prohibition hook, create one shared reserved-semantic-selector table in the class compilation/installation path and include `is(_)`, `is!(_)`, and `===(_)` rather than introducing a `===`-only special case.

A class declaration that attempts:

```phalcom
class Bad {
  ===(_ other) { true }
}
```

must fail at compile/install time with a dedicated diagnostic such as:

```text
semantic.selector_reserved: '===(_)' defines compiler-guaranteed sameness and cannot be overridden
```

Future message interception must also exempt this selector from semantic alteration.

## A.4 Lowering/optimization

The minimum implementation may invoke the sealed native method through ordinary `Invoke`. Prefer an intrinsic/fused bytecode only if profiling shows a benefit; do not require a new opcode solely to make `===` work.

The LSP may always infer Bool for `===` and must never search user overrides.

---

# 4. Task Set B — bilateral cooperative operator dispatch

This is the architectural foundation for `<=>` and mixed-type arithmetic.

## B.1 Protocol contract

For a bilateral arithmetic/bitwise operator `OP`:

```phalcom
lhs OP rhs
```

has two possible protocol candidates:

```phalcom
lhs.OP(rhs)          // direct candidate
rhs.OP(from: lhs)    // reflected candidate
```

The reflected method is defined to compute the **original** `lhs OP rhs` expression. This matters for non-commutative operations.

Examples:

```text
lhs - rhs
  direct:    lhs.-(rhs)
  reflected: rhs.-(from: lhs)   // means lhs - rhs

lhs / rhs
  direct:    lhs./(rhs)
  reflected: rhs./(from: lhs)   // means lhs / rhs
```

Use Phalcom's ordinary labeled-selector identity. Do not introduce Python-style hidden `__radd__` names.

Required direct/reflected pairs:

| Operator | Direct selector | Reflected selector |
|---|---|---|
| `+` | `+(_)` | `+(from)` |
| `-` | `-(_)` | `-(from)` |
| `*` | `*(_)` | `*(from)` |
| `/` | `/(_)` | `/(from)` |
| `~/` | `~/(_)` | `~/(from)` |
| `%` | `%(_)` | `%(from)` |
| `**` | `**(_)` | `**(from)` |
| `<<` | `<<(_)` | `<<(from)` |
| `>>` | `>>(_)` | `>>(from)` |
| `&` | `&(_)` | `&(from)` |
| `^` | `^(_)` | `^(from)` |
| `\|` | `\|(_)` | `\|(from)` |

A raw direct message send does **not** recursively trigger bilateral dispatch:

```phalcom
a.+(b)           // raw protocol call; may return unsupported
a + b            // complete operator resolution
b.+(from: a)     // raw reflected protocol call
```

## B.2 `unsupported` versus errors

The existing canonical singleton from `core/universe/src/errors/unsupported.ph` is the cooperative decline value.

Protocol candidate outcomes:

```text
normal value
    success; stop candidate resolution

exact canonical `unsupported`
    this candidate declines; try the next legal candidate

raised Error / runtime error
    genuine failure; propagate immediately; DO NOT try another candidate
```

A missing candidate method is semantically equivalent to a decline, but **must be detected by lookup**. Never implement fallback by sending the method, catching `doesNotUnderstand`, and guessing that the miss belonged to the candidate itself; an implementation can legitimately raise/message-miss internally.

`unsupported` must not escape complete operator syntax. If every legal candidate is absent or returns `unsupported`, `lhs OP rhs` raises an unsupported-operands error.

Direct protocol sends may expose it:

```phalcom
a.+(b) == unsupported
```

## B.3 Candidate precedence

Normal precedence:

```text
1. direct candidate
2. reflected candidate
```

Subtype exception:

If the RHS runtime class is a **strict subclass** of the LHS runtime class and it resolves the reflected selector to an implementation defined in the strict RHS-side portion of the hierarchy (not merely inherited from the LHS class or an ancestor), try the reflected candidate first.

This allows a subtype to refine mixed base/subtype operations without being swallowed by a broad base implementation.

Add a lookup helper alongside the current hierarchy lookup, not by changing the hot existing API in place:

```rust
lookup_method_with_definer(heap, start_class, selector)
    -> Option<(ObjRef /* method */, ClassId /* defining class */)>
```

The preference predicate is approximately:

```text
rhs_class is strict subclass of lhs_class
AND
reflected method exists
AND
reflected defining class is strict subclass of lhs_class
```

If false, direct-first order applies.

Do not deduplicate candidates merely because the same underlying method object is inherited: the same implementation can observe a different receiver/argument orientation.

## B.4 VM/compiler architecture

Do **not** implement bilateral dispatch as exception-driven source desugaring.

The first implementation should add small VM bytecode primitives that let the compiler emit the fallback state machine while preserving ordinary user method calls. This is less invasive than a monolithic `InvokeBilateral` continuation frame and is easier to test.

Recommended bytecodes (names may be adjusted to local style):

```rust
/// Inspect lhs/rhs runtime classes and the RHS reflected selector.
/// Push Bool: should reflected candidate run before direct candidate?
BilateralPreferReflected(u16 /* reflected selector const */),

/// Invoke an exact fixed-arity selector if present.
/// On lookup miss: discard receiver+args and jump to `missing_offset`.
/// On hit: enter the ordinary method call path; its eventual result is left
/// on the operand stack for the following instruction.
TryInvokeExact {
    arity: u8,
    selector: u16,
    missing_offset: i32,
},

/// If TOS is the exact VM-rooted `unsupported` singleton, pop it and jump.
/// Otherwise leave TOS intact.
JumpIfUnsupported(i32),
```

Why exact lookup: bilateral operators have fixed arity and exact semantic protocol selectors. A rest-family catch-all should not silently become an arithmetic protocol implementation. Ordinary explicit sends retain normal exact/rest/dNU behavior.

Update all bytecode plumbing:

- `Bytecode` enum;
- `BYTECODE_NAMES`;
- `Bytecode::VARIANTS` / `index()` exhaustive match;
- VM dispatch handlers;
- bytecode/disassembly tests;
- any histogram/perf instrumentation keyed by opcode index.

### Compiler skeleton

Evaluate operands once, left-to-right, into scratch locals using the existing `ReserveScratchLocal` discipline when necessary.

Conceptual lowering:

```text
lhs_slot = eval lhs once
rhs_slot = eval rhs once

prefer_reflected = runtime_preference(lhs_slot, rhs_slot, reflected_selector)
if prefer_reflected:
    try reflected
    if result != unsupported: done
    try direct
    if result != unsupported: done
else:
    try direct
    if result != unsupported: done
    try reflected
    if result != unsupported: done

raise UnsupportedOperationError(OP, lhs.class, rhs.class)
```

Candidate invocation uses ordinary call frames and ordinary user bytecode/native methods. Only lookup-miss handling and fallback control are special.

## B.5 VM-rooted `unsupported`

Bilateral dispatch and built-in primitives need an identity-safe reference to the canonical `.ph` singleton without converting `Unsupported` into a native immediate.

Add a late-bound semantic-root structure to the universe/VM, conceptually:

```rust
struct SemanticRoots {
    unsupported: Value,
    ellipsis: Value,          // Task Set F
    ordering_class: ClassId,  // Task Set C validation
}
```

Populate these only after the corresponding universe `.ph` modules/classes have executed. Do not allocate replacement instances in Rust.

Requirements:

- `unsupported` must be the exact value exported by `errors/unsupported.ph`;
- trace/root any heap `ObjRef` stored here in `vm/gc.rs`;
- add `universe/invariants.rs` assertions after initialization;
- initialization failure is a bootstrap error, never silently replaced by `None`;
- do not key semantics by `==`, `_kind`, or rendered text; use exact identity.

## B.6 Unsupported-operands error

Prefer a dedicated catchable `UnsupportedOperationError < Error` if the current error/bootstrap architecture can add it cleanly. Otherwise use the existing runtime-error-to-surface-error bridge with a stable kind such as `#unsupportedOperation`.

Diagnostic must include:

- operator spelling;
- LHS runtime class;
- RHS runtime class;
- direct selector;
- reflected selector;
- source range of the operator expression.

Do not raise the `Unsupported` singleton itself. `Unsupported` is a protocol value, not the final user-facing failure.

## B.7 Built-in numeric migration

Current `primitive/number.rs` raises a type error when `promote_pair` receives a non-number. That prevents a custom RHS type from ever receiving its reflected candidate.

Refactor numeric operand classification so it can distinguish:

```text
not a Number at all        -> return canonical unsupported
valid Number operands      -> perform operation
supported domain but error -> raise existing numeric error
```

Examples:

```text
5 + CustomNumber
    Number.+(_) -> unsupported
    CustomNumber.+(from:) gets a chance

5 ~/ 0
    Number knows this operation; DivideByZero raises immediately
    no reflected fallback
```

Do not move numeric coercion into the bilateral resolver. Int/BigInt/Float promotion remains the Number protocol's own policy.

If primitive macro metadata currently declares a bilateral numeric method parameter as `Number`, widen the protocol-facing metadata to `Object` where necessary so a non-number can reach the function and receive `unsupported`; keep the documented semantic return broad enough to include `Unsupported` until union metadata is supported.

## B.8 Performance follow-up, not first-cut complexity

The initial state-machine bytecodes are deliberately factored for correctness. After benchmarks:

- add negative lookup caching to `TryInvokeExact` keyed by receiver class + selector + `world_version`;
- reuse the existing method inline-cache machinery for hits;
- consider a fused `InvokeBilateral` only if dispatch profiles justify it;
- a future fused opcode may cache both candidate method handles and preference decision for a stable `(lhs_class,rhs_class,operator)` pair;
- do not fuse before tests prove missing-method, `unsupported`, subtype priority, and error propagation separately.

---

# 5. Task Set C — `Ordering`, `compare(_)`, and `<=>`

## C.1 Temporary `Ordering` representation

Until `@data` / zero-payload `@variant` semantics provide canonical unit variants, implement Ordering as the current manual singleton pattern.

Create:

`phalcom-core/core/universe/src/scalar/ordering.ph`

Recommended definition:

```phalcom
@sealed
class Ordering {
  @get _kind

  @class _less
  @class _equal
  @class _greater
  @class _unordered

  @private
  @constructor
  create(_ kind) {
    _kind = kind
  }

  @class
  less {
    if (_less == None) {
      _less = Ordering.create(#less)
    }
    _less
  }

  @class
  equal {
    if (_equal == None) {
      _equal = Ordering.create(#equal)
    }
    _equal
  }

  @class
  greater {
    if (_greater == None) {
      _greater = Ordering.create(#greater)
    }
    _greater
  }

  @class
  unordered {
    if (_unordered == None) {
      _unordered = Ordering.create(#unordered)
    }
    _unordered
  }

  @class
  new() {
    Error.new("Ordering values cannot be constructed directly").raise()
  }

  reverse {
    if (self === Ordering.less) {
      return Ordering.greater
    }
    if (self === Ordering.greater) {
      return Ordering.less
    }
    self
  }

  ==(_ other) { self === other }

  hash { kind.hash }

  toString { toRepr }

  toRepr {
    "Ordering.\(kind.toString.trimStart(\"#\"))"
  }
}

export Ordering
```

Required behavior:

```phalcom
Ordering.less === Ordering.less      // true
Ordering.less == Ordering.less       // true
Ordering.less is Ordering            // true
Ordering.less.reverse                // Ordering.greater
Ordering.equal.reverse               // Ordering.equal
Ordering.unordered.reverse           // Ordering.unordered
System.print(Ordering.less)           // Ordering.less
```

Do not use `@variant Less()` yet. Current `@variant` expansion creates ordinary sibling classes/instances, not canonical unit values.

Expose `ordering.ph` from `scalar/package.ph` and ensure the universe/prelude makes `Ordering` available consistently with the other scalar core classes.

## C.2 Meaning of the four values

```text
Ordering.less
    lhs precedes rhs

Ordering.equal
    lhs and rhs occupy the same position under this ordering
    DOES NOT imply lhs == rhs

Ordering.greater
    lhs follows rhs

Ordering.unordered
    the comparison relation is defined but these particular values are incomparable
```

`unordered` is a successful result and stops bilateral fallback.

`unsupported` is outside `Ordering` and means the current method declines the operand pair.

## C.3 `compare(_)` protocol

A user-defined natural ordering is written once:

```phalcom
class Version {
  compare(_ other) {
    if other is! Version {
      return unsupported
    }

    let order = major <=> other.major
    if (order === Ordering.equal).not {
      return order
    }

    order = minor <=> other.minor
    if (order === Ordering.equal).not {
      return order
    }

    patch <=> other.patch
  }
}
```

Conceptual contract:

```text
compare(Object) -> Ordering | Unsupported
```

A method may raise a genuine domain error; such an error is propagated and does not cause fallback.

## C.4 `<=>` bilateral semantics

Add a longest-match token for `<=>` (`Token::Spaceship` or equivalent) before `<=` / `<` scanning.

Add a dedicated AST operator kind, but do not treat it like an ordinary symbolic method named `<=>`.

Resolution:

```text
candidate A: lhs.compare(rhs)
candidate B: rhs.compare(lhs)
```

Use the same subtype-priority rule as Task Set B, based on the RHS `compare(_)` defining class.

When the RHS candidate succeeds, validate it as an Ordering and return:

```phalcom
result.reverse
```

When the LHS candidate succeeds, return it unchanged.

If both candidates decline/miss, raise unsupported operands for `<=>`.

A successful `compare` result must be an `Ordering`; anything else is a protocol contract violation. Add a VM/compiler guard using the late-bound `ordering_class` root. A plain symbol such as `#less`, integer `-1`, Bool, or arbitrary object must fail loudly.

Do not accept `unsupported` as the final operator result.

## C.5 Numeric `compare`

For the first implementation, it is acceptable—and less bootstrap-sensitive—to define `Number.compare(_)` in `scalar/number.ph` over the existing numeric `<`, `>`, `==` primitives:

```phalcom
compare(_ other) {
  if (other is Number).not {
    return unsupported
  }

  if self < other { return Ordering.less }
  if self > other { return Ordering.greater }
  if self == other { return Ordering.equal }
  Ordering.unordered
}
```

This maps NaN-like cases, for which all three ordinary numeric relations are false, to `Ordering.unordered`, and preserves current numeric promotion behavior.

Later, a native `Number::compare(_)` may eliminate repeated numeric conversion/comparison once the semantics are stable.

## C.6 Derived ordinary relations for user objects

Do not remove existing specialized `< <= > >=` implementations in this change-set. Instead add default methods on `Object` (or the narrowest common ordering root if the project already has one) so a class that implements only `compare(_)` automatically participates in ordinary relational operators:

```phalcom
<(_ other) {
  (self <=> other) === Ordering.less
}

<=(_ other) {
  const order = self <=> other
  (order === Ordering.less) or (order === Ordering.equal)
}

>(_ other) {
  (self <=> other) === Ordering.greater
}

>=(_ other) {
  const order = self <=> other
  (order === Ordering.greater) or (order === Ordering.equal)
}
```

`Ordering.unordered` makes all four predicates false.

Existing Number/native overrides remain fast and compatible. A later dedicated ordering cleanup can decide whether independent relational overrides should be deprecated; do not make that unrelated compatibility break here.

---

# 6. Task Set D — chained relational expressions

Adopt Python-style chained relation semantics.

```phalcom
0 <= x < 10
```

is semantically equivalent to:

```phalcom
0 <= x and x < 10
```

except that `x` is evaluated exactly once.

## D.1 AST

Do not encode a chain as left-associated nested `BinaryExpr`; that loses the shared-middle-operand rule.

Add an explicit node, e.g.:

```rust
ComparisonChainExpr {
    operands: Vec<Expr>,        // length >= 3 for a true chain
    operators: Vec<RelationOp>, // len = operands.len - 1
    range: SourceRange,
}
```

A single relational operation may remain an ordinary `BinaryExpr`.

`RelationOp` should be able to represent all already-landed relation families plus:

- `<`, `<=`, `>`, `>=`;
- `==`, `!=`;
- `===`;
- existing `is` / `is!` / particle-combined forms;
- existing `in` / `not in` / `is in` families;
- `matches`;
- `understands`.

`<=>` is **not** a chain relation because it returns `Ordering`, not Bool.

Reject an unparenthesized construction such as:

```phalcom
a <=> b < c
```

rather than silently interpreting an `Ordering` as the next relational operand. Require explicit grouping if the user intentionally compares the `<=>` result:

```phalcom
(a <=> b) === Ordering.less
```

## D.2 Evaluation order

For:

```phalcom
a() < b() <= c() === d()
```

required order is:

1. evaluate `a()` once;
2. evaluate `b()` once;
3. evaluate `a < b`;
4. if false, return false without evaluating `c` or `d`;
5. evaluate `c()` once;
6. evaluate `b <= c` using the saved `b`;
7. if false, return false without evaluating `d`;
8. evaluate `d()` once;
9. evaluate `c === d`;
10. return that Bool.

Use compiler scratch locals for middle operands; do not duplicate their AST or reevaluate them.

Every relation result must follow its existing Bool contract. Preserve current Bool guard/diagnostic behavior rather than adding truthiness.

## D.3 Contextual `matches` / `understands`

Do not globally reserve these words in the lexer. Parse `Token::Identifier("matches")` and `Token::Identifier("understands")` contextually only at comparison-operator position.

This preserves ordinary method/property names:

```phalcom
pattern.matches(value)
object.understands(selector)
```

and avoids unnecessary source compatibility loss.

The LSP semantic-token layer should still classify their infix occurrences as operators/keywords according to the project's token taxonomy.

---

# 7. Task Set E — `matches`

## E.1 Operator ownership

Ratified lowering:

```phalcom
candidate matches pattern
```

means:

```phalcom
pattern.matches(candidate)
```

The RHS owns pattern semantics. Do not send `matches` to the candidate.

The method is ordinary and overridable.

## E.2 Minimal pre-pattern-language behavior

Add a literal-pattern fallback to `Object`:

```phalcom
matches(_ candidate) {
  self == candidate
}
```

Thus ordinary objects act as equality patterns.

This is intentionally small. Do not attempt to encode future structural pattern bindings through this value-level method.

## E.3 Selector-pattern specialization

Current repository facts:

- exact `#foo(_)` selector literals lower to interned `Symbol` values;
- structural selector patterns such as `#foo...` already materialize `heap::SelectorPatternObject`;
- `phalcom-common::selector::SelectorPattern::matches` already implements the rich structural predicate.

Give runtime selector-pattern objects a proper surface class rather than leaving them indistinguishable from generic Object dispatch.

Recommended changes:

1. bootstrap a `SelectorPattern` class in `universe/core_classes.rs` alongside other native-representation classes;
2. map `Object::SelectorPattern(_)` to `selector_pattern_class` in `Value::class`;
3. add `core/universe/src/reflection/selector-pattern.ph` as the surface reopen/stub and expose it from `reflection/package.ph`;
4. install native `SelectorPattern::matches(_)` in the primitive registration path.

Native behavior:

```text
receiver must be Object::SelectorPattern
candidate Symbol -> decode as exact selector identity -> rich_pattern.matches(selector)
non-selector/non-decodable candidate -> false
```

Required example:

```phalcom
#+(_) matches #+...       // true
#+(_, _) matches #+...    // true
#-(_) matches #+...       // false
```

Reuse `phalcom-common/src/selector.rs`; do not write a second punctuation matcher in the primitive.

Keep `#name...` / internal-gap selector syntax exactly as the already-landed selector-pattern parser defines it. This unit adds the operator/method bridge, not another selector-pattern grammar.

---

# 8. Task Set F — `Ellipsis` and expression `...`

## F.1 Surface object

Create a sealed singleton class using the same current pattern as `Unsupported`.

Suggested location:

`phalcom-core/core/universe/src/object/ellipsis.ph`

Suggested implementation:

```phalcom
@sealed
class Ellipsis {
  @class _instance

  @class
  instance {
    if (_instance == None) {
      _instance = Ellipsis.create()
    }
    _instance
  }

  @private
  @constructor
  create() {}

  @class
  new() {
    Error.new("There can be only one Ellipsis instance").raise()
  }

  @class
  call() { Ellipsis.instance }

  ==(_ other) { self === other }
  hash { #ellipsis.hash }
  toString { "..." }
  toRepr { "..." }
}

const ellipsis = Ellipsis.instance

export Ellipsis
export ellipsis
```

Expose the module from `object/package.ph`.

## F.2 Parser context

`Token::DotDotDot` already exists. Add ordinary primary-expression handling:

```phalcom
let marker = ...
[1, 2, ...]
foo(...)
```

only where `...` is not already claimed by a higher-priority grammar production such as selector-pattern syntax.

Add `Expr::Ellipsis` or an equivalent explicit AST node; do not lower it to an identifier that user code could shadow.

Compile the expression to the canonical VM-rooted `ellipsis` singleton captured in `SemanticRoots` after universe initialization.

Required:

```phalcom
... === Ellipsis.instance
```

## F.3 Pattern metasyntax separation

The expression value and pattern grammar are deliberately distinct.

```text
expression context: ...  => Ellipsis singleton
selector pattern:  #foo... => syntax gap, not Ellipsis
future structural pattern: [head, ..., tail] => pattern metasyntax, not Ellipsis
```

Document and test this parser-context distinction now so the runtime object does not constrain future pattern syntax.

---

# 9. Task Set G — `understands`

Ratified surface:

```phalcom
object understands selector
```

lowers to:

```phalcom
object.understands(selector)
```

## G.1 Default implementation

Install a native default `Object::understands(_)` because lookup-without-send is VM knowledge.

It must:

1. accept the exact selector representation used by current `#selector` literals (`Symbol`), and optionally an actual `Selector` instance if the reflection class is already constructible/useful;
2. perform exact method lookup;
3. if exact lookup misses, check whether an existing rest-family method would accept that exact call shape using `vm/send.rs::lookup_rest_method`;
4. return Bool;
5. never call the target method;
6. never route a miss through `doesNotUnderstand`;
7. never treat the existence of the universal DNU path as understanding every selector.

An invalid RHS type should raise the project's normal argument/type diagnostic rather than silently treating arbitrary values as selector names.

The method remains overridable so a future explicit interception/proxy class can truthfully extend its advertised message capability. When message interception lands, the default semantics should be revisited so declared interception participates without actually sending the message.

## G.2 Examples

```phalcom
console understands #print(_)
object understands selector
```

A concrete/inherited method or accepting rest-family method => true.

Only eventual DNU => false.

---

# 10. Task Set H — general pattern AST and two match modes

The existing `Pattern` node was intentionally designed for reuse. Extend it rather than creating separate tuple destructuring, Option unpacking, `if let` patterns, and loop patterns.

## H.1 Pattern forms required by this change-set

Retain:

```text
Name
Tuple
List (+ trailing *rest)
```

Add:

```rust
Variant {
    constructor: StaticSymbolRef-or-pattern-name,
    arguments: Vec<Pattern>,
    range: SourceRange,
}

Record {
    entries: Vec<RecordPatternEntry>,
    range: SourceRange,
}

Map {
    entries: Vec<MapPatternEntry>,
    range: SourceRange,
}
```

Use the existing concrete aggregate delimiters in pattern position:

```phalcom
// tuple
let (x, y) = pair

// list
let [head, *tail] = values

// record — mirrors current #{...} record literal syntax
let #{name: name, age: age} = person

// map — mirrors current { key: value } map syntax
let {#name: name, #age: age} = dictionary

// sealed/Option/Result variant
if let Some(value) = maybeValue { ... }
if let Ok(value) = operation() { ... }
if let Err(error) = operation() { ... }
if let Circle(radius) = shape { ... }
```

Do not add shorthand record fields in this unit unless the corresponding literal/product syntax already has an identical shorthand. Explicit `label: pattern` is sufficient.

For initial map patterns, restrict keys to stable literal/static keys accepted without arbitrary user evaluation (at minimum Symbol/String/Int literals). Dynamic key expressions in pattern position are deferred.

Record/map patterns are **open** with respect to extra fields/keys: every specified field/key must exist and match, but unspecified bindings do not cause failure. This aligns structural record matching with future row-style typing and makes map destructuring useful without requiring an immediate rest-map syntax.

## H.2 Variant pattern disambiguation

A variant pattern is spelled with parentheses, including zero-payload variants:

```phalcom
Some(value)
None()
Ok(value)
Err(error)
UnitVariant()
```

Do not make bare `None` contextually ambiguous between "bind a local named None" and "unit variant" in the first implementation.

A variant pattern matches the exact variant class, not arbitrary subclasses. This preserves disjoint sealed variants.

For `@sealed` + `@variant`, retain enough generated metadata to map positional pattern arguments to the variant declaration's label order. The current `@variant` expander in `compiler/attributes.rs` temporarily has the complete variant list/labels and currently discards the closed-set metadata after generating sibling classes and `match(...)`; stop throwing away the information needed by the compiler/LSP.

At minimum retain:

```text
sealed root
variant class
variant ordinal
ordered payload labels
family variant list
```

Store it in the compiler/module semantic metadata and expose it to the LSP. Runtime class metadata is preferred if cross-module compiled code needs to match variants without source AST availability.

`Some`/`None` and `Result`/`Ok`/`Err` should participate through the same abstract pattern evaluator, even if their bootstrap representations need small adapters.

## H.3 Two evaluation modes

The same pattern tree is interpreted in two modes:

```rust
enum PatternMode {
    Irrefutable, // mismatch raises
    Refutable,   // mismatch branches false
}
```

Used as follows:

| Construct | Mode |
|---|---|
| `let pattern = expr` | Irrefutable |
| `const pattern = expr` | Irrefutable |
| `for pattern in iterable` | Irrefutable per yielded element |
| `if let pattern = expr` | Refutable |
| `while let pattern = expr` | Refutable per iteration attempt |

A plain `for Some(x) in values` is **not** a filter. A `None` element causes an irrefutable pattern mismatch error.

## H.4 Atomic binding

Refutable matching must not leak partial bindings.

Bad implementation:

```text
bind x
later nested pattern fails
branch false with x partially installed
```

Required implementation:

1. evaluate scrutinee once into a scratch local;
2. walk/test pattern and store candidate leaf values in compiler-owned scratch slots;
3. on any mismatch, branch to failure with no user-visible bindings committed;
4. only after the entire pattern succeeds, establish the branch/body user bindings.

Use the same principle for nested tuple/list/record/map/variant patterns.

For irrefutable mode, mismatch raises; it may share the same test plan and commit sequence for consistency.

## H.5 Refactor `compiler/lib/patterns.rs`

The current `compile_pattern_bind_top_of_stack` declares leaf locals as it recursively walks. Generalized loops and refutable matching need declaration and assignment to be separable.

Refactor toward a plan such as:

```rust
PatternBindingPlan {
    leaves: Vec<PatternLeafBinding>,
    // source ranges / slots / mutability
}

fn declare_pattern_bindings(...)
fn compile_pattern_match_to_scratch(..., mode, failure_target)
fn commit_pattern_bindings(...)
fn assign_pattern_to_existing_slots(...) // loop rebind path
```

Keep existing arity checks and `at(_)` reads as reusable helpers.

For record/map matching, use their stable lookup APIs; absence is match failure in refutable mode and a pattern-mismatch raise in irrefutable mode.

Introduce a dedicated `PatternMatchError < Error` if it can be added without bootstrapping friction; otherwise retain current clean `Error.new(message).raise()` behavior but standardize message kinds/spans.

## H.6 Duplicate names

Reject duplicate leaf bindings inside one pattern, including across nested pieces and across parallel `for` lanes:

```phalcom
let (x, x) = pair                 // error
for x in xs, x in ys {}           // error
for (x, y) in pairs, y in ys {}   // error
```

Report both the duplicate and first-binding source spans.

---

# 11. Task Set I — `if let` and `while let`

## I.1 Grammar

Add explicit parser paths before ordinary `if` / `while` condition parsing:

```text
if-let:
    "if" "let" pattern "=" expression block ["else" ...]

while-let:
    "while" "let" pattern "=" expression block
```

Examples:

```phalcom
if let Some(user) = findUser(id) {
  use(user)
}

if let Ok(value) = operation() {
  consume(value)
} else {
  recover()
}

while let Some(order) = orderNumbers.next() {
  fulfill(order)
}

while let Some((key, value)) = iterator.next() {
  consume(key, value)
}
```

## I.2 AST

Do not desugar these into ordinary method calls at parse time; bindings and failure edges are compiler-visible semantics.

Add explicit nodes, e.g.:

```rust
IfLetExpr {
    pattern: Pattern,
    value: Expr,
    then_body: BlockExpr-or-statements,
    else_body: Option<...>,
    range: SourceRange,
}

WhileLetStatement-or-Expr {
    pattern: Pattern,
    value: Expr,
    body: Vec<Statement>,
    range: SourceRange,
}
```

Choose statement/expression placement consistent with current `if`/`while` value semantics. The crucial rule is: **do not invent a different branch-result policy**. If ordinary one-armed/two-armed `if` has Some/None lifting or another established result shape, `if let` must mirror it exactly.

## I.3 Semantics

`if let`:

1. evaluate RHS once;
2. refutably match pattern;
3. on success, commit bindings scoped to then-body;
4. on failure, execute else if present / use ordinary one-armed-if result semantics;
5. bindings do not exist after the construct.

`while let`:

```text
loop:
    evaluate RHS once for this attempt
    refutably match
    failure -> exit
    success -> commit fresh per-iteration bindings
    execute body
    continue -> next attempt
    break -> exit
```

RHS is evaluated once **per attempt**, not once for the entire loop.

Pattern-bound closures must get per-iteration snapshots just like the current `for` binding. Reuse the `CloseUpvalue` technique from `compiler/lib/loops.rs` for every captured pattern leaf before the next iteration rebind.

`break`/`continue` must use the existing `LoopContext` machinery and behave exactly like ordinary `while`/`for`.

---

# 12. Task Set J — remove mandatory control-flow wrapper parentheses

Canonical forms:

```phalcom
if condition {
  ...
}

while condition {
  ...
}

for item in items {
  ...
}
```

Parentheses remain legal when they are themselves an expression/pattern:

```phalcom
if (a or b) { ... }
for (x, y) in pairs { ... }
```

The outer `for (x in xs) {}` compatibility wrapper is not part of the new grammar. Update current source/tests/docs that still use it.

Parser work:

- `parse_if`: parse condition directly until the following brace block; grouped `(condition)` naturally continues to work through expression grammar;
- `parse_while`: same;
- `parse_for`: entirely replace old parenthesized-header assumptions with lane parsing described next.

Do not mechanically strip all parentheses from source: `(x, y)` in a `for` header is now a tuple pattern and is load-bearing.

---

# 13. Task Set K — Option C `for` lanes, indexing, and zip

## K.1 AST

Replace the single-name `ForStatement` shape with explicit lanes:

```rust
pub struct ForLane {
    pub pattern: Pattern,
    pub index: Option<ForIndexBinding>,
    pub iter: Expr,
    pub range: SourceRange,
}

pub struct ForIndexBinding {
    pub name: String,
    pub range: SourceRange,
}

pub struct ForStatement {
    pub lanes: Vec<ForLane>, // non-empty
    pub body: Vec<Statement>,
    pub range: SourceRange,
}
```

Do not keep a parallel legacy `binding: String` path in compiler semantics. Migrate all consumers.

## K.2 Grammar

```text
for-statement:
    "for" for-lane ("," for-lane)* block

for-lane:
    pattern ["@"-token-as-`at` keyword?] "in" expression
```

Surface spelling is the existing word/punctuation token sequence for `at`:

```phalcom
for item at index in items {}
```

The lexer already has `Token::At` for `@`; **do not confuse the decorator punctuation token with the word `at`**. If `at` is currently lexed as an identifier, parse it contextually as `Identifier("at")`. Do not globally reserve it unless current language policy already has an `At` keyword token distinct from `@`.

Canonical examples:

```phalcom
for item in items {}
for item at i in items {}
for (key, value) in entries {}
for (key, value) at i in entries {}
for user in users, card in cards {}
for user at i in users, card in cards {}
for user at i in users, card at j in cards {}
for (name, age) in people, score in scores {}
```

Only **top-level** commas in the `for` header delimit lanes. Commas nested in tuple patterns, calls, tuple literals, lists, etc. belong to those constructs.

## K.3 Lane semantics

A lane is always understandable locally:

```text
pattern [at ordinal] in iterable
```

The `at` binding is:

- zero-based;
- an iteration ordinal;
- independent of the iterable's internal cursor;
- not a Map key;
- not a Range value;
- incremented exactly once after each successfully formed iteration step.

Alternative starts belong to explicit iterable APIs, not more loop grammar.

## K.4 Parallel semantics

Two or more lanes advance in lockstep and are semantically equivalent to zipping the iterables and destructuring the produced tuple:

```phalcom
for x in xs, y in ys {
  body
}
```

is equivalent to:

```phalcom
for (x, y) in (xs, ys).zipped {
  body
}
```

N lanes generalize naturally.

Evaluate every lane's iterable expression exactly once, **left-to-right**, before beginning traversal.

Nested loops remain nested/Cartesian/dependent:

```phalcom
for x in xs {
  for y in ys {
    ...
  }
}
```

Do not add compact Cartesian syntax.

## K.5 Strict zip

Use strict zip as the default semantics for `.zipped` and comma-separated loop lanes.

For each attempted step, advance/probe every lane exactly once. Outcomes:

```text
all lanes produce next cursors -> yield tuple/body iteration
all lanes end together         -> normal end
some lanes end, others continue -> ZipLengthMismatchError
```

Do not silently truncate to the shortest source.

If a future API needs shortest/longest behavior, add explicit value-level operations (`zippedShortest`, `zippedLongest`, or final names chosen later). Do not alter comma syntax.

Document that strict mismatch detection may probe another stateful source for the attempted mismatching step; after mismatch the zip raises and no body value is yielded.

## K.6 Compiler lowering and optimization

Do **not** implement comma-loop sugar by allocating an actual `ZippedIterable` wrapper and tuple on every iteration. Preserve explicit `.zipped` as the semantic model, but compile multi-lane `for` directly.

Generalized `compile_for` should:

1. begin one enclosing loop scope;
2. evaluate/store each iterable once;
3. initialize one cursor per lane;
4. initialize one ordinal counter for each `at` lane (the compiler may share one physical counter for equal zero-based lockstep ordinals, but source bindings remain independent);
5. on every attempted step, determine all lane cursor states and enforce strict all-end/all-live;
6. fetch each lane value with `iteratorValue(cursor)`;
7. irrefutably assign each value to that lane's predeclared pattern binding plan;
8. assign ordinal bindings;
9. execute body;
10. close captured pattern/index upvalues before rebinding;
11. advance all cursors left-to-right;
12. increment ordinals;
13. back-edge;
14. patch existing `break`/`continue` targets so `continue` lands at the common step/close/advance point.

Do not introduce a body block call. Maintain the current fiber/yield guarantee.

## K.7 Pattern locals in loops

Current `compile_for` declares one immutable loop binding once, then rebinds its slot internally and closes captured upvalues per iteration.

Generalize this to **every leaf name** of every lane pattern plus every `at` binding.

Create the pattern slots before the loop, mark them immutable to user assignment, and use internal slot writes for each successful iteration. After the body, emit `CloseUpvalue` for every captured leaf before the next rebind.

This is essential for:

```phalcom
for (x, y) in pairs {
  closures.append(|| { (x, y) })
}
```

Each closure must retain that iteration's pair, not the final loop values.

---

# 14. Task Set L — explicit `.indexed` and `.zipped`

These are not merely compiler concepts; they are ordinary first-class iterable transformations.

## L.1 `.indexed`

Add to `Iterable`:

```phalcom
indexed { IndexedIterable.new(self) }
```

`IndexedIterable` must carry source traversal state and ordinal independently because the source's cursor is not guaranteed to be an integer.

Conceptual cursor:

```text
(sourceCursor, ordinal)
```

Semantics:

```text
iterate(None):
    c = source.iterate(None)
    c == None ? None : (c, 0)

iterate((c, i)):
    next = source.iterate(c)
    next == None ? None : (next, i + 1)

iteratorValue((c, i)):
    (i, source.iteratorValue(c))
```

Thus:

```phalcom
for item at i in items {}
```

is semantically equivalent to:

```phalcom
for (i, item) in items.indexed {}
```

## L.2 `.zipped`

Add `.zipped` to `Tuple`, not to arbitrary `Iterable`:

```phalcom
(xs, ys).zipped
(xs, ys, zs).zipped
```

Return a lazy `ZippedIterable` implementing the ordinary cursor protocol.

The wrapper stores the Tuple of source iterables. Its cursor is a Tuple of source cursors. `iteratorValue` returns a Tuple of source values in lane order.

Use existing tuple-pack/positional expansion machinery when dynamically assembling cursor/value tuples. The current compiler/runtime already has `BuildTuple`, `FinishTuplePack`, and positional expansion support; do not add a second tuple representation for zip.

Require at least two source lanes for `.zipped` in the first implementation; reject an empty source tuple and avoid defining a surprising infinite zip-of-zero-lanes behavior.

Strict length behavior must exactly match K.5.

## L.3 Explicit/sugar equivalence tests

For every loop sugar test, include an explicit operation twin:

```phalcom
for x in xs, y in ys { ... }
for (x, y) in (xs, ys).zipped { ... }
```

and:

```phalcom
for x at i in xs { ... }
for (i, x) in xs.indexed { ... }
```

They must produce the same user-visible sequence of bindings/results/errors.

---

# 15. Parser implementation details

## 15.1 Longest operator scanning

`lexer.rs` operator scanner ordering must ensure:

```text
<=> before <= before <
=== before == before =
```

Keep `...` handling unchanged except for permitting it in primary expression context.

## 15.2 Comparison precedence

Put `<=>` at the comparison/relational precedence level but exclude it from chain aggregation. `matches` and `understands` belong to the relation level.

Preserve existing precedence of `and`/`or`, coalescing, ranges, arithmetic, etc.

## 15.3 Contextual words

Prefer contextual parsing for:

```text
matches
understands
at (inside a for lane)
```

This allows all three to remain ordinary method names elsewhere.

## 15.4 Pattern parser

Extend `Parser::parse_pattern` with ordered lookahead:

1. tuple `(`;
2. list `[`;
3. record `#{`;
4. map `{` **only when in pattern-target grammar**, not block grammar;
5. identifier followed by `(` => Variant pattern;
6. bare identifier => Name binding.

Keep expression and pattern parsing separate; never parse a RHS expression and reinterpret it as a pattern afterward.

---

# 16. Compiler implementation details

## 16.1 Binary expression split

Refactor the current `Expr::Binary` compiler branch in `compiler/lib/expr.rs` into explicit semantic classes:

```text
lazy logical
ordinary direct-send binary
bilateral cooperative
intrinsic sameness
right-owned relation
left-owned relation
spaceship comparison
```

Do not grow one giant `match op { ... }` with repeated selector strings. Centralize operator descriptors.

Suggested descriptor:

```rust
struct BilateralOpSpec {
    spelling: &'static str,
    direct_selector: &'static str,
    reflected_selector: &'static str,
    reflected_result: ReflectedResult,
}

enum ReflectedResult {
    Identity,
    ReverseOrdering,
}
```

`<=>` may use a separate descriptor because its direct/reflected selector is `compare(_)` rather than the token spelling.

## 16.2 Single-evaluation scratch discipline

Use the existing `reserve_pack_scratch` / scratch-local infrastructure when a complex expression occurs under an already-live operand stack. Do not introduce ad-hoc `$tmp` locals that break nested expression stack windows.

This applies to:

- bilateral operands;
- comparison-chain middle operands;
- `if let` scrutinees;
- `while let` per-attempt scrutinees;
- multi-lane iterable expressions;
- explicit pattern-match candidate captures.

## 16.3 Source ranges

Diagnostics for generated operator/pattern/zip branches should use the smallest useful source range:

- unsupported operation -> operator token/expression range;
- invalid compare return -> `compare` operator token / candidate call origin;
- pattern mismatch -> pattern range;
- strict zip mismatch -> comma-separated `for` header or `.zipped` call range;
- reserved `===` override -> method selector range.

Do not report synthetic local/compiler-helper ranges to users.

---

# 17. Runtime and bootstrap implementation details

## 17.1 Semantic roots

Add late-bound roots only for objects that are semantically canonical but defined in `.ph`:

```text
unsupported
ellipsis
Ordering class identity
```

Do not duplicate them as native Values/classes unless representation forces it.

Populate them after universe source load, assert them, trace them.

## 17.2 SelectorPattern class

Because `Object::SelectorPattern` is a native heap representation, it needs a stable class mapping for method dispatch once it has a specialized `matches(_)`. Add the class ID to `CoreClasses`, native-representation allocation restrictions if appropriate, `Value::class`, primitive registration, invariants, and docs.

## 17.3 No new native representation for Ordering/Ellipsis

Both remain ordinary `InstanceObject` values created by their `.ph` singleton classes. VM roots are references to those existing values/classes, not alternate native representations.

---

# 18. LSP obligations

This change-set is incomplete until `phalcom-lsp` models the same semantics.

## 18.1 AST/source indexing

Update all exhaustive AST visitors in at least:

- `phalcom-lsp/src/index.rs`;
- `semantic/analyzer.rs`;
- `semantic/occurrence.rs`;
- `semantic/scope.rs`;
- `semantic/flow.rs`;
- `semantic/surface.rs`;
- `semantic/infer.rs`;
- `semantic/dispatch.rs`.

## 18.2 Pattern definitions and scopes

Every pattern leaf is a definition occurrence.

- `let`/`const`: binding scope follows existing binding rules;
- `for`: every lane pattern and `at` name is in scope only in the body;
- `if let`: bindings exist only on the success branch;
- `while let`: bindings exist only in the body and are per-iteration;
- failure/else paths must not see them;
- duplicate binding diagnostics should align with compiler diagnostics.

## 18.3 Flow narrowing

When variant identity is known:

```phalcom
if let Some(user) = maybeUser { ... }
```

inside the body, the scrutinee can be narrowed to `Some` and `user` to the payload inference available from the variant declaration/native Option metadata.

Likewise `Ok`/`Err` and user `@variant` classes.

Record/map structural matches can add presence facts for the selected labels/keys.

## 18.4 Operator inference

- `===` => Bool, intrinsic, no override search;
- `matches` => Bool; resolve method on RHS type;
- `understands` => Bool; resolve method on LHS type;
- `<=>` => Ordering after bilateral `compare` candidate analysis;
- bilateral arithmetic => union/join of direct and reflected successful return facts, excluding `Unsupported` as an operator result;
- chained comparison => Bool;
- `< <= > >=` on a class with `compare(_)` should discover inherited Object defaults.

Dispatch/hover for a bilateral operator should surface both possible protocol selectors. If one candidate is statically impossible, present the remaining one. Go-to-definition may choose the statically preferred candidate when known, but hover should explain cooperative fallback.

## 18.5 Semantic tokens

Contextual `matches`, `understands`, and `at` in a for-lane should be highlighted as syntax/operator words while the same spelling in:

```phalcom
obj.matches(x)
obj.understands(sel)
```

remains a method/property token.

---

# 19. Required test matrix

Do not treat parser snapshots alone as sufficient. Add compiler/runtime/LSP tests.

## 19.1 `===`

Positive:

```phalcom
1 === 1
1 === 1.0            // false
None === None
unsupported === unsupported
Ordering.less === Ordering.less
```

Heap identity:

```phalcom
let a = "x" + ""
let b = "x" + ""
System.print(a == b)   // true
System.print(a === b)  // false unless explicitly the same handle
System.print(a === a)  // true
```

Negative: user override of `===(_)` rejected.

## 19.2 Bilateral dispatch

Create small user classes covering:

1. direct candidate succeeds;
2. direct returns unsupported -> reflected succeeds;
3. direct missing -> reflected succeeds;
4. direct raises -> reflected is **not** called;
5. both missing -> unsupported-operation error;
6. both return unsupported -> same error;
7. reflected method for subtraction/division preserves original lhs/rhs orientation;
8. RHS strict subtype reflected override gets priority;
9. RHS only inherits reflected method from LHS hierarchy -> normal direct priority;
10. candidate called at most once;
11. direct raw message returns `unsupported` instead of triggering operator fallback.

Numeric:

- Int + Float existing behavior unchanged;
- Int + custom reflected numeric can now cooperate;
- divide-by-zero still raises immediately;
- non-number direct primitive returns `unsupported`, not TypeError.

## 19.3 Ordering / `<=>`

- singleton identity and representation;
- `reverse` table;
- user `Version.compare`;
- LHS compare success;
- LHS unsupported / RHS compare success with reverse;
- subtype priority;
- both unsupported;
- `Ordering.unordered` stops fallback;
- invalid compare result (`#less`, `-1`, Bool) raises protocol error;
- `Ordering.equal` does not imply `==` using a class whose comparator ignores an identity field.

## 19.4 Chained relations

Instrument side effects/counters:

```phalcom
low() < middle() <= high()
```

Assert `middle()` once and later operands skipped after false.

Cover mixed chains with `===`, existing `is`/`in` families, `matches`, `understands`.

Reject `<=>` inside an unparenthesized chain.

## 19.5 `matches`

- literal equality fallback;
- user-defined pattern object override;
- `#+(_) matches #+...` true;
- arity/suffix selector-pattern cases already supported by common matcher;
- wrong selector base false;
- non-selector candidate against SelectorPattern false.

## 19.6 Ellipsis

- `... === Ellipsis.instance`;
- printable/repr `...`;
- construction blocked;
- usable in list/tuple/call values;
- selector-pattern `...` remains pattern gap, not expression singleton.

## 19.7 `understands`

- own method;
- inherited method;
- exact missing method false;
- DNU availability alone false;
- accepting rest-family true for matching selector shape;
- no target method side effect occurs;
- invalid RHS diagnostic.

## 19.8 Patterns / `if let` / `while let`

Tuple/list existing cases remain green plus:

- nested tuple/list;
- record field subset match;
- map required-key subset match;
- missing record/map field refutable failure;
- missing field irrefutable raise;
- `Some`, `None()`, `Ok`, `Err`;
- user `@sealed` + `@variant` payload destructuring;
- nested variants `Some(Ok(x))`;
- RHS evaluated once;
- failure creates no visible partial bindings;
- binding scope ends at body;
- `while let` terminates on first mismatch;
- break/continue;
- closure captures distinct per-iteration values.

## 19.9 `for` Option C

Basic:

```phalcom
for x in xs {}
for (x, y) in pairs {}
for x at i in xs {}
```

Parallel:

```phalcom
for x in xs, y in ys {}
for x at i in xs, y in ys {}
for x at i in xs, y at j in ys {}
for (name, age) in people, score in scores {}
```

Equivalence with `.indexed` / `.zipped`.

Strict zip:

- equal finite lengths;
- both empty;
- mismatch on first step;
- mismatch after several steps;
- three-lane mismatch;
- break before potential later mismatch does not probe future steps;
- continue advances all lanes exactly once.

Single evaluation:

- every iterable factory called once, left-to-right.

Closure capture:

- all pattern leaves and index bindings snapshot per iteration.

Fiber regression:

- yielding from body remains valid; disassembly/taken path contains no body `block_call` introduced by zip/destructuring sugar.

---

# 20. Documentation changes

Update or add normative docs in the same change train. At minimum reconcile:

- `docs/spec/current/iteration.md` — new header, pattern lanes, `at`, comma zip, strictness, `.indexed`, `.zipped`;
- control-flow documentation — parentheses no longer required; `if let`, `while let`;
- `docs/adr/accepted/0046-destructuring-bindings.md` — no longer only Name/Tuple/List; document shared irrefutable/refutable engine or supersede with a new ADR;
- `docs/spec/current/selectors.md` — selector pattern `matches` bridge and `understands` selector argument semantics;
- lexical structure — `===`, `<=>`, expression `...`, contextual relation words;
- object/equality docs — distinguish `==` from `===`;
- new comparison/ordering spec — `Ordering`, `compare`, bilateral protocol, unsupported contract, chained relations;
- `unsupported` documentation — explicitly state "return to decline; final operator raises only after resolution is exhausted";
- LSP semantic docs — bilateral candidate resolution and pattern scopes/narrowing.

Do not rewrite the already-landed `is` / `in` particle family in this unit beyond documenting participation in chains.

---

# 21. Implementation order / dependency graph

Use this order to keep intermediate commits buildable and reviewable.

## Phase 1 — AST/token foundations

1. add `===`, `<=>` tokens and lexer coverage;
2. add AST operator/node shapes (`Same`, `Compare`, comparison chain, new patterns, new for lanes, if-let/while-let, Ellipsis);
3. update AST visitors/tests until `phalcom-ast` is green.

## Phase 2 — `===`

1. `Value::same_as`;
2. native method/interface;
3. reserved semantic selector enforcement;
4. compiler lowering;
5. tests/LSP intrinsic typing.

## Phase 3 — bilateral dispatch substrate

1. hierarchy lookup-with-definer helper;
2. semantic roots for `unsupported`;
3. bytecodes + VM handlers;
4. compiler bilateral lowering helper;
5. unsupported operation diagnostic;
6. exhaustive bilateral unit/runtime tests.

Do not add `<=>` until this substrate is independently green.

## Phase 4 — Ordering / `<=>`

1. `ordering.ph` + exports;
2. late-bound Ordering class root;
3. `<=>` parse/lowering/validation/reverse;
4. `Number.compare`;
5. Object default `< <= > >=`;
6. tests/LSP.

## Phase 5 — relation surface

1. comparison-chain AST/parser/compiler;
2. contextual `matches`/`understands`;
3. SelectorPattern surface class/native matcher;
4. Object matches fallback;
5. Object understands primitive;
6. Ellipsis singleton + expression lowering;
7. tests/LSP.

## Phase 6 — generalized patterns

1. Pattern AST extensions;
2. retain variant metadata;
3. refactor declaration/test/commit pattern engine;
4. record/map/variant matching;
5. `if let`;
6. `while let`;
7. tests/LSP flow narrowing.

## Phase 7 — for Option C and iterable views

1. no-wrapper `for` parser;
2. ForLane AST migration;
3. generalized `compile_for` with pattern plans / index locals;
4. strict multi-lane direct lowering;
5. `.indexed` wrapper;
6. `.zipped` wrapper;
7. migrate current `for (...)` source/tests;
8. closure/fiber/strictness/LSP tests.

## Phase 8 — docs/performance/full gate

1. normative docs/ADR updates;
2. benchmark operator hot paths and multi-lane loops;
3. only then add cache/fusion optimizations;
4. full workspace verification.

---

# 22. Optimization requirements and preclusions

## Required now

- single evaluation of all syntax operands/scrutinees/iterables;
- no exception-driven method-missing fallback;
- no zip wrapper allocation for comma-loop sugar;
- no body block call in `for`/`while let` compiler loops;
- per-iteration upvalue closure for all rebound pattern leaves;
- reuse existing selector encoding/common matcher;
- reuse existing cursor protocol;
- exact identity check for `unsupported`;
- preserve current Number coercion policy inside Number, not resolver.

## Profile before adding

- fused bilateral opcode;
- two-class bilateral PIC;
- native `Number.compare`;
- specialized two-lane zip opcode;
- no-allocation indexed cursor representation;
- specialized pattern matching opcodes.

## Explicitly precluded

- swallowing arbitrary errors and trying the other operand;
- interpreting any `Unsupported`-looking symbol/string/kind as protocol decline;
- returning `-1/0/1` or symbols from `<=>`;
- conflating `Ordering.unordered` with `unsupported`;
- making `===` overridable;
- silently filtering mismatched `for` patterns;
- silently truncating zip to shortest;
- treating internal iterator cursors as `at` ordinals;
- evaluating a comparison-chain middle expression twice;
- lowering `matches` to `candidate.matches(pattern)`;
- treating DNU as universal `understands`;
- interpreting expression `...` as a pattern wildcard;
- implementing current zero-payload `@variant` objects as Ordering values without separately changing variant semantics.

---

# 23. Acceptance gate

The work is complete only when all of the following are true:

1. `cargo test --workspace` (or repository canonical equivalent) passes.
2. `cargo clippy --workspace --all-targets` passes under repository policy.
3. `cargo doc --workspace --no-deps` passes if it is part of current gate.
4. `./scripts/verify.sh` or current canonical repository verification script passes.
5. `phalcom-ast` lexer/parser snapshot tests cover every new syntax token/path.
6. `phalcom-core/tests/lang/MANIFEST.md` includes dedicated corpus coverage for comparison/bilateral, patterns, and iteration additions.
7. Runtime tests prove direct/reflected ordering, subtype precedence, unsupported/error distinction, and no duplicate candidate calls.
8. Runtime tests prove `Ordering` singleton semantics and invalid compare return rejection.
9. Runtime tests prove `===` differs from `==` and cannot be overridden.
10. Runtime tests prove chained operands evaluate exactly once and short-circuit.
11. Runtime tests prove selector `matches`, Ellipsis, and `understands` semantics.
12. Runtime tests prove `if let`/`while let` scope, atomic binding, variant destructuring, and single evaluation.
13. Runtime tests prove Option C zipped loops, `at`, strict mismatch behavior, explicit `.zipped`/`.indexed` equivalence, closure capture, and fiber/yield safety.
14. LSP tests cover definitions/references for pattern bindings, flow narrowing, operator hover/dispatch, semantic tokens for contextual words, and intrinsic Bool/Ordering result knowledge.
15. Documentation reflects the new canonical syntax and does not continue teaching `for (x in xs)` as the primary form.

---

# 24. Final semantic examples

The finished language should support the following coherently.

```phalcom
// Basic iteration + pattern binding
for item in items {
  consume(item)
}

for (key, value) in entries {
  consume(key, value)
}

// Ordinal binding
for item at index in items {
  System.print((index, item))
}

// Parallel Option-C zip
for good in goods, store in stores {
  ship(good, to: store)
}

for good at i in goods, store at j in stores {
  System.print((i, good, j, store))
}

// Explicit object-model equivalents
for (index, item) in items.indexed {
  ...
}

for (user, card) in (users, cards).zipped {
  ...
}

for (user, card) at i in (users, cards).zipped {
  ...
}

// General destructuring
const (width, height) = dimensions
let #{name: name, age: age} = userRecord

if let Some(user) = findUser(id) {
  use(user)
}

if let Ok(value) = operation() {
  consume(value)
} else {
  recover()
}

while let Some((key, value)) = iterator.next() {
  consume(key, value)
}

// Ordering
const order = a <=> b

if order === Ordering.less {
  ...
}

// User-defined natural ordering
class Version {
  compare(_ other) {
    if other is! Version {
      return unsupported
    }

    let order = major <=> other.major
    if (order === Ordering.equal).not { return order }

    order = minor <=> other.minor
    if (order === Ordering.equal).not { return order }

    patch <=> other.patch
  }
}

// Chained relations: x evaluated once
if 0 <= x < 10 {
  ...
}

// Exact sameness
if value === None {
  ...
}

// Pattern-object relation
if #+(_) matches #+... {
  ...
}

// Reflection
if object understands #render(_) {
  ...
}

// Ordinary Ellipsis value
const marker = ...
System.print(marker === Ellipsis.instance) // true
```

The implementation should make these examples feel like one language rather than a collection of unrelated parser tricks: ordinary methods own extensible semantic protocols; the compiler owns binding/evaluation structure and sacred guarantees; the VM owns lookup-sensitive bilateral resolution; and the LSP sees the same model as the runtime.
