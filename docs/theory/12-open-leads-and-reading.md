# 12 — Open leads and the reading map

> Two things live here: a **bibliography audit** — what this repository actually cites, verified by
> exhaustive search, versus what it is widely believed to cite — and a ranked list of **leads worth
> pulling**, each stated concretely enough to act on.

---

## 1. The bibliography audit

Run 2026-07-20 across `docs/` for author names, venue names, year-in-parens patterns, "et al",
"proceedings", "thesis", and "dissertation".

### `[V]` What the decision records cite: nothing

**There is not one academic citation anywhere in the 78 ADR and PDR files.** No author-year
reference, no paper title, no journal or conference name. The only formal external documents cited
by identifier are four Python Enhancement Proposals — standards documents, not papers — plus
Nygard for the decision-record format itself and "C3 linearization" as a bare algorithm name.

This is worth sitting with rather than treating as a deficiency. The records are dense with
*implementation* precedent — Smalltalk in nineteen of them, Wren in fourteen, Lua, CPython, Ruby,
Java, Swift, Rust, Dart, Go, Erlang, Eiffel, PHP — and the house rule governing those references is
strict: **"Cite precedent with consequence. Not 'Ruby does X' but 'Ruby does X, which forces Y.'"**

So the corpus is not under-referenced. It is referenced against *shipped systems* rather than
*papers*, which is a defensible choice for a decision record and an odd one for the theory layer
that sits above it. Which is part of why this directory exists.

### `[V]` What the teaching documents cite: a real, careful bibliography

Verified literal citations in `docs/learn/`:

| Citation | Where | Note |
|---|---|---|
| L. Peter Deutsch and Allan Schiffman, *Efficient Implementation of the Smalltalk-80 System*, POPL 1984 | `vm/caches-and-fusion.md:40`; also `vm/upvalues.md:160` | full author + title + venue + year |
| Hölzle, Chambers, Ungar — SELF polymorphic inline caches, ECOOP 1991 | `vm/caches-and-fusion.md:110` | authors + venue + year; title omitted |
| Ierusalimschy, de Figueiredo, Celes, *The Implementation of Lua 5.0* | `vm/execution-loop.md:227` | with link |
| Joseph Weizenbaum — origin of the funarg problem, late 1960s | `vm/upvalues.md:41`; `upvalue/REQUIREMENTS.md:89` (as "Weizenbaum, 1968") | |
| Joel Moses, *The Function of `FUNCTION` in LISP, or Why the FUNARG Problem Should Be Called the Environment Problem*, MIT AI Lab memo, 1970 | `vm/upvalues.md:42-43` | full title + venue + year |
| David Abrahams — basic/strong/nothrow exception-safety taxonomy | `vm/frame-identity.md:425` | explicitly downgraded to *attribution*, venue uncertain |
| Bob Nystrom, *"What Color is Your Function?"* | `concurrency/restricted-loop.md:455` | essay, not a paper; noted as also the author of Wren |
| Goldberg and Robson, *Smalltalk-80: The Language and its Implementation*, 1983 | draft documents only | did **not** survive into a shipped document |

