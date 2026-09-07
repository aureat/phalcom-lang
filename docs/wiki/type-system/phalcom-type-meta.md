# Phalcom Type Metadata

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-type-meta source snapshot](../raw/type-system/2026-09-07-phalcom-type-meta.md)
> Updated: 2026-09-07

## Overview

`phalcom-type-meta` is a standalone, store-independent model for serialized semantic metadata. It represents artifact headers, stable identities, kind and type graphs, scoped type-lambda bodies, generic signatures, declarations, callable and field surfaces, module/runtime roots, type-use occurrences, extension sections, fingerprints, JSON encoding, and structural validation.

## Artifact header and compatibility

The header records the type-metadata schema version, semantic model version, producer identity/version, native-surface schema version, retention profile, enabled features, identity scheme, and source/interface fingerprints. The implementation accepts type-metadata schema versions in the inclusive range `1..=2`; the semantic model version is `1`, and the native-surface schema version is `1`.

Feature flags explicitly advertise type lambdas, record rows, runtime type constants, source occurrences, and advanced sections. Schema compatibility is not only a version check: schema 1 rejects the record-rows feature and record-row/open-record constructs, while schema 2 requires the record-rows feature whenever those constructs occur.

## Stable identity and graph representation

Stable project references distinguish builtins, packages, source artifacts, and sessions. Stable module and declaration references extend those project identities with paths; callable and field references add dispatch side plus selector/name. Source spans retain start and end offsets.

Kinds form an indexed graph containing ordinary types, record-row kinds, and arrows from parameter kinds to a result kind. The global type graph contains nominal, applied, union, tuple, record, callable, generic-parameter, self, type-lambda, and open-record forms. Each indexed entry stores its kind and a structural fingerprint.

Scoped types alpha-normalize lambda bodies. A scoped node may refer to a bound variable by depth/index or to a free global type node; it also supports nested lambdas, applied/union/tuple/record/callable forms, and open records. Scoped record tails are either bound row variables or free stable parameters.

## Generic and declaration surfaces

Generic metadata gives each parameter a stable owner and index, a name, kind, variance, and optional source span. Signature records own parameter lists and constraints represented as subtype or equivalent relations over global type-node IDs.

Declaration records connect a stable declaration to its type form, kind, optional generic signature, optional superclass template, callable and field references, declaration flags, and an optional source span. Published type slots preserve uncertainty and provenance explicitly: a slot can be known with an authority, dynamic with a reason, unknown with a reason, or unavailable with a reason. Callable records add parameter labels/local names/rest modes and a published return slot; field records add mutability and a published type slot.

## Bundle roots and serialization

`SemanticMetadataBundle` is the complete immutable artifact/program bundle. It contains the header plus indexed kinds, global and scoped types, parameters, signatures, declarations, aliases, callables, fields, module roots, runtime type roots, occurrences, and extension sections. Module roots provide index membership and an interface fingerprint; runtime roots map local keys to type forms; occurrences record the role, status, optional written spelling, and source span of a type use.

`encode_metadata_json` serializes a bundle with `serde_json`. `decode_metadata_json` first enforces the configured total-byte budget, then deserializes and validates the bundle, returning either a JSON or validation error.

## Fingerprints and validation invariants

`Fingerprint128` stores a 128-bit value as sixteen bytes and exposes zero, conversion from `u128`, byte access, debug formatting, and hexadecimal display. `FingerprintBuilder` provides deterministic streaming writes for bytes, integers, strings, byte slices, and nested fingerprints before producing the final fingerprint.

Validation applies configured budgets, checks schema/model compatibility and feature floors, and validates references across the indexed graphs. Kind, global type, and scoped type graphs must be topologically ordered: referenced nodes cannot be future entries. It also checks kind/type/scoped-type/signature indexes, duplicate parameter identities, generic constraint references, declaration references, known callable/field type references, lambda depth limits, and record-row tail ownership.

Record and open-record fields must be sorted and unique. Global open-record tails must resolve to a parameter whose kind is `RecordRow`. Scoped open-record tails must resolve either to an in-scope row binder or to a free parameter with that kind. These checks make malformed graph structure and feature/version mismatches explicit validation errors rather than deferred consumer failures.

## Evidence and compatibility tests

The compatibility tests decode a schema-v1 JSON fixture through the current decoder, reject unsupported versions below and above the supported range, reject schema-v1 record-row constructs, accept a valid schema-v2 open record, and exercise failures for unsorted fields, duplicate fields, an incorrect tail kind, and a missing feature bit. A separate test validates a scoped open record whose row tail is bound inside a lambda.

## See Also

- [Phalcom Type Syntax](phalcom-type-syntax.md)
