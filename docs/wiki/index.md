# Phalcom Knowledge Base

A concept-oriented map of Phalcom's language, semantic, execution, tooling, and evidence boundaries. Package names remain in source provenance; article names follow the future decomposition.

## Language

Source syntax and the language surface.

- [Language surface](language/overview.md)
- [Lexical structure](language/lexical-structure.md)
- [Expressions and control flow](language/expressions-and-control-flow.md)
- [Blocks and closures](language/blocks-and-closures.md)
- [Strings and interpolation](language/strings-and-interpolation.md)
- [Patterns and matching](language/patterns-and-matching.md)
- [Annotations and Phaldoc](language/annotations-and-phaldoc.md)

## Type system

Symbolic types, callable contracts, families, rows, and published metadata.

- [Type system](type-system/overview.md)
- [Symbolic type notation](type-system/symbolic-type-notation.md)
- [Callable and generic contracts](type-system/callable-and-generic-contracts.md)
- [ADT/GADT families](type-system/adt-gadt-families.md)
- [Records and rows](type-system/records-and-rows.md)
- [Semantic type metadata](type-system/semantic-type-metadata.md)

## Modules and project topology

Project identity, source loading, interfaces, linking, dependency graphs, and sessions.

- [Modules overview](modules/overview.md)
- [Identity and module paths](modules/identity.md)
- [Project structure](modules/project-structure.md)
- [Project manifests](modules/project-manifest.md)
- [Module resolution](modules/module-resolution.md)
- [Source providers](modules/source-providers.md)
- [Interfaces](modules/interfaces.md)
- [Linking and symbols](modules/linking-symbols.md)
- [Dependency graphs](modules/dependency-graphs.md)
- [Sessions and incremental updates](modules/sessions.md)

## Semantic analysis

Canonical authority, inference, identity, snapshots, capabilities, and proof products.

- [Semantic analysis](semantic/overview.md)
- [Type formation and inference](semantic/type-formation-and-inference.md)
- [Authority and identity](semantic/authority-and-identity.md)
- [Workspace incrementality](semantic/workspace-incrementality.md)
- [Capability and flow](semantic/capability-and-flow.md)
- [Products and reflection](semantic/products-and-reflection.md)
- [Pattern coverage and constructors](semantic/pattern-coverage-and-constructors.md)

## Collections

Product values, maps, indexing, traversal, and argument packs.

- [Collections](collections/overview.md)
- [Product model](collections/product-model.md)
- [Maps](collections/maps.md)
- [Indexed access and ranges](collections/indexed-access-and-ranges.md)
- [Collection traversal](collections/traversal.md)
- [Argument expansion](collections/argument-expansion.md)

## Runtime

Object/value representation, compiler lowering, execution, lifecycle, and dispatch identity.

- [Runtime](runtime/overview.md)
- [Representation and object model](runtime/representation-and-object-model.md)
- [Compiler and VM boundary](runtime/compiler-vm-boundary.md)
- [Lifecycle and memory](runtime/lifecycle-and-memory.md)
- [Selector identity](runtime/selector-identity.md)

## Concurrency

Fibers, scheduling, reactor integration, and futures.

- [Concurrency](concurrency/overview.md)
- [Fibers and scheduling](concurrency/fibers-and-scheduling.md)
- [Reactor](concurrency/reactor.md)
- [Fiber reflection](concurrency/fiber-reflection.md)
- [Futures](concurrency/futures.md)

## Native surfaces

Authored declarations, canonical universe records, generated surfaces, and host capabilities.

- [Native surfaces](native/overview.md)
- [Universe catalog](native/universe-catalog.md)
- [Primitive contracts](native/primitive-contracts.md)
- [Declaration pipeline](native/declaration-pipeline.md)
- [Canonical native surface](native/canonical-surface.md)
- [Generated surface drift](native/generated-surface-drift.md)
- [Host capabilities](native/host-capabilities.md)

## Diagnostics

Structured reports, source context, terminal rendering, and runtime error products.

- [Diagnostics](diagnostics/overview.md)
- [Report model](diagnostics/report-model.md)
- [Source snippets and locations](diagnostics/source-snippets-and-locations.md)
- [Terminal rendering](diagnostics/terminal-rendering.md)
- [Result, error, and traceback surfaces](diagnostics/result-error-traceback.md)

## Editor integration

LSP architecture, semantic projections, snapshots, and clickable locations.

- [Editor integration](editor/overview.md)
- [LSP architecture](editor/lsp-architecture.md)
- [Semantic editor products](editor/semantic-editor-products.md)
- [Workspace snapshots and incrementality](editor/workspace-snapshots-and-incrementality.md)
- [Editor source locations](editor/source-locations.md)
- [Editor performance](editor/performance.md)

## Tooling

REPL sessions, interactive intelligence, Phaldoc, and documentation organization.

- [Tooling](tooling/overview.md)
- [REPL sessions](tooling/repl-sessions.md)
- [Interactive intelligence](tooling/interactive-intelligence.md)
- [REPL commands](tooling/repl-commands.md)
- [Phaldoc and documentation](tooling/phaldoc-and-documentation.md)

## Performance

Measurement protocol, benchmark evidence, hot paths, and optimization seams.

- [Performance](performance/overview.md)
- [Benchmark corpus](performance/benchmark-corpus.md)
- [Measurement and instrumentation](performance/measurement-and-instrumentation.md)
- [Hot paths and inline caches](performance/hotpaths-and-inline-caches.md)

## Standard library

User-facing numeric and collection-library contracts.

- [Standard library](standard-library/overview.md)
- [Numeric contracts](standard-library/numeric-contracts.md)
- [Numeric literals and arithmetic](standard-library/numeric-literals-and-arithmetic.md)
- [Float, text, and numeric errors](standard-library/float-text-and-errors.md)
- [Collection library surface](standard-library/collection-library-surface.md)

## Evidence and provenance

Compiled pages cite immutable raw snapshots under [raw/](raw/). The append-only [operation log](log.md) records source ingests, taxonomy decisions, and migrations. Normative language and semantic rules remain in [docs/spec/](../spec/); implementation lifecycle remains in [docs/implementation/](../implementation/).

The wiki is an explanatory index, not a release certification. Each page labels proposed, partial, implemented, focused-tested, and unverified boundaries explicitly.
