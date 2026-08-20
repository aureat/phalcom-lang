# Implementation Specification: Membership and Type-Membership Operators

Status: implementation blueprint; no code implementation is performed by this document.

Target repository: /Users/altunhasanli/dev/phalcom/phalcom.

This specification adds: x in y, x not in y, x is in ys, x is! in ys, x is not in ys, and x is! not in ys.

The repository's strict selector is literally is!(_). Some older documentation calls it isExactly(_); the implementation must follow code and existing fixtures, then correct stale documentation.

## 1. Objective and semantic contract

### 1.1 Ordinary membership

The right-hand operand owns ordinary membership:

    x in y      ≡ y.contains(x)
    x not in y  ≡ (y.contains(x)).not

contains(_) is an ordinary overridable Phalcom method with one positional argument and a Bool result. The compiler must not try x.in(y), x.isIn(y), y.includes(x), bilateral fallback, iteration, or equality fallback.

Operands evaluate left-to-right exactly once. For left() in right(), evaluate left(), then right(), then dispatch rightValue.contains(leftValue). Missing contains(_), a method error, or an invalid result follows ordinary message-send/runtime behavior.

### 1.2 Lifted kind-of/exact membership

These forms lift the existing Object relation over a candidate iterable. They do not call contains(_).

    x is in ys       ≡ ys.any(where: |candidate| { x.is(candidate) })
    x is! in ys      ≡ ys.any(where: |candidate| { x.is!(candidate) })
    x is not in ys   ≡ (ys.any(where: |candidate| { x.is(candidate) })).not
    x is! not in ys  ≡ (ys.any(where: |candidate| { x.is!(candidate) })).not

Negation applies to the complete existential result, never inside the predicate. The candidate iterable is evaluated once, the left expression once, and existing any(where:) short-circuiting is retained. The left value must be captured after evaluation and must not be re-evaluated once per candidate.

### 1.3 Existing is contract

    x is T       ≡ x.is(T)
    x is! T      ≡ x.is!(T)
    x is not T   ≡ (x.is(T)).not
    x is! not T  ≡ (x.is!(T)).not

is walks the current class/superclass chain. is! compares the current direct class. Both are ordinary overridable sends defined in core Phalcom source; negation is Bool#not.

Deliberately unsupported: no isIn(_), belongsTo(_), doesNotContain(_), isNotIn(_), class-only restriction for candidates, union-type implementation, or optimizer that changes evaluation order or duplicates evaluation.

## 2. Current implementation

### 2.1 Lexer/parser

phalcom-ast/src/lexer.rs is a hand-written scanner. scan_identifier maps in to Token::In, is to Token::Is, and not to Token::Not. No lexer token is needed. Bang remains relevant to contiguous is! and !=; general prefix bang is retired.

phalcom-ast/src/parser.rs uses precedence climbing:
parse_expr -> parse_assignment -> parse_range -> parse_coalesce -> parse_binary -> parse_unary.

parse_binary recognizes is before the punctuation binary_op table, gated at min_prec <= 3. Parser::parse_is consumes contiguous bang, greedily consumes not, parses its RHS with parse_binary(4), and creates Expr::MethodCall to is or is!, optionally wrapped in Expr::Unary(UnaryOp::Not). There is no is AST node.

That direct desugar is safe for is. It is insufficient for in: a normal MethodCall with object rhs and argument lhs compiles receiver before arguments and would reverse source-side effects.

Token::In is also used by the dedicated for (binding in iterable) grammar. That grammar must remain separate.

### 2.2 AST/compiler/VM

phalcom-ast/src/ast.rs has Expr::Binary, Expr::MethodCall, and Expr::Unary. BinaryExpr stores op, optional op_range, left, right, and range. BinaryOp has arithmetic, comparison, bitwise, And, and Or, but no membership.

