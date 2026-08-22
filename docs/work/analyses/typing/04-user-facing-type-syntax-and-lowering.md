# Phalcom User-Facing Type Syntax and Lowering

**Date:** 2026-08-22
**Status:** Ratified syntax and lowering specification, with named gates
**Authority:** source-facing typing grammar and compiler-authoritative annotation lowering
**Depends on:** [01 — Implementation Architecture](01-implementation-architecture.md), [02 — Runtime Reification and Metadata](02-runtime-reification-and-metadata.md), and [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md)
**Owners:** `phalcom-ast`, `phalcom-semantic`, `phalcom-type-syntax`, native metadata adapters, source formatter ownership when established
**Scope:** source annotations, type applications, tuples, callables, unions, `Self`, `Dynamic`, generic binders, variance, kinds, constraints, record rows, aliases, parser recovery, semantic lowering, and formatter behavior
**Non-goals:** runtime type expressions, selector changes, class specialization, ambient generic context, proof/effect syntax, or implementation in this document

## 1. Purpose and authority

This document fixes source-facing syntax and its path into compiler-owned semantic facts. It does not alter runtime class/metaclass behavior, make type arguments affect selector identity, or turn annotations into runtime expressions.

Evidence labels follow the [series reading contract](README.md#1-reading-contract-and-evidence-labels):

- **Observed current implementation** records live source behavior.
- **Observed test coverage** records visible tests or supplied Task 13 evidence; this documentation task did not rerun tests.
- **Ratified/normative design** authorizes implementation.
- **Proposed design needing ratification** is a hard stop before public syntax or metadata is emitted.
- **Untracked forward-design input** is useful context without authority.
- **Pyrefly architectural transfer** transfers execution mechanisms, never Python syntax or type rules.

Normative distinction:

```text
value.class     runtime class identity
value : T       static value-typing judgment
T :: K          kinding judgment over a type form
value ⇝ T       explicit reification/description operation
```

`Type` is the atomic proper-type kind. `TypeForm` is a semantic and reflective role described by [03](03-reflection-api-and-capabilities.md), not a superclass of runtime classes and not a replacement for `Type`.

## 2. Current implementation inventory

### 2.1 AST is ahead of source parsing

**Observed current implementation.** [`TypeAnnotationExpr`](../../../../phalcom-ast/src/ast.rs#L406) already represents:

- a statically resolved reference;
- type application;
- union;
- labeled or unlabeled tuple elements;
- callable parameters and result.

[`Parser::parse_type_annotation`](../../../../phalcom-ast/src/parser.rs#L1396) accepts only an identifier followed by zero or more qualified-name segments. It is invoked for local bindings, fields, method parameters, rest parameters, getters/setters/index methods, and callable return annotations. Generic declaration binders are absent from [`ClassDef`](../../../../phalcom-ast/src/ast.rs), and variant payload syntax is not typed.

Consequence: hand-built AST tests can exercise semantic forms that no source program can spell. Parser acceptance and semantic acceptance must be tested separately.

### 2.2 Semantic lowering already covers core forms

**Observed current implementation.** [`resolve_type_form`](../../../../phalcom-semantic/src/types/annotation.rs) lowers the existing AST variants through `TypeStore`: reference lookup, kind-checked application, tuple construction, callable construction, and union normalization. `Never`, `Unit`, and `Dynamic` receive special handling.

Two repairs are required before expanding syntax:

1. application and kind errors currently collapse to `UnknownReason::UnannotatedDeclaration`, merging invalid syntax/semantics with missing annotation knowledge;
2. the canonical source spelling for unit is `()`, while the resolver currently recognizes implementation-facing `Unit` as a reference.

This specification introduces an explicit resolved-annotation result so neither repair is hidden inside parser work.

### 2.3 Native metadata has a separate parser

**Observed current implementation.** [`phalcom-type-syntax`](../../../../phalcom-type-syntax/src/lib.rs) parses native metadata into `TypeExpr` and `CallableType`. It supports named references, applications, unions, tuples, callables, `Self`, `Never`, `Unknown`, and `universe.<name>`. Its generic parameters carry names only. It has no source spans, recovery tree, owner-qualified binders, variance, constraints, record rows, or diagnostic provenance.

**Ratified/normative design.** Source parsing and native metadata parsing may share a token-independent semantic lowering vocabulary, but they remain separate front ends. Source recovery requirements must not be weakened to fit a metadata parser. Native `Unknown` is migrated to an explicit metadata knowledge/status result; it does not become a source type or canonical `TypeData` member.

### 2.4 Current record and operator syntax

**Observed current implementation.** Runtime record literals and patterns use `#{ ... }`. Record expansion uses `**`. `|` is already an operator token and a valid method selector. Type annotations are contextual after `:`, `->`, or a type-declaration binder, so using `|` within type grammar does not change selector lexing.

## 3. Lexical and contextual boundary

**Ratified/normative design.** Type syntax is contextual. Parser enters type mode only at:

- a declaration or binding annotation after `:`;
- a callable result after `->`;
- a type alias right side after `=`;
- a generic parameter kind after `::`;
- a generic constraint right side after its relation token;
- a record-row field type or tail;
- native metadata's dedicated parser entry point.

No type atom is an arbitrary runtime expression. Name resolution in annotations remains static and side-effect free. Annotation normalization cannot send messages, invoke `Type.currentApplication`, evaluate a class method, perform reflection, or allocate runtime type descriptor objects.

### 3.1 Annotation positions

Every position declares its required kind and missing-annotation policy:

| Position | Expected kind/status | Missing behavior |
|---|---|---|
| local/constant/field annotation | proper `Type` | explicit absence; inference or unannotated policy decides later |
| callable parameter/rest parameter | proper `Type` | explicit absence; never rewritten to `Dynamic` |
| callable return annotation | proper `Type` | explicit absence; does not imply `()` |
| tuple/callable/record field component | proper `Type` | syntactically required once component begins |
| union member | proper `Type` | syntactically required |
| type application origin | declared constructor kind | syntactically required |
| type application argument | constructor parameter's kind | syntactically required |
| generic parameter kind ascription | any admitted kind | omitted kind defaults to `Type` for ordinary type parameters |
| upper/finite constraint member | proper `Type` unless constraint declares a constructor kind | syntactically required |
| record tail | `RecordRow` | omitted tail means closed row |
| transparent alias right side | alias declaration's inferred/declared result kind | syntactically required |

Future protocol/ADT payload positions reuse this table only after their declaration semantics are ratified. An unsaturated constructor in a proper-type position is invalid even when partial application is supported internally.

### 3.2 Reserved contextual spellings

Reserved contextual spellings:

| Spelling | Meaning | Status |
|---|---|---|
| `()` | unit type and empty tuple type | Ratified |
| `Never` | bottom proper type | Ratified |
| `Dynamic` | explicit dynamic boundary | Ratified |
| `Self` | owner-relative proper type | Ratified |
| `Any` | top proper type | Name reserved; introduction gated by `DEC-ANY-SURFACE` |
| `Unknown` | no source type | Rejected as an annotation |
| `Type` | atomic proper-type kind | Ratified in kind positions |
| `where` | constraint-clause introducer | Ratified |

Identifiers with these spellings remain ordinary identifiers outside type/kind contextual positions where existing grammar permits them.

## 4. Core annotation grammar

### 4.1 Active grammar

The following EBNF is normative for the first public syntax tranche:

```ebnf
type-annotation     ::= union-type

union-type          ::= callable-type ("|" callable-type)*

callable-type       ::= postfix-type
                      | callable-domain "->" type-annotation

postfix-type        ::= type-atom type-arguments*

type-arguments      ::= "<" type-annotation ("," type-annotation)* ">"

type-atom           ::= static-type-reference
                      | "()"
                      | "Never"
                      | "Dynamic"
                      | "Self"
                      | parenthesized-type
                      | tuple-type
                      | record-type

static-type-reference
                    ::= identifier ("." identifier)*

parenthesized-type  ::= "(" type-annotation ")"

tuple-type          ::= "(" tuple-type-element ","
                          (tuple-type-element ("," tuple-type-element)*)? ")"

tuple-type-element  ::= (identifier ":")? type-annotation

callable-domain     ::= "(" callable-parameter-list? ")"

callable-parameter-list
                    ::= callable-parameter ("," callable-parameter)* ","?

callable-parameter  ::= (identifier ":")? type-annotation
                      | "..." type-annotation

record-type         ::= "#{" record-type-body? "}"

record-type-body    ::= record-type-field ("," record-type-field)* ","?
                      | record-type-field ("," record-type-field)* ","
                        "|" record-row-tail
                      | "|" record-row-tail

record-type-field   ::= identifier ":" type-annotation
record-row-tail     ::= type-parameter-reference
```

`postfix-type` application binds tighter than callable arrow. Callable arrow binds tighter than union on its left and is right associative on its result:

```text
List<Int> | None             Union(List<Int>, None)
(Int) -> String | None       Callable((Int), Union(String, None))
(Int) -> (String) -> Bool    Callable((Int), Callable((String), Bool))
```

Parentheses disambiguate a one-element tuple from grouping and a one-parameter callable domain:

| Source | Meaning |
|---|---|
| `()` | unit/empty tuple proper type |
| `(T)` | grouped type `T` |
| `(T,)` | one-element tuple type |
| `(T) -> R` | one-parameter callable type |
| `((),) -> R` | callable taking one unit value |

Tuple labels and callable labels are semantic labels, not type names. Duplicate labels are diagnosed during lowering even if parser recovery retains the complete node.

### 4.2 Union syntax

**Ratified/normative design.** `A | B` is the only active infix type relation constructor in the first tranche. Parser builds every written member with its own range. TypeStore later flattens, sorts, deduplicates, removes `Never`, and collapses singleton unions under the rules in [01](01-implementation-architecture.md).

Intersection spelling is reserved but inactive. Parser must not silently treat `A & B` as a type until conformance and intersection normalization are ratified in [05](05-advanced-kinds-constraints-effects-and-proofs.md).

### 4.3 Callable syntax

**Ratified/normative design.** Callable left side is tuple-shaped. Zero parameters use `() -> R`; one positional unit parameter uses `((),) -> R`. Rest type syntax is `...T`, matching native metadata and remaining distinct from runtime collection expansion.

Callable type annotations describe parameter labels/types and normal return type. Effects, exits, termination, contracts, and proof evidence are separate declarations or metadata described in [05](05-advanced-kinds-constraints-effects-and-proofs.md). They are never inferred from `Never` alone.

### 4.4 Record row syntax

Options were checked against current record literals/patterns, value expansion, union tokens, maps, and selectors:

| Candidate | Assessment |
|---|---|
| `#{ name: String | R }` | Rejected: final union member and row tail cannot be distinguished syntactically. |
| `#{ name: String, ...R }` | Rejected: resembles value spread/rest and conflicts with native callable ellipsis vocabulary. |
| `#{ name: String, **R }` | Rejected: `**` already means record/value expansion. |
| `#{ name: String } where row R` | Rejected for initial surface: verbose and separates tail from structural form. |
| `#{ name: String, | R }` | Selected: existing record/union tokens, contextual, unambiguous through mandatory comma. |

**Ratified/normative design.** Open record rows use existing record delimiters and selected tail form:

```phalcom
#{ name: String, age: Int }
#{ name: String | None, | R }
#{ | R }
```

Known fields and tail are different AST fields. `| R` is accepted at the beginning of a tail-only body or after a mandatory comma following the final known field. Therefore a union in the final field remains unambiguous:

```phalcom
#{ value: Int | String, | R }
```

The earlier draft form `#{ value: Int | String | R }` is rejected as ambiguous. This decision preserves `|` as union everywhere inside a field type and avoids precedence dependent on whether an identifier happens to denote a row parameter.

Closed record `#{}` is valid. Tail must resolve to a generic parameter with kind `RecordRow`; arbitrary type expressions, `Dynamic`, and nominal classes cannot be row tails. Known fields are sorted by field name when canonicalized, while source order and ranges remain in syntax/provenance data.

Record rows describe structural record values only. They do not expose or approximate nominal object layouts. Variant rows and effect rows are different semantic domains even if a future implementation shares a generic row-solver utility.

### 4.5 Intersection and effect staging

Callable, tuple, union, and record forms in §4 are active target grammar. Intersection `A & B` is reserved until semantic conformance, normalization, distribution, and overload interaction are ratified. Parser must diagnose it as unsupported type syntax, not reinterpret `&` as a runtime selector.

Effect sets, effect variables, raises/exits, totality, and handler syntax do not occur inside the initial `type-annotation` grammar. Callable annotations describe parameter and normal-result types only. A future effect surface may follow a callable declaration/signature under a distinct contextual introducer; it must lower to the separate effect/exit domains in [05](05-advanced-kinds-constraints-effects-and-proofs.md), not become another `TypeAnnotationExpr` member by convenience.

## 5. Generic declarations

### 5.1 Binder grammar

Generic binder syntax is normative:

```ebnf
generic-parameters ::= "<" generic-parameter ("," generic-parameter)* ">"

generic-parameter  ::= variance? identifier kind-annotation? parameter-bound?

variance           ::= "+" | "-"
kind-annotation    ::= "::" kind-expression
parameter-bound    ::= "<:" type-annotation

kind-expression    ::= kind-atom
                      | kind-atom "->" kind-expression
kind-atom          ::= "Type" | "RecordRow" | "(" kind-expression ")"
```

Examples:

```phalcom
class Box<+T> { ... }
class Encoder<-T> { ... }
class Functor<F :: Type -> Type> { ... }
map<U>(transform: (T) -> U) -> List<U> { ... }
```

`+T` means covariant, `-T` contravariant, and bare `T` invariant. `out` and `in` are rejected. Signs are contextual variance markers over parameter declarations; they do not form part of a selector or identifier.

### 5.2 Attachment points

**Ratified/normative design.** First implementation attaches generic binders to:

- class declarations, immediately after class name;
- method declarations, immediately after selector/name and before parameter list;
- transparent type aliases, immediately after alias name.

Generic binder AST must be reusable by future protocol and algebraic-data declarations. Their declaration wrappers remain behind their own syntax/semantics gates; parser work here must not invent them.

### 5.3 Stable binder identity

Syntax names are not semantic identity. Lowering allocates `TypeParameterId` from declaration owner plus binder index. The source name and range are presentation/provenance. Renaming a binder changes display and source fingerprint but does not make two binders with equal text interchangeable.

**Ratified/normative design.** Kind-polymorphic generalization is prenex. `KindParameterId` is stable in generalized interfaces; solver-local `KindVarId` never escapes. This document does not expose public syntax for kind parameters. No unsolved kind variable may reach an interface, snapshot, metadata DAG, or reflection result.

### 5.4 Partial application

**Observed current implementation.** `TypeStore::apply_type_form` already returns a residual arrow kind for partial application.

**Ratified/normative design.** Partial type application is legal only where a constructor kind is expected. A value annotation, field annotation, parameter annotation, return annotation, tuple element, union member, record field, or callable slot requires kind `Type`. Thus `Map<String>` may satisfy a higher-kinded parameter of kind `Type -> Type` but cannot directly annotate a runtime value.

No partial application changes runtime class-side dispatch. `Map<String>.new()` is not introduced by this syntax; explicit reflection/construction follows [03](03-reflection-api-and-capabilities.md).

## 6. Constraints and aliases

### 6.1 Constraint grammar

The initial constraint clause is:

```ebnf
where-clause       ::= "where" constraint ("," constraint)*

constraint         ::= type-parameter-reference "<:" type-annotation
                     | type-parameter-reference "in" "("
                       type-annotation ("," type-annotation)+ ")"
```

Examples:

```phalcom
class SortedSet<T> where T <: Comparable<T> { ... }
parseNumber<T>() -> T where T in (Int, Float) { ... }
```

Upper bounds and finite exact-set constraints are ratified. A finite set does not create subtyping between its members. `T in (Int, Float)` constrains inference/instantiation; it does not authorize implicit numeric conversion.

Lower bounds, equality constraints, associated types, negative constraints, and arbitrary boolean constraint expressions are **Proposed design needing ratification**. F-bounds such as `T <: Comparable<T>` are allowed only after occurs checks and recursive-relation budgets in [05](05-advanced-kinds-constraints-effects-and-proofs.md) pass their gate.

### 6.2 Alias declaration

Transparent alias surface is ratified:

```ebnf
type-alias-declaration
                  ::= "type" identifier generic-parameters? where-clause?
                      "=" type-annotation
```

```phalcom
type UserId = Int
type Pair<T> = (T, T)
type Named<R :: RecordRow> = #{ name: String, | R }
```

A transparent alias has declaration identity for navigation, diagnostics, metadata provenance, and invalidation, but expands for semantic equivalence under a bounded, cycle-checked alias resolver. It does not create a runtime class, constructor, allocation identity, or distinct runtime value.

Lowering records the decision explicitly:

```rust
enum AliasTransparency {
    Transparent,
    Opaque,
}

struct AliasDeclaration {
    transparency: AliasTransparency,
    // owner, binders, kind, normalized body, provenance, dependencies
}
```

Bare `type` is deliberately the transparent syntax and lowers to `AliasTransparency::Transparent`; transparency is not inferred later from a missing flag. `Opaque` remains impossible to construct from source until a separate opaque/newtype spelling and runtime/representation policy are ratified. Recursive transparent aliases require an explicit guarded-recursion decision and remain rejected initially.

## 7. `Self`, top/dynamic/unknown, and literal facts

### 7.1 `Self`

**Ratified/normative design.** `Self` is contextual and resolves through an explicit semantic owner:

- in instance-side class members, current instance nominal type with current generic arguments;
- in class-side members, the owner-relative class-object form required by the declaration surface;
- in protocol-like future declarations, conforming receiver form under that feature's own rules;
- outside an owning declaration, invalid.

`Self` is not a global declaration and is not resolved through ordinary module lookup. It must preserve owner/side identity in lowered semantic form so inherited and overridden signatures can substitute it correctly.

### 7.2 Epistemic separation

**Ratified/normative design.** These states never collapse:

| State | Source spelling | Canonical type? | Meaning |
|---|---|---|---|
| `Never` | `Never` | Yes | no normal value inhabits form |
| `Dynamic` | `Dynamic` | Explicit dynamic knowledge/type boundary | runtime-checked or open-world behavior |
| `Any` | reserved | Proper type when ratified | semantic top, not checker failure |
| missing annotation | none | No | program omitted an annotation |
| unresolved name | written name | No | static resolution failed |
| invalid annotation | written invalid form | No | parse/kind/arity/constraint failure |
| inference variable | none | Solver-local only | pending constraint solution |
| budget/cancelled/internal | none | No | computation terminal status |

`Unknown` is rejected in source. Native metadata's legacy `Unknown` becomes an explicit metadata status such as `MetadataTypeKnowledge::Unknown(reason)` outside `TypeStore`.

### 7.3 Numeric literal facts

Annotation syntax does not reinterpret literals. `1` has runtime class/type `Int`; `1.0` has `Float`. An expected type can constrain method selection or diagnose incompatibility, but cannot make the literal another runtime numeric class. Exact spelling/value knowledge is `ConstantFact`, not a singleton type produced by syntax lowering.

## 8. Syntax AST and recovery contract

### 8.1 Target AST

**Ratified/normative design.** Extend `phalcom-ast` without embedding semantic IDs:

```rust
enum TypeAnnotationExpr {
    Reference(StaticSymbolRef),
    Application { origin: Box<TypeAnnotation>, arguments: Vec<TypeAnnotation>, range: SourceRange },
    Union { members: Vec<TypeAnnotation>, range: SourceRange },
    Tuple { elements: Vec<TypeTupleElement>, range: SourceRange },
    Callable { parameters: Vec<TypeCallableParameter>, result: Box<TypeAnnotation>, range: SourceRange },
    Unit { range: SourceRange },
    Dynamic { range: SourceRange },
    Never { range: SourceRange },
    SelfType { range: SourceRange },
    Record { fields: Vec<RecordTypeField>, tail: Option<RecordRowTailSyntax>, range: SourceRange },
    Invalid { diagnostic: ParseDiagnosticId, range: SourceRange },
}
```

Existing reference spellings may remain represented as references during a compatibility patch, but final AST gives reserved atoms explicit nodes. This prevents a local declaration named `Dynamic` from changing annotation meaning and prevents semantic lowering from string-matching ordinary references.

Generic syntax uses `GenericParameterSyntax`, `VarianceSyntax`, `KindSyntax`, `ConstraintSyntax`, and `TypeAliasDecl`. These syntax objects carry ranges and names, never `TypeId`, `KindId`, `TypeParameterId`, or solver variables.

### 8.2 Recovery

Parser recovery must retain a typed syntax node and continue at the nearest type boundary. Synchronization tokens:

- `,`, `)`, `>`, and `}` inside composite type forms;
- `|` inside union/row contexts;
- `where`, `=`, `{`, or declaration terminator at binder level;
- outer expression/declaration recovery set after annotation completion.

Examples of required recovery:

| Input | Diagnostic | Recovery |
|---|---|---|
| `List<>` | missing type argument | invalid argument node; consume `>` |
| `Map<String,>` | trailing argument missing | invalid final argument; preserve application |
| `(Int -> String` | expected `)` before arrow/domain end | retain callable-shaped node if unambiguous |
| `#{ name String }` | expected `:` | retain field name and invalid field type |
| `#{ name: String | R }` intended row | missing comma before row tail | parse union field; no semantic row guess |
| `class Box<+>` | missing parameter name | invalid binder; continue at `,` or `>` |
| `T ::` | missing kind | invalid kind node |

Recovery must not invent `Dynamic`, `Any`, or a canonical unknown type. Invalid syntax lowers to an explicit invalid result and cannot be cached as a successful annotation.

### 8.3 Nested `>` tokens

Type mode must parse nested `List<List<Int>>` even if ordinary expression lexing has a `>>` operator token. Preferred implementation is parser-context token fission with precise ranges; globally changing shift-selector lexing is forbidden. Formatter emits adjacent `>>` for canonical nested type arguments only after parser and lexer tests prove round-trip behavior.

## 9. Semantic lowering pipeline

### 9.1 Stages

**Ratified/normative design.** Annotation lowering is staged:

```text
TypeAnnotation syntax
    -> resolve contextual atoms and static names
    -> allocate owner-qualified binders
    -> build TypeTerm / KindTerm / ConstraintTerm
    -> validate arity, kind, proper-position, variance, and bounds
    -> normalize through TypeStore
    -> publish ResolvedAnnotation with provenance and dependencies
```

Target result:

```rust
enum AnnotationStatus {
    Resolved { form: TypeId, kind: KindId },
    Missing,
    Unresolved { name: StaticNameId },
    Invalid { diagnostic_ids: Box<[DiagnosticId]> },
    Cancelled,
    BudgetExceeded { operation: BudgetClass },
    InternalError { incident: IncidentId },
}

struct ResolvedAnnotation {
    syntax: SyntaxId,
    owner: DeclarationId,
    status: AnnotationStatus,
    dependencies: DependencySet,
    provenance: AnnotationProvenance,
}
```

`Resolved` may hold a constructor kind only in an explicitly constructor-kinded position. Value positions call `expect_proper_type` and produce an invalid result when kind is not `Type`.

### 9.2 Name and binder resolution

Resolution order in a type annotation:

1. contextual atoms (`()`, `Never`, `Dynamic`, `Self`, later `Any`);
2. lexically bound type parameters;
3. declaration-local aliases/imports under ordinary static module rules;
4. qualified module/static references;
5. unresolved result.

Every successful lookup records a semantic dependency key. Alias expansion records both alias identity and dependencies of its body. `Self` records declaration surface/side dependency. No runtime namespace lookup or DNU participates.

### 9.3 Variance and constraints

Declaration lowering validates variance on the normalized declaration surface, not parser text. It computes every occurrence position through application arguments using the referenced constructor's declared variance. Unknown or unresolved variance data blocks interface publication with reasoned status; it does not default to covariance.

Constraint lowering occurs after all binders have identities but before bodies are checked. Constraint solving may use temporary variables; published generic signatures contain only stable binder IDs, normalized bounds, and validated kind schemes.

### 9.4 Records

Record syntax lowers to:

```rust
RecordType {
    fields: Box<[(FieldName, TypeId)]>,
    tail: RecordTail::Closed | RecordTail::Parameter(TypeParameterId),
}
```

Duplicate field names diagnose at all duplicate source ranges. Canonical form sorts unique fields. A row parameter may occur once as the tail; it is not a normal field and cannot appear as a union member to simulate openness.

## 10. Formatting and rendering

No canonical semantic printer may be used as a source formatter without source-shape data. Formatter rules:

- spaces around `|`, `->`, `<:`, `::`, and `=`;
- no space between constructor and `<`;
- comma plus space between type arguments, tuple entries, fields, binders, and constraints;
- trailing commas preserved where multiline layout requires them;
- `+T` and `-T` have no internal space;
- open row rendered `#{ field: T, | R }`;
- unit always rendered `()` in source, never `Unit`;
- parentheses inserted from precedence, not preserved gratuitously;
- invalid/recovered syntax formatted conservatively or left source-stable until fixed.

Semantic/reflection rendering may show qualified owners or kind schemes that source omitted. It must label that output as a description, not promise parseable source.

## 11. Diagnostics

Required diagnostic families:

| Code family | Trigger | Required context |
|---|---|---|
| `AnnotationParse*` | malformed contextual type grammar | expected token/form and recovery range |
| `AnnotationUnresolved` | static name missing | complete written path and module context |
| `AnnotationNotAType` | value/runtime name in type position | resolved declaration kind |
| `AnnotationUnsaturatedConstructor` | constructor in proper-type position | actual and expected kind |
| `Application*` | arity or argument kind mismatch | constructor signature, argument index |
| `GenericDuplicateParameter` | repeated binder name | first and repeated ranges |
| `VarianceInvalidPosition` | parameter used against declaration variance | occurrence path and composed position |
| `ConstraintUnsatisfied` | instantiation violates bound/set | normalized constraint and evidence |
| `RecordDuplicateField` | duplicate structural field | both ranges |
| `RecordTailKindMismatch` | tail not `RecordRow` | actual kind |
| `SelfOutsideOwner` | `Self` lacks semantic owner | enclosing declaration context |
| `AnnotationUnknownSpelling` | written `Unknown` | explain missing/dynamic alternatives |

Parser owns syntax diagnostics. Semantic module owning the declaration owns resolution, kind, variance, and constraint diagnostics. LSP adapts published diagnostics and never recomputes these rules.

IDE behavior:

- recovered invalid nodes retain ranges so semantic tokens, document symbols, and later diagnostics remain aligned;
- completion after `<`, `::`, `<:`, `in (`, and a record-tail `|` filters candidates by expected kind/domain;
- hover distinguishes written `TypeUse`, resolved/canonical form, kind, and failure status;
- signature help renders source grammar, not runtime descriptor identity;
- formatter/code actions never replace missing or invalid syntax with `Dynamic`;
- stale snapshot/document pairs return stale/blocked status instead of diagnostics for the wrong revision.

## 12. Compatibility and migration

Migration is additive until the final cleanup:

1. add parser tests for current qualified-reference syntax and failure states;
2. introduce explicit reserved atom/invalid AST nodes;
3. add core composite parsing without generic declarations;
4. replace string-matched builtins in `resolve_type_form`;
5. introduce `ResolvedAnnotation` and stop using `UnannotatedDeclaration` for invalid forms;
6. add binder/kind/constraint AST and lowering;
7. add record row syntax only with `RecordRow` kind/store support from [05](05-advanced-kinds-constraints-effects-and-proofs.md);
8. add transparent aliases with cycle checks and invalidation;
9. converge native metadata lowering on the semantic term/result layer;
10. remove legacy `Unit` source acceptance if it was ever observable, while retaining reflective `Unit` naming.

No migration step changes existing runtime class identity, selector identity, class-side lookup, construction, or serialized metadata schema without the gates in [02](02-runtime-reification-and-metadata.md).

## 13. Implementation units and verification

### Unit S1 — Parser parity for existing AST forms

**Files/symbols:** `phalcom-ast/src/ast.rs` (`TypeAnnotationExpr`), `phalcom-ast/src/parser.rs` (`parse_type_annotation` and new type-mode helpers), `phalcom-ast/tests/integration.rs` plus focused parser fixtures.

**Test first:** source cases for application, nested application, union, tuple/group/unit, callable precedence, labels, rest parameters, qualified names, nested `>>`, and recovery.

**Command:** `cargo test -p phalcom-ast --test integration type_annotation`

**Gate:** parser AST must match hand-built semantic test shapes; no generic declarations or records yet.

### Unit S2 — Explicit annotation result algebra

**Files/symbols:** `phalcom-semantic/src/types/annotation.rs` (`TypeFormResolution`, `resolve_type_form`, `resolve_type_annotation`), `phalcom-semantic/src/types/evidence.rs`, `phalcom-semantic/src/diagnostic.rs`, `phalcom-semantic/tests/type_annotations.rs`.

**Test first:** invalid application, unsaturated constructor, missing annotation, unresolved name, `Unknown` spelling, cancellation/budget status, and unit source node.

**Command:** `cargo test -p phalcom-semantic --test type_annotations`

**Deletion criterion:** no invalid/kind error path returns `UnknownReason::UnannotatedDeclaration`.

### Unit S3 — Generic binders and kinds

**Files/symbols:** `phalcom-ast/src/ast.rs`, `phalcom-ast/src/parser.rs`, `phalcom-semantic/src/types/parameter.rs`, `phalcom-semantic/src/types/kind.rs`, `phalcom-semantic/src/declarations.rs`, declaration/interface builders, AST and semantic focused tests.

**Test first:** class/method/alias binders, `+`/`-`/invariant, owner/index identity, kind arrows, partial application position checks, duplicate binders, and nested delimiter recovery.

**Gate:** stable `KindParameterId` design in [05](05-advanced-kinds-constraints-effects-and-proofs.md) must land before kind-polymorphic binders; this unit initially accepts explicit monomorphic kind expressions only.

### Unit S4 — Constraints and transparent aliases

**Files/symbols:** AST/parser declarations, `phalcom-semantic/src/declarations.rs`, new constraint/alias modules under `phalcom-semantic/src/types/`, interface/export fingerprint code, workspace tests.

**Test first:** upper bounds, exact finite sets, F-bound cycle budgets, alias equivalence, generic substitution, recursive alias rejection, and invalidation after alias-body change.

**Gate:** no opaque alias syntax; no recursive alias acceptance without separate ratification.

### Unit S5 — Record rows

**Files/symbols:** AST/parser record-type nodes, `phalcom-semantic/src/types/kind.rs`, `store.rs`, new row solver module, annotation lowering, metadata lowering in `phalcom-semantic/src/metadata/`, focused tests.

**Test first:** closed/open records, deterministic field order, duplicate fields, tail kind, union before tail comma, substitution, relation outcomes, and metadata round-trip.

**Gate:** `RecordRow`, `RecordTail`, and solver-local row variables from [05](05-advanced-kinds-constraints-effects-and-proofs.md) must exist. Syntax cannot land first as an untyped placeholder.

### Unit S6 — Native/source convergence and formatting

**Files/symbols:** `phalcom-type-syntax/src/lib.rs`, semantic metadata adapters, parser/source rendering seam if established by then, native metadata tests.

**Test first:** equivalent source/native inputs lower to structurally equivalent store-independent forms; native unknown remains status; source spans/recovery stay source-only.

**Gate:** convergence is below parser ASTs. Replacing both parsers with one parser is not an acceptance criterion.

## 14. Acceptance matrix

| Requirement | Evidence required |
|---|---|
| Every AST core form is source-spellable | parser snapshots plus semantic lowering tests |
| Unit spelling is coherent | `()` parser/lowering/render tests; no canonical source `Unit` |
| Precedence is stable | nested application/callable/union round trips |
| Generic identity is sound | owner/index tests across same-name binders and incremental rebuilds |
| Variance is enforced | positive/negative/invariant occurrence-path tests |
| Constructor kinds remain distinct | partial application accepted only in constructor-kinded positions |
| Error states remain distinct | missing/unresolved/invalid/cancel/budget assertions |
| Record openness is explicit | closed/tail forms, comma disambiguation, deterministic canonicalization |
| Native metadata remains trustworthy | schema parser failures explicit; no source `Unknown` leakage |
| Runtime model is unchanged | object-model invariants remain green; no new dispatch path |

## 15. Intentional gates

Implementation must stop for ratification before:

- exposing `Any` in source (`DEC-ANY-SURFACE`);
- enabling intersections or an `&` grammar;
- adding lower/equality/negative/boolean constraints;
- accepting recursive transparent aliases;
- designing opaque/newtype aliases;
- exposing kind-parameter syntax;
- adding protocol or algebraic-data declaration wrappers;
- giving effects, exits, totality, or proofs inline type-expression syntax;
- treating an annotation as a runtime expression or constructor dispatch target.

These gates are semantic boundaries, not unfinished parser chores.

## 16. What this must not preclude

Syntax and lowering must preserve extension seams for:

- public kind-parameter syntax after its ratification;
- `Any` as a proper top type after lattice policy is fixed;
- intersections and overloads after normalization/coherence rules exist;
- protocol and algebraic-data declaration wrappers reusing generic binders;
- explicit opaque/newtype aliases and guarded recursive aliases;
- separate effect/variant row grammars;
- source formatting that preserves comments and recovered syntax;
- richer metadata without weakening source recovery.

It need not preserve compatibility with `Unknown` as a source type, `out`/`in`, an ambiguous row tail without comma, arbitrary annotation evaluation, or ambient applied-class forwarding.

## 17. Take directly / Adapt / Reject

### Take directly

- existing `TypeAnnotationExpr` core shapes;
- current contextual annotation entry points and static qualified references;
- `TypeStore` kind-checked application, tuple, callable, and union construction;
- owner/index parameter identity;
- native parser's compact applications, unions, tuples, callables, `Self`, and `Never` as semantic input cases.

### Adapt

- source parser into a precedence/recovery grammar with explicit atom and invalid nodes;
- native/source overlap into shared normalized lowering below separate front ends;
- current partial application into proper-position checking;
- existing record delimiter into unambiguous `#{ fields, | R }` type syntax;
- current `TypeFormResolution` into explicit resolved/missing/unresolved/invalid terminal states.

### Reject

- claiming AST-only forms as implemented source syntax;
- `Unknown` as a source or canonical type;
- string-matched reserved atoms as final AST architecture;
- one parser shared between recoverable source and schema metadata;
- type annotations that execute runtime messages or reflection;
- `Type.currentApplication`, type-directed selector identity, `out`/`in`, and unsaturated constructors in proper-type positions.

## 18. Final normative summary

1. Source annotations are contextual, static, recoverable syntax.
2. Applications, unions, tuples, callables, `()`, `Never`, `Dynamic`, and `Self` have fixed spellings and precedence.
3. `Unknown` is never a source type; invalid and missing remain distinct.
4. `+T`, `-T`, and `T` are declaration-site variance; `out`/`in` are rejected.
5. Higher-kinded forms use `::`; partial application is valid only in constructor-kinded positions.
6. Open record rows use `#{ fields, | R }`, with mandatory comma before tail.
7. Transparent aliases normalize semantically but retain declaration provenance.
8. Parser AST contains ranges and recovery nodes, never semantic/store IDs.
9. Lowering produces explicit terminal status, provenance, and dependencies.
10. No syntax in this document changes runtime class objects, selectors, dispatch, or construction.
