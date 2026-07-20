# 15 — Performance Methodology

Knowing whether you actually made it faster. The through-line: *a number is a claim, and
most performance claims are false for reasons that have nothing to do with the code.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — The benchmark that measured nothing

A colleague benchmarks a hash function:

```java
long t0 = System.nanoTime();
for (int i = 0; i < 100_000_000; i++) {
    hash("hello");
}
long t1 = System.nanoTime();
// reports 0.31 ns/op
```

A 0.31 ns/op result on a ~3 GHz machine is roughly one cycle per call.

1. Give the three distinct compiler transformations that could each independently produce
   this number, and say which one you would suspect first and why.
2. JMH exists specifically to prevent this. Name the two mechanisms it provides for the
   transformations above, and explain why a naive `if (result == 42) System.out.println()`
   sink is not equivalent to the real one.
3. JMH forks a fresh JVM for each benchmark by default, and this is not about isolation from
   other processes. What is it actually defending against? Name the mechanism.

### Q2 — Two numbers from the same build

You run the same workload two ways:

- Single process, one iteration: **840 ms**.
- Single process, iteration 200 of 200: **95 ms**.

Nothing changed between them.

1. Account for the 8.8× gap with at least three distinct mechanisms, in the order they stop
   contributing over the run.
2. Your users run this workload once, in a short-lived process, and exit. Which number is
   the honest one to report, and what does that imply about which optimisations you should
   be working on?
3. LuaJIT and HotSpot both have a failure mode where *more* warmup makes a benchmark
   diverge further from production behaviour. Explain the mechanism for one of them.

### Q3 — The microbenchmark that lies in the right direction

Your new string-interning scheme:

- Microbenchmark of `intern()` in a tight loop: **40% faster**.
- Full application benchmark: **2% slower**, reproducible.

1. Give three mechanisms by which a component can get faster while the whole system gets
   slower. At least one must not be about memory.
2. Explain the phrase "a microbenchmark can be anti-correlated with the real workload"
   using a concrete cache-resident-versus-not argument.
3. You believe the microbenchmark anyway and want to prove the app benchmark is wrong.
   Describe the measurement that would actually settle it — not "profile it", a specific
   measurement.

### Q4 — The box is not quiet

You have a build A and a build B, and you run them alternately on your laptop while a
compile is running in another terminal. B comes out 6% faster. You ship it.

1. State a defensible load gate — an actual number, with the reasoning for it — that a
   benchmark harness should refuse to run below. Your reasoning must account for the fact
   that the benchmark process itself contributes to load.
2. Even on an idle machine, three hardware-level effects will move your number between two
   identical runs. Name them and say which direction each biases a *sequential* A-then-B
   comparison.
3. Give the two mitigations that are cheap and the one that is expensive but sound.

### Q5 — The optimisation that was reverted

An engineer moves a hot function to the top of a translation unit and adds
`__attribute__((aligned(64)))`. Benchmarks show 4% improvement, reproducible across ten
runs. It is merged. Three weeks later, after unrelated changes, the benchmark is 5% *worse*
than before the change, and the change is reverted with no explanation anyone believes.

1. Explain what was actually measured. Name the paper-grade result that established this
   effect is large enough to flip conclusions.
2. There is a documented CPU erratum that made exactly this class of effect a first-class
   toolchain concern. Name it and say what the mitigation does.
3. What is the methodologically sound way to measure a change whose effect is the same
   order of magnitude as layout noise? Name the technique and its cost.

### Q6 — The mean is the wrong summary

Twenty runs of a request handler, in milliseconds:

```
12 11 12 13 11 12 11 340 12 13 11 12 12 11 13 12 380 11 12 12
```

Mean: **46.65**. Median: **12**.

1. Report this honestly in one line. Then say what the mean is *actually measuring* here —
   it is a real quantity, just not the one people think.
2. Someone proposes reporting "mean ± standard deviation" and running 3 iterations. Give
   the two independent reasons this is wrong for latency data, one about the distribution
   and one about the sample size.
3. For an A/B on *throughput* (not latency), what is the honest statistical procedure, and
   why is "run it 30 times and compare means" still not sufficient?

### Q7 — The load generator that lied

A benchmark harness sends one request, waits for the response, records the latency, and
repeats — for 60 seconds at a nominal 1000 req/s. It reports p99 = 8 ms. In production the
same service shows p99 = 900 ms under the same nominal load.

1. Name the phenomenon and explain the mechanism precisely — what happens to the recorded
   samples during a 500 ms stall?
2. Give the two fixes, and say which one is a change to the harness's *architecture* rather
   than its arithmetic.
3. Explain why this makes the reported p99 not merely optimistic but *systematically*
   optimistic in proportion to how bad the stall was.

### Q8 — Instrumentation changes the thing

You add opcode counters to an interpreter's dispatch loop:

```rust
#[cfg(feature = "opcode-stats")]
{ self.counts[op as usize] += 1; }
```

The feature is off in the timing build.

1. Even compiled out, adding this can change the timing build's performance. Give two
   mechanisms.
2. State the two-build protocol and the rule that follows from it. Why is "just measure
   both with the counters on, and compare relatively" not a valid escape?
3. A tracing framework left in with a `LevelFilter::OFF` check at the top of each hot-path
   call site is the same problem in a worse form. Explain what is worse about it, and what
   the honest fix is.

### Q9 — Where the time actually goes

Two tools disagree about the same program:

- A sampling profiler: 62% of samples in `Interpreter::run`.
- `perf stat`: IPC 0.4, `branch-misses` 3.1% of branches, `LLC-load-misses` high.

1. `Interpreter::run` being 62% is nearly content-free. Explain why, and name the two
   things that destroyed the attribution.
2. Sampling profilers on managed runtimes have a specific, documented bias. Name it, name
   the mechanism, and name the class of tool that avoids it.
3. When would you reach for Callgrind instead, and what specifically must you not conclude
   from its output?

### Q10 — Reading the counters

```
        3,201,993,441      cycles
        1,290,442,001      instructions        #  0.40  insn per cycle
          402,113,882      branches
           12,571,918      branch-misses       #  3.13% of all branches
           31,004,552      L1-icache-load-misses
            2,904,113      LLC-load-misses
```

1. IPC 0.40 on a 4-wide machine means the core is stalled roughly 90% of its issue
   capacity. Rank the three candidate causes visible above by how much of that stall they
   can plausibly explain, and justify the ranking with cycle costs.
2. Branch-miss rate of 3.13% sounds small. Convert it into a cycle budget and say whether
   it is small.
3. The top-down methodology would classify this workload into one of four buckets. Name the
   buckets, say which one you would bet on here, and name the counter you would collect next
   to confirm.

### Q11 — "Allocation is slow"

A team removes an allocation from a hot loop by reusing a buffer. Expected: large win.
Measured: within noise. Separately, a different change that allocates *more* but stores
results contiguously is 25% faster.

1. Explain why bump-pointer allocation in a generational nursery is not, by itself,
   expensive. Give the actual instruction sequence.
2. Given that, name the three costs that people are *really* observing when they say
   "allocation is slow", and say which one the contiguous-storage change fixed.
3. When *is* removing an allocation the right fix? Give the specific signature in the
   measurements that tells you so.

### Q12 — Two layouts