phalcom-core/src/compiler/lib/expr.rs compiles ordinary binary expressions left then right and emits Bytecode::Invoke through emit_operator_send. And and Or are special only because they create lazy blocks and use inliner.rs.

phalcom-core/src/compiler/lib/scope.rs::emit_operator_send constructs selectors with encode_selector, interns them, and emits Invoke. Dispatch in phalcom-core/src/vm/dispatch.rs and vm/send.rs performs ordinary lookup, overrides, access control, and doesNotUnderstand. There is no stack-swap opcode. The default implementation therefore uses hidden compiler locals, not a new VM opcode.

### 2.3 Object and collection model

phalcom-core/core/core.ph and phalcom-core/core/universe/src/object/object.ph define Object is(_) and is!(_) in Phalcom source. They are not Rust primitives. They use class, superclass, ==, and a loop; root termination is None. phalcom-core/src/universe/primitives.rs installs the native floor around Object but not these methods.

phalcom-core/core/universe/src/collections/iterable.ph defines iterate(_), iteratorValue(_), and any(where:). Its current generic membership method is includes(_), implemented by cursor traversal and ==.

Current names are inconsistent:

- List, Tuple, and Bytes inherit includes(_) from Iterable.
- Map and Set includes(_) delegate to native _$has(_).
- Range includes(_) performs direct bound checks.
- docs/spec/collections/04-sets.md already describes values.contains(value), while docs/spec/current/core/collection-protocol.md calls includes(_) derived behavior.
- Reflection objects already expose native contains(_) in phalcom-native-surface/src/lib.rs; this proves selector compatibility but not collection membership.

Make contains(_) canonical and retain includes(_) as a compatibility alias. Do not make the compiler choose between spellings.

### 2.4 Tooling/tests

phalcom-lsp/src/semantic_tokens.rs already classifies In, Is, and Not as keywords. phalcom-lsp/src/hover.rs::keyword_spelling already knows their spellings. phalcom-lsp/src/selectors.rs::binary_selector_name maps punctuation BinaryOp values only and must not invent a punctuation selector for in.

phalcom-lsp/src/semantic/flow.rs::refine_condition_state recognizes desugared method calls named is and is!, with surrounding UnaryOp::Not. New lifted forms are not simple variable tests and must not be fed to that narrowing rule.

Relevant existing tests: phalcom-ast/tests/lexer.rs and parser.rs; phalcom-core/tests/lang.rs; class fixtures is_kind_of_and_exact.ph, is_subclass_inclusion.ph, is_not_particle.ph, is_metaclass_discriminator.ph; sequence any short-circuit fixtures; phalcom-core/tests/invariants.rs::isa_is_reflexive_and_superclass_closed; disasm_golden.rs; and phalcom-lsp/tests/semantic_tokens_current_syntax.rs.

docs/spec/current/is-tests.md is marked implemented but uses isExactly(_) in parts of its prose while code and fixtures use is!(_). Correct this inconsistency instead of adding a second strict selector.

## 3. Design decisions

D1. Choose RHS ownership: y.contains(x). The domain knows whether membership means equality, key lookup, hashing, bounds, substring, identity, or custom policy. Reject candidate-owned and bilateral forms.

D2. contains(_) is canonical; includes(_) remains an ordinary alias. New syntax always targets contains(_).

D3. is…in is syntactic lifting over any(where:), not another protocol. Strictness selects is!(_).

D4. Negate after any. This preserves existential logic and short-circuiting.

D5. Use explicit compiler lowering for evaluation order. Do not direct-desugar ordinary in to a MethodCall.

D6. Add no native membership primitive, ABI entry, VM dispatch branch, or bytecode opcode.

D7. Do not statically reject a receiver that lacks contains(_); use ordinary dynamic dispatch errors.

## 4. Proposed architecture

### 4.1 AST

