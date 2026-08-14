# Semantic Consumers and Their Policies

## Shared truth, different policies

A fact can be useful to one consumer and insufficient for another. Keep the shared fact;
put the policy in the consumer.

## Hover

Hover should answer the exact source target under the cursor and may render:

- declaration identity/category;
- inferred runtime shape;
- declared/inferred language type when available;
- defining class/module;
- selector/callable signature;
- confidence/provenance;
- relevant documentation.

Do not highlight/render an entire enclosing method when the cursor targets one parameter,
selector fragment, local, field, or literal.

## Completion

Completion needs context:

- lexical bindings visible at offset;
- receiver semantic fact;
- dispatch side;
- current lexical class for visibility;
- super/implicit-self context;
- union receiver policy;
- static versus computed selector/pack state.

For a union receiver, distinguish:

- members safe on every alternative (intersection of capabilities);
- members available on some alternatives (optional/lower-ranked, if UI policy allows).

The checker may reject the latter even if completion shows them.

## Definition/references/rename

These should be identity-driven.

Rename requires more than references:

- target identity;
- declaration/reference set;
- lexical collision analysis;
- selector identity transformation if renaming methods;
- module/import alias semantics;
- dynamic/reflection limitations.

Never implement semantic rename as text replacement.

## Signature help

Use resolved/candidate callable identity and exact selector/argument mapping. Dynamic packs
may force partial information.

Future generic typing can supply substitutions and expected parameter types.

## Semantic tokens

Prefer resolved semantic category where available:

- local/parameter/import/class/module/field/method/type parameter etc.

Fall back to syntax for incomplete code. Token classification must not require expensive
whole-project solving if the needed category is lexical.

## Diagnostics

Diagnostics consume facts plus a policy/obligation.

Good diagnostic structure:

```text
primary claim
primary source range
expected requirement
observed fact
reason/provenance chain
secondary ranges
repair guidance/code action where safe
```

An inference fact alone is not necessarily an error.

## Lints

Every lint should declare the weakest semantic tier required:

```text
token
syntax
binding
occurrence
local-flow
dispatch
type
module/project
proof/effect
```

This helps correctness and performance. If a lint needs type proof, it should suppress or
report uncertainty according to lint policy rather than guess from heuristics.

## Refactorings

Refactorings need stronger guarantees than diagnostics because they modify code.

Prefer refusing a transformation when semantic identity is ambiguous over applying a
plausible but unsafe edit.

Useful shared facts:

- definitions/references;
- scope visibility;
- capture sets;
- call graph;
- purity/effect summary;
- type compatibility;
- module dependencies.

## Formatter

Formatter is mainly syntax-owned, but semantic architecture still constrains it:

- formatting must preserve semantic parse;
- source ranges/targets after edits should remain deterministic;
- formatter must not need type inference to disambiguate valid syntax unless language
  grammar itself depends on semantics (avoid such grammar designs where possible).

## Parser

Parser recovery determines how much semantic tooling survives incomplete edits.
When adding syntax, ensure semantic target extraction can distinguish the new construct and
that recovered AST nodes retain usable ranges.

## Checker

Checker consumes identity, control flow, dispatch surfaces and type metadata. It owns
normative legality, not basic name lookup semantics.

Never duplicate class/member lookup in the checker with slightly different inheritance or
selector rules.

## Static prover

The prover should consume CFG/program points, type facts, effects and contracts. It must
label unknown proof states honestly.

## Optimizer

Semantic facts used for optimization require stronger discipline than editor hints.

Any speculative assumption must have:

- a guard whose truth implies the fast path is semantically equivalent;
- invalidation/versioning when mutable class/module state can violate the assumption;
- deoptimization or fallback preserving exact behavior.

Never feed heuristic LSP facts directly into code generation.

## REPL

REPL semantics introduces re-evaluation and evolving runtime state. When LSP-backed REPL
support is implemented, distinguish:

- source snapshot semantics;
- persisted runtime bindings/classes;
- redefinition/version identity;
- partial snippets.

Do not assume a file-based module lifetime.
