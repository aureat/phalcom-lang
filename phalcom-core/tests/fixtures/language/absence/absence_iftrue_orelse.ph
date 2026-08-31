// area: absence
// spec: values-and-absence.md §3.3; control-flow.md §1; ADR-0007; ADR-0018
// status: PASS
// U-CORE-2: `orElse(_)` (core.ph, derived over `match`) passes a `Some`
// through unchanged and replaces a `None` with the block's own `Option`
// result (§3.3's "Transform" group — the block, like Rust's `or_else`,
// already returns an `Option`). Exercised directly against an `ifTrue`
// result, which only type-checks post-U-CORE-2 now that `ifTrue` returns a
// well-formed `Option` rather than a raw value on its taken arm.

System.print(true.ifTrue || { "yes" }.orElse || { Some("fallback") }.match(some: |v| { v }, none: || { "unreachable" }))
System.print(false.ifTrue || { "yes" }.orElse || { Some("fallback") }.match(some: |v| { v }, none: || { "unreachable" }))
