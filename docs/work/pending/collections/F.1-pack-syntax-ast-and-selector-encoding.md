# Spec F.1 — Pack Syntax, AST, and Selector-Slot Encoding

Status: implementation specification. This phase is parser/AST/selector infrastructure. It must land before F.2/F.3.

## 1. Mission

Represent argument-pack contributions explicitly in the AST and enforce the ratified two-phase source grammar for both:

```text
call/subscript argument lists
Tuple construction
```

At the same time, make selector slot encoding reversible for the full Symbol label domain introduced by Spec A.

No runtime expansion occurs in F.1.

Expected primitive-floor delta: **0**.

## 2. Baseline problem

Current baseline call compilation assumes:

```rust
method_call.args: Vec<Arg>
Arg {
    expr,
    label: Option<String>
}
```

and constructs a selector before argument evaluation by mapping labels directly into:

```rust
encode_selector(name, labels, SignatureKind::Method(arity))
```

`Index` and `SetIndex` reuse the same call-shaped argument list.

This shape cannot represent:

```text
*expr
**expr
***expr
computed label expression
```

and cannot distinguish a dynamic label from a static one.

Replace it instead of layering sentinel strings into `label`.

## 3. Shared pack-item AST

Introduce an explicit shared AST type, conceptually:

```rust
pub enum PackItem {
    Positional {
        expr: Expr,
        range: SourceRange,
    },

    Labeled {
        label: PackLabel,
        value: Expr,
        range: SourceRange,
    },

    Expand {
        mode: ExpansionMode,
        expr: Expr,
        range: SourceRange,
    },
}

pub enum PackLabel {
    Static {
        text: String,
        range: SourceRange,
    },
    Computed {
        expr: Expr,
        range: SourceRange,
    },
}

pub enum ExpansionMode {
    Positional, // *
    Labeled,    // **
    Complete,   // ***
}
```

Exact names may follow `phalcom-ast` conventions.

Do not represent expansion using:

```text
label = Some("*")
label = Some("**")
label = Some("***")
```

because those are legitimate Symbol labels.

## 4. AST consumers

Use the shared pack-item representation for:

- `Expr::MethodCall`;
- `Expr::Index`;
- `Expr::SetIndex`'s user-supplied subscript items;
- A's explicit Tuple/product construction node.

Do not convert Record/Map literals to pack-item AST. Their construction semantics are already covered by A/B.

## 5. Lexer

F assumes A.1's longest-token work has landed:

```text
*** before ** before *
```

If actual HEAD still has only `Asterisk` plus parser lookahead, land the dedicated token forms now rather than parsing three adjacent `*` tokens inconsistently across grammar sites.

`...` remains a distinct token and is **not** expansion.

Keep Range `..` / `..=` behavior from C.2 intact.

## 6. Call/Tuple source phase parser

Track:

```rust
enum PackSourcePhase {
    Positional,
    Labeled,
}
```

Start in `Positional`.

### 6.1 Positional phase

Accept:

```text
expr
*expr
***expr
```

An explicit/computed label or `**expr` switches permanently to `Labeled`.

### 6.2 Labeled phase

Accept only:

```text
label: expr
[labelExpr]: expr
**expr
```

Reject:

```text
ordinary positional
*expr
***expr
```

after the transition.

### 6.3 Important `***` rule

The runtime operand of `***expr` may later contain labeled entries.

That does not switch parser phase.

Only source syntax switches phase.

Therefore this is valid:

```phalcom
target(***first, x, ***second, label: y)
```

provided all three pre-label contributions occur before the first explicit labeled-phase source item.

## 7. Multiple complete expansions

Do not preserve the older repository rule "at most one `***`".

Multiple `***` expansions are ratified.

Do not preserve the older "complete expansion excludes split expansion" rule either.

This is legal:

```phalcom
target(*prefix, ***first, *middle, ***second, label: x, **tail)
```

subject to runtime operand capability and duplicate-label checks.

What is prohibited is source-phase violation, not mixing expansion modes.

## 8. Labeled expansion phase

`**expr` begins the labeled source phase.

After it:

```phalcom
target(**a, b)       // syntax error
target(**a, *b)      // syntax error
target(**a, ***b)    // syntax error
```

but this remains legal:

```phalcom
target(**a, x: 1, **b)
```

Older drafts that said "only further `**` may follow `**`" are obsolete.

Explicit labels and further `**` contributions may interleave in the labeled phase.

## 9. Static labels

Static labels are normalized to Symbol text through the same label parser Spec A uses for Tuple/Record.