Add to phalcom-ast/src/ast.rs:

    pub struct MembershipExpr {
        pub left: Expr,
        pub right: Expr,
        pub negated: bool,
        pub op_range: SourceRange,
        pub range: SourceRange,
    }

    pub struct IsMembershipExpr {
        pub left: Expr,
        pub candidates: Expr,
        pub strict: bool,
        pub negated: bool,
        pub op_range: SourceRange,
        pub range: SourceRange,
    }

Add Expr::Membership and Expr::IsMembership. Do not add these to BinaryOp unless the implementation deliberately replaces the named nodes; named nodes prevent accidental use by compound assignment and binary_selector_name.

Update every AST walker: compiler purity/attribute analysis, boundedness/source facts, LSP analysis, semantic-token traversal, debug/snapshot traversal, and expression matchers. Preserve source ranges.

### 4.2 Parser

Extend Parser::parse_binary at the existing is/equality gate (min_prec <= 3). Recognize in and produce MembershipExpr. Recognize not in as one compound operator: not must precede in. Parse RHS at parse_binary(4). Do not accept in not y as negated membership.

Refactor parse_is into two branches after contiguous strictness and greedy not handling:

    is_suffix := "is" ["contiguous !"] ["not"]
                  ("in" candidate_expr | ordinary_rhs_expr)

If next token is in, consume it and produce IsMembershipExpr; otherwise preserve current direct is/is! send desugaring.

Exact flags:

    x is in ys       -> strict=false, negated=false
    x is! in ys      -> strict=true,  negated=false
    x is not in ys   -> strict=false, negated=true
    x is! not in ys  -> strict=true,  negated=true

is not T remains ordinary is-negation because only an exact following in selects the lifted branch. Retain non-chaining behavior.

### 4.3 Compiler

Add compile_membership(MembershipExpr) and compile_is_membership(IsMembershipExpr) in phalcom-core/src/compiler/lib/expr.rs, using existing hidden local-slot allocation. Hidden slots must not become user-visible bindings or affect closure capture metadata except through normal upvalue capture.

Ordinary sequence:

    compile left
    store hidden left
    compile right
    store hidden right
    load hidden right       // receiver
    load hidden left        // argument
    Invoke(1, contains(_))
    if negated: Invoke(0, not)

Construct contains(_) with encode_selector("contains", [None], SignatureKind::Method(1)); construct not with emit_getter_send.

Lifted sequence:

    compile left once
    store hidden left
    compile candidates once
    build one-parameter closure:
        load hidden left
        load candidate parameter
        Invoke(1, is(_)) or Invoke(1, is!(_))
    send candidates.any(where: closure)
    if negated: send result.not

Reuse existing BlockExpr, upvalue capture, PackItem::Labeled, and call-selector machinery. The generated call is any(where), not positional any(_). any(where:) remains ordinary dispatch, so overrides and existing inlining/deoptimization rules remain authoritative.

Generated invokes use the surface operator range for source maps and diagnostics. No new bytecode is required.

### 4.4 Core protocol

In phalcom-core/core/universe/src/collections/iterable.ph, make contains(_) canonical and keep a non-recursive alias:

    contains(_ x) { ... existing cursor equality walk ... }
    includes(_ x) { self.contains(x) }

List, Tuple, and Bytes inherit it; add no native membership methods.

In map.ph:

    contains(_ k) { self._$has(k) }
    includes(_ k) { self.contains(k) }

In set.ph:

    contains(_ v) { self._$has(v) }
    includes(_ v) { self.contains(v) }

In range.ph, move the direct bound-check body to contains(_) and retain includes(_) as alias. Preserve open bounds and exclusive/inclusive upper behavior.

In scalar/string.ph, add substring contains(_) over indexOf(_). Recommended contract: empty needle is contained; non-String arguments raise the same ArgumentError family as indexOf. Pin this in tests.

Do not add collection contains methods to phalcom-native-surface/src/lib.rs or native metadata. Existing reflection contains entries remain untouched.

### 4.5 LSP/reflection

