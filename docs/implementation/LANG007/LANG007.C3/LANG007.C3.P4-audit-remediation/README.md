# LANG007.C3.P4 audit remediation

Starting revision: `0d6a6e9bbf74c82e5c3587eb4153fde589cbedd5`

P4 state at start: `IN_PROGRESS / PARTIAL / UNVERIFIED`.

The attached post-implementation audit is the remediation authority for this work. This directory records implementation evidence for each audit finding as it is addressed. Projection-solver soundness, conditional runtime evidence propagation, and complete-pack canonical identity are addressed; later findings remain open.

- [01 — projection solver soundness](01-projection-solver-soundness.md)
- [02 — complete-pack canonicalization](02-complete-pack-canonicalization.md)
- [03 — nested derived conformance runtime layout](03-nested-derived-conformance-runtime-layout.md)

Conditional runtime evidence propagation is implemented in local commits
`6e2197e6` and `ccde8670`. The semantic capability retains selected impl
bindings and Exact/Forwarded/Derived proof provenance; core projects this into
the selected callable's existing runtime slot layout. Focused semantic and core
tests cover direct calls, captured families including escape beyond the creator
frame, distinct same-trait subjects, nested derived evidence, and strict
subclass override isolation. Complete-pack source-backed identity is recorded
in `02`. Priority 2's nine named selectors are individually green, including
CV50 nested-slot isolation, CV51 and identical-environment reuse, CV44 repeated
derived calls, direct-field delegation, bounded generic witness/default, and
the distinct/combined generic evidence cases. Narrow input changes to `const`
preserve original assertions and flow policy. The live owner/method test is not
ignored: ignored-mode selected zero, while its normal exact selector passed.
Full per-test evidence and commits are in `03`.

The selected-fallback paths of `InvokeConditional` and conditional-family
dispatch carry their explicit environments. The rest and module-export
reachability probes did not publish a callable, specialization, or recipe, so no
runtime environment drop was demonstrated and neither adapter was changed.
Next: classify the audit's 40 semantic failures (Priority 3), then resume T4/T5
as directed by the audit. The overall audit and release gates remain open.

Finishing revision and final P4 outcome are intentionally left open until the full remediation program is complete.
