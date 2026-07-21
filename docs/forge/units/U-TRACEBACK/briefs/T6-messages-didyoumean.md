# T6 brief — dispatch AFTER T3 (error.rs, dispatch.rs) and T2 (diagnostics/)

Implement traceback plan unit T6 — message hygiene + did-you-mean.
READ FIRST: implementation-spec.md §8.2 (style rules) + §9 (suggest engine spec, normative
parameters); plan.md §T6.
Deliverables:
1. diagnostics/suggest.rs: best_match(miss, candidates) — Damerau-Levenshtein OSA; thresholds
   len≤4→1, 5-8→2, >8→3; rank case-insensitive-equal > transposition > substitution >
   ins/del, then distance, then shorter, then lexicographic; emit only strictly-unique best.
   Pure, table-tested.
2. Wire: object_does_not_understand (primitive/object.rs:230-251) — candidates = receiver
   class + ancestors method tables (class-side send walks metaclass chain naturally);
   arity-miss with exact base-name match → dedicated hint ('sum(_)' exists — did you mean to
   pass 1 argument?). Undefined variable (dispatch.rs:664/:716 area) — structured variant with
   name as data, candidates = locals (compile-time list where resolvable) + module globals +
   core globals. Unknown class/import member in compiler.
3. Style-guide sweep per IS §8.2: rewrite live messages (single quotes, lowercase start, no
   trailing period, selector signature shape, receiver via toString ≤40 cols).
4. Internal-leak fixes: dispatch.rs:1009/:1012 + :956/:959/:984/:987 → typed #type errors,
   message like: cannot instantiate from 'Foo': it is a Number, not a class. Internal reserved
   for real VM bugs, prefixed: internal error (this is a Phalcom bug, please report): …
5. Dead variant deletion: UndefinedVar, UnsupportedOperation, BinaryNotSupported,
   UnaryNotSupported, ZeroDivision (error.rs:96-128 area) — ZeroDivision's IEEE-754 rationale
   moves to a doc comment on number_div (primitive/number.rs:104-114).
Write-set: phalcom-core/src/error.rs, src/vm/dispatch.rs, src/primitive/object.rs,
src/primitive/number.rs (doc only), src/diagnostics/suggest.rs (new) + mod.rs decl,
compiler unknown-class site, tests/**.
Tests: suggest table tests (metric/threshold/tie-suppression/determinism); one fixture per
rewritten message; negative-control that old {:?} leak strings are gone.
Gate + GIT + rustdoc: as above.
Return: engine parameters as implemented, message diff summary, SHAs, test evidence.
