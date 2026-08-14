# Semantic Feature Recipes

These are implementation playbooks. Adapt to current source; do not copy paths blindly.

## Recipe 1 — Add a new binding syntax

Example: a future `catch error` binding or pattern arm binding.

1. Confirm grammar/AST/spec.
2. Decide lexical scope and declaration visibility point.
3. Add/update `SemanticBindingKind` only if semantically distinct.
4. Extend scope builder to create scope/binding.
5. Extend occurrence builder for declaration/read/write targets.
6. Extend flow transfer to seed/project value facts if construct supplies them.
7. Add visible-bindings/completion query coverage.
8. Add rename/shadowing tests.
9. Add malformed/incomplete source test.
10. Add incremental insert/remove binding test.

## Recipe 2 — Add a new literal/container shape

1. Define abstract shape and join behavior.
2. Decide structural depth/size cap.
3. Add exact syntax inference in expression analyzer.
4. Handle spreads/rest/dynamic elements conservatively.
5. Add render/debug representation.
6. Add element/member capability helpers only if general.
7. Test homogeneous/heterogeneous/empty/dynamic forms.
8. Verify no future type semantics are accidentally encoded in shape domain.

## Recipe 3 — Add a new send/member syntax

1. Identify canonical selector formation from parser/compiler.
2. Extend occurrence targeting to exact selector fragment(s).
3. Extend analyzer to evaluate receiver/arguments in source order.
4. Map static labels; detect dynamic packs.
5. Route lookup through `DispatchResolver`.
6. Emit `ResolvedCall` when exact; mark dynamic otherwise.
7. Propagate return summary/effects.
8. Test instance/class/super/implicit-self forms.
9. Test inheritance/visibility.
10. Test computed labels/packs do not fabricate target.

## Recipe 4 — Improve field inference

1. Confirm field identity: owner + name + side.
2. Enumerate evidence sources:
   - declaration initializer;
   - constructor write;
   - general write;
   - future native/trusted declaration.
3. Ensure flow evaluates RHS before record.
4. Attach exact write site and evidence kind.
5. Join writes using domain operation.
6. Decide inherited field ownership semantics from runtime/spec.
7. Expose query by `FieldId`/class/name/side.
8. Test conflicting constructor branches, multiple constructors, class-side field, inherited field.
9. Test edit/removal invalidation.
10. Keep declared field type (future) separate from observed shape.

## Recipe 5 — Improve parameter inference

1. Resolve call target exactly.
2. Map arguments to declared parameter slots respecting labels/rest.
3. Record argument facts with call-site provenance.
4. Join contributions in `ParameterFacts`.
5. Feed facts into callable analysis seed.
6. Iterate summary fixed point.
7. Mark dynamic call sites conservative/unobserved.
8. Test multiple call sites/conflicts/recursion/higher-order calls.
9. Do not promote observed facts to normative type contract without typing-spec authority.

## Recipe 6 — Add branch refinement

Example: `x != None` or sealed variant pattern.

1. Confirm predicate has trusted semantics (not arbitrary overridable method).
2. Define refinement domain.
3. Implement `refine_condition` returning true/false states.
4. Apply before branch transfer.
5. Invalidate refinement on assignment/mutation according to stability rules.
6. Join branch exits.
7. Retain predicate provenance.
8. Test true/false, nested branch, mutation, union, unknown predicate.
9. Ensure LSP heuristic refinement is not used as checker proof unless exact.

## Recipe 7 — Add a callable effect

Example: `may_yield`.

1. Define may/must polarity and default unknown behavior.
2. Add field to `SummaryEffects` or a scalable effect set.
3. Mark direct origin operations/native metadata.
4. Propagate through resolved calls.
5. Propagate through invoked block/callback parameters.
6. Dynamic call fallback must be conservative.
7. Include effect in summary equality/invalidation.
8. Add direct/transitive/dynamic/higher-order tests.
9. Add intended consumer only after semantic tests pass.

## Recipe 8 — Add a module dependency kind

1. Specify edge meaning.
2. Parse/resolve target identity.
3. Retain unresolved edge.
4. Add forward/reverse graph representation.
5. Define effect on name/type/runtime initialization separately if needed.
6. Extend affected-frontier logic.
7. Handle file add/remove/rename.
8. Test cycles and same module names in different packages when package layer exists.

## Recipe 9 — Add checker type facts

1. Do not edit `ValueShape` for type algebra.
2. Add canonical `TypeStore`/`TypeId` design.
3. Resolve annotation syntax through semantic names.
4. Add per-binding/callable declared/inferred type maps keyed by existing IDs.
5. Build `synthesize`/`check` operations using shared dispatch/scope/flow.
6. Add constraint solver/substitution.
7. Add flow refinement on shared program points.
8. Add provenance and diagnostic obligations.
9. Integrate invalidation with module/callable dependencies.
10. Let LSP consume typed facts when available and fall back to shape facts.

## Recipe 10 — Add a contract proof

1. Parse/resolve contract expression.
2. Determine whether operation semantics are trusted for logic translation.
3. Lower callable body/CFG program points.
4. Seed parameter/type/precondition facts.
5. Run cheap abstract analyses.
6. Build obligation.
7. Return `Proved/Refuted/Unknown`.
8. Optionally pass residual supported formula to SMT.
9. Store proof evidence/source ranges.
10. Typed runner inserts runtime check only according to checker-mode policy.

## Recipe 11 — Add native/stdlib semantic signature

1. Prefer visible `.ph` declaration/type annotation.
2. If Rust-only behavior remains, centralize metadata in native/core semantic registry.
3. Specify return fact/type and effects.
4. Make unknown fields conservative.
5. Add runtime conformance tests.
6. Ensure metadata participates in same dispatch/callable summary path as source methods.
7. Avoid feature-specific hardcoded LSP branches.

## Recipe 12 — Add fiber/yield/block semantics

1. Read concurrency/runtime spec first.
2. Distinguish `may_yield_fiber` from `may_block_thread`.
3. Annotate direct scheduler/native origins.
4. Propagate through callable summaries.
5. Define callback effects across fiber transfer.
6. Decide refinement invalidation/reentrancy implications.
7. Expose diagnostics/lints only after effect semantics are complete.
8. Test nested calls, dynamic calls, FFI, callbacks, cancellation/join primitives.

## Recipe 13 — Migrate legacy LSP inference into semantic engine

1. Characterize old observable behavior with fixtures.
2. Identify semantic question old code answers.
3. Add/extend shared semantic fact/query.
4. Run old and new implementations differentially on corpus if practical.
5. Switch consumer to shared query.
6. Delete old AST walker/cache.
7. Add regression that prevents reintroduction of duplicate path.
8. Measure update/query latency.

## Recipe 14 — Introduce explicit CFG

1. List analyses requiring CFG; do not build for one cosmetic feature.
2. Define source-mapped body/block/program-point IDs.
3. Lower current AST constructs preserving exact dynamic semantics.
4. Differentially compare current structured-flow outputs.
5. Port one analysis at a time.
6. Keep occurrence/source maps intact.
7. Add loop/exception/non-local return control edges.
8. Delete old duplicated flow code only when parity achieved.
9. Benchmark build cost and per-query reuse.
