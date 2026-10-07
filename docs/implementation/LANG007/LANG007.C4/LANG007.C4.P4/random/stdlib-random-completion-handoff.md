# `universe.random` completion continuation handoff

Plan state: IN_PROGRESS / PARTIAL / FOCUSED_TESTED. The source-authored random
API is committed in `b87e0cec`; continuation started on `main` at `ad1db231`. The user subsequently requested
lifecycle updates and commit/push of the entire current tree, including existing
COLL006, slotmap, and math-document relocation work. Those additional changes
are included as requested without claiming verification by this random task.

## Established implementation

The public deterministic API is source-authored: explicit mutable
`Xoshiro256PlusPlus` state, SplitMix64 seeding, xoshiro256++ transition,
sampling helpers, typed `UniformInt`/`Bernoulli` distributions, and collection
helpers. `SecureRandom` remains a distinct capability with one narrow OS-byte
fill native operation and source-owned allocation/error lifting. Stable
callable ownership, generic trait recipe transport, and trait-local slot
numbering were repaired at their general owning boundaries before the source
API landed. See the implementation plan and incident packet for the invariants.

## Continuation changes and evidence

- Public source docs clarify deterministic replay and explicitly warn that
  Xoshiro is not cryptographically secure.
- The random corpus fixture checks the seed-42 sequence across forced GC.
- A secure byte-count regression confirms an unrepresentable count fails at
  `Bytes.new` before invoking host fill. It does not establish safe handling of
  very large counts that fit the immediate integer range.
- The native census test checks a single canonical `SecureRandom.rawFill(_)`
  source anchor, one descriptor, exact mutation/nondeterminism/blocking
  metadata, and no native deterministic Generator methods.
- The builtin provider test checks the facade's public exports and all six
  child source modules.
- Manual self-checking benchmark programs cover `nextU64`, `nextFloat`, small
  and 128-bit bounded integers, 1 KiB bytes, 100/10,000-element shuffle, and a
  source MINSTD baseline. They are in `benchmarks/random/`.

Focused checks completed during this continuation:

- Core canonical secure-native owner/effect assertion: PASS 1/1.
- Secure random unrepresentable byte count: PASS 1/1.
- Random language corpus, including forced-GC fixture: PASS 2/2.
- Numeric-policy sampling: PASS 1/1.
- Builtin random provider: PASS 1/1.
- Native surface generated check: PASS, 400 primitive declarations.
- Scoped Rust formatting check and `git diff --check`: PASS.

Previously completed evidence remains: core random lane PASS 4/4, core trait
suite PASS 18/18, secure native unit tests PASS 2/2, four crate checks, and
semantic binder/proof plus cold/incremental regressions. These are inherited
from the earlier delivery record, not rerun during this continuation.

## Architect decisions and blocker repairs (2026-10-08)

The user renewed fix authority in the architect role. Work remains inline on
main without subagents. R11's source mutation-effect presentation deferral
stands; native metadata and canonical source identity retain their obligations.

R21 is resolved at the general owner boundary. `VM.max_byte_buffer_len` defaults
to 64 MiB and may be configured by the embedder independently of numeric
budgets. `Bytes.new` checks the limit before a fallible backing reservation,
then zeroes and publishes the fixed-length buffer. Policy rejection and host
reservation failure have distinct controlled runtime errors. Ordinary source
and secure/deterministic random helpers share the boundary. This does not claim
a total VM memory budget or recovery from OS process termination. Small limit,
zero/exact boundary, huge representable index, capacity overflow, and the public
secure caller have focused PASS evidence. No giant allocation was attempted.

Release startup was blocked by semantic expression-owner stack mutation inside
a debug assertion. Moving the pop outside the assertion preserves the same
canonical ownership in both build profiles. An optimized build and the original
syntax fixture now PASS; the fixture printed `1` (13.17 s, 619,413,504-byte peak
RSS before the subsequent idle-polling repair).

Retained compilation now uses a fallible semantic publication API and cannot
silently compile last-known-good fixture source after an update failure. The
general session repairs schedule exact surface member prerequisites, revalidate
consumed effective definitions, preserve constructor body/factory signature
correspondence, and retain witness body/access facts through accepted source
coordinates. Seed changes, import removal, and restoration pass current-source
and cold parity checks. The expanded actual-fixture regression FAILS at `session.rs:6169`: accepted
inherent callable must have a canonical signature. The simple four-step sequence
passed before expansion; retained actual-fixture acceptance remains blocked.

A stopped million-word release run spent virtually all sampled main-thread time
in idle OS polling. The reactor now skips OS polling only when its canonical
registration table has no active source, while always draining worker ingress.
Active polling, executor waits, rooting, and scheduling remain unchanged.

## Verification and remaining gates

The affected incremental suite completed: 208 PASS / 36 FAIL / 4 ignored.
All 36 failures reproduced with the session repair removed, including the three
large reuse cases (205.16 seconds). They are Category D baseline failures; the
walkthrough retains exact identities. No broad PASS is claimed.

Final focused core-lib selection: 20 PASS / 1 FAIL / 147 filtered, 28.74 seconds.
All 16 reactor tests, the direct Bytes boundary test, and three existing retained
session tests passed. The new expanded retained random fixture regression failed
at the inherent-callable signature invariant above. Do not weaken that invariant.

The user requested delivery at this point. The final optimized CLI rebuild,
renewed workload timings, allocation/GC measurements, fresh random corpus/core
trait gates, full corpus, broad core/release certification, and cross-platform
execution remain incomplete. The prior optimized binary passed syntax startup;
the million-word run was interrupted at 225.51 seconds, before idle reactor
repair. No completed random workload throughput is available.
Keep IN_PROGRESS / PARTIAL / FOCUSED_TESTED. Next action: diagnose retained
inherent-signature publication, rerun the exact retained regression, then resume
remaining R20/R21 and plan acceptance gates.

Delivery hygiene: scoped Rust formatting and `git diff --check` PASS. The nested
expression-owner unit regression PASS 1/1. No broad test PASS is claimed.
