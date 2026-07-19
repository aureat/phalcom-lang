# Authoring a `docs/learn` file — the procedure

This is the executable procedure for producing one learner's-course document about Phalcom.
Follow it in order. It is written so that a lower-effort model can run it and still catch the
errors a higher-effort model would catch by reflex, because the guards are mechanical, not
matters of taste.

The method behind it (why these docs exist, what shape they take) lives in the project's
`docs-learn-method` memory and is restated compactly, self-contained, in
[§9](#9-reference-the-standing-method-self-contained). This file is the *how*. Read §9 first if you have never
written one of these.

---

## The one obligation (everything serves this)

> **After reading, the reader can re-derive Phalcom's choice from the constraints alone.**

Delete the source, hand the reader the pressures, and they rebuild the design. A doc that only
*describes* what the code does has failed, no matter how accurate. Two consequences the
procedure enforces later: the doc must contain at least one **predict-then-check** moment (§5.4),
and every branch not taken must be made genuinely tempting before it is rejected (a strawman
teaches the answer without the question).

---

## The five phases and their deliverables

| Phase | Who | Produces | Cost |
|---|---|---|---|
| 1. Recon | **you (orchestrator)**, graphify + bounded reads | `recon.md` | cheap — a scout, not a survey |
| 2. Requirements | you | `REQUIREMENTS.md` | grip now **grounded**, not assumed |
| 3. Two agents | sonnet A + sonnet B, in parallel | `draft-concept.md`, `source-map.md` | the bulk research |
| 4. Synthesis | you | the shipped doc | judgment layer — where value is added |
| 5. Self-review gate | you | pass/fail against §6 | mandatory before "done" |

Working folder: `docs/learn/<concept>/` holds phases 1–4's scratch (`recon.md`,
`REQUIREMENTS.md`, `draft-concept.md`, `source-map.md`). The shipped doc lands in its part
folder (`docs/learn/vm/`, `docs/learn/object-model/`, …), **not** the working folder.

Never run more than two agents at once. Use them sparingly.

---

## Phase 1 — Recon (this is the new step; it exists to kill errors before they enter a brief)

**The single most expensive mistake is writing the grip from an assumption about the
representation, then feeding that assumption into an agent brief.** It happened on the upvalue
doc: the draft's thesis ("the read path never branches") was Lua's design, asserted before
reading Phalcom's type. One 15-second type read would have killed it. Recon is that read, made
mandatory and done *first*.