```
// A: array of structs
struct Particle { x: f32, y: f32, z: f32, mass: f32, id: u64, name: String }
particles: Vec<Particle>

// B: struct of arrays
xs: Vec<f32>, ys: Vec<f32>, zs: Vec<f32>, masses: Vec<f32>, ids: Vec<u64>, names: Vec<String>
```

The hot loop reads `x`, `y`, `z` and writes `x`, `y`, `z`.

1. Compute the bytes-per-useful-byte ratio for both under a 64-byte cache line, and say what
   that predicts about the memory-bound case.
2. Name two things layout B costs that do not appear in that calculation, one about the code
   and one about the workload.
3. There is a third layout that is usually better than both for this specific loop. Name it
   and say what constraint it imposes.

### Q13 — The scaling that collapsed

A work-stealing scheduler with per-worker statistics:

```rust
struct Worker { queue: Deque<Task>, tasks_run: u64, steals: u64 }
workers: Vec<Worker>   // contiguous
```

Throughput: 1 thread 1.0×, 2 threads 1.9×, 4 threads 2.1×, 8 threads 1.7×.

1. Name the effect, and explain the coherence-protocol mechanism that turns an increment of
   a *private* counter into cross-core traffic.
2. Give the fix, and say why the padding constant is not 64 on every machine you care about.
3. Explain why this bug is nearly invisible to a sampling profiler, and name the counter
   that does show it.

### Q14 — Refute the dispatch proposal

A proposal: "replace the `switch` in the interpreter loop with computed-goto threaded
dispatch. Published work shows large reductions in indirect-branch mispredictions, so this
should be a significant win."

1. State the actual mechanism by which threaded dispatch improves branch prediction — be
   specific about what the predictor gets that it did not have.
2. This argument was substantially correct in 2003 and substantially weaker on modern
   hardware. Explain what changed, and name the class of predictor responsible.
3. Given the `perf stat` output in Q10, what is the largest win threaded dispatch could
   deliver on that workload? Show the arithmetic, and decide whether to build it.

### Q15 — Refute it without running it

A proposal: "our interpreter loop is thrashing L1i. If we reorder the opcode handlers so
the hot ones are adjacent and pad the loop to fit in cache, we should see a large win."

The interpreter has ~180 opcodes. Disassembly shows handlers averaging ~90 bytes.

1. Do the back-of-envelope calculation for two targets: a typical x86 core with a 32 KiB
   L1i, and an Apple M-series performance core, whose L1i is far larger. State the
   conclusion for each.
2. The proposal cited a measured `L1-icache-load-misses` count as evidence. Explain why
   that counter being nonzero does not support the claim, and what you would need instead.
3. Generalise: give the shape of a back-of-envelope refutation, and say what makes one
   legitimate rather than an excuse not to measure.

### Q16 — The number you remember

In a design review someone says: "we already know allocation is the number one cost — we
measured it, it was 19× slower than the reference implementation."

1. Give three independent reasons that sentence should not be allowed to influence the
   decision, even if every word of it was true when it was said.
2. Describe the discipline that replaces it. Be specific about what a single row records —
   there are more fields than you think.
3. Why must the number be recorded *at the time of the change* rather than reconstructed
   later from the current tree? Give the mechanism, not the platitude.

### Q17 — Three percent

You have a change that is 3% faster on your benchmark suite. It adds a second
representation for one data type, with a conversion at four boundaries, and roughly 300
lines.

1. Make the rigorous version of "this is not the bottleneck" using Amdahl. State exactly
   what you need to measure to make the argument, not just the formula.
2. Give the conditions under which you ship the 3% anyway. There are at least two, and one
   of them is not about performance at all.
3. Give the conditions under which you refuse a change that is 30% faster.

### Q18 — Comparing against another implementation

Your language runs a benchmark in 2.4 s; a well-known implementation of a comparable
language runs "the same" benchmark in 0.9 s.

1. Name five ways this comparison is probably unfair, in either direction. At least two must
   favour *you*.
2. State the rules a fair cross-implementation comparison must follow. Name the methodology
   work that formalised this for language runtimes.
3. You do the work and the honest number is 2.7× slower. What is the *useful* next step —
   and why is "find where the 2.7× is" the wrong framing?

---

## Answers

### A1 — The benchmark that measured nothing

**1.** (a) **Dead code elimination** — `hash("hello")`'s result is unused and the call has no
side effects, so the whole body is removed and the loop becomes empty. (b) **Loop-invariant
code motion / constant folding** — the argument is a compile-time constant and `hash` is
pure, so the call is hoisted out of the loop and evaluated once; you measured one call
divided by 10⁸. (c) **Empty-loop elimination** — having removed the body, the loop itself is
removed, and you measured two `nanoTime` calls. Suspect (b) first: the argument is a
*literal*, which is the loudest possible signal, and it is the transformation that survives
even if you do sink the result — the sink sees the same constant every iteration and the
compiler is entitled to compute it once. DCE is the more famous answer and the less likely
one when a constant is present.

**2.** JMH provides **`Blackhole.consume()`** against DCE and **`@State`-held, non-`final`
fields** for inputs against constant folding. A hand-rolled `if (result == 42) print()` sink
is not equivalent for two reasons: it is a *branch the compiler can prove is almost never
taken*, so the value may still be computed speculatively or narrowed, and more importantly
it introduces a conditional whose cost is data-dependent and which perturbs the loop's own
branch prediction. Blackhole's whole design problem is consuming a value in a way the
compiler cannot see through while itself costing a known, small, *constant* amount — which
turns out to be genuinely hard, and is why the implementation is more elaborate than a
volatile store, and why JMH later added compiler blackholes (OpenJDK 17), where the JIT
carries the value through the optimisation phases and then emits nothing at all for the sink
— no call and no barrier — cutting the per-op sink cost from roughly 3.2 ns to about 1 ns.

**3.** **Profile pollution.** In a shared JVM, benchmark A's execution fills the profiling
data at shared call sites — a send site that A made megamorphic stays megamorphic for B,
inlining decisions made for A's types persist, and a class B loads may invalidate
assumptions the compiler made while running A. The result is order-dependence: running A
then B gives different numbers from B then A, and it is not noise, it is deterministic
history. Forking gives each benchmark a fresh profile. The same reasoning applies far beyond
Java — any runtime with inline caches, hidden classes, or a tiered compiler has this, and it
is the reason "run all the benchmarks in one process to save startup time" is a trap.

**Trap.** "I stored the result in a variable, so it can't be eliminated." A local assignment
that is never read is dead too, and even a field store can be eliminated if the field is
provably unread. The only reliable sinks are ones the compiler cannot analyse — a volatile
write, an escape, or a purpose-built blackhole — and each of those has its own cost you must
know and subtract.

### A2 — Two numbers from the same build

**1.** In the order they stop mattering:

- **Interpretation before compilation.** Early iterations run in the interpreter or a
  baseline tier. HotSpot goes interpreter → C1-with-profiling → C2; V8 goes Ignition →
  Sparkplug → Maglev → TurboFan. The first iterations run code that is 10-50× slower than
  the final tier. This dominates the first few iterations and then stops.
- **Profile accumulation and re-compilation.** The optimising tier needs enough samples to
  decide what to inline and which types are monomorphic. Until the profile is representative,
  the compiled code is compiled for the wrong thing, and there are deoptimisation/recompile
  cycles in the middle of the run.
- **Class loading, linking, resolution, and lazy initialisation.** Every first call to a
  method resolves a symbolic reference; every first touch of a class runs its initialiser.
  One-time, front-loaded.
