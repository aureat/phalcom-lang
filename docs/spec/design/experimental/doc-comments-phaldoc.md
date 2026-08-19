# Phaldoc — documentation comments (`///`, `//!`)

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Resolves: untracked — Phalcom has no documentation standard (pydoc / tsdoc / rustdoc analogue)
- Related: ADR-0012 (selector identity), ADR-0016 (hand-written lexer/parser), ADR-0008 (layered errors), ADR-0007 (Option/absence)
- Interacts-with: [annotations-core.md](annotations-core.md), [annotations-legality-grammar.md](annotations-legality-grammar.md) (the code-level `@` attribute registry — **disjoint namespace**, see §6), [typing.md](spec/design/experimental/typing.md) (erasable optional types)
- Grounding: `lexical-structure.md` §2 (comments); lexer `skip_trivia` at `phalcom-ast/src/lexer.rs:88`

## Context

Phalcom documents nothing in a machine-readable way. Rust ships `///` + markdown,
Python ships docstrings, TypeScript ships TSDoc `/** @param */`. Phalcom needs the
same: a **convention over comments** a future `phalcom doc` tool can harvest, and
that reads well in source *today* even though nothing parses it.

The request is explicitly scoped: **the parser ignores Phaldoc for now.** That is
not a limitation to design around — it is the enabling fact. `skip_trivia`
(`lexer.rs:88`) consumes any line beginning `//` as trivia, up to the newline.
`///` and `//!` both begin `//`, so **Phaldoc is already lexically inert on the
current tree** — no lexer, parser, AST, or bytecode change is required to adopt it.
It is a pure *convention* until a doc tool is built, at which point that tool
re-lexes the trivia bytes the compiler discards.

> **Latent gap (flagged, not fixed here):** `lexical-structure.md` §2 documents
> `/* block comment */`, but `skip_trivia` (`lexer.rs:80–100`) only handles `//`
> line comments — there is **no `/*` handling**. So the block form `/** … */`
> (§2) is *not* inert today; it would currently lex as `/` `*` operators. This is
> why Phaldoc's **primary and canonical form is the line comment `///`**, which is
> provably trivia today. The block form is defined but marked *contingent* (§2).

## Decision

Phaldoc is a **comment convention**, not language syntax. It has three markers, a
markdown body, and a closed vocabulary of `@`-tags. A doc tool associates a doc
block with **the item on the next non-blank, non-`///` line**, and — this is the
Phalcom-specific core — **indexes methods by their canonical comma-form selector
symbol** (ADR-0012), not by bare name.

### §1 Markers

| Marker | Role | Precedent | Inert today? |
|---|---|---|---|
| `///` | **Outer** doc — documents the item *below* it (class, method, static, getter/setter, top-level binding) | rustdoc `///`, JSDoc `/** */` | **Yes** — `//`-prefixed trivia (`lexer.rs:88`) |
| `//!` | **Inner** doc — documents the *enclosing* scope (the file/module, or a class from inside its body) | rustdoc `//!`, Elixir `@moduledoc` | **Yes** — `//`-prefixed trivia |
| `/** … */` | Block outer doc — multi-line alternative to `///` | Javadoc / TSDoc | **No — contingent** on block-comment lexing (see Context gap) |

Rule of thumb: `///` points **down** at the next item; `//!` points **up** at the
thing that contains it. A plain `//` comment is *not* Phaldoc — it stays an
ordinary implementation aside and is never harvested. This mirrors rust's
`//` vs `///` split exactly and keeps "documentation" opt-in.

### §2 Body: markdown, summary-first

The doc body is **CommonMark**. The **first paragraph is the summary** (the
one-liner a doc index shows); everything after is detail — the pydoc/rustdoc
convention. Fenced code blocks are Phalcom source. Inline `` `code` ``,
`#selector` symbol refs, and `[text](link)` all render.

```phalcom
/// Reduce a rational to lowest terms, sign-normalised so the denominator is positive.
///
/// The result is canonical: `Rational.of(2, 4)` and `Rational.of(1, 2)` are `==`.
/// Uses Euclid's `gcd` and forces a positive denominator so `==` is structural.
static of(n, d) { … }
```

### §3 Tag vocabulary (the "all annotations for documenting" set)

Tags begin a `///` line as `@tag`. Everything after the tag on that line (and any
following indented continuation lines) is the tag's payload, markdown-formatted.
The **complete** Phaldoc tag set:

