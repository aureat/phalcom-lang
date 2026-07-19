# Authoring a `docs/learn` file — the LEAN procedure (experimental)

**Status: experimental, under test.** This is a stripped variant of
[`AUTHORING.md`](AUTHORING.md), written after C1 to test a hypothesis:

> **The cost of a doc is brief size, not phase count. The quality comes from recon, ground truth,
> and the five synthesis passes — not from the number of artifacts produced along the way.**

C1 ran the full five-phase procedure in ~25 minutes. This variant targets the same doc in ~10.
Run it on **C2**, then compare against C1 using [§8](#8-how-to-judge-the-experiment). If the doc is
worse, this file is deleted and the full procedure stands.

**Not duplicated here, deliberately:** [`AUTHORING.md` §9](AUTHORING.md#9-reference-the-standing-method-self-contained)
(the standing method: doc kinds, comparison filter, grip gate, spiral rule, reader) and
[§10](AUTHORING.md#10-the-errors-this-procedure-exists-to-prevent-concrete) (the error log). Those
are shared and must **not** fork — two copies of an error log drift, and a drifted error log is
worse than none. Read them from `AUTHORING.md`.

---

## The one obligation (unchanged, and non-negotiable)

> **After reading, the reader can re-derive Phalcom's choice from the constraints alone.**

Everything below serves this. If a cut in this file ever threatens it, the cut is wrong, not the
obligation.

---

## What was cut, and why each cut is safe

Stated up front so the experiment is honest about what it is trading.

| Cut | Rationale | Risk if wrong |
|---|---|---|
| **`REQUIREMENTS.md` as a separate artifact** | On C1 it was ~80% a restatement of `recon.md`. Only its *forbidden list* and *open risks* did work not already done. | Those two sections get dropped instead of merged. **Mitigation:** they are now mandatory sections of `recon.md` (§1). |
| **Agent A, for mechanism docs only** | A exists to keep the *design space* honest. A mechanism doc has no design space. On a fork doc A stays. | Applying the mechanism exemption to a doc that is secretly a fork. **Mitigation:** the doc-kind gate in §2, which must be answered *before* deciding. |
| **~⅔ of each agent brief** | C1's Agent B ran 216k tokens / 57 tool calls across 12 questions; both of its load-bearing findings came from two narrow targeted asks. Breadth produced citations, not insight. | A finding that would have surfaced from breadth is missed. **Mitigation:** §8's comparison explicitly looks for this. |

**Not cut, and never cut:** recon, the A/B isolation *when A runs*, the five synthesis passes, the
gate. Those are where the quality is.

---

## The phases

| Phase | Who | Produces | When |
|---|---|---|---|
| 1. Recon + requirements, merged | **you** | `recon.md` | always |
| 2. Doc-kind gate | **you** | one line in `recon.md` | always — decides whether phase 3a runs |
| 3a. Agent A (theory, blind) | sonnet | `draft-concept.md` | **fork and tension docs only** |
| 3b. Agent B (source map) | sonnet | `source-map.md` | always |
| 4. Synthesis | **you** | the shipped doc | always |
| 5. Gate | **you** | pass/fail | always |

So: **3 working phases for a mechanism doc, 4 for a fork or tension doc.** Never more than two
agents at once; usually one.

---

## Phase 1 — Recon, now carrying requirements

Recon is **yours**, not an agent's — this is what sharpens the briefs and arms you for synthesis.
Keep it a scout, not a survey.

### Run, in order

1. `graphify query "<the concept as a question>" --budget 2000` — entry symbols.
2. `graphify explain "<the core type or function>"` — neighbors. Usually settles the architecture
   question by itself.
3. **Read exactly one thing: the core type definition.** One file region, not a sweep. This is the
   read that prevents the representation error.
4. `graphify affected "<core symbol>"` — blast radius; which subsystems the doc must mention.
5. Grep `docs/adr/` for the concept. Read **only** *Decision* + *Alternatives considered*.
6. **Run the spec's own example programs.** Cheap, and on C1 this caught that ADR-0030's canonical
   snippet does not compile at HEAD. Do not skip because it looks like transcription.

### `recon.md` — six sections, nothing else

The first four are unchanged from the full procedure. **5 and 6 are what absorbed
`REQUIREMENTS.md`; they are the reason no second artifact is needed.**

1. **Architecture vs representation.** The *shape* (states, structures, algorithm family) versus
   what the live state *holds* (address? name? handle?). Different axes; the consequences live in
   representation. Cite the type definition line.
2. **The grip, grounded.** One sentence, derived from #1, not assumed. If you cannot state it,
   recon is not done.
3. **What was actually deliberated** — from the ADR. Everything else in a design-space walk is
   *your pedagogical reconstruction* and the doc must say so. **If the ADR genuinely deliberated,
   say that too** — C1's did, and copy-pasting the "this is a reconstruction" caveat there would
   have been a false statement.
4. **Findings that change the doc.** Anything recon settled that the plan or your priors got
   wrong. Number them; they are cited throughout the rest of the run.
5. **Forbidden list.** *(absorbed from REQUIREMENTS)* Which shipped docs already own which
   material, and which *future* docs own what this one must only name in passing. Be specific —
   "C2 owns the four-field swap as mechanism; this doc uses it as a one-line fact." Highest-value
   section in the file when tracks overlap.
6. **Open risks.** *(absorbed from REQUIREMENTS)* Every assumption the doc rests on that recon did
   **not** settle, and what happens to the doc if it is wrong. Each one becomes either a B
   question or an in-prose "unverified" label. This section is load-bearing — on the upvalue doc
   it caught a contaminated grip.

---

## Phase 2 — The doc-kind gate (one line, decides phase 3a)

Answer explicitly in `recon.md`, from [`AUTHORING.md` §9](AUTHORING.md#9-reference-the-standing-method-self-contained)'s
four kinds:

- **Fork** — a live decision with real occupants on each branch. **→ Agent A runs.**
- **Tension** — two committed features colliding. **→ Agent A runs** (a tension has competing
  framings, and you will bias toward the one the code implies).
- **Mechanism** — no fork, just machinery. **→ skip Agent A.**
- **Knot** — genuine circularity. **→ skip Agent A**, but see the warning below.

> **Answer this from recon's findings, not from the track plan.** C1's plan called it a fork and
> was right, but the *same* plan was wrong about why the restriction exists. A plan's doc-kind
> guess is a hypothesis.
>
> **When in doubt, run A.** The exemption exists to save ~9 minutes, and a doc that needed an
> honest design space and did not get one cannot be repaired by editing — it has to be rewritten.
> Nine minutes is not worth that risk. "I am not sure whether this is a mechanism" means run A.

---

## Phase 3a — Agent A (fork/tension only)

Brief template: [`AUTHORING.md`](AUTHORING.md) **§7**,
**including its redaction note** — never point A at a requirements file, and here that means never
point A at `recon.md`, which now contains the grip, the findings, *and* the forbidden list. A is
blind or A is worthless.

Two lean changes to the template's Cover list:

- **Cut items 3 and 7** (the distinguishing program; vocabulary import) **as separate numbered
  asks.** On C1 both arrived anyway, embedded in the branch walk where they belong. Asking
  separately produced duplication you then had to cut.
- **Keep items 2, 4, 6, 8** (branches, mechanism, cast, tensions) — that is where every line of
  C1's draft that survived into the doc came from.

---

## Phase 3b — Agent B (always)

Brief template: [`AUTHORING.md`](AUTHORING.md) **§8**.
The headline architecture-vs-representation question is **non-negotiable and comes first**.

**Hard budget: ≤6 questions after the headline.** C1 asked 12 and spent 216k tokens; both findings
that mattered came from asks of this shape:

- *"Confirm the <X> claim **mechanically** — disassembly if reachable; if not, mark INFERRED and
  say exactly what you could and could not observe."* → produced C1's central reframe.
- *"I have a claim I want you to try to **REFUTE**: <claim>. Find every writer and try to construct
  a path that breaks it. Report VERIFIED-TRUE or REFUTED-with-path. **Do not agree with me by
  default** — I will act on this."* → produced the adversarial verification.

**Always include** (these are cheap and load-bearing):
- the core type definitions, quoted;
- the hot-path site — does it branch or not, shown;
- **run the programs**, report verbatim observed output including error text.

**Do not ask for** (C1 paid for these and used almost none):
- a full fixture census — name the ≤4 fixtures you actually want, or ask for none;
- every use site as a table, unless the blast radius *is* the subject;
- a broad ADR sweep — one ADR, *Decision* + *Alternatives*, bounded.

State the budget in the brief itself: *"Six questions. If you find yourself opening a seventh area,
stop and report it as an unexplored lead instead."*

---

## Phase 4 — Synthesis (unchanged; this is where the quality is)

Do not paste agents together. Rebuild from your own judgment. All five passes run, every time.
**These were not cut and must not be** — each prevents a specific error that actually happened.

### 4.1 Reconciliation

- **If A ran:** table of *A's claim → Phalcom's reality (from B, with the line)*. Every differing
  row is something the doc must **teach**, not average. An empty table means you did not
  reconcile.
- **If A was skipped:** the table becomes *recon's assumption → B's ground truth*. **Run it
  anyway.** On C1 the single best finding — that the collection combinator is written in Phalcom,
  not Rust — was exactly this kind of row, and it contradicted recon, the track plan, *and* Agent
  A simultaneously. Without A, this pass is your only contamination check. Treat it as such.

### 4.2 Honesty pass

For every "designed that way for reason Y" claim: did the design *reason* its way there, or does
the outcome merely *land* somewhere nice? Mechanical rule: distinguish **"the code does X"** (cite
the line) from **"the code does X in order to achieve Y"** (cite the ADR/comment that says so, or
downgrade to "the effect is Y"). Bug fixes and absences get labelled as bug fixes and absences.

### 4.3 Claims ledger

Every forward-looking, comparative, or performance claim: **cite a line, label it
unverified/unmeasured in the prose, or cut it.** Perf claims quote a number or say "unmeasured" —
silence reads as "someone checked," and `perf-log/SCOREBOARD.md` is the only source of numbers.
Every markdown link resolves to a file or a real anchor. **Check links with absolute paths** — a
relative check from the wrong working directory reports false failures.

### 4.4 The re-derive moment

Pose the puzzle **before** the reveal and let the reader predict. At least one per doc. This is the
piece most easily dropped under time pressure, which is why it is a named pass and a gate item.

> **Lean addition, from C1.** The best predict-then-check has a **productive wrong answer** — one
> most readers will reach, which is wrong for an interesting reason. C1's: *"`each` can't yield
> because it's a built-in written in Rust."* Wrong — it is written in Phalcom — and the correction
> *is* the doc's thesis. When A runs blind, watch its confident errors: A is a good model of the
> reader, and A's wrong assumption is a gift, not a defect.

### 4.5 Trace the counterintuitive case, then cut to weight

Trace the case where the reader's model **breaks**, not the textbook one. Weight sections by what
they teach about Phalcom's choice, not evenly — the design-space walk is the usual site of bloat,
expect to cut ~30%.

---

## Phase 5 — The gate

Run [`AUTHORING.md` §6](AUTHORING.md#6-the-gate-checklist) unchanged. A "no" is a blocker.

Two extra boxes for this procedure specifically:

- [ ] **The A-skip was justified.** If Agent A was skipped, the doc-kind call in phase 2 still
      holds *after* writing. If the doc grew a real design space during synthesis, the call was
      wrong — say so in §8's log; that is a finding about the procedure, not a failure to hide.
- [ ] **The forbidden list held.** Nothing in the doc spends material `recon.md` §5 assigned to
      another doc.

---

## 8. How to judge the experiment

The point of a lean variant is to be *measurably* not-worse. Record these for the doc produced,
and compare against C1 ([`concurrency/restricted-loop.md`](concurrency/restricted-loop.md), full
procedure).

**Cost** — wall-clock; agent count; per-agent tokens and tool calls.

**Quality — the things that would show a regression first:**

| Signal | C1 baseline (full procedure) |
|---|---|
| Findings that contradicted the plan or priors | 2 (the combinator's implementation; the Doc-1 loop claim) |
| Adversarial checks that changed a claim | 1 (`floor_depth` verified, not assumed) |
| ADR-vs-HEAD gaps found and stated | 3 |
| Claims labelled unverified rather than smuggled | 3 |
| Predict-then-check moments | 1, with a productive wrong answer |
| Gate items failed on first pass | 0 |

**The specific regression to watch for:** a doc that is *accurate but flat* — correct, well
anchored, and with no moment where it corrects a belief the reader (or the plan, or you) actually
held. That is what the cut phases were buying, if they were buying anything. Accuracy is the easy
half; the contradiction-finding is the half that costs.

**If the lean run produces zero rows in the 4.1 reconciliation table**, that is the strongest
possible signal the cuts went too far — it means nothing checked your assumptions. Escalate back to
the full procedure for the next doc regardless of how the prose reads.

Log the outcome here. If C2 comes out as good, this file replaces `AUTHORING.md`'s phase structure
and §10 gains an error 8 about ceremony. If it comes out worse, this file is deleted and the
result is recorded in `AUTHORING.md` §10 so nobody retries it blind.

### Run log

| Doc | Kind | A ran? | Wall | Agent tokens | Recon rows | Gate fails | Verdict |
|---|---|---|---|---|---|---|---|
| [C2 — the parked fiber](concurrency/parked-fiber.md) | mechanism | no (see below) | ~50 min | 170k / 44 calls (one agent) | 4 | 2, both fixed | **keep the variant; fix the gate wording** |
| [C3 — when a fiber fails](concurrency/fiber-failure.md) | tension | **yes** | ~25 min | 243k / 91 calls (A 75k/12, B 168k/79) | 6 | 0 | **keep; A earned its place** |

**Cost, against C1.** C1: five phases, two agents, Agent B alone 216k tokens / 57 tool calls,
~25 min. C2: three phases, one agent, B 170k / 44 calls — **~21% fewer agent tokens.** But wall-clock
was *worse*, ~50 min, and none of that is the procedure's fault: the first two background agents died
without writing output and B had to be relaunched synchronously. Discount the wall number; the token
number is the real comparison and it favours lean.

**Quality, against §8's table.**

| Signal | C1 baseline | C2 (lean) |
|---|---|---|
| Findings that contradicted the plan or priors | 2 | **4** — (i) C1 already owned most of C2's *planned* content, which the track plan did not know; (ii) the fourth swapped field is unexercisable at HEAD; (iii) "swap" is wrong in ADR-0030 §3, Doc 3, and C1 alike; (iv) recon's own retention finding was cut down by the source map |
| Adversarial checks that changed a claim | 1 | **2** — both REFUTE-asks landed: the `checking` retention came back PARTIAL (traced *edge*, not root; unreachable in any program), and the GC-reachability claim came back REFINED (resumer chain is one path of several) |
| ADR-vs-HEAD gaps found and stated | 3 | **2 new** — §3's "pointer swap" is a `mem::take` move; §7's "fibers are GC roots" is satisfied by transitive reachability, with no fiber registry. (§5's typed-signal gap is C1's, not re-counted) |
| Claims labelled unverified rather than smuggled | 3 | **3** — the general GC invariant (INFERRED beyond the one tested shape); the regression golden's non-observability, from its own header; "no other writer of `checking`" as inferred-not-proven |
| Predict-then-check moments | 1, productive wrong answer | **1, productive wrong answer** — two `f.call(x)` in a row; the natural prediction is that both deliver the same way, and the correction (parameter vs. `yield`'s return value) *is* the two-path asymmetry |
| Gate items failed on first pass | 0 | **2** — comparison filter (no cut list named; zero languages is defensible but must be *stated*), and one claim stated more strongly than its evidence. Both fixed before shipping |

**Verdict: the cuts held.** §8's stated kill-switch — *"zero rows in the 4.1 reconciliation table"* —
did not fire; the table had four rows and two came from the adversarial asks, which are the cheap
part. The specific regression to watch for (*accurate but flat*) did not appear either: the doc's
thesis is a correction to three shipped artifacts, which is the opposite of flat.

**Three procedure findings, recorded rather than smoothed over.**

1. **The doc-kind gate was answered twice, and that is a wording bug in this file.** §2 says
   "when in doubt, run A"; §5's extra checkbox says an A-skip that proves wrong is a finding, not a
   failure. On this run both were followed at once — recon concluded *mechanism* and A was dispatched
   anyway "to be safe" — which is not a permitted state. It was caught and reverted before any of A's
   output was read (in the event, A died having written nothing, so the discard cost nothing).
   **Fix §2:** the escape hatch should read *"if you cannot name the doc's kind, you have not finished
   recon — go back to phase 1"*, not *"run A."* Hedging between the two answers costs a whole agent
   and, worse, makes the run useless as a measurement of the variant.
2. **Add a phase-1 step: read the sibling docs that claim to hand off to this one.** The single most
   consequential recon finding was that C1 had already spent the four-`mem::take` block, the
   no-rebasing note, *and* the entire execution-model design space — so the plan's content list for
   C2 was ~40% already-shipped. That came from reading C1, which the phase-1 list does not mention.
   `recon.md` §5 (forbidden list) cannot be written honestly without it.
3. **The B budget worked and should not be relaxed.** Six questions, two of them explicit
   REFUTE-asks, produced both claim-changing results; §3's "do not ask for a full fixture census"
   held. What the brief got wrong was operational, not methodological — background agents that die
   silently. **Run B synchronously.**

**Disposition: do not delete this file.** Run the next doc (C3) on it as well. C3 is a *tension*
doc, so it is also the first real test of the part C2 could not exercise — whether Agent A is worth
running when the procedure says it must.

---

### C3 — the four-phase run (tension; A ran)

**Cost.** Four phases, two agents, 243k agent tokens — *more* than C1's 216k, as expected for a doc
that runs A. Wall-clock ~25 min, the best of the three, because both agents were launched in one
message and run **synchronously**, which is C2's finding 3 applied. Neither died.

**The A/B split paid, and here is the evidence rather than the assertion.** A was briefed blind and
redacted (no `recon.md`, no findings, no proper nouns from the tree) and asked, among other things,
to name *the specific bug it would expect a real implementation of its recommended branch to have*.
It answered: skipping upvalue-closing on a failing fiber's last frames because "it's dying anyway",
manifesting only when a closure captured from that frame had already escaped. **That is E002,
predicted from theory alone, before A had seen a line of Phalcom.** A doc-kind gate that had called
this a mechanism and skipped A would have lost the single strongest piece of evidence that the defect
is structural rather than a local slip.

**Quality, against §8's table.**

| Signal | C1 baseline | C2 (lean, no A) | C3 (lean, A ran) |
|---|---|---|---|
| Findings that contradicted the plan or priors | 2 | 4 | **6** — (i) E001, one of the two scars the plan and C2 both handed to this doc, was fixed three commits before writing; (ii) E002's recorded repro no longer compiles; (iii) the three `clear()`s are a **no-op for the failing fiber** — its state dies by drop-on-reassignment elsewhere, which refines C2's own framing; (iv) `unwind_to` has exactly one caller, so the two paths share nothing; (v) the cascade has zero test coverage past its first hop; (vi) A's blind prediction matched the shipped defect |
| Adversarial checks that changed a claim | 1 | 2 | **3** — all three REFUTE-asks resolved: `close_upvalues_from` unreachable from the failure arm VERIFIED-TRUE (with an independent repro in a different mode/API to show it is not API-specific); E001 VERIFIED-FIXED (which *inverted* a planned section); cascade-vs-cleanup unobservable VERIFIED-TRUE |
| ADR-vs-HEAD gaps found and stated | 3 | 2 new | **1 new** — §6's "the unwind … stops at the fiber floor" implies unwinding *to* the floor; the implementation abandons *at* it, and nothing records that as a decision |
| Claims labelled unverified rather than smuggled | 3 | 3 | **3** — Lua's `resume`/`wrap` split (recalled, no implementation checked); the `block_on` unrooted-`error` lead (surfaced, not reproduced, explicitly "not a claim"); E002's fix direction (marked a hypothesis) |
| Predict-then-check moments | 1 | 1 | **1, with the productive wrong answer built in** — two programs one wrapper apart; the natural prediction is that the *contained* one is safer, and it is the one that panics |
| Gate items failed on first pass | 0 | 2 | **0** |

**Two procedure findings.**

1. **C2's finding 1 (the gate wording) was followed and worked.** Recon concluded *tension* from §1
   and F5 — two shipped features colliding at one `Err` arm — not from the plan's guess, and A was
   dispatched once, without hedging. No "run A to be safe" state occurred.
2. **C2's finding 2 (read the siblings first) is now load-bearing twice over.** Reading C1's headings
   and C2's forbidden list *before* writing recon is what kept the comparison cast from repeating
   Go-on-stacks and Wren-as-port, and it is what turned "the two confirmed scars" into a check rather
   than a copy — which is how the E001 inversion was found at all.

**Verdict across three runs: the lean variant holds, including the branch where it costs more.** The
mechanism exemption saved an agent on C2 without flattening the doc; the tension gate spent one on C3
and got the doc's structural claim underwritten by an independent blind prediction.

**The promotion condition stated at the top of §8 is now met, and promoting it is not done here** —
it is one decision with two edits attached, and it should be taken deliberately rather than as a side
effect of shipping a doc:

1. flip this file's Status header from *experimental, under test* and make it the phase structure
   `AUTHORING.md` points at;
2. add `AUTHORING.md` §10's **error 8, about ceremony** — the concrete form is *"`REQUIREMENTS.md`
   was ~80% a restatement of `recon.md`, and a second artifact whose only load-bearing content is a
   forbidden list and an open-risks table is ceremony; fold both into recon."*

Until someone does both, this file stays experimental and `AUTHORING.md` stays authoritative.
