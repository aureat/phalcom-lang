# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `COLL006`
- Checkpoint: `COLL006.C1`
- Plan: `COLL006.C1.P5`
- Task/Gate: T7 / G6.B, complete parse-package certification
- Repository: `aureat/phalcom-lang`
- Worktree: `.worktrees/telos-collections-upgrade`
- Branch: `feat/telos-collection-upgrade`
- Starting revision: `80a889f0` (clean P6 T1 state)
- Current revision: `80a889f0` plus the scoped user-directed audit follow-up
  diff recorded in the P5 walkthrough

## 2. Escalation trigger

The remaining failure enters compiler associated-family invocation lowering,
an owning subsystem outside the P5 parser adapter and collection storage work.
The live compiler explicitly rejects the path with
`AssociatedInvokeNotLoweredYet` / “associated family invocation lowering is
not implemented until associated semantic resolution lands.” This triggers
the escalation rules for an unexpected compiler-layer contract and for a
cross-layer behavior required by the parser package gate. P5 implementation
must not invent a fallback or change associated semantic authority.

## 3. Required invariant

- Preserve `ListSource<T>`'s private Array snapshot retained through canonical
  `Slice<T>`; caller List mutation must not alter its membership/order.
- Preserve Cursor and parser behavior for a source whose `Source::Token` is a
  user-defined algebraic type.
- Keep associated type resolution canonical in the semantic layer. Compiler
  lowering must consume canonical checked products, not reconstruct the
  associated target or erase the token type.
- Keep P5 G6.A's independently proven snapshot acceptance intact. G6.A does
  not certify the full parse package or waive G6.B.

## 4. Expected architecture

`Cursor<S>` requires `S |= Source`; `Cursor.current` has result type
`Option<<S as Source>::Token>` and delegates to `Source.at`. `ListSource<T>`
implements `Source` with `type Token = T`, snapshots the caller List into an
Array, and retains a Slice. A `Cursor<ListSource<Token>>` should therefore
expose `Option<Token>` while preserving the canonical `Source::Token = Token`
projection. The P5 plan requires this parser/source integration without a new
snapshot type or alternate associated-type model.

## 5. Observed behavior

After correcting the audited String/Substring adapter and optional-byte
consumer mismatches, the full command
`RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package`
selects four Rust tests: three pass and one fails. The declared parse project
compiles, both independent ListSource snapshot/owner tests pass, and 11 of 12
runtime fixtures pass. The remaining fixture is
`packages/parse/tests/runtime/src/core/source-test.ph`.

The failure occurs in the token-backed Cursor `current`/`advance` region after
constructing `ListSource<Token>` and mutating the caller List. The runtime
compiler reports:

```text
associated family invocation lowering is not implemented until associated semantic resolution lands
```

An exact single-expression isolation was not established: a truncated probe
including the paired Cursor assertions reached the same compiler error, while
the smaller prefix ending before those assertions passed. Treat the exact
trigger within that region as unconfirmed.

## 6. Minimal reproduction

The minimal failing region is the token-backed Cursor use in
`source-test.ph` after the `ListSource<Token>` snapshot and caller mutations:

```telos
let tokenCursor = Cursor.new(tokenSource)
Testing.Test.expect(tokenCursor.current == Some(Token::Integer(3)), ...)
Testing.Test.expect(tokenCursor.advance.current == Some(Token::Comma), ...)
```

The complete package gate is the authoritative reproducer. A standalone
truncated probe reproduced the associated-family compiler error when it included
the paired token-Cursor assertions; it was stopped during a follow-up attempt
to isolate the first expression because that probe consumed sustained CPU
without producing additional diagnostics.

## 7. Relevant code path

```text
phalcom-core/tests/core/modules/parse_package.rs
  -> runtime fixture runner
  -> packages/parse/tests/runtime/src/core/source-test.ph
  -> Cursor<S>::current / Cursor<S>::advance.current
       result projection: Option<<S as Source>::Token>
  -> S = ListSource<Token>; Source::Token = Token
  -> compiler associated-invocation lowering fallback
       phalcom-core/src/compiler/lib/associated.rs
       Err(CompilerError::AssociatedInvokeNotLoweredYet(...))
```

The exact failing expression may be the associated result projection, a
generic invocation used by the getter, or an interaction with the ADT variant
comparison. Current evidence does not distinguish these causes.

## 8. Scoped implementation diff

The follow-up diff changes only the parse adapters/fixture consumers and the
Map contract regression oracle. It does not edit Cursor, ListSource's
associated-type declaration, semantic associated resolution, compiler
associated lowering, VM dispatch, P5 ownership, or P6 public collection APIs.

The original audit residuals were corrected at their local consumer boundary:

- `TextSource.slice` now materializes `Substring` to preserve its declared
  `String` result.
- TextSource, `parse.text`, and the operator fixture explicitly unwrap
  `Option<Int>` byte reads, preserving the existing `-1` out-of-range
  sentinel where applicable.