| Tag | Applies to | Payload | Notes / Phalcom grounding |
|---|---|---|---|
| `@param name — desc` | method / static / block | one param, by **label position** | Name must be a real parameter; for keyword selectors, one `@param` per label (see §4). |
| `@returns desc` | method / static | the return value | Alias `@return`. Document **absence as `Option`** (`Some`/`None`), never "returns nil" — there is no `nil` (ADR-0007, values-and-absence §2). |
| `@throws Class — when` | method / static | an `Error` **subclass** + trigger | Only `Error` subclasses are throwable (ADR-0008); a value-failure is documented on `@returns` as `Result` (`Ok`/`Err`), not here. Alias `@raises`. |
| `@example` | any | fenced Phalcom block on the following lines | Runnable snippet; the future doc tool may execute these as doctests (rustdoc model). |
| `@see #sel / Class / [text](url)` | any | a cross-reference | Prefer the **selector symbol** form `#of(_,_)` so links survive `foo` vs `foo(_)` (§4). |
| `@since version` | any | a version string | API-stability marker; pairs with `@deprecated`. |
| `@deprecated reason` | any | why + what to use instead | Advisory only — no compile effect (Phaldoc never changes semantics). |
| `@author name` | file (`//!`) / class | attribution | Discouraged in library code (rust omits it); allowed for corpus/benchmark provenance. |

**Reserved, not in v1** (defined so tools reject typos rather than silently drop):
`@typeparam` (awaits generics, typing.md), `@invariant-doc`, `@group`,
`@internal`. A tool encountering an unknown `@tag` **warns** (never errors — Phaldoc
is inert by construction) and treats the line as prose.

### §4 The Phalcom-specific rule: docs key to the *selector*, not the name

This is what makes Phaldoc Phalcom's standard rather than a JSDoc reskin. Under
ADR-0012, `foo`, `foo()`, `foo(_)`, and `move(_,to,duration)` are **four distinct
methods** with four distinct selector symbols. Therefore:

1. A doc block attaches to the **selector** of the item below it, so overloaded-by-
   arity methods each carry their own docs — a name-keyed doc model (JSDoc, pydoc)
   *cannot* express this and would collapse them.
2. `@param` entries map **positionally to the selector's labels**. For a keyword
   selector `move(_,to,duration)` the params are the receiver-slot `_`, `to`, and
   `duration`, documented in that order.
3. `@see` and any cross-reference SHOULD use the canonical **comma-form selector
   symbol** (`#move(_,to,duration)`, per ADR-0012/selectors §1), which is stable
   and unambiguous, over a bare name.
4. When a doc block is **detached** from its item, an explicit first line
   `/// selector: move(_,to,duration)` pins the target. (Optional; adjacency is
   the default.)

### §5 Placement legality

| Placed above… | `///` documents | `//!` documents |
|---|---|---|
| top of file | first item, if adjacent | **the file/module** |
| inside a class body, first lines | — | **the class** |
| a `class`/method/`static`/getter/setter/`let`/`var` | that item | (n/a) |
| an operator method (`+`, `==`, …) | that operator's selector | — |