- **Caches, everywhere.** Cold I-cache, cold branch predictors, cold TLB, cold OS page cache
  and file-system cache, cold CPU frequency (the core is at a low P-state until there is
  demand). Also cold *heap*: the first iterations touch fresh pages and take soft page
  faults; the nursery has never been collected so the collector's costs have not appeared
  yet.
- **GC reaching steady state.** Early iterations may not have collected at all, so a
  short-run number can be *artificially good* by never paying for the garbage it made.

**2.** The honest number is **840 ms**, and it implies that essentially none of the
optimisations that pay off in a steady-state benchmark are the ones you should be working
on. Your levers become startup cost, lazy initialisation, tiering thresholds, whether you
need an optimising tier at all, ahead-of-time compilation, snapshotting an initialised heap,
and reducing the work done before first output. This is why CLI tools, serverless functions,
and shells have a completely different optimisation profile from servers, and why runtimes
grew features specifically for it — class-data sharing and AOT in the JVM, V8 startup
snapshots, image-based startup in Smalltalk lineages. Reporting the steady-state number for
a short-lived workload is not a rounding error; it is measuring a different program.

**3.** **LuaJIT**: it is a *tracing* compiler — it records a linear trace of a hot loop
including the branches actually taken, then compiles it with guards. A benchmark that
exercises one path produces a trace specialised to that path; production traffic that takes
the other branch exits through a guard. A hot side exit gets its own **side trace**, so the
artifact grows a shape fitted to whichever branch the benchmark took; and if recording that
side trace repeatedly *aborts*, LuaJIT penalises and eventually **blacklists** the originating
bytecode, so the region falls back to the interpreter for the rest of the process. Note the
causal chain: blacklisting is driven by recording aborts, not by guard exits directly. So more
warmup on unrepresentative data makes the compiled artifact *more*
wrong for production, and the failure is a cliff rather than a slope. **HotSpot**: the
analogous case is a call site that is monomorphic during warmup, gets an inlined
monomorphic-dispatch fast path, and then goes megamorphic in production — a deoptimisation
and a permanently worse compilation. In both cases the general lesson is that a
speculating runtime does not merely run your benchmark, it *learns from it*, and what it
learns is part of the artifact you measured.

**Trap.** "Warm it up until the numbers stabilise, then measure — that's the standard
protocol." Stability is not convergence. A benchmark can settle into a stable number while
still in an intermediate tier, or stable *because* it deoptimised to a state it will never
leave, and a flat curve looks identical in all three cases. Worse, "until it stabilises" is a
stopping rule that depends on the data, so it systematically selects the quietest window.
Check what the runtime actually did — compilation logs, tier transitions, deopt counts — and
report the warmup policy as a fixed number of iterations decided in advance.

### A3 — The microbenchmark that lies in the right direction

**1.**
- **Cache and TLB footprint.** The intern table is now larger or has worse locality. In the
  microbenchmark it is the only live data and sits in L1/L2; in the app it competes with
  everything else, and the extra footprint evicts something on a hotter path. The
  microbenchmark cannot observe this because it has no competitors.
- **Code size and inlining.** The new `intern` is bigger — more branches, an extra fast
  path. In isolation that is free. In the app, the larger callee stops being inlined into
  its caller, which removes a set of optimisations that were worth more than the intern
  speedup, and the extra code adds I-cache pressure at a dozen call sites rather than one.
- **Non-memory:** **a change in allocation or synchronisation behaviour that shifts cost to
  another component.** A faster intern that allocates more advances the GC schedule; the
  intern loop is faster and the collector runs 15% more often, and the cost lands somewhere
  the microbenchmark does not run. Same shape with a lock: reduced hold time that increases
  acquisition frequency can increase total contention.

**2.** In a microbenchmark, the working set is whatever the benchmark touches, and it is
small, so essentially every access is an L1 or L2 hit — call it ~1-4 ns. Optimising the
*instruction count* of an operation whose data is cache-resident produces a real, measurable
win. In the real workload the same operation's data is one of thousands of live structures
and the access is an LLC miss — call it ~80 ns of DRAM latency. Now the instruction-count
win is 2 ns out of 90, i.e. invisible, while any change that *increases* the footprint costs
you additional misses at ~80 ns each. So the two effects have opposite signs in the two
regimes: the change that reduces work per operation while increasing bytes per operation
wins the microbenchmark by exactly the mechanism that loses the real workload. That is what
"anti-correlated" means here — not "unrelated", but *inversely* related through footprint.

**3.** Measure the app benchmark with `perf stat` on both builds and compare **cycles,
instructions, and cache misses at each level**, plus the runtime's own GC and allocation
counters. Three specific outcomes discriminate: (a) instructions retired went *down* and
cycles went *up* → the change is a locality/stall regression, confirmed by an increase in
LLC misses or a drop in IPC; (b) instructions went up → your "faster" version is doing more
work in context, which means the microbenchmark's fast path is not the one the app takes,
and you should count which path it takes; (c) instructions and cycles both roughly
unchanged, but GC counters moved → the cost moved to the collector. The point is that
"profile it" gives you a flat attribution that will point at the interpreter loop and tell
you nothing (see Q9); the counter deltas between two builds of the same program are far more
diagnostic than a profile of either one.

### A4 — The box is not quiet

**1.** Gate on **1-minute load average below ~0.5 per core-equivalent you intend to use**,
checked immediately before the run and again after. The reasoning that matters: the naive
gate is "load < 1.0 because I have spare cores", and it is wrong because **your own
benchmark process contributes about 0.63 to the one-minute average after 60 seconds**, and
approaches 1.0 only after several minutes — the figure is exponentially damped with a
60-second time constant, so it both understates a load that just started and lags one that
just ended.
A harness that gates at 1.5 will pass, start the run, push the load to 2.5, and then spend
the measurement window competing with whatever was already there — and because the load
average is a decaying average, it lags the actual contention by tens of seconds, so a gate
checked once at the start is measuring the *past*. Practical rule: require the pre-run
one-minute average below 0.5, re-check after, and discard the run if the post-run average
exceeds the pre-run figure by more than your process's own expected contribution. Also
verify no other run is in flight — the single most common cause of a poisoned A/B is a
second benchmark you started yourself.

**2.**
- **Frequency scaling / turbo.** Modern cores boost above base clock and drop back based on
  thermal and power budget and on how many cores are active. In a sequential A-then-B run, A
  starts on a cold package with the most thermal and power headroom, so it typically sustains
  the highest clock — the bias **favours whichever ran first**. The low-P-state ramp at
  process start pushes the other way, but it is a millisecond-scale transient (Speed Shift
  settles in about 1 ms) and is noise for any benchmark over ~100 ms. State your run length
  before claiming a direction.
- **Thermal throttling.** After sustained load, the package hits its limit and clocks drop.
  In a sequential comparison this systematically **penalises the second build**, and it is
  the reason a long benchmark suite shows a downward drift unrelated to any code.
- **Frequency-dependent measurement units.** Wall-clock time is frequency-sensitive;
  retired-instruction counts are not. A run at a different clock is a different experiment
  measured in seconds, and identical in instructions.