Keep the new words as keyword semantic tokens; add all six forms to semantic_tokens_current_syntax.rs. Update hover descriptions for RHS-owned membership and lifted kind/exact membership; actual strict selector remains is!(_). Add AST traversal for new nodes. Do not add an entry to binary_selector_name. Do not apply simple variable narrowing to ordinary in or lifted forms. Reflection needs no special case: contains(_) is found through ordinary respondsTo, methodFor, and Behavior#methods; is…in has no reflected selector.

## 5. Detailed change-set

Parser/AST: modify ast.rs, parser.rs, lexer/parser tests. Cover variants, flags, ranges, precedence, malformed forms, and for regression.

Compiler: modify expr.rs, the minimal temporary-slot support in mod.rs/scope.rs, attributes.rs, and boundedness.rs. No VM semantic branch is required; verify ordinary Invoke path.

Core: modify Iterable, Map, Set, Range, and String Phalcom sources. Synchronize core/core.ph and the object mirror comments only as bootstrap/build requires; no Object semantic change.

Docs: update current/is-tests.md, current/syntax/expressions.md, current/core/collection-protocol.md, and collections/04-sets.md.

## 6. Algorithms and lowering rules

| Surface | Semantic lowering | Owner |
|---|---|---|
| x in y | y.contains(x) | y |
| x not in y | (y.contains(x)).not | y |
| x is in ys | ys.any(where: |c| { x.is(c) }) | ys then x |
| x is! in ys | ys.any(where: |c| { x.is!(c) }) | ys then x |
| x is not in ys | negate completed any | ys then x |
| x is! not in ys | negate completed any | ys then x |

Place ordinary in/not in at the effective is/equality tier used by min_prec <= 3. and/or bind more loosely; arithmetic/comparisons bind inside operands. Forms are non-chaining.

Collection behavior:

| Receiver | Meaning | Implementation |
|---|---|---|
| List, Tuple, Bytes | cursor element == x | Iterable Phalcom method |
| Map | key exists | wrapper over _$has |
| Set | member under hash/equality contract | wrapper over _$has |
| Range | bound membership | direct Phalcom check |
| String | substring occurrence | over indexOf |
| arbitrary object | no default | ordinary missing send |

## 7. Examples

    2 in [1, 2, 3]                 // true
    4 not in [1, 2, 3]             // true
    "ph" in "phalcom"              // true
    "answer" in map                // key membership
    3 in 1..=5                     // true
    5 not in 1..5                  // upper-exclusive

    3 is in (Number, String)
    3 is! in (Number, Int)
    3 is not in (String, Bool)
    3 is! not in (Number, String)

Evaluation-order fixture:

    var events = []
    fn left() { events.append("left"); return 1 }
    fn right() { events.append("right"); return [1] }
    left() in right()
    events == ["left", "right"]

Override fixture:

    class Domain {
      contains(_ x) { x == 42 }
    }
    42 in Domain.new()

Malformed forms: x in not y is a syntax error; x is in is missing its candidate; a is B is C retains the existing non-chaining error. 3 in 4 is syntactically valid and fails at runtime through missing contains(_).

## 8. Errors and diagnostics

Use existing Parser::error_here, error_message_here, and SyntaxErrorKind conventions.

Required parse diagnostics: missing RHS after in; not after in with explanation that valid spelling is not in; missing candidate after every is…in form; preserved contiguous is! rule; preserved non-chaining diagnostic and precise span.

Do not add a compile-time “not a container” error.

Missing contains(_) is ordinary MessageNotUnderstood for selector contains(_), with existing reified Message, receiver rendering, and suggestions. Custom contains, is, is!, any, or predicate errors propagate ordinary Phalcom errors and tracebacks. Operator source range identifies the complete surface operator.

## 9. Compatibility and migration

in is already reserved for for syntax; expression support must not alter for (x in xs). Existing is/is!/is not/is! not AST and bytecode behavior must remain unchanged.

