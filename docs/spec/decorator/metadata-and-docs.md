# Passive metadata and the documentation boundary

- Status: **Mechanism built; one proposal (`@Deprecated`); one boundary
  defended.** Retention/reflection ground truth in
  [on.md](../v0.2/decorators/on.md); the documentation system in
  [doc-comments-phaldoc.md](../v0.2/experimental/doc-comments-phaldoc.md)
  (Proposed).

## Passive metadata — the honest Java/C# case, built

An `Attribute` subclass with no tier and no hook is inert, retained,
reflectable (`Engine.attributesOfType(Author).first.name`). This works on
HEAD end-to-end (fixture `decorators_attribute_retention.ph`), modulo the
mechanism divergences (labeled args, `inherited:` — [mechanism.md](mechanism.md)).
It is the zero-cost tier: stripping a passive attribute leaves bytecode
identical; only the retained instance disappears. Store representation keeps
the un-annotated case free (empty `Vec`, no allocation).

Design rule for library authors, worth stating once: **passive metadata is
for consumers you can name.** An attribute nobody reads is schema without a
reader — the Java ecosystem's graveyard of decorative annotations is the
precedent, and its consequence (annotation cargo-culting) is the thing the
"attr.unknown is a hard error" posture already leans against.

## `@Deprecated(reason:, since:)` — proposed (PDR candidate)

The strongest passive-metadata candidate because its consumers already have
homes:

1. **Now (retention only):** `@Deprecated(reason: "use v2", since: "0.2")`
   on any member or class; reflection exposes it; the REPL/doc tooling can
   render it. Pure library — an `Attribute` subclass in `core.ph`, zero new
   mechanism, land-anytime.
2. **Later (warn tier):** when diagnostics grow warnings
   ([compiler-directives.md](compiler-directives.md)), a compile-time
   use-site warning keyed off the retained instance. The attribute's data
   shape is designed for that future now (reason + since strings) so the
   upgrade is additive.
3. **Boundary with Phaldoc:** the `///` tag `@deprecated` remains prose for
   *humans reading docs*; the attribute is the *machine-readable* fact.
   The doc generator should render the attribute and lint prose that
   duplicates it — the same authoritative-attribute rule as the contract
   view (`redundant_doc_of_contract` precedent).

## The documentation boundary — docs are not decorators

The user-facing question this file exists to answer: should Phalcom have
rustdoc/tsdoc-style *documentation decorators*? **No — and the corpus already
decided this correctly.** Phaldoc (`///` outer, `//!` inner, CommonMark,
`@param`/`@returns`/`@throws`/`@example`/`@see`/`@since` tags) is comment
trivia: lexically inert, zero AST/bytecode presence, harvested by tooling.
The reasons this is right, not merely chosen:

1. **Docs must be free.** A doc-as-decorator (`@doc("…")`) is a retained
   heap object per documented member — Elixir/Smalltalk pay this for runtime
   `@moduledoc` reflection; Phalcom declines (doc-comments §7's explicit
   trade). Ten thousand documented members must cost nothing at runtime.
2. **The selector rule.** Docs key to *selectors*, not names — `foo`,
   `foo(_)`, `move(_,to,duration)` each carry their own block. Phaldoc §4
   handles this; a decorator surface would have to reinvent it.
3. **The disjointness contract (COLL-5).** Phaldoc's `@`-tags live in
   comments and never collide with code decorators lexically; §6's standing
   obligation — every new Phaldoc tag is checked against the decorator
   registry — is the semantic guard. This tree adds the reverse obligation:
   **every new decorator name is checked against the Phaldoc tag vocabulary**
   too. One list, two directions.
4. **Where the two systems touch, the attribute wins.** The contract view
   renders real `@requires`/`@ensures`/`@invariant` attributes (needs DEF-11's
   metadata emission — [contracts.md](contracts.md) plan §3); auto-derives
   `@throws` for contract errors; lints prose restating machine facts. This
   is Eiffel's short-form tool rendered in Phalcom terms, and it is the
   design's best idea: *documentation states intent; attributes state facts;
   neither repeats the other.*

One latent bug inherited from the doc spec, restated so it isn't lost:
`/* block comments */` are documented in lexical-structure.md but
`skip_trivia` never lexes them — `/** … */` is not inert today. Either lex
block comments or strike them from the lexical spec; until then `///` is the
only doc form. (Owner: lexer, not this tree; flagged.)

## `@Author` and friends

Legal, built, discouraged in library code (doc spec's own posture) — allowed
where provenance matters (benchmark/corpus files). No spec needed beyond the
mechanism; listed to show the passive tier needs no per-name blessing.

## What this precludes

- Runtime-reflectable documentation without a separate, explicit future
  mechanism (a harvest pass stashing strings on `MethodObject` — possible,
  additive, ungated by anything here).
- A Phaldoc tag and a decorator sharing a name with different meanings —
  the two-direction check above is the guard, and it is a review-time
  obligation, not tooling, until someone builds the check.
