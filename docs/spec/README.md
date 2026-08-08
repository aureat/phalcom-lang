# Phalcom Language Specification

## Status

This file is the documentation-governance charter for the language
specification reorganization. It defines where normative language rules belong;
it does not introduce or amend language semantics.

During the reorganization, the existing specification sources remain in force
until their replacements are promoted here. A moved document replaces its source
only when its migration commit updates all affected links and records the source
path in the archive manifest.

## Authority

`docs/spec/` is the sole home for the normative Phalcom language specification.
A specification document states the effective language rule a programmer,
compiler, and runtime must follow. It may describe behavior that is ratified but
not implemented; that implementation gap belongs outside the specification.

Normative documents use `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` in
their usual specification sense. A document under this directory must not contain
competing alternatives, unresolved recommendations, or a second version of an
effective rule.

## Decision and implementation records

The specification records **what** the language means. Other documentation has
separate roles:

- `docs/decisions/` records **why** a rule was chosen, including accepted,
  proposed, and retired decisions.
- `docs/implementation/` records **what HEAD does**, evidence, as-built work,
  divergence, and verification status.
- `docs/design/` records active proposals and research that have not become
  effective language rules.
- `docs/archive/` preserves superseded, duplicate, stale, and closed artifacts.

An accepted decision does not by itself make a document a specification chapter.
Its effective rule must be reconciled into one canonical topic chapter. A proposed
decision or design document never overrides `docs/spec/`.

## Target layout

The completed specification uses stable topic directories:

```text
docs/spec/
  foundations/    values, objects, messages, classes, lookup
  syntax/         lexical structure, grammar, expressions, declarations
  semantics/      blocks, control flow, errors, modules, iteration
  runtime/        bootstrap, memory, execution, concurrency
  library/        numeric tower, collections, Result, System, standard library
  extensions/     ratified optional language extensions
  conformance/    normative conformance and compatibility requirements
```

Every topic directory will have an index that names its canonical chapters and
their governing decision records. A topic has one canonical rule; subsidiary
chapters must link to it rather than restate it.

## Promotion rule

A document may enter `docs/spec/` only when all of these are true:

1. It states a single effective rule, with no unratified alternatives.
2. Its terminology and examples agree with the relevant accepted decisions.
3. Its implementation status, plans, and historical narrative have been removed
   or moved to their proper records.
4. It has one canonical destination and all inbound links are updated.

If a source contains both normative and non-normative material, split it. Promote
the reconciled language rule; preserve the remainder in `implementation/`,
`design/`, or `archive/`.

## Retirement and archive rule

Never delete documentation because it is stale. Move it with `git mv` into a
dated archive subtree that preserves its former relative path. Each archived file
must identify its replacement, or state that no replacement exists. Retired
language syntax and superseded rules belong in the archive or decision history,
never beside their active replacements.

## Migration gate

Each migration commit must:

1. change one coherent topic only;
2. include a source-to-destination manifest or archive note;
3. update internal and inbound documentation links;
4. leave no duplicate claim of normative authority; and
5. pass documentation-link and wording checks.

No migration changes language behavior, source code, tests, or implementation
status unless that work is explicitly requested separately.
