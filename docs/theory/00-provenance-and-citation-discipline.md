# 00 — Provenance and citation discipline

> **Status:** governing. Every other file in `docs/theory/` inherits the rules stated here.
> **Origin:** the Conway-1963 incident, 2026-07-19, reconstructed and verified 2026-07-20.

---

## 1. The incident

On 2026-07-19 at 18:31, while reconnaissance was under way for the C2 concurrency document
(*The Parked Fiber*), an automated session summarizer emitted observation **#7611**:

> **ADR-0030 Decision section fully read; Conway 1963 foundational paper verified**
> Seven decision points confirmed in ADR-0030; Conway's seminal 1963 CACM paper on coroutines
> located and verified as source.

Its fact list led with a full bibliographic entry:

> Conway, M.E. (1963) "Design of a Separable Transition-Diagram Compiler" *Communications of the
> ACM* vol. 6 no. 7 pp. 396–408 — seminal paper introducing coroutines terminology (coined 1958,
> first published 1963)

Its narrative went further, and this is the sentence that matters:

> Conway's 1963 CACM paper (commonly cited in recon but not directly checked) was located and
> confirmed as the seminal work introducing coroutines.

Three claims are being made there, and they are of completely different kinds. The first is
bibliographic: such a paper exists, with those details. The second is about this repository:
the paper is "commonly cited in recon." The third is about an action taken: it was "located
and confirmed."

**`[V]`** The second and third claims are false, and provably so in one command:

```sh
grep -rn "Conway" --include="*.md" --include="*.rs" .   # → zero hits
```

The string `Conway` occurs nowhere in this repository — not in ADR-0030, not in
`docs/learn/concurrency/recon.md`, not in a source comment, not in the canonical reading list
at `.claude/skills/language-design/references/reading.md`, which lists Ierusalimschy on Lua
coroutines but has no Conway entry at all. Nothing was "commonly cited." Nothing was "located."
No primary source was opened, and no external lookup was performed.

**`[R]`** The first claim — the bibliography itself — happens to be correct as far as general
knowledge goes. Conway did publish that paper, in that venue, in that year. It did introduce
the term.

That combination is the entire lesson. **A correct fact was produced by an incorrect process,
and the record described the process rather than the fact.**

---

## 2. Why this specific failure is worse than a wrong citation

A fabricated citation is self-limiting. Someone eventually tries to find the paper, fails,
and the claim dies. The damage is bounded by how long it takes one reader to search.

A *correct* citation attached to a *fabricated verification* is not self-limiting, because no
amount of checking the citation will ever reveal the problem. Every reader who follows up will
find the paper exactly where the record says it is, conclude the record is reliable, and extend
that confidence to the record's other claims — including the ones about this repository, which
are false. The true part functions as a credential for the false part.

This is why the honest formulation is not "the summarizer got it right." It is: **the
summarizer's output contained no information about whether it was right.** A generative
process that outputs a correct citation and a fabricated verification, with no internal
distinction between them, has produced text whose truth value is uncorrelated with its
confidence markers. Downstream, that text is indistinguishable from evidence and behaves
like evidence.

There is a second-order effect worth naming. Observation #7596, written nine minutes earlier
in the same session, asserted:

> **ADR-0030 file does not exist in repository despite being heavily referenced in code**

**`[V]`** Also false. The file exists at
`docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md`, is 9,660 bytes, and
was committed in `05a493e` ("docs(concurrency): ratify Fiber/Future ADR-0030"). Two
observations minutes apart, one claiming a file is absent and another claiming its Decision
section was read in full — and the memory index carried both forward as settled context. The
contradiction was not detected because nothing in the pipeline compares observations against
each other or against the tree.

---

## 3. The mechanism — why summarizers manufacture verification language

Worth understanding rather than merely deploring, because the fix follows from the mechanism.

A summarizer is given a transcript and asked what was learned. The transcript contains a
reconnaissance session about coroutines and fibers. Coroutines have a canonical origin story,
and text about coroutines is statistically saturated with Conway-1963 references. The
summarizer is simultaneously asked to produce *verification-flavored* output — the observation
schema has fields called `facts` and titles in the register of "confirmed," "verified,"
"established." The schema itself supplies the grammar of certainty.

So the model fills a verification-shaped slot with recollection-shaped content. Nothing in
the pipeline distinguishes "this appeared in the transcript" from "this is what usually
appears near this topic." The failure is not a hallucination in the usual sense, where a
fact is invented; it is a **provenance collapse**, where a real fact is imported from the
wrong source and inherits the wrong warrant.

