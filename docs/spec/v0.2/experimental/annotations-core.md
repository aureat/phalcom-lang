# Annotations (`@`) — core mechanism

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Resolves: selectors.md §4 (attributes planned, undetailed); classes.md §1/§3 (`@construct`/`@get`/`@set` relationship TBD)
- Related: ADR-0012 (selector identity), ADR-0011 (fixed slot layout), ADR-0016 (hand-written lexer/parser)
- Siblings: [annotations-contracts.md](annotations-contracts.md), [annotations-construct.md](annotations-construct.md)

## Context

selectors.md §4 reserves `@` for attributes and asserts they "compile to ordinary
method-table entries — macros over the method table, not new machinery," but
leaves the mechanism unspecified. The member-compilation loop
(`phalcom-core/src/compiler/lib.rs` L457–528) consumes exactly three member
kinds — `ClassMember::{Method,Getter,Setter}` — encodes a selector, calls
`compile_block`, allocs a `MethodObject`, and emits `Bytecode::Method`. It knows
nothing about attributes.

## Decision

Annotations are a **compile-time AST→AST desugaring pass** that runs after parse
and before the member loop. It takes a `ClassDef` carrying attributes and returns
a plain `ClassDef` whose members are only the three existing variants with
rewritten bodies. **No new bytecode, no new `Value` arm, no VM change.**

### Model: derive-macro, not runtime decorator

`@` picks the **Rust `#[derive]` model** — deterministic, compile-time,
order-independent expansion — over the Python-decorator model (arbitrary runtime
wrapping) and the Java-annotation model (inert metadata read by an external
processor). Consequences: no execution-order semantics, no runtime attribute
objects, nothing DNU-hookable. Reflection over `@` (if wanted) goes through
`Behavior`/`perform`, never through the attribute itself.

### Four-layer change

1. **Lexer**: add `Token::At` (single char). `@requires(x > 0)` lexes as `At`,
   `Identifier`, `LParen`, ordinary expr, `RParen` — argument reuses expression
   parsing.
2. **AST** (`phalcom-ast/src/ast.rs`): new `Attribute { name: String, args:
   Vec<Expr>, range }`. Add `attributes: Vec<Attribute>` to `ClassDef`
   (class-level: `@construct`, `@data`) and to `MethodDef`/`GetterDef`/`SetterDef`
   (member-level: `@requires`, `@ensures`, `@invariant`, `@get`, `@set`).
3. **Parser** (`parse_class_member`, `parser.rs` L548): before eating `static`,
   loop collecting `@`-attributes; attach to the following member.
4. **Pass**: `expand_class_attributes(ClassDef) -> Result<ClassDef,
   CompilerError>`, called at the top of the `Statement::Class` arm. All
   attribute logic lives here; builds AST from existing nodes (`Expr::Block`,
   `Expr::MethodCall`, `Statement::Let`, `Expr::Var`). Independently
   snapshot-testable on the desugared `ClassDef`.

### Open/closed: reserve user-defined attributes (D1)

The expander is a **name-keyed registry of `AttributeExpander`s**, not a Rust
`match`:

```rust
trait AttributeExpander {
    fn expand(&self, class: &ClassDef, target: Target) -> Result<Vec<ClassMember>, CompilerError>;
}
```

Builtins are the first rows. This is ~20 lines over a `match` and avoids
hardwiring "attributes are a closed set" — the assumption Rust paid a
language-level project to undo (`#[derive]` → proc-macros). It keeps open the
Smalltalk-native end state where an attribute is a compile-time metaobject
(CLOS-MOP / Racket-`syntax`). **We do not ship user-defined attributes now**; we
only avoid precluding them.

### Composition is phase-ordered (D2)

Multiple attributes on one target are not independent (all rewrite the same
body). Fixed pipeline:

1. **generate** — member-adding attributes (`@construct`, `@variant`) → raw methods.
2. **weave** — body-wrapping attributes; within weave, order is **invariant
   (outermost) → postconditions → preconditions (innermost)**, matching Eiffel.
3. **finalize** — layout/index passes (slot assignment, base-name index).

### Span hygiene (D3)

Every synthesized node inherits the triggering `Attribute.range`, so a woven-check
failure blames the `@ensures`/`@invariant` line, never phantom generated code.
Derive-macro `Span` discipline; decided up front (miette-based diagnostics per
CLAUDE.md conventions).

## Consequences

- The compiler loop at L457–528 is **unchanged**: if the pass emits only the
  three existing member variants, nothing below the pass moves.
- Two tiers emerge (see siblings): **method-table macros** (`@requires`,
  `@ensures`, `@invariant`, `@get`, `@set` — body/method only) are buildable on
  the current tree; **layout derives** (`@construct`, `@data`, `@observable` —
  grow the slot vector) are gated behind [annotations-construct.md](annotations-construct.md).
- Order-independence of contract attributes falls out of the derive model for free.

## What this precludes

Committing to derive-macro semantics forecloses ever making `@` a Python-style
runtime decorator hook without a second mechanism — no real loss, since that
pattern is an ordinary message send in a class body and needs no sigil.

> **Amended by [ADR-0054](../../../adr/0054-two-speed-ratification-annotation-decorator-tiers.md).**
> This foreclosure is scoped to the Compile/Layout tier described in this
> document. `docs/spec/v0.2/decorators/README.md`'s Install/Dispatch/Runtime
> tiers are the admitted second mechanism, gated on
> [ADR-0053](../../../adr/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)
> (satisfied) and `attribute-classes.md`'s open questions A-1–A-6 (not yet
> resolved). Read the sentence above as historical rationale for the
> Compile/Layout design, not as a live restriction on `@` as a whole.