**3.** Cheap: (a) **interleave** — run ABABAB… rather than AAA…BBB, so drift and thermal
trend affect both arms equally, and compare paired differences; (b) **pin and normalise** —
pin to a fixed core, disable turbo or fix the frequency governor, and report
instructions-and-cycles alongside wall time so a frequency change is visible as a
discrepancy between them. Expensive but sound: **many independent process invocations with
randomised order and a non-parametric test on the resulting distributions**, which is the
only approach that actually gives you a confidence interval rather than a story. That is
what "statistically rigorous performance evaluation" means in the literature and it is
expensive precisely because it needs dozens of runs, not three.

### A5 — The optimisation that was reverted

**1.** What was measured is **a change in code layout**, not a change in code. Moving a
function changes the addresses of everything after it, which changes: which cache sets
instructions map into, whether hot branches straddle alignment boundaries, branch-target
buffer aliasing, and — for the data side — nothing directly, but the stack and heap
addresses shift too if anything about the binary's size changed. The measured 4% was a draw
in a lottery, and three weeks of unrelated changes redrew it. The established result is
**Mytkowicz, Diwan, Hauswirth and Sweeney, "Producing Wrong Data Without Doing Anything
Obviously Wrong!" (ASPLOS 2009)**, which showed that changing the size of UNIX environment
variables — which shifts the stack's starting address — or changing link order produced
performance swings large enough to reverse the measured conclusion about whether `-O3` beat
`-O2` on real benchmarks. The environment variables are the punchline: nothing about the
program changed at all.

**2.** The **Intel jump-conditional-code (JCC) erratum**. On affected Skylake-family cores,
a conditional jump whose instruction ends at or crosses a 32-byte boundary is not eligible
for the decoded-instruction cache, so the frontend must re-decode that region. The microcode
mitigation avoids the erratum's correctness issue and costs performance in exactly those
cases; toolchains responded with assembler and compiler options that pad instructions so
branches do not land on those boundaries. The relevance here: it turned "where your
instructions happen to sit" from an academic curiosity into something Intel documents at
0–4% on industry-standard benchmarks, with unspecified higher outliers — and it makes
layout-sensitivity a permanent property of the
measurement environment rather than a fluke.

**3.** **Randomise the layout and measure the distribution**, rather than measuring one
layout. The technique is layout randomisation across many runs — repeatedly re-randomising
function order, stack offsets, and heap placement so that layout becomes a random variable
you can average over instead of a fixed confound; **Stabilizer** (Curtsinger and Berger,
ASPLOS 2013) is the reference implementation of the idea. The cost is that you need many
runs to get a usable interval, you need tooling that can actually perform the randomisation,
and you lose the ability to reason about any single run. The cheap approximation available to
everyone: run the A/B across several *different builds* — different link orders, different
inlining seeds — and require the effect to survive all of them. A change whose effect is the
same size as the layout lottery is not a change you can defend with a single number, and the
honest thing is to say so rather than to produce a tighter-looking mean.

**Trap.** "We ran it ten times and got a tight standard deviation, so it's real." Ten runs
of the *same binary* re-measures the same layout draw ten times. The variance you computed
is run-to-run noise within one layout, which says nothing about the variance across layouts —
and the layout variance is the larger one. Tight error bars on a repeated-identical-binary
experiment are evidence of a stable machine, not of a real effect.

### A6 — The mean is the wrong summary

**1.** "Median 12 ms, p95 ≈ 340 ms, max 380 ms, n=20; the distribution is bimodal with two
outliers around 350 ms — the interesting question is what those two are." The mean of 46.65
is a real quantity: it is **total time divided by count**, i.e. a *throughput* statistic. It
answers "how long will 1000 of these take" correctly and answers "what will a user
experience" wrong, because no user experienced 28 ms — they experienced 12 or they
experienced 350. Latency distributions are almost always multi-modal (fast path, slow path,
retry path, GC pause), and the mean of a multi-modal distribution names a value the system
never produces.

**2.** (a) **The distribution.** Standard deviation summarises a symmetric, unimodal
distribution. Latency is bounded below (there is a minimum possible time) and unbounded
above, hence strongly right-skewed, so "mean ± sd" implies a symmetric interval that extends
below the physical minimum and understates the tail. Reporting sd here is not conservative,
it is meaningless. (b) **The sample size.** A p99 requires at least ~100 samples to have a
single observation in it and far more to estimate it with any stability; 3 iterations cannot
estimate a 99th percentile at all, and with a bimodal distribution 3 iterations has a
substantial chance of missing the slow mode entirely and reporting a clean, confident, wrong
number. The right report is quantiles from a histogram with enough samples that the quantile
you claim is populated — which is exactly what HdrHistogram is designed to make cheap.

**3.** For throughput: **many independent process invocations, interleaved between arms,
with a non-parametric comparison** — report the median difference and a confidence interval,
using a rank-based test such as Mann-Whitney rather than a t-test, because the run
distributions are not normal and often not even unimodal. "30 runs, compare means" is
insufficient for three reasons: it assumes normality that does not hold; it compares point
estimates without an interval, so it cannot distinguish a 2% effect from 2% noise; and — the
one people miss — **30 iterations inside one process is n=1**, not n=30, because everything
in Q1 and Q2 (layout draw, profile state, page placement, frequency state) is shared across
those iterations and is the dominant source of variance. Independent *invocations* are the
unit of replication; iterations within a run are a sub-sample.

### A7 — The load generator that lied

**1.** **Coordinated omission**, named by Gil Tene. The mechanism: the harness's send rate is
coupled to the system's response time, because it waits for a response before sending the
next request. When the server stalls for 500 ms, the harness sends *nothing* during the
stall. It records exactly one sample of ~500 ms — the request that was in flight — and then
resumes. But at a nominal 1000 req/s, 500 requests *should* have been issued during that
window, and each of them would have observed a latency from ~500 ms down to ~1 ms as the
queue drained. Those 500 samples are missing from the histogram, replaced by one. The
harness has quietly conspired with the system under test to omit exactly the samples that
were bad.

**2.** (a) **Arithmetic fix**: back-fill the missing samples. Record each measured value with
an expected interval, and synthesise the samples that should have been taken during the
stall — this is what HdrHistogram's interval-aware recording does. It is a correction, and it
assumes the intended rate. (b) **Architectural fix**: decouple the send schedule from the
response. The harness computes each request's *intended* send time in advance and measures
latency from the intended time, not the actual send time — so a stalled system produces a
backlog whose queueing delay is measured directly. This is the constant-throughput load
generator design; `wrk2` was written specifically to do this after `wrk` was shown to have
the bug. (b) is the architectural change and is the correct one; (a) is the patch you apply
when you cannot change the generator.

**3.** Because the number of omitted samples is **proportional to the duration of the
stall**. A 10 ms hiccup omits ~10 samples; a 1-second stall omits ~1000. So the worse the
outage, the more of its evidence disappears, and the *ratio* of recorded-bad to actual-bad
shrinks as the problem grows. This is why a coordinated-omitting harness can report a p99 of
8 ms for a system that spends 3% of its wall time completely stalled: the stall contributed
a handful of samples out of 60,000. It is not a small optimistic bias; it is a bias that
vanishes precisely when the system is behaving well and grows without bound as it behaves
badly, which is the worst possible shape for a measurement error.

**Trap.** "We measure at the server, so there's no coordinated omission." Server-side timing
starts when the request is *dequeued*, so a request that spent 400 ms waiting for a worker is
recorded as 3 ms of service time. Strictly, that is a *different* defect with the same
signature — not coordinated omission, since the generator's send schedule is unaffected, but
an incomplete definition of latency. The distinction is worth keeping, because coordinated
omission is the term most often misstated. Either way: the
queueing delay — the part the user experiences — is excluded by construction, and the metric
looks best exactly when the queue is deepest. Latency must be measured from the earliest
moment the client could have been served, which for a closed-loop harness means the intended
send time and for a server means arrival, not dispatch.

