# Random stable-owner prerequisite walkthrough

Completion: IMPLEMENTED. Verification: FOCUSED_TESTED.
Parent random plan: BLOCKED at R0.

The user consultation fixed the architecture: preserve the semantic owner
algebra in durable callable identity. `StableCallableOwnerRef` now has
Declaration, Variant, and Conformance variants. Variant selectors and callable
selectors use canonical encoding. Both semantic projections share the same
owner conversion. Exact applications retain source impl ownership.

Metadata schema is v11. Tagged owners are written; legacy declaration-shaped
owners remain readable. The validator accepts v11. Partial owner accessors
distinguish lexical declaration ownership from enclosing nominal ownership.
CallableSignature reflection uses the latter and does not fabricate a nominal
owner for an impl. Generic parameter fingerprints use the same discriminated
callable encoding. No compiler evidence workaround was introduced.

Changed production owners are `phalcom-type-meta/src/{identity,header,validate}.rs`,
`phalcom-semantic/src/metadata/{stable_identity,export}.rs`, and
`phalcom-core/src/primitive/typing.rs`. Direct construction sites in core tests
and VM test setup were migrated. `generic.rs` and runtime typing environment
already embed `StableCallableRef` and required no parallel representation.

Tests prove contextual/context-free projection, all owner-kind noncollisions,
generic parameter fingerprints, v11 whole-bundle roundtrips, v10 backward decode,
and unchanged generic witness identity across two exact applications and
cold/incremental reconstruction. Exact commands/results and the unrelated
record-domain metadata failure are recorded in
`stdlib-random-implementation-incident.md`.

The unchanged original random syntax probe exposed a second blocker: missing
runtime conformance recipe for method-generic `sample(using:)`. The parent
plan's R0 acceptance has not passed. No random library source or native primitive
was implemented, and performance/final random gates were not run.

Work remains on main at the starting HEAD
`c4db36bbab312cd41a589a34dad55dfbfd4ed456`, with uncommitted changes. No commits,
pushes, subagents, or unrelated work modifications were performed.