Recon is **yours**, not an agent's. You grounding yourself is precisely what sharpens both
briefs. Keep it cheap — a scout, not the survey (that is Agent B's job, and it runs deeper).

### Run, in order:

1. `graphify query "<the concept as a question>" --budget 2000` — find the entry symbols.
2. `graphify explain "<the core type or function>"` — get its neighbors. This alone usually
   settles the architecture question.
3. **Read exactly one thing: the core type definition** (the enum/struct at the center). Not a
   sweep — one file region. This is the read that prevents the representation error.
4. `graphify affected "<core symbol>"` — cheap reverse-impact, tells you the blast radius and
   which subsystems the doc must mention (GC, fibers, compiler).
5. Grep `docs/adr/` for the concept. If an ADR exists, read **only** its *Decision* and
   *Alternatives considered* sections. This is what tells you what was **actually deliberated**
   versus what you will reconstruct pedagogically (see the honesty guard, §5.2).

### `recon.md` must answer four questions and nothing else:

1. **Architecture vs representation.** What is the *shape* (states, data structures, algorithm
   family — e.g. "Lua-style two-state open/closed")? What is the *representation* (what does it
   hold — an address? a name? a box?)? **These are different axes and the consequences live in
   representation.** State both. Cite the type definition line.
2. **The grip, now grounded.** One sentence that collapses the confusion, derived from #1, not
   assumed. If you cannot state it, recon is not done.
3. **What was actually deliberated.** From the ADR's *Alternatives considered*. Everything else
   in the design-space walk is *your pedagogical reconstruction* and the doc must later say so
   (§5.2). Name the one or two real alternatives.
4. **Brief-steering notes.** Which design-space branches deserve depth vs a sentence (steers
   Agent A's *emphasis*). Which exact symbols Agent B must confirm (steers B's *verification*).

Recon does **not** decide the doc's prose, trace anything, or run programs. It arms the briefs
and arms you for synthesis.

---

## Phase 2 — Requirements

Write `REQUIREMENTS.md` into the working folder using the structure from the upvalue exemplar
([REQUIREMENTS.md](upvalue/REQUIREMENTS.md)): the obligation, the reader, the doc kind, **the
grip (now copied from `recon.md`, grounded)**, the design space as a table, the comparison
filter, the tensions to surface, the structural rules, the checklist, the build sequence, and an
**open-risk** section.

The open-risk section is load-bearing: name every assumption the doc rests on that recon did not
fully settle, and state what happens to the doc if the assumption is wrong. On the upvalue doc
this section fired — it caught the contaminated grip. Keep writing it even after recon, because
recon is bounded and Agent B goes deeper.

---

## Phase 3 — The two agents

Spawn both in parallel, sonnet, `run_in_background: false`. Their briefs are the templates in
§7 and §8, **with recon's findings injected**. The division is deliberate and must be preserved:

- **Agent A (theory) is given no source access and is told nothing about which branch Phalcom
  took.** This is structural, not stylistic. An agent that does not know the answer cannot
  flatter it, so the design space stays honest. Recon steers A's *emphasis* (go deep here, one
  sentence there) but never hands A the conclusion. **A's brief is inlined and redacted — never
  a pointer to `REQUIREMENTS.md`, which names the branch. See §7's note; this is error 7 in §10.**
- **Agent B (source map) leads with the architecture-vs-representation question and must answer
  it first, with the line that settles it.** B runs `.ph` fixtures live rather than reading
  behaviour off code, because observed output is the strongest evidence and it is cheap.

You supply the judgment neither can: A supplies uncontaminated theory and prose bulk, B supplies
ground truth, and you reconcile them in phase 4.

---

## Phase 4 — Synthesis (write the doc; this is where the raw power goes)

Do not paste A and B together. Rebuild the doc from your own judgment over both. Run these five
passes; each one exists to prevent a specific error that actually happened.

### 5.1 Reconciliation table (prevents: propagating the theory agent's wrong-for-Phalcom claims)

Agent A wrote correct *theory* and will be wrong exactly where the theory meets Phalcom's real
representation. Build a table: *A's claim → Phalcom's reality (from B, with the line)*. Every row
where they differ is a place the doc must teach the difference, not average it. On the upvalue
doc this table had eight rows (branchless→branches, linked-list→BTreeMap, per-thread→per-fiber,
etc.). If your table is empty, you did not reconcile — recheck.

### 5.2 The honesty pass (prevents: flattering the codebase)

For every claim that the code was *designed* a certain way *for a reason*, check B and the ADR:
did the design actually reason its way there, or does the outcome merely *land* somewhere nice?

The upvalue doc failed this: it wrote that Phalcom "drew the same line" as C# on principle. The
truth is the `for` behaviour is a documented **bug fix** (U-ITER-FIX) and the `while` behaviour
is an **absence of machinery** that happens to match how a plain `var` reads. A bug-fix plus an
absence is not a principle. The honest form: *the outcome lands on C#'s line, and C#'s rationale
is available as a strong post-hoc justification — but Phalcom did not reason its way there.*

Mechanical rule: **distinguish "the code does X" (cite the line) from "the code does X in order
to achieve Y" (cite the ADR/comment that says so, or downgrade the claim to "the effect is Y").**

### 5.3 The claims ledger (prevents: smuggled unverified claims)

List every forward-looking, comparative, or performance claim in the doc. For each, one of:

- **cite a source line** (from B), or
- **label it unverified/unmeasured** in the prose, or
- **cut it.**