### A8 — Instrumentation changes the thing

**1.** (a) **Layout.** The `#[cfg]`'d block changes the source, which changes the compiled
output even when the block is empty — different inlining costs, different register pressure
in the enclosing function, a different function size, and therefore different addresses for
everything downstream. That is the Q5 lottery, redrawn. (b) **Structural changes that
survive the `cfg`.** The counters array has to live somewhere — usually a field on the
interpreter struct — and if the field exists in both builds, you have changed the struct's
size and the offsets of every field after it, which changes cache-line occupancy of the hot
state. Even if the field is also `cfg`'d out, the *presence* of the feature usually forces
`self` to be borrowed mutably in the loop or forces a value out of a register into memory,
and those effects do not always disappear cleanly.

**2.** **Two-build protocol: one build with counting enabled, used only to answer "how
many"; one build with counting absent, used only to answer "how long". Never time a counting
build, and never trust an instruction-count attribution from a timing build.** The rule that
follows: any number in the scoreboard must name which build produced it. "Measure both with
counters on and compare relatively" fails because the counter's cost is **not uniform across
the arms**. A change that reduces the number of executed opcodes also reduces the number of
counter increments, so the counting build exaggerates the improvement; a change that shifts
work from many cheap opcodes to few expensive ones is *penalised* in the timing build and
*flattered* in the counting build. The instrumentation's cost is correlated with the very
thing you are changing, so it does not cancel — which is the general reason "relative
comparison makes the overhead cancel" is a fallacy whenever the overhead is proportional to
something the change affects.

**3.** A tracing framework with a level check per call site is worse because the cost is
**distributed and unauditable**. A counter block is one place you can find and delete; trace
macros are at hundreds of sites, each expanding to a load of a global filter, a comparison,
and a branch, plus — this is the real damage — an argument-construction path the compiler
must keep alive enough to be able to take. The branch is well-predicted so it looks free in a
microbenchmark, but it inflates code size at every site, which is I-cache pressure exactly
where you can least afford it, and it constrains the optimiser at every site. It is also
worse epistemically: because the framework is "off", nobody accounts for it, and it becomes
invisible permanent overhead. The honest fix is compile-time removal — a feature gate that
makes the macro expand to nothing, verified by checking that the symbol does not appear in
the disassembly of the release binary — plus a timing build that contains no tracing at all.
A runtime filter is not "disabled"; it is "enabled and returning early".

### A9 — Where the time actually goes

**1.** Because `Interpreter::run` is the interpreter's dispatch loop, and in a
direct-threaded or big-switch interpreter **every opcode handler is inside it**. Saying 62%
of time is in `run` is saying 62% of time is spent running bytecode, which you knew. The two
things that destroyed attribution: (a) **inlining** — the handlers are inlined into the loop
body, so there is no callee frame to attribute to, and the samples all land in one symbol;
(b) **the absence of a language-level frame concept in the profiler's view** — the profiler
sees native frames, and the user's actual program is data being interpreted, not frames on
the stack, so no amount of native call-stack sampling can tell you which *bytecode method*
was hot. Fixing this needs either per-opcode attribution from the instruction pointer plus a
map from address range to handler, or runtime-level instrumentation that reports the
interpreted frame — which is why every serious VM ships its own profiler rather than relying
on the system one.

**2.** **Safepoint bias.** A profiler that collects stacks by asking threads to stop at a
safepoint samples only at safepoint locations, and safepoints are placed at specific
points — loop back-edges, calls, allocation sites — and are *elided* in code the compiler
proved does not need them, such as counted loops the JIT decided cannot block. The result is
that samples cluster at safepoints and the truly hot straight-line region between them is
invisible; the bias is systematic, not random, so more samples do not fix it. The documented
demonstration is Mytkowicz et al., "Evaluating the Accuracy of Java Profilers" (PLDI 2010),
which showed four widely used profilers disagreeing with each other and with the truth on
the same programs. The class of tool that avoids it: profilers that sample from a signal
handler using the OS's performance-counter interrupt and walk the stack directly rather than
requesting a safepoint — `async-profiler` on the JVM, or plain `perf` with appropriate
unwinding — combined with hardware precise-event sampling (PEBS) to reduce skid.

**3.** Reach for Callgrind when you want **deterministic, exactly reproducible attribution of
instruction counts and call graphs**, especially for a change whose effect is smaller than
machine noise, or on a machine you cannot make quiet. Its output is repeatable to the
instruction, which makes it excellent for "did my change reduce work" and for finding
unexpected call-count explosions. What you must not conclude from it: **anything about
elapsed time**. Its cache model is a simulation of a simplified hierarchy — it does not model
out-of-order execution, store buffers, hardware prefetching, memory-level parallelism, or
branch prediction realistically — so its miss counts are indicative, not predictive, and its
instruction counts are not a proxy for cycles on a machine where IPC swings between 0.4 and
3. Use it to explain *why* wall-clock moved after you have measured that it moved; never as
the source of the claim that it moved.

### A10 — Reading the counters

**1.** Ranked by explanatory power:

- **LLC misses first.** 2.9M misses at roughly 200-300 cycles each is on the order of
  600M-900M cycles — a substantial fraction of the 3.2G total, and no other line in this
  output has that kind of budget. Memory stalls are almost always the largest single bucket
  when IPC is this low.
- **Branch misses second.** 12.6M at a ~15-20 cycle recovery penalty is roughly
  190M-250M cycles, i.e. 6-8% of total. Real, worth fixing, not the main story.
- **L1i misses third**, and this one needs care: 31M L1i misses is a large *count*, but most
  L1i misses hit in L2, which costs on the order of a dozen cycles, not hundreds. At ~12
  cycles that is ~370M cycles — comparable to branch misses. The number of misses is not the
  cost; the level they are served from is.

The honest summary: memory-hierarchy stalls dominate, with branch and frontend costs each
contributing single-digit-percent-scale time. Note the total already exceeds what a naive sum
would allow, which is the reminder that these events overlap — an out-of-order core services
several misses concurrently, so you cannot add penalties naively. Which is exactly why
top-down exists.

**2.** 12.6M misses × ~15-20 cycles of pipeline refill ≈ **190M-250M cycles**, against 3.2G
total: **6-8% of all execution time spent recovering from mispredictions.** For a percentage
that reads as "3%, basically perfect", it is worth more than most optimisations anyone will
ship this quarter. The general conversion is the point: a miss rate is a rate over *branches*
and the cost is in *cycles*, and the two are related by a penalty of 15-20 cycles and by how
branch-dense the code is. In an interpreter, roughly one instruction in three is a branch, so
branch-miss rates translate into time far more aggressively than in numeric code.

**3.** The four top-down buckets, which partition every issue slot: **Retiring**, **Bad
Speculation**, **Frontend Bound**, **Backend Bound** (the last usually split into
Core-Bound and Memory-Bound). I would bet **Backend Bound / Memory Bound**, on the LLC miss
budget above. The next counter to collect: the top-down level-1 breakdown itself — via
`toplev` or the `topdown-*` events — because it apportions stall slots directly instead of
making you infer them from event counts with assumed penalties. Failing that, `cycle_activity`
stall events, which separate cycles stalled on L1D, L2, and memory. The methodological point:
this list of raw counters invites double-counting and penalty guesswork, and the top-down
methodology exists precisely because summing event×penalty is not a valid model of an
out-of-order core.