**`[V]`** The densest single page is outside `docs/learn/` — a sealed-classes draft chains
Chambers & Ungar (POPL '89) → Hölzle, Chambers, Ungar (ECOOP '91, LNCS 512) → dynamic deoptimization
(PLDI '92, pp. 32–43). The interview-drill directory adds Dijkstra, Yuasa, Deutsch–Bobrow,
Hindley-Milner/Damas-Milner, Wadler, Appel, and Hoare, but is explicitly **not a Phalcom document**.

### `[V]` What is absent

**Zero literal hits** for: Conway, Reynolds, Knuth, Aho, Ingalls, Landin, and (outside two draft
files) Sussman and Steele.

The Conway absence is the one that matters, and it is the reason
[`00-provenance-and-citation-discipline.md`](00-provenance-and-citation-discipline.md) exists. The
origin of the concept the entire concurrency subsystem descends from is **absent from the
bibliography while being confidently cited in the memory database as "located and verified."** That
inversion is the incident in one sentence.

**`[O]` Concrete gap to close:** the canonical reading list at
`.claude/skills/language-design/references/reading.md` has a Concurrency section listing Armstrong,
Hoare, and structured-concurrency sources, and a Closures section listing Ierusalimschy on Lua
coroutines — but **no Conway entry**. Adding it is a one-line fix; reading the paper first, so the
entry is `[V]` rather than `[R]`, is the actual task.

---

## 2. Leads worth pulling, ranked

Ranked by *decision-changing potential per unit of effort*, which is not the same as importance.

### Tier 1 — cheap, and the answer changes a decision

**`[O]` L1. Does anything transcode across the native/`.ph` boundary?**
The boundary tax argument in [`06`](06-mechanism-versus-policy.md) §3 is entirely hypothetical for
Phalcom because nobody has checked. If a Rust primitive and a guest method exchange the same value
enum, the crossing is nearly free and there is no tax to find. If anything transcodes — strings are
the obvious candidate — there is a cost hiding in the split that no dispatch optimization touches.
Bounded, cheap, and it gates any decision to move a hot operation across that boundary in either
direction.

**`[O]` L2. Re-profile at current HEAD.**
The attribution table ranking the remaining optimization tiers describes a binary two large cuts in
the past, and shares are shares of a denominator that has moved. Cheap next to any unit it would
justify, and it would give the tiers a clean baseline for the first time.

**`[O]` L3. Resolve the once-observed fiber-spawn regression.**
Recorded as an open question, deliberately unexplained. Build binaries either side of the suspected
commit and run them interleaved. "The collector costs fiber-heavy code ~37%" would be a material
fact about the collector design; "it was noise" is equally worth knowing, and the current state —
a plausible unexamined story — is the worst of the three.

### Tier 2 — substantial, and the design space is genuinely open

**`[O]` L4. Structured concurrency and cancellation.**
Single-fiber `abort` exists; cascading cancellation of children does not. **`[R]`** The nursery
literature's claim is that spawn without a join point is the concurrency `goto`. Phalcom currently
has the `goto`. Note the interaction: adding cancellation touches the parked-state representation
that [`01`](01-coroutines-and-the-suspension-problem.md) is about, so it is not a library-level
addition.

**`[O]` L5. `select` / `race`.**
Not mentioned in any record. Hard to retrofit for a specific reason worth naming in advance: it
requires a coroutine to be blocked on *several* wake conditions at once, which the current parked
representation does not express.

**`[O]` L6. Scheduler fairness.**
A ready queue exists as mechanism; no fairness policy is specified. Consistent with the
mechanism-over-policy stance — but an unspecified policy is still a policy, and it is currently
"whatever the queue does."

**`[O]` L7. Does sealing let a call site skip its guard entirely?**
If a class provably cannot be reshaped or have methods redefined, a site whose receiver class is
sealed needs no epoch check at all — making sealing an inline-cache *accelerant* rather than merely
a deleted invalidation case. Requires reading what the existing sealed-class machinery actually
guarantees, and is gated behind the cache preconditions. Do not design on it first: an inline cache
without an invalidation story is unsound, and sealing is a shortcut *after* the epoch mechanism
exists, not instead of it.

**`[O]` L8. Precompiled bootstrap image.**
The core library is lexed, parsed, and compiled on **every** startup — every CLI invocation, every
REPL start, every golden test. Filed with two constraints already attached: measure the split first,
and it may **never** be re-sold as a throughput lever, because the steady-state benchmark harness is
blind to it by construction.

### Tier 3 — known-latent, will bite on a schedule

**`[O]` L9. The hash-equality contract breaks when the numeric split lands.**
A large integral float will compare equal to an integer under the new promotion rules while hashing
differently. `a == b ⇒ a.hash == b.hash` fails, and the hash-based collections depend on it.
Harmless today; broken the day the split lands. Already identified — the lead is to make sure it is
*scheduled*, not merely known.

**`[O]` L10. Generation wraparound.**
A bare 64-bit counter with wrapping increment, no guard, no tombstone, no test, nothing in any
record. Almost certainly unreachable in practice; that judgment has never actually been made by
anyone, and the documentation deliberately declines to manufacture it.

**`[O]` L11. The traceback path is built and unwired.**
Rendering, runtime-error, and source-location machinery all exist and are correct; the CLI run path
bypasses them, so every traceback is empty by construction. Note the search hazard recorded
alongside: grepping for "backtrace" misses it entirely and produces a wrong premise about what
exists.

**`[O]` L12. Reification without re-dispatch.**
`doesNotUnderstand` receives a first-class `Message`, but `perform` accepts only a symbol — so a
handler can observe an intercepted call and cannot forward it, which blocks the proxy pattern
outright. The general rule: ship `perform(Message)` in the same change as `doesNotUnderstand`.

---

## 3. Reading the corpus itself

For someone arriving cold, ordered by density of transferable content rather than by dependency:

1. **`docs/learn/vm/upvalues.md`** — the densest comparative document in the repository, and the
   one carrying the real bibliography. Java, Algol/Pascal, Scheme, Smalltalk, ML, C++, JavaScript,
   Go, C#, Swift, Python, Rust, each with a stated bill.
2. **`docs/design-notes/js-redesign-transferable-notes.md`** — the mechanism-versus-policy thesis,
   the boundary tax, and an honest section of counterevidence against its own argument.
3. **`docs/learn/concurrency/`** (four documents, read in order) — the clearest worked example in
   the corpus of four individually correct decisions composing into a broken feature.
4. **`docs/design-notes/optimization-method-and-harness-fidelity.md`** — five findings on why the
   measurement apparatus is itself a subject requiring verification.
5. **`docs/adr/accepted/0030-…md`** — the concurrency decision, and the best single example of a
   record that argues its alternatives properly rather than narrating a foregone conclusion.
6. **`.claude/skills/language-design/phalcom/overlay.md`** — roughly forty-five committed positions
   in table form, plus the original hazard catalogue. **Known to drift**: it currently lists at
   least one retired record as accepted and one accepted record as proposed. Read it as a map, not
   as a source of truth.
7. **`docs/learn/proving-ground/`** — general design-space drilling, explicitly not about Phalcom,
   with a three-grade rubric worth stealing: *Recalled* (worth little) / *Derived* / *Traded*
   ("named what the design forecloses, and what you would pick instead under a different
   constraint — the only grade that reads as senior"), and a mandatory **Trap** per answer naming
   the plausible, confident, wrong thing a strong candidate says.

That rubric is, not coincidentally, the same three-way distinction this directory's provenance tags
enforce. Recalled is not derived; derived is not traded; and knowing which one you are doing is most
of the skill.
