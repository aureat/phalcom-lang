# Spec A.1 — Product Syntax and AST Foundation

Status: implementation specification. This phase is intentionally front-end-heavy and must leave the repository green on its own. It establishes the syntax model that later phases lower into the new Unit/Tuple/Record runtime, while preserving the currently working positional Tuple path as a temporary compatibility bridge.

## 1. Mission

Replace the old parser-only Tuple desugaring model with an explicit product syntax representation capable of expressing Phalcom's ratified two-lane Tuple and Symbol-field Record forms. Add the missing lexical forms required by product labels—most importantly `#{...}` and quoted/operator Symbols—without implementing product runtime construction yet. After this phase the parser must retain enough structure to distinguish `()`, positive positional Tuples, labeled Tuples, and Records; the compiler must continue compiling the currently supported positive positional Tuple literals through the old runtime path so the workspace remains green.

This phase is not allowed to invent typing, reflection, destructuring, expansion, slicing, or Record update semantics. It is a syntax/AST foundation with a narrow compatibility seam.

## 2. Authority and precedence

The normative semantic sources for this phase are the supplied `collections-next/tuple-record-and-symbols-spec.md`, especially §§2–7, 9–10, 23–25, 34, and 39, and `collections-next/product-normalization-and-unit-spec.md`, especially §§5–9 and 38. The old `collections` archive is advisory only; its diagnostic examples are useful, but any old rule that conflicts with `collections-next` is superseded.

Repository planning documents such as `docs/forge/units/U-COLL/plan.md` and `docs/forge/units/U-COLLTYPES/plan.md` describe the implementation that exists today, not the target semantics. In particular, `U-COLL` lowers `(a, b)` directly to `Tuple.fromList(...)`; this phase begins retiring that architectural choice.

## 3. Repository baseline to verify before editing

Re-read actual HEAD before implementation. At the repository state inspected for this specification:

- `phalcom-ast/src/token.rs` already contains `NameSymbol(String)` and `SelectorSymbol { name, labels }`, ordinary brace/paren/bracket tokens, `Asterisk`, `Question`, and `DotDotDot`, but has no Record opener token and no quoted-Symbol token.
- `phalcom-ast/src/lexer.rs` routes every `#` through `scan_symbol()`. `#{` therefore cannot currently form a Record opener, and `#"..."` is not a Symbol literal. The scanner is hand-written and already contains the `#!` shebang carve-out.
- `phalcom-ast/src/ast.rs` has `Expr::Symbol(Box<SymbolExpr>)` and `SymbolLiteralKind::{Name, Selector}`, but no Tuple/Record literal AST nodes. `Argument.label` remains `Option<String>` and is call-specific; do not reuse it as the product-label representation.
- `phalcom-ast/src/parser.rs::parse_primary()` dispatches `Token::LParen` to `parse_paren_or_tuple()`. That helper currently detects a top-level comma and immediately rewrites a Tuple into `Tuple.fromList(List.new().add(...))`. It cannot retain labels or distinguish an empty product semantically.
- `phalcom-core/src/compiler/lib/expr.rs` exhaustively matches `Expr`, so adding new AST variants requires compiler handling in the same phase even if final lowering is deferred.
- Current positive positional Tuple goldens under `phalcom-core/tests/lang/collections/` must remain green throughout A.1.

Run `./scripts/verify.sh` before the first edit and record the baseline result. The repository's gate runs `cargo test --workspace` and `cargo clippy --workspace --all-targets`; use `./scripts/verify.sh --full` before declaring the phase complete.

## 4. Required syntax model

### 4.1 Record opener

Add a dedicated token for contiguous `#{`. Name it `RecordLBrace` unless a better established repository naming convention is found on HEAD. The lexer must recognize `#{` before generic `#` Symbol scanning. The close delimiter remains ordinary `RBrace`.

This token is deliberately distinct from `{`. It must not pass through the existing block/Map brace-disambiguation branch. `#{...}` is Record syntax exclusively. Ordinary `{...}` remains governed by the Map/Set/block work in later specs.

The `#!` offset-zero shebang carve-out must remain unchanged. A source starting `#!` must never be reclassified as a Symbol or Record opener.

