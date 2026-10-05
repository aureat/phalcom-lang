# Phase 10 Derivation Design

## Purpose

Phase 10 makes reflected domain models usable with bare `@Given` without creating a second generation engine. Derived strategies are ordinary `Strategy` values: they consume `DrawData`, emit semantic spans, replay deterministically, shrink through the existing passes, and participate in database identity through deterministic fingerprints.

## Opt-in metadata

`@arbitrary` is passive class metadata. It authorizes automatic derivation but performs no runtime checking, wrapping, or method installation.

`@strategy(Type)` is passive method metadata for a zero-argument provider. A registry installs a provider explicitly with `register(ProviderClass)`. The provider result is stored as an exact entry, so it has the same precedence as `register(type:, strategy:)`.

There is no process-global registry mutation.

## Resolution precedence

The registry checks exact entries, cached derived entries, built-in applied types, and finally opt-in derivation. Exact registration always wins. Registering an exact strategy invalidates any cached derived entry for the same type.

Nested resolution carries an immutable path. Constructor parameter and applied-container segments are appended before recursion, so the public `StrategyResolutionError` names the outer model, field, container, and unsupported leaf.

## Constructor derivation

A derivable class has exactly one reflected constructor. Every ordered parameter must have a reflected type and may not be a rest parameter. The derivation worker resolves each parameter strategy, retains parameter name and label in the fingerprint, and builds `_ConstructorStrategy`.

At draw time `_ConstructorStrategy` opens a non-discardable `#derivedConstructor` span, draws each argument in constructor order, and invokes the reflected constructor. Child strategies retain their own discardable spans, so collection and recursive shrinking remain structural.

## Safety boundary

Constructor preconditions are not translated into `.filter`. General contracts may be expensive, stateful, undecidable, or nearly unsatisfiable. A constrained constructor is therefore rejected before search with a recommendation to register a custom strategy.

The same fail-closed policy applies to missing annotations, multiple constructors, and rest parameters.

## Sealed derivation

Reflected variants are sorted by class name. A non-recursive hierarchy derives `Gen.oneOf` over constructor strategies.

For a recursive hierarchy, parameter type expressions are inspected recursively. Variants with no reference to the root form the terminal base. Recursive variants are rebuilt inside `Gen.recursive` using the supplied child strategy wherever the root appears, including inside supported standard containers.

At least one terminal variant is required. This establishes a size-zero base case and prevents unbounded eager recursion.

## Persistence and compatibility

Derived fingerprints contain the root type, stable variant ordering, constructor selectors, parameter names and labels, reflected types, and child strategy fingerprints. Relevant model changes therefore create a new database identity.

Existing explicit strategies, named `@Given` overrides, direct registrations, stateful testing, reporting, and persistence semantics are unchanged.