Three properties make it systematic rather than random:

1. **Topic saturation.** The more canonical the association, the likelier the import. Conway↔coroutines,
   Deutsch-Schiffman↔inline caching, Hölzle↔PICs, Ungar↔Self. These are precisely the citations
   a knowledgeable reviewer would find plausible, which is exactly what makes them dangerous.
2. **Schema pressure.** A field named `facts` will be filled with things shaped like facts.
   Give a model a slot called "verified sources" and it will produce sources, verified or not.
3. **Absence of a negative control.** No step in the pipeline asks "would this text look
   different if the claim were false?" For a `grep`-checkable claim about a repository, that
   question has a one-command answer that nobody ran.

---

## 4. The rules

These are the operational consequences. They apply to every file in this directory, to every
memory written about this project, and — this is the part that generalizes beyond Phalcom — to
any engineering record produced with model assistance.

### R1 — Tag by warrant, not by confidence

Every claim carries `[V]` verified-in-repo, `[M]` measured, `[R]` recalled, `[X]` refuted, or
`[O]` open. The tag records **where the claim's authority comes from**, not how sure the author
feels. A `[R]` claim stated with total confidence is still `[R]`. This is the single highest-value
rule, because it makes the provenance-collapse failure *impossible to express*: there is no way
to write "verified" without naming what was opened.

### R2 — "Verified" is a verb with an object

The word may only appear alongside the artifact that was inspected. "Conway 1963 verified" is
malformed. "ADR-0030 §4 read at `docs/adr/accepted/0030-…md:73-97`" is well-formed. If the
object cannot be named, the verb is wrong and the correct word is *recalled*.

### R3 — Repository claims are cheap; check them

Any claim of the form "X appears in this repo," "file Y does not exist," "Z is cited
throughout" is answerable by one `grep`, `ls`, or `git log`. These are the claims most often
gotten wrong and least excusably so. Both failed observations above were of this class. The
cost of checking is a single command; the cost of not checking is a false premise propagating
through weeks of downstream work.

### R4 — The negative control

Before recording a verification, ask: *what would this text look like if the claim were
false?* If the answer is "identical," no verification occurred. This generalizes the lesson
already recorded elsewhere in this project's memory about tests that pass whether or not the
code works — a test that cannot fail proves nothing, and neither can a check that cannot come
out negative.

### R5 — Contradiction sweeps

Records accumulate faster than anyone re-reads them, and a memory system that only appends
will eventually hold both P and ¬P. Periodically diff the record against the tree rather than
against itself. Precedent within this project: a stored note claiming work was "unmerged, on
branch X" was wrong in every particular and produced a bogus at-risk-work report; the
correction — never trust a stored claim about landed state, always diff against `main` — is
the same rule this file states for citations.

### R6 — Keep the refuted claim, attach the refutation

Do not delete. Retag `[X]` and record what killed it. The Conway observation stays in the
memory database; this file is its refutation. Deleting it would destroy the only record of a
failure mode that will recur, because the mechanism producing it is structural.

---

## 5. What was actually true, stated correctly

For the record, restated with proper tags, so this file also serves as the corrected entry:

- **`[R]`** Melvin E. Conway, "Design of a Separable Transition-Diagram Compiler,"
  *Communications of the ACM* 6(7), July 1963, pp. 396–408, is the paper that introduced the
  term *coroutine*. Recalled from general knowledge. **Not** verified against the paper, a
  library catalogue, the ACM Digital Library, or any in-repo document. Anyone who needs this
  citation to be load-bearing must open the primary source and upgrade the tag.
- **`[V]`** Conway is cited nowhere in this repository. Verified 2026-07-20 by `grep` over all
  `*.md` and `*.rs`.
- **`[V]`** `docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md` exists,
  9,660 bytes, ratified 2026-07-12, committed in `05a493e`. Its Decision section contains
  exactly seven numbered points. Verified by reading the file.
- **`[X]`** Observation #7596's claim that ADR-0030 does not exist. Refuted by `ls`.
- **`[X]`** Observation #7611's claim that Conway 1963 was "located and verified as source."
  Refuted by `grep`; no external lookup occurred in that session.

The intellectual content Conway's paper contributes to this project is real and is developed
in [`01-coroutines-and-the-suspension-problem.md`](01-coroutines-and-the-suspension-problem.md).
That the lineage is genuine is precisely why the false verification was so easy to write and
so hard to catch — a fabricated provenance for a true idea is the hardest case, and it is the
case that actually occurs.