**Trap.** Adding up event counts times textbook penalties and reporting the result as a
breakdown. On an out-of-order core, misses overlap, some stalls are hidden entirely, and a
"200-cycle" DRAM access can cost near zero if there is independent work to issue. The sum
routinely exceeds 100% of measured cycles, which should be the tell. Penalty arithmetic is
for establishing *orders of magnitude and upper bounds* — which is exactly how it is used in
Q14 and Q15 — and not for attribution.

### A11 — "Allocation is slow"

**1.** In a bump-pointer nursery, allocation is: load the allocation pointer, add the object
size, compare against the limit, conditionally branch to the slow path, store the new
pointer, then write the header. That is on the order of five instructions with a
perfectly-predicted branch, and the pointer is in cache because you just used it. It is
competitive with a stack push and considerably cheaper than a general-purpose `malloc`,
which has to consult size classes and freelists. So the naive model — "allocation is a
function call into the memory manager" — is simply wrong for a generational runtime, and the
buffer-reuse change measured within noise because it removed something that cost almost
nothing.

**2.** What people are actually observing:

- **Collection cost, deferred.** Allocation is cheap; *reclamation* is not free, and the
  cost lands later, in a different part of the profile. High allocation rate means frequent
  nursery collections, and each one costs proportional to the *surviving* data. This is the
  one that makes allocation look free in a microbenchmark and expensive in an app.
- **Initialisation and zeroing.** The runtime must zero or initialise every field; that is
  real memory traffic proportional to bytes, and for large objects it is the dominant cost.
- **The resulting layout.** This is the big one. Allocating each element separately means the
  elements are wherever the nursery pointer was, interleaved with everything else allocated
  in the same window, and reached through a pointer. Iterating them is pointer chasing: one
  dependent load per element, no prefetching, a cache miss whenever the object is not
  adjacent. The contiguous-storage change fixed exactly this — it allocated *more* total
  bytes but put the hot fields adjacent, so iteration became a sequential stream the
  prefetcher handles and one cache line serves many elements.

**3.** Removing an allocation is right when the measurement shows **(a) the allocation is in
the hottest loop and the object is short-lived and does not escape** — in which case you are
really removing the *initialisation* and the GC pressure, not the pointer bump — or **(b) the
runtime's GC counters show collection time is a significant share of wall clock and the
allocation rate traces to this site.** The signature to look for is not "there is an
allocation here"; it is *collector time* moving, or *bytes allocated per operation* being
large relative to the useful output. If you remove an allocation and the collection count
does not change, you removed five instructions. If you remove an allocation and the object
was going to be iterated later, check that you did not make the layout worse in exchange.

**Trap.** "Escape analysis will stack-allocate it, so it's free." Escape analysis is real and
routinely fails: one path where the object is stored in a field, passed to a virtual call the
compiler cannot devirtualise, or captured by a closure, and the whole allocation is
reinstated — often the path your production data takes and your benchmark does not. Treat it
as an optimisation that may apply, never as a guarantee you can design against, and check the
allocation counters rather than assuming.

### A12 — Two layouts

**1.** Layout A: `Particle` is 4 f32s (16 B) + u64 (8 B) + `String` (24 B on a 64-bit target
with pointer, length, capacity) = 48 B, plus alignment. A 64-byte cache line therefore holds
roughly 1.33 particles. The loop uses 12 bytes per particle (x, y, z), so per line fetched
you use about 16 useful bytes out of 64 — a **ratio of ~4:1 waste**. Layout B: `xs` is a
dense `f32` array, so one 64-byte line holds 16 x-values, all of them used. Three streams
(x, y, z) are each fully utilised, so the ratio is **1:1**. Prediction for the memory-bound
case: layout B moves roughly a quarter of the bytes for the same work, so if the loop is
bandwidth-limited it should approach a 3-4× improvement, and the improvement will *grow*
with the number of particles as the working set outgrows each cache level. Also relevant:
three sequential streams are exactly what hardware prefetchers are best at.

**2.** (a) **Code cost**: any operation that needs a whole particle now touches six arrays
and cannot be expressed as a single value — you lose the ability to pass a `&Particle`, to
push or remove one particle atomically, to sort particles as units, and to keep the six
arrays' lengths in sync without a type that enforces it. Constructing or deleting one entity
is six operations that can fail independently. (b) **Workload cost**: if some *other* loop
touches all fields of one particle at a time — serialisation, a debugger view, a per-particle
callback — layout B is strictly worse for it: six cache lines instead of one. SoA is a bet
that your access pattern is column-major, and it loses badly on any row-major access. Which
means the right answer depends on the *set* of loops, not the hot one.

**3.** **AoSoA / hybrid — an array of small blocks, each block holding a fixed number of
entities' fields in SoA form** (say, 8 or 16 particles' worth of x, then y, then z, then the
cold fields). You get contiguous, vectorisable, prefetch-friendly access for the hot loop,
and a single entity's data stays within a bounded number of cache lines for the row-major
loop. The constraint it imposes: **a fixed, compile-time block width**, and every access
becomes an index split into a block index and a lane index, so indexing arithmetic gets more
complex and inserting/removing single elements gets harder. It is also markedly harder to
express in a language without good support for it — which is the language-design point:
layout is one of the few performance properties that a *type system* can either enable or
foreclose, and a language where the only aggregate is a heap-allocated object with a header
cannot express any of this.

### A13 — The scaling that collapsed

**1.** **False sharing.** `Worker` is small and the vector is contiguous, so several
workers' `tasks_run` fields land in the same 64-byte cache line. The coherence protocol
tracks ownership at *line* granularity, not at variable granularity: when worker 0
increments its counter, its core must acquire the line in Modified/Exclusive state, which
invalidates that line in every other core's cache. Worker 1's counter is untouched, but it
lives in that same line, so worker 1's next increment misses, must request the line back,
and invalidates worker 0. The line ping-pongs between cores at roughly the cost of a
cross-core transfer per increment — tens to a hundred-plus cycles — for a variable that is
logically private and never contended. More threads means more participants in the
ping-pong, which is why throughput goes *down* past four.

**2.** Pad and align each worker to a cache line — `#[repr(align(128))]`, a `CachePadded`
wrapper, or `@Contended` on the JVM. The constant is not 64 everywhere. On Apple Silicon the
**L1D line is 64 bytes but the L2/system-level-cache line is 128**, and macOS reports
`hw.cachelinesize` as 128 — so the effective cross-core sharing unit is 128. On modern Intel
the L2 spatial prefetcher pulls adjacent 64-byte pairs, with the same consequence. Padding to 128 is the portable-safe choice
and costs memory; this is why Rust's `CachePadded` uses a target-dependent alignment rather
than a hard 64. Getting this wrong by choosing 64 on a 128-byte-line machine reproduces the
original bug at half the rate, which is worse than not fixing it because it looks fixed.

