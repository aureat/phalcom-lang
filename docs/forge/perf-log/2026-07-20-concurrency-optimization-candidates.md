# Concurrency optimization candidates — 2026-07-20

Third lens of the concurrency audit (after
[correctness](../../logs/2026-07-20-concurrency-correctness-audit.md) and
[edges](../../logs/2026-07-20-concurrency-edge-matrix.md)): where the
fiber/scheduler/Future stack spends work it doesn't have to. **Everything here
is a candidate, not a claim** — per ADR-0051, nothing lands without an in-repo
benchmark, a profile naming the mechanism, and a before/after row in
`SCOREBOARD.md`. No numbers appear below because none were measured in this
pass; `SCOREBOARD.md` is the only number authority.

Standing context from this log (measured previously, respected here):
fiber-pool buffer recycling measured **net negative** (2026-07-14) — the lever
is the fiber shell's lifetime, not its buffers. Candidates below avoid that
alley.

## Candidates, ranked by expected mechanism strength

### 1. Future `_state` is compared as strings on every hot operation
`core.ph`: `_state` holds `"pending"`/`"fulfilled"`/`"rejected"`; `isReady`,
`settleValue`, `settleError`, `value`, `await`, `then`, `map`, `catch` all do
string `==` against a literal — content comparison, twice per settle, once per
await-check, and the literal may be freshly built per evaluation. Symbols
(`#pending`) or the `True`/`False` pair for a split `_settled`/`_rejected`
representation are identity comparisons. Measure: a settle/await microbench
plus `opcode_stats` (copy the `opcode_stats.rs` pattern — the vm-trace feature
is not usable for this) over a then-chain workload.

### 2. Every `then`/`map`/`catch` continuation becomes a full Fiber
`Future#drain` funnels **block** waiters through `System.schedule(_)`, which
wraps each in a fresh `FiberObject` (`primitive/system.rs:56-63` →
`new_fiber_ref`): two Vecs, a BTreeMap, a HashSet, a heap slot — per
continuation whose body is usually three sends. A pump-side thunk path (run a
block waiter directly on the pumping fiber, keeping fiber-wrapping only for
actual `await` waiters) removes the whole shell. Interaction to preserve:
failure isolation — a continuation that raises must not kill the pump; the
flat-entry world (`5ba6101`) makes "run a block on the current loop" cheap, but
the capture boundary needs an explicit design decision. Couples with the E007
fix — if completion machinery reworks `drain`, fold this in rather than
optimizing the current shape twice.

### 3. `Future.async` allocates two fibers per task
Driver + action fiber (`core.ph:1642-1655`). The E007 repair (completion
machinery) most likely deletes the driver entirely — completion notification
replaces the babysitting fiber. Halves fiber shells per async task as a *side
effect* of the correctness fix. Sequence: fix E007 first, then measure; do not
tune the current driver.

### 4. Root-await pump pays an Option heap fetch per drain step
`await`'s root branch does `System.nextScheduled` → `Some(fiber)` allocation +
`isNone` dispatch + `unwrapOr` per queue pop (`core.ph:1621-1628`), where the
native root-drive pump (`dispatch.rs:296-299`) pops the queue raw. This is the
ADR-0044 deferral (Option niche-encoding) showing up on a concurrency path;
also fixable locally by a native "pump until settled or quiescent" primitive —
but that trades against the ADR-0019 floor-admission rule (speed is never
sufficient), so the niche-encoding pass is the legitimate route.

### 5. Flat-entry's own win is unmeasured
`5ba6101` moved bytecode block calls from recursive `run_until` re-entry onto
the flat dispatch loop — a per-`each`/`map`/`filter`-element saving completely
apart from its yield-legality purpose. Worth one SCOREBOARD row (combinator
microbench, for-vs-each gap) to know what it bought; the for-loop seam
(`core-ph-call-chain-seam`) numbers predate it.

### 6. Switch path is already right — leave it
`store_live_into`/`load_live_from` are four `mem::take` pointer moves; resume
delivery is truncate+push into a recorded slot. O(1), allocation-free,
matches ADR-0030 §3's design intent. The only structure that *grows* with
fiber count is parked-stack mark time in GC (linear in parked live data,
non-moving, no barrier) — nothing to do until a workload shows it.

## Non-candidates (measured or ruled dead)

- **Fiber-pool buffer recycling** — measured negative 2026-07-14; also
  interacts badly with E002 (recycled buffers turn the dangling-upvalue panic
  into a silent stale read). Leave the feature off.
- **Eager per-fiber stack presizing** — measured negative (F18 family).
- **"Move the pump into Rust for speed"** — blocked by ADR-0019's admission
  rule as a speed-only motive; the legitimate versions are candidate 4's
  niche-encoding or a capability argument, not a performance one.

## Measurement discipline for whoever picks these up

`uptime` gate before any A/B (`ab-guarded` refuses a loaded box; wait for
`load1 < 0.5` — the runner's own child adds ~1.0). Two-build rule for
instruction counting (never time a counting build). Copy `opcode_stats.rs`
for new instrumentation; the `vm-trace` feature is double-gated and unusable
as-is.
