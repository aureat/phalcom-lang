# Phase 06 Design — Property Attributes, Reflective Inference, Builder API, and Runner

## Scope

Phase 06 replaces the temporary property-facing compatibility slice with the canonical public API while preserving the Phase 05 search kernel. It implements passive method attributes, reflective parameter inspection, recursive type-to-strategy inference, explicit and partially overridden strategies, source-aware assertions, the fluent builder, property discovery, suite execution, and named failure arguments.

Database persistence, rich event reporting, targeting, JSON output, and stateful expansion remain later phases.

## Ownership

The authoritative property slice is:

- `property/attributes.ph`: `Given`, `GivenArgs`, `Case`, and `WithSettings`;
- `property/inference.ph`: reflected parameter descriptors and strategy resolution;
- `property/target.ph`: method and block invocation targets;
- `property/assertion.ph`: source-aware assertion errors and `Assert`;
- `property/builder.ph`: `Property`, `PropertyBuilder<T...>`, and `PropertySuite`;
- `property/discovery.ph`: `PropertyId`, discovered definitions, and metadata validation;
- `property/runner.ph`: suite execution and named result values.

The root façade exports these modules directly. The compatibility adapter may import them for Phase 07–09 bridges, but it may not define a second property runner, attribute set, assertion implementation, or builder.

## Given modes

`@Given` has exactly three modes:

1. `@Given` infers every parameter strategy from reflected annotations.
2. `@Given(strategy, ...)` supplies exactly one explicit strategy per parameter.
3. `@Given(GivenArgs.new().for(#name, use: strategy))` overrides named parameters and infers the rest.

Unknown override names, duplicate override names, missing annotations, unsupported types, and explicit arity mismatches are discovery errors before search begins.

## Type inference

`StrategyRegistry.standard` resolves exact primitive entries and recursively decomposes reflective applied types. Phase 06 requires `Option<T>`, `List<T>`, and tuples, and also supports `Set<T>`, `Map<K,V>`, and `Result<T,E>` because the underlying strategies already exist.

Annotations remain reflective metadata only. Inference does not enforce values at runtime or alter selector identity.

## Discovery and execution

`PropertyDiscovery` inspects method parameters and retained attributes, resolves settings and strategies, collects explicit cases, and produces immutable `PropertyDefinition` values. `PropertyRunner` creates one suite instance, converts each definition to the Phase 05 `PropertySpec`, executes it through `SearchEngine`, and returns a `PropertySuiteResult` containing `PropertyRun` records.

Explicit examples remain first-phase inputs to the engine and are never shrunk. `PropertyRun.namedArguments` zips reflected parameter names with the falsifying arguments so later reporters do not need to rediscover methods.

## Assertions

Every public assertion captures the caller source location and throws one `_PropertyAssertionError` carrying a concrete `FailureOrigin`. Two assertions with the same error class therefore remain distinct by module, selector, line, column, and optional label.

## Builder

`Property.given(*strategies)` returns `PropertyBuilder<T...>`. `.using(settings)` returns an updated builder, and `.check { ... }` creates an engine specification and invokes `SearchEngine`. `Property.forAll` remains a compatibility spelling that delegates to the builder. `Property.find` continues to use the same engine-level value search.

## Verification

The Phase 06 verifier checks fixtures first, records a red state before implementation, validates public ownership and recursive inference, confirms adapter runner removal, checks root façade migration, resolves every internal import, preserves Phase 01–05 gates, verifies checksums, and re-runs all gates from a clean extraction.