**3.** Nearly invisible to a sampling profiler because the cost is **spread evenly across
every increment site** — each one is a few extra cycles, at a source line that reads
`self.tasks_run += 1` and looks unimpeachable. There is no hot function; the whole program
is uniformly slower, and profile percentages barely move because *everything* moved. The
counter that shows it: coherence-traffic events —`MEM_LOAD_L3_HIT_RETIRED.XSNP_HITM` /
"HITM" events on Intel, which count loads served from another core's modified cache line —
or the equivalent shared-cache-line analysis in a memory-access-focused tool. The
scaling curve itself is also diagnostic and is often the first evidence: **a throughput curve
that goes up, flattens, and then turns down as thread count increases is a coherence problem
until proven otherwise**, because pure contention flattens the curve while coherence traffic
bends it back down.

### A14 — Refute the dispatch proposal

**1.** In a `switch`-based loop, every opcode's dispatch goes through **one** indirect jump —
the jump table lookup at the bottom of the loop. The predictor sees a single branch site
whose target changes with the bytecode stream, so its history is a mixture of every
opcode-to-opcode transition in the program. Threaded dispatch **replicates the dispatch at
the end of every handler**, so there are N indirect branches instead of one. Each one now
predicts the successor of *a specific opcode*, which is far more predictable — after a
`LoadLocal` the next opcode is very often the same handful — so the predictor effectively
gets one extra opcode of context for free. That is the mechanism: not fewer branches, but
**more branch sites, each with a purer history**. Ertl and Gregg's work established this
empirically.

**2.** What changed is the predictor. The 2003 result assumed a BTB with limited per-branch
history, where a single indirect site could not learn a mixture of targets. Modern cores
predict indirect branches using **long global branch history** — no x86 vendor documents the
implementation, but Rohou et al. show Haswell performing at least as well as an ITTAGE
predictor, which is the class of design the behaviour is consistent with — meaning a
single indirect branch site *can* learn "if the previous few opcodes were X, Y, Z, the target
is now T". That is precisely the context threading was hand-building. Rohou, Swamy and Seznec
("Branch Prediction and the Performance of Interpreters — Don't Trust Folklore", CGO 2015)
measured this and found the misprediction advantage of threading dramatically reduced on
contemporary hardware, to the point where the folklore recommendation no longer follows from
the data. It is one of the cleanest available examples of a correct optimisation whose
justification expired without anybody noticing.

**3.** From Q10: mispredictions cost roughly 190M-250M cycles out of 3.2G, i.e. **6-8% of
total time**, and that is *all* mispredictions — dispatch branches, plus every conditional
in every handler, plus the guest program's own control flow. Threaded dispatch can only
address the dispatch share of that, and on a modern predictor it removes only part of that
share. So the ceiling is well under 6%, and a realistic expectation is **1-3%**. Decision:
**do not build it for the branch-prediction argument.** The honest counter-argument for
building it anyway is different and should be stated separately — threading removes the
range check and the jump-table indirection, shortens the dispatch to a single indirect jump
per opcode, and lets each handler keep the instruction pointer in a register — which is an
*instruction-count* argument, not a prediction argument, and should be justified with the
instruction-count numbers. If someone wants it, make them defend the real reason.

**Trap.** "Published results show 20-30%, so it's a known win." Published interpreter
results are from specific interpreters on specific hardware, usually with a naive switch
baseline compiled by a compiler that generated a bounds check plus a jump table plus a jump
back to the loop head. Modern compilers sometimes duplicate the dispatch tail themselves,
which recovers part of the benefit without any source change — so your baseline may already
be threaded and you would be measuring nothing. Check the disassembly of your actual loop
before citing anyone's number.

### A15 — Refute it without running it

**1.** 180 opcodes × ~90 bytes ≈ **16 KB** of handler code, plus the loop's own prologue and
the runtime helpers the handlers call.

- **x86 with a 32 KiB L1i**: the handler code is roughly half of L1i. It *fits*, but not with
  much room — helper functions, the allocator's fast path, and the GC write barrier all
  compete, and a program that exercises many opcodes plus several helpers can plausibly
  exceed capacity. So on this target the hypothesis is not absurd, and the correct conclusion
  is "not refuted, go measure" — specifically, measure whether L1i misses are served from L2
  (cheap) and what fraction of cycles are frontend-bound.
- **Apple M-series performance core**, with a **192 KiB** L1i (Firestorm and successors)
  rather than 32 KiB — 16 KB of handler code is about a twelfth of capacity, so the entire
  interpreter, its helpers, and a large amount of surrounding runtime fit
  several times over. Capacity misses are essentially impossible for a loop of this size.
  **The hypothesis is refuted on this target without running anything**, and any observed
  frontend cost must have a different cause — conflict misses from unlucky set mapping, TLB,
  or decode-path effects, none of which handler reordering fixes.

The general form: compute the footprint, compare against the capacity, and if it fits with an
order of magnitude to spare, capacity is not your problem.

**2.** Because **a nonzero miss count is not a cost, and L1i misses have a compulsory
component that no layout change can remove.** Every distinct cache line of code must be
fetched at least once, so any program with 16 KB of hot code has at least ~250 compulsory
L1i misses, and a program that periodically leaves the interpreter (a GC cycle, a syscall, a
context switch) reloads them. What you need instead is: (a) **the fraction of cycles that are
frontend-bound**, from the top-down breakdown — if that is 3%, the entire opportunity is 3%
and reordering will capture part of it; and (b) **where the misses are served from** — an L1i
miss that hits in L2 costs ~12 cycles, and 31M of those is a very different story from 31M
misses going to DRAM. A raw miss count with no denominator and no service level supports no
conclusion in either direction. Demanding the denominator is the single highest-value habit
in performance review.

**3.** The shape: **identify the resource the proposal claims is saturated, compute the
demand and the capacity from documented parameters, and compare — then compute the maximum
achievable win as a fraction of total time and compare that against the cost of the change.**
Both halves are required; a proposal can be about a real bottleneck and still be worth
refusing because the ceiling is 1%. What makes it legitimate rather than an excuse: (a) the
numbers are **checkable** — footprint from the disassembly, capacity from the vendor's
documentation, penalties from a published table, all of which the other person can dispute
with their own arithmetic; (b) it is stated as an **upper bound**, so being generous in every
assumption still refutes the claim; and (c) it **names the measurement that would overturn
it**. A refutation that does not say what evidence would change your mind is not a
refutation, it is a preference. Done properly, the envelope calculation is worth more than
the measurement it replaces, because it also tells you the ceiling for every *future* version
of the same idea.

### A16 — The number you remember

**1.** (a) **It has no provenance.** Nobody in the room can say which commit, which build,
which machine, which workload, or which build configuration produced it — so it cannot be
disputed, reproduced, or invalidated, which makes it authority rather than evidence. (b) **It
has almost certainly expired.** Between then and now, the code changed, the allocator
changed, the compiler version changed, and — most likely — somebody already fixed the thing
it describes; a number about a bottleneck that has since been removed will point the whole
team at an empty room. (c) **"Number one cost" is a *ranking*, and rankings are the least
stable thing a measurement produces.** Fixing the number-one cost promotes number two, so a
ranking is invalidated by the very work it motivates. The 19× figure is worse still: a
cross-implementation ratio depends on the benchmark, the other implementation's version, and
the machine, none of which are stated.

**2.** A scoreboard with **one row per change, appended at the time of the change, never
edited**. Fields: the commit or change identifier; the benchmark and its input; the metric
and unit; the before and after values *and the spread* (interval or quantiles, not a bare
mean); the number of independent invocations; the machine and its state (CPU model, core
count, frequency policy, load at the time); the build configuration and compiler version; the
build *kind* (timing build or counting build — Q8); and a one-line statement of what the
change was and what mechanism it was supposed to exploit. The last field is the one people
omit and the most valuable in six months, because it lets you tell whether a later regression
undid the mechanism or merely the number. Rows are never revised; a superseded row gets a new
row that references it.

