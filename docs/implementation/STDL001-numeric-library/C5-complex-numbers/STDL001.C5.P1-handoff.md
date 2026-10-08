# STDL001.C5.P1 handoff

State: COMPLETE / IMPLEMENTED / FOCUSED_TESTED. CHECKPOINT.md is authoritative.

## Repository anchor

Worktree `.worktrees/complex-math`, branch `feat/complex-math`, base `ad1db231`.
Implementation commit `a72d1f79` is published on `origin/feat/complex-math`.
Preserve unrelated main-worktree work.
The user prohibits subagents and requires approval for significant choices.
D1–D9 are approved. Commit/push were authorized after focused completion.

## Established surface

`universe.numbers` exports Complex and Addable/Subtractable/Multipliable/Negatable/
IntegerPower. The source data owner supplies exact Int arithmetic, guarded Float
arithmetic and exact Int/Float integer-power witnesses. Negative Int powers raise
ArgumentError; Float negative powers use reciprocal division. Zero exponent is one.

`universe.math.complex` exports 16 elementary Families plus powInt and principal
complex pow. Signed zeros select cut sides; special values return IEEE components.
Construction/conversion is explicit. The public contract and zero-base power
endpoints are in `docs/specs/complex-numbers.md`.

## Takeover map

- `numbers/complex.ph`: immutable data, capabilities, conditional operations,
  exact IntegerPower witnesses.
- `numbers/float_complex.ph`: scalar-pair scaling, division, signs and polar.
- `math/complex/kernel.ph` and `package.ph`: principal numerical kernels/Family
  facade; README and CPYTHON-LICENSE retain pinned adaptation provenance.
- `typing/identity.rs`: complete metadata/overlay type graph equivalence using
  stable declarations and full arguments. Fingerprints alone never prove equality.
- `data.rs::intern_exact_with_component_slots`: representative exact descriptors;
  incoming handles alias the first descriptor and retain its physical layout.
- `modules/materialize.rs`: relocation of conditional invocation and captured
  Family call-environment recipes to the materialized metadata pool.
- `compiler/lib/scope.rs`: proven operator transport and body-kind receiver
  emission. Methods use GetSelf; block literals capture lexical self.
- `vm/mod.rs::semantic_same`: Int content exactness across heap/immediate
  representations without Float coercion.

## Evidence and residuals

Final Complex group: 12 passed. Adjacent Float IEEE/backend matrix: one passed.
Builtin provider: four passed. Scalar Int exactness and reentrant receiver/capture
regressions passed with observed red/green evidence. Scoped Rust format and diff
checks passed. Exact commands and inherited D4/D5 evidence are in the walkthrough.

The broad data suite was not green: four unrelated pre-runtime fixtures remain
deferred after the patch's fifth failure was corrected. Two fixtures mutate
immutable lets; two lack static-const lowering projections. Names and commands
are in CHECKPOINT.md. Leave these to their owning work.

Workspace build/test/clippy and release certification have not been run.
FOCUSED_TESTED must not be promoted to RELEASE_COMPLETE without completed gates.

## Next action

No Complex implementation task remains in this checkpoint. The brief
`deferred-data-fixtures-handoff.md` directs the four-fixture repair follow-up.
Begin with `git status --short` and the scoped diff;
read CHECKPOINT.md, the normative Complex specification and the walkthrough.
Use the completed focused evidence unless source changes create a new concern.

The resolved D4/D5/D6 and power incidents record the causes and adopted fixes.
Do not redesign canonical identity, add Dynamic arithmetic fallbacks, create
Complex runtime tags, or reopen inherent specialization to implement power.
No Rational/probability/Iterable follow-up is authorized by this handoff.
