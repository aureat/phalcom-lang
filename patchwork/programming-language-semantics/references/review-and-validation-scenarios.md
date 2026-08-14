# Review Checklist and Validation Scenarios

Use these as pressure tests for agents proposing semantic, compiler, analyzer, or optimizer changes. Correct answer must invoke semantic invariant, not merely say "add a test."

## Core checklist

- Evaluation order explicit and exactly-once preserved.
- Binding identity separate from spelling/storage.
- Block construction separate from invocation.
- Abrupt outcomes carry target/meaning explicitly.
- `super` preserves receiver and changes lookup start.
- Class-side dispatch follows metaclass semantics.
- Selector identity excludes type metadata unless new mechanism is explicit.
- Direct/cached/reflective dispatch share access rules.
- Module identity/loading/initialization separated.
- Fiber yield/blocking/cancellation explicit.
- Static facts state dynamic guarantee/uncertainty.
- Optimizations state observations/effects preserved.
- Native assumptions identified as trusted/checked.
- Recovery-only semantics not called executable semantics.

## Scenario 1 — argument storage reorder

Proposal: canonical selector storage sorts labels, so compiler evaluates arguments in sorted order.

Expected: reject. Association/storage can canonicalize after evaluation; lexical order is observable through effects/errors/yield.

## Scenario 2 — `super` as superclass object

Proposal: compile `super.foo()` by loading superclass object and sending.

Expected: reject. `super` preserves `self`; only lookup start changes from lexical defining class.

## Scenario 3 — eager block effect

Proposal: block body writes `x`, therefore record `x` changed immediately after literal.

Expected: reject. Construction captures; invocation performs latent write. Propagate when surrounding call is known to invoke block.

## Scenario 4 — private reflection bypass

Proposal: reflection returned `Method`, therefore caller may invoke it.

Expected: reject unless API explicitly grants authority. Discovery/identity is separate from permission.

## Scenario 5 — textual include modules

Proposal: import concatenates source.

Expected: reject if module semantics promise identity, namespace ownership, initialization-once, cycles, or module-qualified classes.

## Scenario 6 — syntax-only algebraic rewrite

Proposal: `x + 0 -> x` everywhere.

Expected: reject. `+` is overridable dispatch and rewrite may remove errors/effects.

## Scenario 7 — cooperative means no concurrency analysis

Proposal: one OS thread means field facts survive `await`.

Expected: reject for shared state; other fibers can run across suspension.

## Scenario 8 — type annotation chooses runtime implementation

Proposal: checker picks same-selector method based on annotations.

Expected: reject under ordinary selector semantics. Separate multimethod layer required.

## Scenario 9 — recovery node executes as Unit

Proposal: compiler turns parser recovery expression into Unit.

Expected: reject unless language explicitly defines recovery execution.

## Scenario 10 — allocation DCE

Proposal: remove unused `new Foo()`.

Expected: require proof constructor/allocation has no observable effects, identity/reflection/resource consequences, errors, or yielding.

## Scenario 11 — immortal inline cache

Proposal: cache keyed receiver class+selector forever.

Expected: reject if method replacement/hierarchy change exists. Require version/invalidation or reflection restriction.

## Scenario 12 — mutable capture by copy

Proposal: copy mutable integer into closure at creation.

Expected: reject if semantics shares mutable cell; later writes must be observed by all captures.

## Scenario 13 — native signature silently trusted

Proposal: checker assumes native return metadata sound without trust policy.

Expected: identify native contract as TCB or validate boundary.

## Scenario 14 — cleanup via Rust Drop

Proposal: rely on Rust destructor unwinding for language ensure.

Expected: only valid if ordering, exception precedence, control outcomes, and cancellation exactly match semantics. Host mechanism does not decide language policy.

## Scenario 15 — cwd as module root

Proposal: resolve relative imports against process cwd.

Expected: reject unless cwd explicitly semantic. Project/package identity should remain stable across invocation directory.

## Scenario 16 — finite unrolling proves loop

Proposal: analyzer executes loop body three times; property accepted for all iterations.

Expected: reject. Unrolling is testing, not inductive/fixpoint proof.

## Scenario 17 — unknown becomes Any

Proposal: analyzer cannot infer value, so checker uses type `Any`.

Expected: reject unless `Any` is exactly semantics of missing knowledge. Epistemic unknown differs from top type/dynamic escape.

## Scenario 18 — flat class-side namespace

Proposal: store class methods directly on class descriptor and semantically ignore metaclasses.

Expected: optimized representation is allowed only if observed lookup/reflection remains equivalent to metaclass model.

## Scenario 19 — helper lowering leaks in stack trace

Proposal: lower source method into compiler helper visible in reflection/stack traces.

Expected: inspect source-observation guarantees; preserve source mapping if needed.

## Scenario 20 — implicit yield in primitive

Proposal: long native primitive yields for responsiveness without semantic review.

Expected: this introduces scheduling/interference point and can invalidate atomicity/proofs. It is semantic unless scheduling points deliberately unspecified.

## Scenario 21 — re-export copies class

Proposal: re-export class by creating equivalent new class descriptor in importing module.

Expected: likely reject because it changes class identity/instance-of/dispatch/reflection; re-export should normally alias same target identity.

## Scenario 22 — old reified method tracks replacement

Proposal: existing `Method` object silently starts calling replacement implementation after method table update.

Expected: must be explicit. Decide whether reified Method captures implementation or is live indirection; both have different identity semantics.

## Scenario 23 — tail-call elimination removes home frame

Proposal: eliminate frame that an escaping block uses as non-local return home.

Expected: reject unless non-local return semantics is preserved via heap/control representation and liveness identity.

## Scenario 24 — static prover keeps shared invariant across yield

Proposal: object invariant established before `await`, assumed after await without ownership/effect reasoning.

Expected: reject; another fiber may mutate shared object.

## Agent self-test

For any proposed answer require:

1. name semantic observation at stake;
2. state normative relation/invariant;
3. state remaining implementation freedom;
4. identify analyzer/checker/prover assumption affected;
5. propose distinguishing conformance fixture.
