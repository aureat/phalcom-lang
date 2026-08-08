# PDR-0032 — Converge lexical namespaces, selectors, visibility, and class placement

- Status: Accepted
- Date: 2026-08-08
- Supersedes: ADR-0061 in full
- Amends: ADR-0060's subscript-setter identity and PDR-0028's legacy-syntax compatibility policy
- Related: `docs/work/pending/transition-1/01` through `06`; current lexical, object-model, functions, and core-class specs

## Context

Several independently reasonable conventions collided: leading underscores meant fields and
pseudo-private methods, trailing underscores meant native internals, `__` named runtime hooks,
setter values occupied ordinary selector slots, and `static` duplicated class-side placement.
These conventions could not be enforced consistently by lexer, compiler, VM reflection, LSP,
and generated metadata.

## Decision

1. Lexical namespaces are structural:

   | Spelling | Meaning |
   |---|---|
   | `name` | ordinary lexical name or selector |
   | `_name` | source field |
   | `__name` | implementation field |
   | `_$name` | implementation selector |

   Ordinary modules cannot declare or access implementation namespaces. Bootstrap-core module
   identity grants the sole source privilege; a module cannot forge it by choosing a name.

2. Ordinary unresolved names use lexical lookup first—local, upvalue, known global—then an
   implicit `self` getter/send. `_field`, `__field`, and `_$selector` are namespace-directed and
   always target `self` when bare.

3. Declaration parameters are `_ local`, `label local`, `label`, and final `*rest`. Call-site
   `label:` syntax is unchanged. Selector identity uses external labels, never local names.

4. Setter values occupy a fixed role: `name=(put)` and `[index-args]=(put)`. Bracket slots
   describe indexing arguments only. Thus `[_ index, default fallback]=(put value)` has identity
   `[_,default]=(put)`.

5. `@private` permits calls only from the defining lexical class. `@protected` permits the
   defining class and subclasses. Runtime authorization applies to direct sends, caches, method
   references, reflection, and dynamic invocation. Implementation selectors are always
   `Internal`, independent of decorators.

6. `@class` is the only canonical class-side placement spelling. `static` remains recognized
   only to produce the targeted error `` `static` member syntax is retired; use `@class` ``; it
   never lowers as an alias.

## Consequences

- Parser and compiler use token/AST kinds for field semantics; string-prefix field heuristics
  are removed.
- Native internal selectors use dedicated registration macros and remain in the ordinary method
  table, with VM authorization as the security boundary.
- Tooling and generated metadata render the same canonical identities as runtime dispatch.
- Cost: source migration is intentionally breaking. Old declarations and `static` must be
  rewritten before code runs.
- This precludes using leading/trailing underscores as informal selector privacy conventions and
  precludes exposing implementation selectors merely by constructing their symbol dynamically.

## Alternatives rejected

- Keeping `static` as an alias leaves two permanent spellings for one placement axis.
- Enforcing internal access only in the lexer leaves reflection and dynamically-built selectors
  as bypasses.
- Encoding assignment `put` inside brackets confuses index arity with the fixed setter operand.
- Inferring privacy or fields from string prefixes makes semantics depend on spelling after the
  lexer already established distinct namespaces.
