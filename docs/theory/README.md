# `docs/theory` — the transferable layer

This directory holds **programming-language and interpreter theory as it was actually
encountered while building Phalcom**. It is deliberately not a textbook, not a tutorial,
and not a second copy of the specification. Every other documentation tree in this repo
answers a question about *this* language; this one answers questions a language implementer
would still care about if Phalcom did not exist.

The distinction is worth stating precisely, because four documentation layers already
compete for the same shelf and confusing them is how each of them rots:

| Layer | Question it answers | Authority |
|---|---|---|
| `docs/spec/current/` | *What is Phalcom?* | Normative. The language is what the spec says. |
| `docs/adr/`, `docs/pdr/` | *Why did we choose this over that?* | Binding. A ratified record is a commitment. |
| `docs/learn/` | *How does the implementation actually work?* | Descriptive, code-grounded, per-mechanism. |
| `.claude/skills/language-design/references/` | *What is the general design space?* | Generic. Language-agnostic axes and precedents. |
| **`docs/theory/` (here)** | ***What did building this teach that transfers?*** | **Non-binding. Leads, lineages, and lessons.** |

Nothing in this directory binds an implementation. A file here may argue a position the
ADRs rejected, may record a paper nobody in this repo has read, and may keep a hypothesis
alive that measurement killed — because the killed hypothesis is often the most instructive
artifact in the whole record. What this directory may **never** do is present any of those
as settled fact. That is what the provenance tags below exist for.

---

## Provenance tags — read this before reading anything else

This directory exists partly because of a specific failure. On 2026-07-19 an automated
summarizer wrote an observation titled *"ADR-0030 Decision section fully read; Conway 1963
foundational paper verified"*, whose body asserted that Conway's 1963 CACM paper had been
"located and verified as source." The citation it produced was, as it happens, bibliographically
correct. It had also never been checked against anything: the string `Conway` does not appear
anywhere in this repository, and no primary source was opened. A recollection had been
written down in the grammar of a verification, and a later reader would have had no way to
tell the difference.

Being accidentally right is not verification. Every factual claim in this directory therefore
carries one of five tags, and the tag is part of the claim:

- **`[V]` Verified in-repo.** The claim is backed by a file in this repository, cited with a
  path and where possible a line number. Anyone can re-check it in seconds. This is the only
  tag that licenses building on a claim without further work.
- **`[M]` Measured.** The claim is a number produced by this repo's own benchmark harness and
  recorded in `docs/forge/perf-log/`. Measured claims decay: they describe a binary that may
  no longer exist. Always check the commit a measurement was taken at.
- **`[R]` Recalled.** The claim comes from general knowledge — mine, or an author's — and has
  **not** been checked against a primary source in this repository or anywhere else. It is a
  lead, not a fact. Most bibliographic citations in this directory are `[R]`, and they are
  honest about it. A `[R]` claim is exactly as trustworthy as an unsourced assertion in a
  conversation, because that is what it is.
- **`[X]` Refuted.** The claim was believed, tested, and killed. These are kept, never deleted,
  with the refuting evidence attached. A dead hypothesis with a cause of death attached is
  more valuable than a live one with no test, because it tells you which reasoning pattern
  produced the error.
- **`[O]` Open.** A genuine unresolved question, with enough context that someone could pick it
  up. Distinguished from `[R]` in that nobody claims to know the answer.

When a claim mixes tags — a verified mechanism explained through a recalled precedent — the
tags are applied per-sentence or per-clause, not per-document. Granularity is the point.

---

## The files

**Foundations and method**

- [`00-provenance-and-citation-discipline.md`](00-provenance-and-citation-discipline.md) —
  the Conway incident in full, the failure mode it represents, why summarizers manufacture
  verification language, and the rules that follow. Read first; it governs everything else.

**Design-space lineages**