Accept the label forms A ratified, including:

```text
ordinary names
suffix forms such as ?
operator Symbols
selector-shaped forms where grammar permits/self-delimits
quoted Symbol forms
```

A call label ultimately must be a Symbol.

Do not implicitly convert String to Symbol.

## 10. Computed labels

Parse:

```phalcom
[labelExpr]: valueExpr
```

as `PackLabel::Computed`.

Do not attempt to prove its runtime Symbol value in the parser.

Static type checking, if available later, may reject known non-Symbol values, but F.2 owns the mandatory runtime guard.

## 11. Trailing closures

### 11.1 Anonymous trailing closure

Preserve the existing anonymous trailing block form if current HEAD supports it:

```phalcom
target(a, b) { ... }
```

Normalize it to one final ordinary positional `PackItem`.

Because ordinary positionals are illegal after labeled phase, reject an anonymous trailing positional closure if the parenthesized argument list already entered labeled phase.

This syntax remains implementation-contingent as previously agreed: if keeping it requires disproportionate parser complexity, it may be disabled without affecting the pack model. The AST must not require it.

### 11.2 Labeled trailing closures

Support one or more:

```phalcom
request.send(url, timeout: 10)
    onError: { e => ... }
    onSuccess: { value => ... }
```

Each is an ordinary `Labeled` pack item appended in source order after the parenthesized items.

The first labeled trailing closure enters/continues labeled phase.

Multiple labeled trailing closures are part of the specification, not parser sugar limited to one.

They contribute to selector identity exactly like labels written inside parentheses.

### 11.3 Evaluation order

Trailing closures are evaluated where they occur lexically: after all earlier parenthesized argument expressions.

## 12. Tuple construction uses the same phase rules

After A:

```phalcom
(1, 2, label: 3)
```

has the same positional→labeled phase rule as a call.

F extends Tuple construction with expansion:

```phalcom
(1, *xs, ***pack, label: 3, **fields)
```

Do not revive the older repository rule that value spread is call-only. The current collections decision explicitly includes Tuple construction.

Record/Map/Set literal expansion remains out of scope.

## 13. Subscript argument lists

Because the existing U-INDEX parser deliberately reuses call-shaped arguments, keep that property.

These are syntactically valid pack forms:

```phalcom
obj[*indices]
obj[***pack]
obj[label: value, **more]
```

Whether the receiver implements the derived subscript selector is ordinary dispatch.

`SetIndex` still appends the compiler-owned `put` labeled argument later in F.2/C.1.

## 14. Method parameter rest AST

F.1 should prepare F.3 by replacing:

```rust
ParameterDef.is_rest: bool
```

with a lane-aware field, conceptually:

```rust
pub enum RestMode {
    None,
    Positional, // *
    Labeled,    // **
    Complete,   // ***
}
```

or:

```rust
Option<RestMode>
```

Do not change binding behavior until F.3.

Parser acceptance for the new forms may land in F.1 behind pending tests, but F.3 activates semantics.

## 15. Parameter source grammar

Target grammar:

```text
fixed positional parameters
optional *positionalRest
fixed labeled parameters
optional **labeledRest
```

or complete mode:

```text
fixed positional parameters
fixed labeled parameters
***completeRest
```

Rules:

- split and complete rest modes do not coexist;
- at most one `*` rest binder;
- at most one `**` rest binder;
- `**rest` is terminal;
- `***rest` is terminal;
- no ordinary positional parameter follows `*rest`;
- no parameter follows `**rest` or `***rest`;
- labels remain ordered.

Do not implement rest type annotations in this collections unit; preserve annotation AST transparently if typing work has already landed on HEAD.

## 16. Selector encoding collision

Current baseline uses:

```text
_     positional slot
raw label text for labeled slot
*     U9 variadic marker
,     slot separator
(...) method framing
[...] subscript framing
```

This stops being injective once labels are arbitrary Symbols.

Examples of collision/parse breakage:

```text
literal label #*       vs rest marker *
literal label #_       vs positional marker _
quoted label #"a,b"    vs two slots
quoted label #"x)"     vs selector framing
```

F MUST fix this before dynamic computed labels derive selectors.

## 17. Backward-compatible component escaping

Preserve the readable encoding for safe existing labels.

Define one shared helper:

```rust
encode_label_component(symbol_text) -> String
decode_label_component(component) -> Result<String, ...>
```

Recommended canonical rule:

1. If the label is a normal safe selector-label token and is not one of the reserved slot markers, keep it unchanged.
2. Otherwise encode its UTF-8 bytes as lowercase hexadecimal prefixed by `~`.
3. Any literal label whose text starts with `~` is therefore encoded, never emitted raw.

Reserved raw components:

```text
_       positional slot
*       positional-rest marker
**      labeled-rest marker where relevant
***     complete-rest marker where relevant
```

Examples:

```text
#timeout     -> timeout
#+           -> +
#==          -> ==
#*           -> ~2a
#**          -> ~2a2a
#***         -> ~2a2a2a
#_           -> ~5f
#"a,b"       -> ~612c62
#"~raw"      -> ~7e726177
```

The exact escape prefix may differ if `~` conflicts with an already-ratified selector grammar on actual HEAD, but the chosen scheme MUST be:

- injective;
- reversible;
- delimiter-safe;
- shared by compile-time and runtime selector construction;
- incapable of colliding with rest markers.

Do not solve this with ad-hoc backslash parsing in only the dynamic path.

## 18. Selector helper API

Refactor toward structured helpers, conceptually:

```rust
encode_concrete_method_selector(
    name,
    positional_count,
    labels: &[SymbolText],
)

encode_concrete_subscript_selector(
    positional_count,
    labels: &[SymbolText],
)

decode_concrete_selector(...)

encode_rest_selector_pattern(...)
```

Existing `encode_selector(name, &[Option<String>], kind)` may remain as a compatibility wrapper while call sites migrate.

The runtime F.2 path must use the exact same component encoder as static compilation.

## 19. Safe-label definition

Do not guess safety from "contains no comma" alone.

Use the lexer/parser's own notion of a self-delimiting bare call label where possible.

At minimum, raw output must exclude:

```text
_
*
**
***
anything beginning with escape prefix
anything containing comma or framing delimiters
anything requiring quoted Symbol syntax
```

Operator labels such as `+`, `==`, `?`, `!` may remain readable if they are already legal self-delimiting bare labels.

## 20. `decode_selector` migration

Update decoding so:

```text
~HEX
```

returns the original raw Symbol label text.

`Message.labels` / dNU reflection must expose the decoded label, not the encoded transport component.

Add malformed-escape handling that is total and non-panicking. `decode_selector` already documents total behavior; preserve that property.

## 21. Rest marker decoding

Do not decode raw:

```text
*
**
***
```

as ordinary literal labels in a rest-pattern selector.

Literal Symbol labels with those texts are always escaped by the concrete label encoder.

This removes the current U9 collision without changing the visible common selector:

```text
sum(*)
```

for a rest pattern.

## 22. Parser diagnostics

Add targeted syntax errors for:

- positional item after labeled phase;
- `*` after labeled phase;
- `***` after labeled phase;
- duplicate explicit static labels;
- multiple `*rest`;
- multiple `**rest`;
- split + complete rest in one declaration;
- parameter after `**rest`;
- parameter after `***rest`.

Point at the offending token/item, not the entire call.

## 23. Tests

### 23.1 Parser positive

- multiple `*` expansions;
- multiple `***` expansions;
- `*` + `***` mixing;
- labels interleaved with `**`;
- computed call label;
- computed Tuple label;
- multiple labeled trailing closures;
- call and Tuple AST snapshot parity.

### 23.2 Parser negative

- positional after explicit label;
- positional after `**`;
- `*` after label;
- `***` after `**`;
- anonymous trailing positional closure after labeled phase;
- illegal rest-parameter ordering.

### 23.3 Selector encoding

Unit tests:

```text
ordinary identifier label round-trips unchanged
operator + round-trips
label * round-trips without rest collision
label ** round-trips
label *** round-trips
label _ round-trips
quoted comma label round-trips
quoted right-paren label round-trips
escape-prefix label round-trips
UTF-8 label round-trips
malformed encoded selector never panics
```

Pin:

```text
method rest selector `foo(*)`
```

as distinct from concrete one-label selector whose label is `#*`.

## 24. Completion checklist

F.1 is complete when:

- pack-item AST is explicit;
- calls, subscripts, and Tuple construction use it;
- parser phase rules match this spec;
- multiple `***` is accepted;
- `**` does not prohibit later explicit labels/`**`;
- computed labels are represented structurally;
- multiple labeled trailing closures parse;
- rest modes are structural AST metadata;
- selector label escaping is reversible;
- literal `#*` and rest `*` are distinct;
- existing safe selector spellings remain unchanged;
- no runtime semantics or public primitive was added.