The upvalue doc smuggled "compaction is a live future option" and "inline-caching an upvalue
access" — both mine, neither checked. One unverified claim poisons a doc whose entire selling
point is that it is grounded. Perf claims specifically: this repo has `perf-log/SCOREBOARD.md`
and a measurement culture. A qualitative cost claim ("two branches per read", "closures in a hot
loop are slow") must either quote a number or say **"unmeasured"** — silence reads as "someone
checked."

Also mechanical: every markdown link must resolve to a file or a real anchor, not a bare
directory. Check them.

### 5.4 Insert the re-derive moment (prevents: the doc failing its own obligation)

Telling the reader the grip is not the same as making them use it. Before the section that
reveals Phalcom's choice, pose it as a puzzle the grip answers, and let the reader predict:

> *Lua's stack is a growable array. So is Phalcom's. Lua needs a fix-up pass on every
> reallocation; Phalcom needs none. Why?*

A reader who answers owns the idea; a reader who is told it has a sentence. At least one such
moment per doc. This is the piece most easily dropped under time pressure — it was dropped on
the first upvalue pass — so it is a named phase, not a nicety.

### 5.5 Trace the counterintuitive thing, then cut to weight

- **Trace the hard case, not the familiar one.** If the doc has a stateful mechanism, trace the
  moment where a reader's mental model *breaks*, not the textbook case their intuition already
  handles. The upvalue doc traced a counter-factory closing at return (easy, Lua-classic) and
  left the genuinely strange case — the for-loop's mid-flight per-iteration close over one reused
  slot — as prose. Trace the strange one.
- **Uneven weight.** Branches, comparisons, and sections get space proportional to what they
  teach *about Phalcom's choice*, not equal space. If flat closures get Java's word count and
  haven't earned it, cut. The design-space walk is the usual site of bloat; expect to cut ~30%
  there.
- **Say the space is a reconstruction.** If §5.2 found the ADR only deliberated one alternative,
  the doc must state that the fuller design-space walk is pedagogical scaffolding, not the
  decision as it happened.

---

## Phase 5 — The self-review gate (§6). Do not declare done until it passes.

---

## 6. The gate checklist

Run every item. Each maps to a real failure. A "no" is a blocker, not a nit.

- [ ] **Re-derive test.** Could a reader rebuild the design from the constraints? Is there ≥1
      predict-then-check moment where they must? *(§5.4)*
- [ ] **Grip grounded.** The thesis came from a read type, not an assumed one. It is stated
      early and *earned* by the end. *(Phase 1)*
- [ ] **Reconciliation done.** Every place theory diverges from Phalcom's representation is
      taught, not averaged. *(§5.1)*
- [ ] **Honesty.** No "designed for reason Y" claim that is really "happens to land at Y." Bug
      fixes and absences are labelled as such. *(§5.2)*
- [ ] **Claims ledger clean.** Every forward-looking/perf/comparative claim is cited, labelled
      unverified, or cut. Perf claims quote a number or say "unmeasured." All links resolve.
      *(§5.3)*
- [ ] **Hard trace.** The traced case is the one that breaks intuition, from real
      output/structure. *(§5.5)*
- [ ] **Weighted, not surveyed.** Sections are proportional to teaching value; the design space
      is not a prose re-table with equal cells; the ~30% of bloat is cut. *(§5.5)*
- [ ] **Comparison filter.** Every language present passes one of the four tests (§9); the cut
      list is named. *(§9)*
- [ ] **Anchors symbol-first.** `file.rs::Type::method` (~Lxxx). A dead symbol should fail
      loudly; bare line numbers rot silently.
- [ ] **Lies marked.** Every simplification is flagged as a lie with a forward pointer to where
      it is destroyed. *(spiral rule, §9)*
- [ ] **Vocabulary imported.** The terms of art the reader lacks are introduced where they do
      work and are visually findable. *(§9, filter test 3)*
- [ ] **Diagram earns its place.** A diagram draws the thing whose *shape is the point*. Do not
      draw pointer arrows for a design whose thesis is that it has no pointers.

If any box is unchecked, fix it before shipping. If a fix needs a fact you do not have, get it
from source — do not guess. Guessing is the error the whole procedure exists to prevent.

---

## 7. Prompt template — Agent A (theory, no source access)

Fill the `<<slots>>`. Inject recon's emphasis notes at `<<EMPHASIS>>`. **Do not tell A which
branch Phalcom took.**

> **Never point A at `REQUIREMENTS.md`.** This template used to say "first read
> `REQUIREMENTS.md`," which contradicted the constraint directly above it. Once recon has done
> its job, `REQUIREMENTS.md` *is* the answer: it states the grip, and its design-space table
> names occupants (*"Lua 5.1, **Phalcom**"*). A that reads it can flatter the branch, which is
> the one thing A exists to make impossible.
>
> **Instead, inline a redacted brief** in A's prompt — the slots below are that brief. Copy from
> `REQUIREMENTS.md` everything that steers *weight* (the branches themselves, the comparison
> cast, the tensions, recon's emphasis notes) and strip everything that reveals *the branch*:
> occupant names tying a branch to Phalcom, the grip, the recon findings, the forbidden list.
> Describe branches by their mechanics, never by who took them.
>
> Do not fix this by softening `REQUIREMENTS.md` — it is *supposed* to be fully grounded. The
> redaction is A's brief's job, not the requirements doc's.

**Evidence it works.** On C1 ([`concurrency/restricted-loop.md`](concurrency/restricted-loop.md))
A was kept blind this way and independently re-derived ADR-0030's own GC-based rejection of
stackful coroutines — an argument it had no access to. It also assumed, wrongly, that the
collection combinator was native. That error was *useful*: it is exactly the reader's error, and
it became the doc's predict-then-check moment. **An uncontaminated A is worth more than a
correct A.**

```
Write a deep conceptual/theoretical document on <<CONCEPT>> in programming-language
implementation.

Write it to: docs/learn/<<CONCEPT>>/draft-concept.md

HARD CONSTRAINTS:
- DO NOT read any source code in this repository. No grep, no .rs files, no graphify. This is
  pure theory. Another agent covers implementation. If you catch yourself wanting to check what
  "Phalcom" does — stop, that is out of scope. Write the theory as it applies to ANY runtime.
- You will NOT be told which design branch Phalcom took, deliberately. Do not guess it, do not
  optimize the writing toward any particular answer. Write the design space honestly, as a space.
- Do NOT go looking for a requirements/spec file for this doc. Everything you are meant to have
  is in this prompt. If you find such a file, you are outside your brief — it names the answer.
- NO fixed skeleton (no Overview/Background/Conclusion). Structure follows the theory; it bottoms
  out where the theory bottoms out.
- NO checkbox comparative table of N languages, one line each. That is the anti-pattern.
  Comparison is a weapon aimed at ONE named confusion. Go DEEP on the few that earn it; name the
  ones you CUT and why.

WHAT I NEED FROM YOU — research recall, historical precision, prose bulk. The judgment layer is
mine. Be exact where I would otherwise have to guess.

Cover, at the depth the theory demands:
1. The problem from first principles. Name it (<<THE NAMED PROBLEM, e.g. upward funarg>>). Get
   the history right.
2. Walk EVERY branch of this design space: <<THE BRANCHES, transcribed from REQUIREMENTS §5 and
   REDACTED — describe each by its MECHANICS, never by an occupant list that includes Phalcom.
   "Restricted: suspension's domain is the frames the loop owns; attempting it under a re-entrant
   native frame raises." NOT "Restricted (Lua 5.1, Phalcom).">> Make each GENUINELY TEMPTING
   before you kill it: who took it, what it buys, what it costs, what it forecloses. A strawman
   is a failure.
3. <<THE DISTINGUISHING PROGRAM: the exact small program that separates the two semantics a
   reader might conflate. Make it concrete and language-specific.>>
4. <<THE MECHANISM in full mechanical detail — every sub-part, transcribed from REQUIREMENTS.
   Phrase it as a mechanism ANY runtime could use. Do not write "the branch Phalcom took"; that
   is the reveal.>>
5. <<EMPHASIS: from recon — "go deep on branches X and Y; keep Z to a sentence." This steers
   weight WITHOUT revealing Phalcom's answer.>>
6. The comparative deep dives that earn their place: <<THE CAST, from REQUIREMENTS §6 — names
   only, WITHOUT the "why it is in this doc" column, which usually leaks the branch.>> For each:
   who/what/bill/scar. Include the famous bug the reader has personally hit, if one exists.
   Name the ones you CUT and why — <<CANDIDATE CUTS>> are worth considering and rejecting.
7. Vocabulary import: the terms of art the reader lacks. Introduce each where it does work; make
   them visually findable; do not ghettoize into a glossary.
8. <<THE TENSIONS, from REQUIREMENTS §7, redacted to the ones that pose a general question rather
   than describing Phalcom's situation. Phrase each as a question about runtimes, not a report:
   "when is a sound-but-wider-than-necessary restriction the right call?" NOT "HEAD restricts
   more than the ADR does.">> At the depth each deserves.

QUALITY BAR: assume the reader knows what <<CONCEPT>> IS; do not explain that. Every paragraph
carries information. Where you are uncertain of a historical fact or language detail, SAY SO with
a marked **[flagged]** and a confidence note. A flagged gap is useful; a confident wrong claim is
worse than useless and I will read adversarially for exactly that. As long as the theory
requires, no longer. No conclusion that restates the document.
```

---

## 8. Prompt template — Agent B (source map, graphify-led)

Fill the `<<slots>>`. Inject recon's must-confirm symbols at `<<CONFIRM>>`. The headline question
is non-negotiable and comes first.

```
Map how <<CONCEPT>> works in the Phalcom source at HEAD. Read-only.

Write findings to: docs/learn/<<CONCEPT>>/source-map.md

The graph at graphify-out/ is up to date — use it FIRST, it is cheaper than grep:
  graphify query "<<the concept as a question>>" --budget 2000
  graphify explain "<<core symbol>>"
  graphify affected "<<core symbol>>"
Then read only the regions it points at. Do not sweep the tree.

## THE QUESTION THAT DOMINATES EVERYTHING — answer FIRST, at the top, with the line that settles it

<<THE ARCHITECTURE-VS-REPRESENTATION QUESTION. State the candidate answers explicitly, e.g.:
"Does Phalcom represent X as [address / name / box / heap object / not-implemented]?" List the
real alternatives so B cannot pattern-match to the expected one.>> If the mechanism is partial or
absent, SAY THAT PLAINLY — that is a valid and important answer. Do not pattern-match to
<<EXPECTED FAMILY>> because you expect it.

## THEN answer each — with file:line and the essential quoted lines

<<Enumerate the specific questions from recon. Always include:>>
- The data structures (quote the type definitions — worth the tokens).
- The representation: what does the "live" state hold — an address, an index, a handle? Quote it.
  In Rust, a self-referential version is what the borrow checker hates — how was that avoided?
- Is the hot-path operation branchless or does it branch? Show the actual site.
- <<CONFIRM: the exact symbols recon flagged. "Confirm X exists at Y and quote it.">>
- Every use site (`graphify affected`), concise table.
- <<The behavioural questions that only source settles — the ones the doc will get wrong if
  guessed. For each, if a .ph fixture exists OR you can write a tiny program, RUN it and report
  the actual observed output. Examples live in examples/*.ph and tests/lang/; the CLI is
  `cargo run -p phalcom-core --bin phalcom`.>>
- Spec/ADR — BOUNDED. Grep docs/adr/ and docs/spec/v0.2/ for the concept. Read the ONE relevant
  ADR's Decision + Alternatives considered. Cite and summarize in a few lines. Do NOT sweep.
- Tests/fixtures exercising it — paths, one line each.

## OUTPUT RULES
- Anchor SYMBOL-FIRST, line second: `file.rs::Type::method` @ ~L120. Line numbers rot; symbols
  are checkable.
- Quote only load-bearing lines. Type definitions and the core functions in full; everything else
  cite + summarize. Do not paste whole files.
- DISTINGUISH what you VERIFIED (read the line, or ran the program and matched output) from what
  you INFERRED. Mark inferences. I read adversarially; a confident wrong claim about the source
  poisons a doc readers are meant to trust.
- If something does not exist at HEAD, "does not exist at HEAD" is a valuable answer. Say it
  plainly. Do not invent, soften, or pad.
```

---

## 9. Reference: the standing method (self-contained)

Condensed from [docs-learn-method memory]. Read this if you have not internalized the method.

**Doc kinds** — discovered by asking whether the concept has a fork, a collision, or a cycle:
- **Fork** — a live decision with real occupants on each branch. Structure = the design space;
  make every branch tempting.
- **Mechanism** — no fork, just machinery. Structure = the mechanism. A trace earns its place
  only if the mechanism is *stateful*, and then it traces the counterintuitive case (§5.5).
- **Tension** — two already-committed features colliding. The best kind; explains why one spot
  of code looks weird.
- **Knot** — genuine circularity (e.g. bootstrap). Show the cycle, fail honestly, show the tie.

**Comparison filter** — a language enters only if it: (1) took the other branch, **with the bill
attached**; (2) has a **scar** — shipped bug, perf loss, spec change; (3) **names something
Phalcom does anonymously** ← highest value, because names are the mental tools the reader lacks;
or (4) is an **ancestor** explaining otherwise-arbitrary shape. Expect ~6 to survive. Name the
cut list and why each failed.

**The grip gate** — every doc needs a nameable thesis before it is written: one reframe that
collapses the confusion. If you cannot state it in a sentence, you do not understand the concept
well enough yet, and the doc will come out as description. (Phase 1 produces it, grounded.)

**Spiral, and mark the lies** — the object model is circular (a Class is an object; an object has
a class), so the course cannot be linearized. Early docs tell simplifications. Every
simplification must be **marked as a lie with a forward pointer** to where it is destroyed. An
unmarked lie destroys trust the moment a later doc contradicts it.

**Truth basis** — HEAD as implemented; spec intent only where v0.2 is unfinished, and say which.
Anchors symbol-first so drift fails loudly, the way stale line numbers do not.

**Reader** — knows PL design, not fluent in implementation. Specific weakness: cannot hold
moving-state mechanisms in their head; lacks stable notation, so complexity accretes until the
thread is lost. The deliverable is a **grip**, not completeness.

---

## 10. The errors this procedure exists to prevent (concrete)

Keep these visible; abstract rules do not bite, instances do. 1–6 are from the upvalue doc;
7 is from C1 and is an error in **this file**, not in a doc.

1. **Grip from an assumption.** Wrote "read path never branches" (Lua's design) before reading
   Phalcom's type. Contaminated Agent A's brief. → **Recon, phase 1.**
2. **Flattered the codebase.** Called a bug-fix-plus-an-absence "principled convergence with
   C#." → **Honesty pass, §5.2.**
3. **Smuggled claims.** "Compaction is a live option," "inline-caching an upvalue" — neither
   checked. Bare directory link. → **Claims ledger, §5.3.**
4. **Told the grip, never made the reader use it.** No predict-then-check. → **§5.4.**
5. **Traced the easy case.** Counter-factory instead of the for-loop per-iteration close. →
   **§5.5.**
6. **Rebuilt the survey I rejected.** Six equal-weight design branches in prose. → **weight,
   §5.5.**
7. **The procedure contradicted itself.** §7's template promised A would not be told Phalcom's
   branch, then told A to read `REQUIREMENTS.md` — which, after recon, states the grip and names
   Phalcom in its design-space table. Following §7 literally would destroy the isolation §7
   exists to create. → **§7's redaction note.** Caught only because C1's orchestrator noticed the
   contradiction while filling the template and deviated deliberately.

Every one was caught only on a second, adversarial read. The gate (§6) is that second read, made
mandatory and up front. **Error 7 is the reminder that the gate applies to this file too** — a
procedure is not exempt from the adversarial read it demands of everything else.