Existing includes(_) calls remain valid through aliases. Custom classes with only includes(_) do not satisfy new in; documentation must tell authors to add contains(_). No compiler fallback is allowed.

No native ABI, NATIVE_MEMBERS count, serialized current opcode, or VM dispatch special case changes. New chunks use existing Invoke, closure, and local instructions.

contains(_) is visible to respondsTo, methodFor, and method reflection. is…in has no reflected selector. Future typing may model contains(_) -> Bool and iterable candidate domains; future union types can be consumed by Object is(_) without changing this syntax.

## 10. Testing strategy

Lexer/parser: test keyword/span sequence, all six forms and flags, AST RHS ownership, precedence with arithmetic/equality/and/or, ranges, tuple grouping, malformed ordering, missing RHS, non-chaining, and for regression.

Compiler/disassembly: assert LHS compiles before RHS; hidden locals are private; receiver/argument order is RHS/LHS; contains(_) and optional not sends are emitted; lifted code stores one LHS and one candidates value, captures LHS once, calls any(where), chooses is versus is!, and negates only after any. Assert no native primitive or new VM opcode.

End-to-end: add phalcom-core/tests/lang/membership/ and register it in lang.rs. Include list/tuple/bytes, map keys, set, open/closed/exclusive ranges, string substring and empty needle, kind-of, exact, all negations, any short circuit, evaluation order, custom contains/is/is!/any overrides, and for regression. Add syntax-error fixtures and a runtime-error fixture for an RHS without contains(_).

Runtime/LSP: extend invariants.rs with direct contains and override tests; extend semantic_tokens_current_syntax.rs with all forms; analyze a document containing all forms without bogus narrowing. Assert every built-in behavior-matrix receiver responds to contains(_) and all includes(_) fixtures remain green.

## 11. Implementation sequence

1. Confirm which core source/bootstrap mirror is canonical; do not edit both blindly.
2. Correct is!(_) documentation and contains/includes migration wording.
3. Add AST nodes and parser support; run AST tests and for regressions.
4. Implement ordinary membership hidden-local sequencing; verify disassembly and side-effect order.
5. Implement lifted closure/capture/any lowering; verify one-time evaluation and aggregate negation.
6. Add core contains(_) methods and aliases for Iterable, Map, Set, Range, and String.
7. Add acceptance and negative fixtures; register the language group.
8. Update compiler walkers and LSP tooling; run LSP tests.
9. Run normal workspace verification and review disassembly/golden changes.
10. If code is later implemented, refresh graphify with graphify update .; this specification itself needs no graph regeneration.

Each milestone must compile. Do not land parser syntax without compiler arms or core methods without operator-path tests.

## 12. Acceptance criteria

- [ ] Lexer accepts all spellings; contiguous is! remains exact.
- [ ] Parser accepts exactly the six operators and preserves for syntax.
- [ ] Malformed ordering, missing RHS, and chaining have precise diagnostics.
- [ ] AST preserves operands, strictness, aggregate negation, and source ranges.
- [ ] x in y dispatches only y.contains(x), with left-before-right once-only evaluation.
- [ ] x not in y is positive membership followed by Bool not.
- [ ] is in uses any(where) plus is(_); is! in uses is!(_).
- [ ] Negated lifted forms negate the completed existential result.
- [ ] List, Tuple, Bytes, Map, Set, Range, and String behavior is tested.
- [ ] includes(_) compatibility remains green; generated syntax uses contains(_) only.
- [ ] User overrides of contains, is, is!, and any are observable.
- [ ] No new native primitive, ABI entry, VM branch, or bytecode opcode is required.
- [ ] Existing is/is! tests and runtime invariants remain green.
- [ ] Compiler walkers, source maps, diagnostics, semantic tokens, hover, and LSP analysis handle new nodes without panic or bogus narrowing.
- [ ] Current documentation names actual selectors and exact lowerings.
- [ ] Full workspace verification passes; changed goldens have semantic reasons.