**3.** Because **the counterfactual is unrecoverable.** The "before" number is a measurement
of a tree that no longer exists — you cannot reconstruct it later without checking out the
old commit, rebuilding it with the compiler of the time, and running it on a machine in the
same state, and the last of those is impossible. Even a perfect rebuild gives you a
*different layout draw* (Q5), a different compiler's inlining decisions, and a different
kernel. So a number reconstructed later is a measurement of a different experiment with the
same name. That is the mechanism, and it is why the discipline is "record at the time" rather
than "record eventually": the information is destroyed by the passage of time, not merely
inconvenient to retrieve. It follows that a change shipped without its row cannot be honestly
defended afterwards, and the correct response to "how much did that buy us?" with no row is
"we don't know", not an estimate.

**Trap.** "We can always re-measure." You can measure the *current* tree, which tells you
where you are, and you cannot measure the delta of a change that landed eleven commits ago
without an expensive and still-inexact bisect-and-rebuild. Re-measuring answers a different
question than the one the row recorded, and treating them as interchangeable is how a project
accumulates a set of optimisations nobody can prove are worth their complexity.

### A17 — Three percent

**1.** The rigorous form needs **the fraction of total time attributable to the region the
change affects**, measured on the workload you care about — call it *p*. Then the ceiling on
any improvement to that region is *p*, and the speedup from making it *s* times faster is
1/((1−p) + p/s). To make the argument you must measure three things, not one: (i) *p* itself,
with attribution you trust — which given Q9 means you probably cannot get it from a flat
profile and need either differential measurement between two builds or runtime-level
instrumentation; (ii) *p* **on the production workload, not the benchmark**, because the
benchmark's *p* is exactly the quantity Q3 shows is unrepresentative; and (iii) the
**variance** of your 3%, because if the interval spans zero you are arguing about the sign,
not the size. "This is not the bottleneck" is rigorous only when you can say "this region is
9% of production time, so the absolute ceiling for any work here is 9%, and this change
captures a third of it".

**2.** Ship it if: (a) **the mechanism generalises** — the 3% here is the same mechanism that
will pay 15% once two other planned changes land, and the representation is the enabling
step. A change that is small alone and load-bearing for a sequence is worth its complexity;
the failure mode is convincing yourself of this without naming the sequence. (b) **The
complexity is negative or neutral** — the second representation actually simplifies four call
sites, or removes a special case, and the 3% is a bonus. (c) The non-performance reason: **it
fixes a correctness or capacity property** — bounded memory, removal of a pathological case,
a latency tail improvement that does not show in the mean. A change that turns a 10× worst
case into a 1.2× worst case while moving the average by 3% is a good change described by a
bad number, and the mean is hiding the actual benefit. Report the quantiles (Q6) and the
argument becomes easy.

**3.** Refuse a 30% change when: it is 30% on a benchmark whose *p* in production is small
(Q3 — verify before believing); it introduces a second representation or a second code path
whose invariants must agree, and you have no way to test the agreement — a correctness risk
that recurs on every future change, forever; it forecloses a design you have already
committed to (a fast path that only works if a class is sealed, in a language that has
promised open classes); it is unmaintainable by anyone but its author; or it wins by
speculating on a property of the current workload that you have no way to detect changing —
the deoptimisation-less speculation, which is fast until the day it silently is not. The
general rule: **performance is a property you can re-acquire and complexity is a property you
cannot easily shed.** A 30% win that costs a permanent invariant is a bad trade; a 30% win
that is local and deletable is nearly always worth taking.

### A18 — Comparing against another implementation

**1.**
- **Different algorithm.** The two "same" benchmarks differ in data structure, memoisation,
  or output — the single most common defect in cross-language comparisons, and the reason
  benchmark suites publish algorithmic constraints.
- **Startup included or excluded.** If your 2.4 s includes 300 ms of runtime initialisation
  and theirs measures steady state after warmup, you are comparing different quantities.
  Which way this cuts depends entirely on which one you chose. **Favours you** if you exclude
  your own startup and they do not.
- **The other implementation has a JIT and yours does not** — or vice versa. A tracing or
  optimising JIT on a numeric loop is not a 2× effect, it is a 10-50× effect, and comparing a
  bytecode interpreter to it is a category comparison, not an implementation comparison.
- **Library work escaping the measurement.** If the benchmark's inner loop calls a native
  library in one implementation and interpreted code in the other, you measured C both times
  in one arm. **Favours you** whenever your primitive happens to be native and theirs is
  library code.
- **Semantic differences that make the work unequal.** Arbitrary-precision integers versus
  machine words; UTF-8 strings with grapheme-correct comparison versus byte comparison;
  bounds and overflow checks present in one and absent in the other. **Favours you** if your
  language's semantics let you skip a check theirs must perform — and this one is easy to
  miss because both programs look identical in source.
- Also: build flags, GC configuration and heap sizing, and — see Q1 — whether either
  implementation eliminated the benchmark.

**2.** The rules: identical algorithm and identical verified output; the same input sizes,
large enough to be above measurement noise; each implementation configured the way its
community would ship it (best available flags, not defaults, and state them); startup either
included in both or excluded in both, stated explicitly; a stated warmup policy that both
arms satisfy; version numbers, machine, and OS recorded; and per-benchmark results published,
never a single geometric-mean number, because the mean hides the fact that you lost 20× on
one benchmark and won on the rest. The formalisation for language runtimes specifically is
**"Are We Fast Yet?" (Marr, Daloze and Mössenböck)**, which defines a benchmark set
restricted to a core language subset that every implementation supports identically —
precisely so that the comparison is of implementations rather than of which language has the
better built-in hash map.

**3.** The useful next step is **choosing which of the two things a 2.7× means**, and the
framing "find where the 2.7× is" presupposes the wrong one. It presupposes a *location* — a
hot spot you can find and fix — whereas a broad factor across every benchmark is almost never
localised. It is usually an **architectural** difference: they have a JIT and you have an
interpreter; their calls are direct and yours go through a dictionary lookup; their integers
are unboxed everywhere and yours box at every boundary. Those are not hot spots; they are
properties of the design that show up as a uniform multiplier, and no profile will point at
them because they are evenly distributed (this is Q13's invisibility, at a different scale).
So the productive question is: **is this factor uniform across benchmarks or concentrated in
a few?** If it is concentrated, you have a real hot spot and the location framing is right.
If it is uniform at 2.5-3× everywhere, you are looking at an architectural gap, and the
decision to make is whether to close it — which is a roadmap decision about inline caches, or
unboxed representations, or a compiler tier — not a bug to find. Reporting "uniform 2.7×,
consistent with the absence of X" is a far more actionable finding than any profile of any
single benchmark, and it is the one that a hot-spot search will never produce.

**Trap.** "We're 2.7× off, so there must be a 2.7× bug somewhere." Broad, uniform factors
decompose into many small ones — 15% from boxing, 20% from dispatch, 10% from bounds checks,
compounding — and the search for the single cause consumes months and finds nothing. The tell
is the *shape of the gap across benchmarks*: a spike on one benchmark is a bug, a flat ratio
across all of them is a design, and knowing which one you have before you start looking is
worth more than any tool.
