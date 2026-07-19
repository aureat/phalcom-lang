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
| *(C2 — fill in)* | | | | | | | |