### 4.2 Quoted Symbols

Extend the Symbol literal representation with a quoted arbitrary spelling. The recommended AST extension is:

```rust
pub enum SymbolLiteralKind {
    Name(String),
    Selector {
        name: String,
        labels: Vec<Option<String>>,
    },
    Quoted(String),
}
```

Add a corresponding token such as `Token::QuotedSymbol(String)` rather than misusing `NameSymbol`: a quoted Symbol may contain whitespace, colons, or other text that is not a name.

`#"..."` must be lexed as one atomic Symbol literal. Its body should follow the language's ordinary string escaping rules where practical, but it must not perform string interpolation. In particular, the scanner must not recursively parse `\(...)` as interpolation merely because ordinary string literals do. The exact future Symbol Unicode-normalization and conversion API remain deferred.

Required examples include:

```phalcom
#"label:withAColon"
#":"
#"a symbol with spaces"
```

### 4.3 Operator-like Symbol literals

The current Symbol scanner already recognizes a subset of operator Symbols. Extend it only as necessary to cover the product specification's required symbolic identities, including at minimum:

```phalcom
#*
#**
#***
#?
#+
```

Use longest-match handling for the star family. `#***` must be one Symbol literal, not `#*` plus two ordinary stars.

Selector-shaped Symbols should continue through the existing `SelectorSymbol` model. If operator bases such as `#+(other)` are not accepted by the current scanner, extend the existing selector-suffix scanner rather than writing a second canonicalization grammar. Full selector-literal grammar remains deferred; this phase must achieve parity between the selector shapes the front end accepts with `#` and those it accepts as bare product-label heads.

### 4.4 `**` and `***` tokenization

Product labels include bare `**:` and `***:`. The preferred foundation is to add `DoubleAsterisk` and `TripleAsterisk` tokens now, scanned by longest match (`***` before `**` before `*`). A.1 must not assign expansion semantics to those tokens; Spec F owns expansion. They are admitted here only so product labels and Symbols have stable lexical identities and so F does not need to alter label syntax later.

If HEAD has acquired these tokens by implementation time, reuse them rather than adding duplicates.

## 5. AST design

Product labels are not call labels. Call labels currently use `Option<String>` because selector construction has historically treated labels as source names. Tuple/Record labels are first-class Symbols and may be explicit or computed. Introduce a dedicated product-label AST family.

Recommended shape:

```rust
#[derive(Debug, Clone)]
pub enum ProductLabel {
    Static {
        symbol: SymbolLiteralKind,
        syntax: ProductLabelSyntax,
        range: SourceRange,
    },
    Computed {
        expr: Box<Expr>,
        range: SourceRange,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductLabelSyntax {
    Bare,
    ExplicitSymbol,
}

#[derive(Debug, Clone)]
pub struct TupleLiteralExpr {
    pub entries: Vec<TupleLiteralEntry>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum TupleLiteralEntry {
    Positional {
        expr: Expr,
        range: SourceRange,
    },
    Labeled {
        label: ProductLabel,
        value: Expr,
        range: SourceRange,
    },
}

#[derive(Debug, Clone)]
pub struct RecordLiteralExpr {
    pub fields: Vec<RecordLiteralField>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct RecordLiteralField {
    pub label: ProductLabel,
    pub value: Expr,
    pub range: SourceRange,
}
```

Add `Expr::TupleLiteral(Box<TupleLiteralExpr>)` and `Expr::RecordLiteral(Box<RecordLiteralExpr>)` and extend `Expr::range()` exhaustively.

The AST intentionally preserves syntax family even for the two zero forms. `()` should parse to an empty `TupleLiteralExpr`; `#{}` should parse to an empty `RecordLiteralExpr`. A.3 will normalize both to `Value::Unit`. This separation is required so diagnostics can still say that the author wrote an empty Record even though the runtime value is Unit.

Do not add an `Expr::Unit` parser node in A.1. Unit is a semantic normalization result, not a reason to erase source syntax at parse time.

## 6. Product-label grammar

Implement one shared parser path for Tuple labels and Record fields. Conceptually it recognizes:

```text
label-head ':' expression

label-head :=
    bare-name-or-operator
  | bare-selector-shape
  | explicit-symbol-literal
  | '[' expression ']'
```

The resulting semantic domain is always Symbol, but A.1 only records syntax; runtime validation of a computed label belongs to A.3.

### 6.1 Bare static labels

A bare label head produces `ProductLabel::Static { syntax: Bare, ... }`. It must cover the currently supported identifier domain and the ratified operator/suffix forms, including examples such as:

```phalcom
name: value
Type?: value
*: value
**: value
***: value
?: value
+(other): value
method(_,_,a,b): value
```

Do not hard-code a second selector canonicalizer in `parser.rs`. Factor or reuse the existing selector-shape parsing logic where possible. The critical invariant is that a bare selector-shaped label and the equivalent explicit selector Symbol compile to the same interned Symbol in A.3.

### 6.2 Explicit Symbol labels

An already-tokenized Symbol immediately followed by `:` is a static label:

```phalcom
#name: value
#":": value
```

Preserve `ProductLabelSyntax::ExplicitSymbol` for diagnostics; semantic identity must later be identical to an equivalent bare label.

### 6.3 Computed labels

`[expression]: value` is a computed product label. The brackets are label syntax and do not construct a List. Therefore the parser must decide this form before routing the leading `[` through `parse_list_literal()`.

A practical implementation is a non-consuming lookahead helper that finds the matching top-level `]` while respecting nested delimiters and verifies that the next significant token is `Colon`. If true, parse the contents as a normal expression and store that expression directly in `ProductLabel::Computed`; otherwise parse `[` as the ordinary List literal it already denotes.

Do not use speculative parsing that mutates parser state and then hand-rewinds unless the parser already has a disciplined checkpoint abstraction. A delimiter-aware lookahead is easier to reason about and test.

The following must remain distinct:

```phalcom
([x],)        // one positional List value in a Tuple
([x]: value,) // one computed Symbol label, assuming x evaluates to Symbol
```

## 7. Tuple parsing

Rewrite `parse_paren_or_tuple()` around the explicit AST instead of constructing a List/send chain in the parser.

Required classification:

```text
()                  -> empty Tuple syntax node
(expr)              -> grouping, exactly as today
(expr,)             -> one positional Tuple entry
(a, b)              -> two positional Tuple entries
(label: value,)     -> one labeled Tuple entry
(a, label: value)   -> positional lane then labeled lane
```

Trailing commas are allowed. A one-component Tuple requires the comma when the first component is positional; `(x)` remains grouping forever.

Tuple source order is constrained by the two-lane model. Once the parser has consumed the first labeled entry, any later non-labeled entry is a syntax error. The diagnostic should describe the source boundary, for example: `positional Tuple entries cannot follow labeled entries`. Do not silently reorder source entries.

A.1 need not decide all duplicate-label timing. It must preserve enough static label structure for A.3 to detect canonical duplicates before execution. It may reject trivially identical duplicates early, but do not create a parser-local canonicalization algorithm merely for duplicate detection.

## 8. Record parsing

Add `Token::RecordLBrace` to `parse_primary()` and parse a `RecordLiteralExpr` directly. Records have no positional entries. Every field must use the shared product-label grammar.

Required forms include:

```phalcom
#{}
#{ name: "Ada" }
#{ #"content:type": mime }
#{ [fieldSymbol]: value }
```

Allow a trailing comma. Reject an expression without `label-head :` inside a Record; do not reinterpret it as a Set or Map element. Duplicate-field identity is preserved for A.3 validation exactly as with Tuple labels.

Record parsing must not enter the existing `Token::LBrace` block/Map branch. Conversely, ordinary `{...}` behavior must remain byte-for-byte compatible in parser snapshots unless a test was previously relying on invalid `#{` input.

## 9. Transitional compiler bridge

Adding new `Expr` variants makes `phalcom-core/src/compiler/lib/expr.rs` non-exhaustive. A.1 must add explicit compiler arms, but final product lowering is deliberately A.3.

For `Expr::TupleLiteral` containing one or more positional entries and zero labeled entries, compile through a small compatibility helper that emits the same effective construction as the old parser desugaring: build the temporary List in source order and send `Tuple.fromList(_)`. This exists only to keep current Tuple programs and goldens working while A.2 changes the runtime underneath them.

