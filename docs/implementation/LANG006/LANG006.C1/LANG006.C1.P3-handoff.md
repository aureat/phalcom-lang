---
id: LANG006.C1.P3
kind: implementation-plan-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
base_revision: f7e2ca1585c755e16f7046e5c24d0858dc2c9574
working_tree: uncommitted scoped implementation changes plus preserved unrelated STDL002 checkpoint work
---

# LANG006.C1.P3 — Handoff

## Current state

LANG006.C1 is complete through P3 at the current worktree revision. The
checkpoint is implementation-complete but broad-verification-blocked. No
additional LANG006 implementation plan is required for this vertical.

P3 focused gates G1–G4 pass, both owning crates compile, and the required
one-time broad runs completed. The broad results are semantic `1,428 passed /
14 failed / 50 ignored` and core `607 passed / 5 failed / 28 ignored`.

## Stable P2 interfaces to preserve

- `phalcom-semantic/src/access.rs` owns canonical declaration/caller access
  relations and exact module/package/project provenance.
- Formal dispatch authorizes the selected callable after lookup and preserves
  inaccessible-versus-missing behavior.
- Source compiler methods carry module-derived runtime origin; native anchors
  retain source module authority and are reconciled after materialization.
- Canonical semantic signatures own native/source contract comparison.
- `ImplementationKind` is the language category; `ImplementationProvenance` is
  native execution provenance.
- `Foreign` remains semantic/presentation data and fails at the explicit
  unsupported binding boundary; do not add an ABI or loader here.

## P3 seams

### Synthetic authority

`InvocationAuthority` is the call-site-local compiler fact. Only generated
constructor `self.allocate()` receives `CompilerInternal`; ordinary calls are
Ordinary. The compiler lowers that fact to `InvokeCompilerInternal`, and the VM
must continue to consume it transiently for one dispatch. Do not restore a
method-wide compiler flag, ambient callee privilege, selector-prefix authority,
or a less restrictive `allocate()`.

### Visibility publication

The canonical publication seam is:

```text
normalized AST member visibility
    -> ClassMember::visibility()
    -> build_declared_class_surface()
    -> published semantic products
```

Do not reintroduce `_`/`__` prefix inference or legacy visibility attributes.
Unmodified fields are Public under the current rule. The parser lookahead fix
for `private [_ index]` is intentionally narrow.

### Migrated authority

Enum behavior belongs in `impl`; iteration evidence is Iterator/Iterable; raw
associated projections are exportable metadata; private subscripts use current
modifiers. `reflection.selector::Selector.slots` is `List<Symbol>` in both
source and native descriptor, with the generated surface current.

## Residual owner ledger

Take the following to the current canonical owner plans; do not create a new
LANG006 fix merely because these tests remain red:

- SEMA flow/evidence: two failures (`opaque_call_drops_refined_field_current_without_rewriting_contract`, `unknown_expansions_do_not_disappear`).
- SEMA canonical subscript application: four structural index/get/set/map failures.
- SEMA incremental/product stability: eight dependency, query, hierarchy, and reuse failures.
- Argument expansion/canonical iteration: positional non-Iterable boundary and list-spread traceback shape.
- LANG005 conformance runtime: conditional-default and bounded-generic default/witness dispatch.
- Typing integration: `traverse_body_resolves_symbolic_inherited_capabilities`.

No residual failure is classified A/B for P3. The constructor `allocate()`
authority cascade is absent from the broad core run.

## Recommended next commands

For a future owner, first run the narrow failing filter for that owner and
compare against the recorded broad result. Do not rerun the broad semantic/core
suites unless the owner plan requires a certification pass. For any future
LANG006 audit, begin with:

```sh
git status --short --branch
git diff --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  execution_compiler_authority::generated_constructor_marks_only_allocator_as_compiler_internal -- --nocapture
```

## Do not redesign or re-explore

Preserve exact access identity, lookup-then-authorize dispatch, source-derived
native origin, the language/provenance split, Foreign's ABI-free boundary,
call-site-local synthetic authority, and normalized AST visibility publication.
Do not revisit retired selector/field prefix semantics or repair broad SEMA,
iteration, conformance, or typing failures inside LANG006.

## Verification truth

P3 and LANG006.C1 are `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`, not
`RELEASE_COMPLETE`. Workspace, formatter, clippy, and release gates were
deliberately deferred. The preserved unrelated STDL002 checkpoint modification
must remain outside any scoped LANG006 commit or push unless the user explicitly
authorizes all-worktree delivery.