- [`01-coroutines-and-the-suspension-problem.md`](01-coroutines-and-the-suspension-problem.md) —
  from Conway 1963 to Phalcom's restricted fibers. Where suspended execution state can live,
  why that single question determines the entire concurrency design, and what the four
  possible answers each cost.
- [`02-dispatch-and-selector-identity.md`](02-dispatch-and-selector-identity.md) —
  message send as the only control primitive, selector encoding as a design lever, inline
  caching's lineage, and the striking result that a dispatch-key decision bought pattern-match
  exhaustiveness for free while making default arguments impossible.
- [`03-object-model-and-the-metaclass-tower.md`](03-object-model-and-the-metaclass-tower.md) —
  the reflexive class/metaclass loop, how to bootstrap it without infinite regress, and why
  a parallel tower turns constructor dispatch from a feature into a consequence.
- [`04-values-absence-and-representation.md`](04-values-absence-and-representation.md) —
  tagged unions versus NaN-boxing, the cost of two absences, niche encoding, bootstrap cycles
  in the absence type itself, and truthiness as an enforcement problem rather than a syntax one.
- [`05-closures-control-flow-and-unwinding.md`](05-closures-control-flow-and-unwinding.md) —
  open and closed upvalues, non-local return via frame tokens, why one unwind primitive is
  better than three, and the generation-counter trick that turns a use-after-free into an error.

- [`09-memory-gc-and-rooting.md`](09-memory-gc-and-rooting.md) — arenas and handles, why the
  collector is non-moving, the latched-safepoint invariant, and the finding that root
  enumeration cannot be audited into correctness — it has to be enforced by the compiler.

**Cross-cutting lessons**

- [`06-mechanism-versus-policy.md`](06-mechanism-versus-policy.md) — the strongest single
  idea recovered from the JavaScript-redesign session: languages fail when they ship policy
  where mechanism was needed. Function coloring, the WASM boundary tax, and the honest
  counterevidence that clean redesigns lose anyway.
- [`07-borrowed-techniques-and-their-preconditions.md`](07-borrowed-techniques-and-their-preconditions.md) —
  why a technique ported from another VM must be justified by a property of *that* VM which
  you then check for in yours. Three ports attempted, one survived, and the general rule.
- [`08-performance-epistemology.md`](08-performance-epistemology.md) — attribution is not
  mechanism, profilers name lines rather than causes, harnesses are subjects rather than
  instruments, and a catalogue of this project's refuted performance hypotheses.
- [`10-hazard-catalogue.md`](10-hazard-catalogue.md) — the feature-interaction hazards this
  project has actually hit, written in the `A ⊗ B` form, each with its resolution or its
  standing cost. Feature interactions, not features, are where language designs die.
- [`11-documented-wrongness-as-method.md`](11-documented-wrongness-as-method.md) — the
  documentation practices this repository invented to carry its own errors forward: numbered
  lies with forward pointers, preserved retractions, graded risk registers, and the rule that
  a correct diagnosis does not imply a correct prescription.
- [`12-open-leads-and-reading.md`](12-open-leads-and-reading.md) — an audit of what this repo
  actually cites versus what it is believed to cite, plus twelve ranked leads worth pulling
  and a reading order for the corpus itself.

---

## How to add to this directory

A new file belongs here if a competent implementer of a *different* language would learn
something from it. If the lesson evaporates once you remove the Phalcom specifics, it belongs
in `docs/learn/` instead. If it binds an implementation, it belongs in an ADR.

Three rules, all downstream of the Conway incident:

1. **Tag every factual claim.** An untagged claim is a defect, not a stylistic lapse.
2. **Never upgrade a tag without doing the work.** `[R]` becomes `[V]` only when someone opens
   the source and cites it. Confidence is not evidence, and neither is repetition — a recalled
   claim restated in three files is still one recollection.
3. **Keep the corpses.** When something here is refuted, retag it `[X]`, attach the refutation,
   and leave it in place. Deleting a refuted claim destroys the only record of why the mistake
   was tempting.