For these forms, the bridge must preserve each entry's original source range as far as the existing compiler representation permits.

For any newly expressible form whose runtime does not yet exist—empty Tuple, labeled Tuple, any Record—return a deliberate `CompilerError` such as `ProductLiteralNotLoweredYet`, carrying the expression range. Never panic, `todo!`, or emit a half-valid send. Parser tests in A.1 should exercise those forms without requiring successful execution.

Document this compiler branch as temporary and tag it for removal by A.3. There must be exactly one compatibility path, not duplicated product-building logic that later has to be unwound.

## 10. Diagnostics

A.1 owns syntax diagnostics only. Required negative cases include:

- positional Tuple entry after the labeled phase has begun;
- malformed `#{...}` field lacking `:`;
- missing `]` or `:` in a computed label;
- malformed quoted Symbol;
- malformed selector-shaped bare label;
- missing closing `)` / `}`;
- a bare `#` that is neither Record opener nor valid Symbol.

Use existing `SyntaxError` / `SyntaxErrorKind` machinery and precise `SourceRange`s. Do not introduce runtime `KeyError`, `IndexError`, duplicate-field runtime errors, or computed-label type errors in this phase.

## 11. Tests

### 11.1 Lexer tests

Extend the existing `phalcom-ast` lexer snapshot/unit lane with at least:

```text
#{ }
#"label:withAColon"
#":"
#*
#**
#***
#?
#+
#+(other)
```

Pin longest-match behavior for `***`, `**`, and `*`, both bare tokens and after `#`. Pin `#!` at byte zero as shebang and verify `#{` after ordinary source position is Record opener.

### 11.2 Parser tests

Add focused parser cases for:

- `()` as empty Tuple syntax node;
- `(x)` as grouping;
- `(x,)` as singleton positional Tuple;
- `(name: x,)` as singleton labeled Tuple;
- mixed positional/labeled Tuple with the lane boundary;
- a negative positional-after-labeled case;
- `#{}` and positive Record literals;
- bare, explicit, quoted, operator, selector-shaped, and computed labels;
- nested List/Tuple/Record expressions inside values;
- computed-label versus List-literal disambiguation;
- ordinary block and Map brace cases to prove `#{` did not perturb `{`.

AST snapshots must preserve ranges and source-form distinctions required for later diagnostics.

### 11.3 Compatibility tests

Keep every existing positive positional Tuple golden green, especially the collection fixtures around literal construction, equality, Map-key use, and nested collections. Add one compiler test proving the A.1 compatibility bridge still reaches the old Tuple runtime for `(1, 2)`.

New labeled/Record execution goldens do not belong here; they begin in A.3.

## 12. Expected write set

Primary files:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/**
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/error.rs   # only if a named transitional error is added
phalcom-core/tests/**                    # compatibility-only tests
```

Do not touch heap representation, `Value`, `core.ph`, Universe class creation, primitive registration, floor census, Map/Set runtime, or iterator code in A.1.

Exhaustive downstream AST consumers such as LSP visitors may fail to compile when the new `Expr` variants land. Fix only the mechanically required exhaustive matches so they preserve existing behavior; do not design product hover/type/reflection features in this phase.

## 13. Completion gate

A.1 is complete only when all of the following hold:

1. the lexer recognizes Record opener and the required Symbol forms without regressing shebangs or ordinary punctuation;
2. Tuple and Record syntax survive in the AST rather than being erased into method sends;
3. grouping and ordinary brace behavior remain unchanged;
4. source positional-to-labeled ordering is enforced for Tuple syntax;
5. currently valid positive positional Tuple programs still execute through the temporary compiler bridge;
6. unsupported new runtime forms fail as explicit compiler errors rather than panics;
7. no typing, reflection, expansion, slicing, or Record-update semantics have been smuggled into the phase;
8. `./scripts/verify.sh --full` is green.

The implementation report should include the commit SHA, lexer/token additions, AST shapes, parser classification table, exact transitional compiler seam, new positive/negative tests, and the final verification tail.