- The operator fixture materializes its byte-sized `Substring` before String
  concatenation.

The Map Rust test now checks the actual public wrapper contract: `remove(_)`
returns the Map for absent and present keys, and present removal changes size.

## 9. Evidence

| Evidence | Result |
|---|---|
| `RUSTFLAGS='' cargo test -p phalcom-core --test core core_collections` | 13 passed; corrected Map test included |
| `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package` | 3 passed / 1 failed; project compile passes; 11/12 runtime fixtures pass |
| Independent `parse_list_source_snapshot` focused tests | 2 passed in the preceding P5 closeout; retained G6.A evidence |

No broad workspace tests, Clippy, or release certification were run. Those
remain P9 obligations.

## 10. Implementer's causal hypothesis

The most likely boundary is lowering of a generic Cursor operation whose
result depends on the associated `Source::Token` projection after specializing
`S` to `ListSource<Token>`. This is consistent with the compiler's explicit
associated-family lowering error and with the fixture succeeding before the
token-backed Cursor assertions. This is a hypothesis, not a verified root
cause.

## 11. Alternative hypotheses

1. `Cursor.current`'s associated result projection lacks a canonical lowering
   product for this generic instantiation.
2. `Cursor.advance.current` introduces a nested owner/invocation case that the
   compiler cannot lower even if the first `current` access is supported.
3. `Token::Integer` or `Token::Comma` comparison in an `Option<Token>` context
   triggers the unsupported family path; direct `current` assignment/matching
   may behave differently.
4. The compiler failure is a fixture/compiler capability gap rather than a
   defect in P5's collection representation or the `Source` contract.

The current run does not distinguish these alternatives.

## 12. Attempts already made

- Corrected the known String/Substring and `Option<Int>` consumer mismatches.
- Reran the complete package gate; the original diagnostics cleared, leaving
  the associated-family failure described above.
- Ran `core_collections`; all 13 tests passed after correcting the stale
  public `Map.remove(_)` test expectation.
- Ran a truncated token-backed Cursor probe through the paired assertions;
  it reached the same compiler error. A further attempt to isolate one
  assertion was interrupted after sustained CPU use without new output.
- Made no compiler, semantic, Cursor, or fixture workaround changes.

## 13. Why local authority is insufficient

The direct repair would enter compiler lowering and potentially change how
canonical associated semantic products are consumed. A test-only workaround
could also hide a real inability to use `Cursor.current` for a user-defined
token type and weaken the full parser integration oracle. The P5 plan forbids
silently redesigning compiler/semantic contracts, while the associated-type
owner plan says such lowering blockers remain at their owning layer rather than
being hidden in standard-library code. An architect must choose the owner and
scope before either path is taken.

## 14. Decision requested

Please decide:

1. Is this a compiler/associated-resolution defect that needs a separate
owner-layer implementation plan, with G6.B remaining blocked until that work
lands; or is the current Cursor use outside the supported contract and the
fixture should be narrowed while retaining a separate record of the compiler
limitation?
2. If the compiler layer owns it, which plan/checkpoint owns canonical
associated result projection and invocation lowering for this path, and what
semantic product must lowering consume?
3. If a bounded fixture adjustment is permitted, specify the replacement
assertion that still proves Cursor behavior and user-defined-token parsing
without masking the unsupported call.
4. What is the minimum focused compiler/semantic/runtime regression set, and
should P9 rerun the complete package gate only after that owner-layer gate
passes?

Until a decision arrives, keep P5 `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`,
G6.A `PASS`, and G6.B/aggregate G6 `BASELINE_BLOCKED`. P6's API consultation
remains independent and in progress; do not implement its new public APIs
without its decision.

## Resolution — 2026-10-18

The failure was a missing semantic context problem at the call site. The
unqualified `Some(...)` appeared inside an equality expression with no expected
enum owner, so semantic analysis could not resolve the nested `Token`
constructor. The fixture now uses `Option<Token>::Some(...)`, supplying the
exact contextual owner while preserving the distinction between expected
type constraints and owner evidence. It checks the resulting ADT by structural
matching rather than requiring a separate `Token` value-equality protocol.

The full package gate passes 4/4 Rust integration checks and 13/13 runtime
fixtures. The established architecture remains intact:

```text
semantic resolution
    -> canonical AssociatedResolution
    -> semantic-lowering projection
    -> AssociatedLoweringSpec
    -> compiler emission
    -> existing VM construction/invocation machinery
```

No compiler fallback, AST-spelling reconstruction, generic erasure, or
lowering-invariant weakening was introduced. This closes P5 G6.B for the
statically owned constructor path. Dynamic associated-family invocation remains
an unsupported compiler capability for a separate plan; it was not required
by this package path. P5 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`, and the
checkpoint retains workspace tests, Clippy, and release certification as not
run.