Blank lines break adjacency: a `///` block separated from the next item by a blank
line documents nothing (a lint-worthy "dangling doc", rust's `unused_doc_comments`).

## §6 Interaction hazard — doc-tag namespace ⊗ code `@`-attribute registry

**The crown-jewel check.** Phalcom already spends `@` on **executable, compile-time
attributes** — `@requires`, `@ensures`, `@invariant`, `@construct`, `@get`, `@set`,
`@data`, `@observable` ([annotations-core.md](annotations-core.md),
[annotations-legality-grammar.md](annotations-legality-grammar.md)). Phaldoc also
uses `@`. Two `@`-worlds now coexist:

- **Lexically:** zero collision — a code `@requires` is a real `Token::At` in the
  grammar; a Phaldoc `@param` lives inside `//`-trivia the lexer never tokenises
  (`lexer.rs:88`). This is exactly how TSDoc `@param` coexists with TypeScript
  decorators `@Component`. Verified inert, not asserted.
- **Semantically / for the reader:** a real risk. If Phaldoc reused `@requires` or
  `@ensures` as *doc* tags, one spelling would mean two things (an executable
  contract vs a prose note). **Resolution: the Phaldoc tag namespace is disjoint
  from the code-attribute registry.** Phaldoc defines **none** of the contract or
  construct attribute names as tags. Preconditions are **not** documented with a
  Phaldoc tag at all: the `@requires`/`@ensures`/`@invariant` attributes are
  *already source*, so `phalcom doc` **harvests the real attributes** and renders
  them in the generated page. Contracts are executable, not commentary — Phalcom's
  one genuine advantage over Javadoc `@throws`-by-prose, and Phaldoc leans into it.

Standing obligation: any future Phaldoc tag must be checked against the code
`@`-attribute registry and rejected if it collides. The two `@` vocabularies must
stay **name-disjoint** even though they are lexically separated.

## §7 Other hazard checks (rubric §1,§2,§4)

- **Soundness:** none — Phaldoc emits no code, changes no dispatch, allocates
  nothing. Worst case is a *wrong doc*, caught only by the doc tool's warnings.
- **Dispatch / representation impact:** zero. No selector, no `Value` arm, no slot.
- **Preclusion:** choosing "convention over comment" forecloses making a doc block
  a *runtime-reflectable* object for free (Elixir/Smalltalk expose `@doc` and method
  comments at runtime). If that is later wanted, it is a **separate** mechanism — a
  `///`-harvesting compiler pass that stashes the string on the `MethodObject`,
  reachable via `Behavior`/`perform` — layered on top without changing Phaldoc's
  surface. Not precluded, just not free. Nothing here blocks it.

## §8 Documenting the code annotations — the "contract view"

§6 established *that* code `@`-attributes are harvested rather than re-documented.
This section pins *how*, for the full experimental annotation surface
([annotations-contracts.md](annotations-contracts.md),
[construct-derive.md](../../../work/pending/ctor/notes/construct-derive.md),
[annotation-paradigm-bridges.md](annotation-paradigm-bridges.md),
[annotations-legality-grammar.md](annotations-legality-grammar.md)).

### §8.1 The layering rule (the one principle)

Two annotation worlds document one member, at two altitudes:

- **Phaldoc prose (`///`) = intent.** *What* the method is for and *why* — the part
  a machine cannot infer. Free text, markdown.
- **Code `@`-attributes = machine-checkable facts.** *Which* inputs are legal
  (`@requires`), *what* is guaranteed (`@ensures`), *what* holds always
  (`@invariant`), *what* type a param is (`x @ Type`), *what* is generated
  (`@construct`, `@get`/`@set`, `@data`).

**They must never restate each other.** Writing `/// @param amount must be
positive` next to `@requires(amount > 0)` is drift waiting to happen — the prose
can rot while the contract stays true. **Rule:** when a fact is expressible as an
attribute, it lives *only* in the attribute; Phaldoc prose never duplicates it. A
prose `@param`/`@returns`/`@throws` that merely echoes a harvested contract is a
**lint** (`redundant_doc_of_contract`), and the **attribute is authoritative**.

This is **Eiffel's contract view / "short form"**: Eiffel has *no* precondition
doc-tag because the `require`/`ensure`/`invariant` clauses *are* the spec, and its
`short` tool extracts signature + contracts + header comment into the published
view. D-contract-1 (contracts reflectable on `MethodObject`, `Method>>contracts`)
is exactly the hook that lets `phalcom doc` do the same.

### §8.2 What `phalcom doc` renders per method (harvest order)

For each method the generator emits a **contract view**, assembling the two layers:

1. **Selector** — canonical comma-form (§4), e.g. `deposit(_)`, `move(_,to,duration)`.
2. **Summary** — first paragraph of the `///` block (§2).
3. **Signature + types** — params; each `x @ Type` (which desugars to
   `@requires(x.is(Type))`, D-contract-2) rendered as a type row.
4. **Requires** — every `@requires` predicate, source-printed.
5. **Ensures** — every `@ensures` predicate, with `old(...)` shown verbatim.
6. **Invariant** — the class-level `@invariant`(s), conjoined
   (annotations-contract-semantics §Multiple), noted on each *public* method they
   guard (they fire outermost-only, so private/nested sends are not annotated).
7. **Raises** — see §8.3.
8. **Detail + `@returns` + `@example`** — the rest of the prose.
9. **Checked-in badge** — see §8.4.

### §8.3 `@throws` is *derived* from contracts, not hand-written

The contract errors `PreconditionError`, `PostconditionError`, `InvariantError`
are `Error` subclasses (ADR-0008). The doc tool **auto-populates a method's Raises
section** from the *presence* of contracts:

- a `@requires` ⇒ `Raises PreconditionError when the precondition is violated`;
- an `@ensures`/`@invariant` ⇒ the corresponding `Post`/`InvariantError`.

The author never writes `@throws PreconditionError` — that would be exactly the
Javadoc `@throws`-drift §6/the layering rule forbid. Author-written `@throws`
therefore documents **only domain errors** the *body* raises (`throw
InsufficientFunds`), and is unioned with the derived contract errors.

### §8.4 Enforcement mode is a rendering flag, not a spec change

Contracts are stripped by compile mode (annotations-contract-semantics §Release):
`@ensures`/`@invariant` are gone in `release`, `@requires` survives (Meyer demand
contracts). The **spec is documented regardless** — the doc tool reads the
*reflectable metadata* (D-contract-1), which is retained in every mode, so the
contract view is complete even against a `release` build. Each clause carries a
**checked-in** badge (`debug` / `debug+release`) so a reader never assumes a
stripped `@ensures` is enforced in production. Docs show the *contract*;
the badge shows the *enforcement*.

### §8.5 Layout & bridge annotations — document the *generated* surface

Layout-tier attributes (construct-derive.md, annotation-paradigm-bridges.md)
generate members; Phaldoc documents **what they generate**, the way rustdoc shows
`#[derive(...)]`-produced impls:

| Attribute | Doc renders | Phaldoc source of prose | Hazard surfaced |
|---|---|---|---|
| `@construct` | the generated `new(x:,y:)` constructor, params in **field-declaration order** | `///` on the class | **field order is API** (construct.md) — the doc states the calling convention and warns reordering fields is a breaking change |
| `@get`/`@set` on `var _x` | the derived accessor pair `x` / `x=(_)` | `///` on the **field decl** flows to both accessors | `@get(priv)` is advisory naming only (never enforcement) — doc labels it, doesn't claim privacy |
| `@data` | derived `==`, `hash`, `with(...)` | `///` on the class | `==`+`hash` are a pair (equality ladder) — doc lists both, never a lone `==` |
| `@sealed` | the **closed** subclass set + a "match is exhaustive over {…}" note | `///` on the class | adding a variant is a breaking change for every `match` |
| `@variant Circle(radius:)` | the case as a constructor-shaped entry | `///` on the variant | — |
| `@observable`/`@computed` | the reactive read/subscribe surface (`c::total.subscribe`) | `///` on the field/computed | glitch policy (eager vs batched) is a documented semantic |

Since attributes are **class-member-only** (annotations-legality-grammar.md §Grammar),
the harvester only ever scans the class-member attribute list — no expression- or
statement-level attribute positions to chase.

### §8.6 `@example` blocks demonstrate contracts (doctest ⊗ contract)

An `@example` that violates a `@requires` **raises `PreconditionError`** at runtime.
So examples double as executable contract demonstrations: a doctest runner
(rustdoc model) can assert both the happy path *and* that an out-of-contract call
raises. This is only sound because contract predicates are **pure**
(annotations-contract-semantics §Purity) — a doctest re-run has no side effects.

### §8.7 Hazard recap for this section (rubric §1,§2,§4)

- **Soundness:** still none — harvesting reads reflectable metadata and trivia; it
  emits documentation, not code. A stale prose tag is the only failure, caught by
  the `redundant_doc_of_contract` lint.
- **Dispatch/representation impact:** zero. The harvest is a read of the
  `Method>>contracts` side table (already built for property testing) plus the
  `///` trivia bytes.
- **Preclusion:** documenting the *generated* surface (not the attribute source)
  forecloses nothing; if a future attribute generates members Phaldoc can't yet
  name, it renders the raw derive and warns — never silently omits.

## Precedent, with consequence

- **Rust (`///` + markdown, `# Examples` headers, *no* `@tags`):** deliberately
  avoided `@`-tags so `#[attr]` stays visually the only "annotation." Phalcom can't
  copy that resolution — its attribute sigil is `@`, not `#` — so it takes rust's
  *markers* (`///`/`//!`) but keeps `@`-tags, paying the §6 disjointness tax instead.
- **TSDoc / JSDoc (`/** @param */`):** proves `@`-doc-tags coexist fine beside code
  `@`-decorators; the cost is a spec devoted to disambiguating the two, which §6 is.
- **Pydoc / Sphinx (docstrings, `:param:`):** docs are *runtime objects*
  (`__doc__`). Phalcom declines that (see Preclusion) to keep docs zero-cost and
  out of the object graph.
- **Javadoc (`@throws`):** documents exceptions as prose that drifts from code.
  Phaldoc's §6 harvest-the-real-`@requires` move is the fix for exactly that drift.

## What this precludes

Only what §7 names: runtime-reflectable docs are not free under the convention
model. Everything else — a `phalcom doc` generator, doctest execution of
`@example` blocks, IDE hover from harvested trivia — is additive and unblocked.
