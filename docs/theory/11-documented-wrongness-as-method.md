# 11 — Documented wrongness as a method

> **Thesis:** the most unusual thing about this repository is not any decision it made. It is that
> its documentation is built to *carry its own errors forward*. Simplifications are labelled as
> lies with forward pointers to where they get destroyed; retracted claims are preserved
> struck-through as the record of the mistake; a shipped document opens with the admission that
> the feature it describes does not work. This file extracts that method, because it transfers to
> any technical writing where the subject is complicated enough that the first accurate sentence
> is too complicated to be the first sentence.

---

## 1. The lie convention

**`[V]`** Teaching documents in `docs/learn/` state simplifications explicitly, number them, and
point at the document that destroys them. From the execution-loop document:

> **Lie #1** — "The loop reads its code array" is not the whole truth; it hoists a shared reference
> across iterations and refreshes it with a one-compare guard, for a measured reason. → Doc 2.
>
> **Lie #2** — the `Invoke` arm "sends a message." A send does not just "run a method." It pushes a
> resumable **frame**. → Docs 3/4.

The successor documents then say so. The compiled-artifact document explicitly **destroys Doc 1's
Lie #1**. The frame-identity document opens its resolution with "`generation` and
`home_frame_token` were never 'just fields.'"

Why this works is worth stating precisely, because it is not merely a nice touch:

**A pedagogical simplification is a debt, and untracked debt is indistinguishable from a wrong
claim.** Every technical explanation simplifies. The ordinary practice leaves the reader unable to
tell which sentences were approximations for their benefit and which were the author's actual
belief. Numbering the simplification and naming its creditor converts an epistemic problem into a
bookkeeping one.

**It also disciplines the author.** You cannot write "Lie #2 — X is not the whole truth" without
knowing what the whole truth is. The convention makes the boundary of your own understanding
visible at the moment you write the sentence, which is the only moment you can cheaply do something
about it.

---

## 2. Preserving the wrong version

**`[V]`** The strongest example in the corpus. A requirements document's central section is headed:

> **SUPERSEDED. The original was wrong about Phalcom.** ⚠

and preserves the wrong thesis struck through, explicitly labelled "the record of the error":

> ~~An upvalue is a pointer that knows how to become an owner… the read path never branches~~
>
> **This describes Lua, not Phalcom, and I asserted it before reading the source.**

Three things make this entry exemplary rather than merely honest.

**It names the mechanism of the error.** The claim was asserted from a lineage label — "Phalcom is
Lua-style" — before opening the source. The generalized lesson, and it is one of the most useful
sentences in the repository:

> **`[V]`** *"X-style" is a claim about architecture and says nothing about representation, and
> representation is where the consequences live.* Two systems can share every structural feature
> and still make opposite trades.

**It tracks contamination.** The wrong grip had been passed to a subagent as its brief, so that
agent's output was "confidently wrong about the target language — through no fault of A, which had
no source access by design." An error's *blast radius* is recorded, not just the error.

**It scores the risk register.** The same document had pre-registered this exact risk, and the
resolution reads: "**FIRED. Resolved.** ✅ — Outcome: half right, and the wrong half was the
load-bearing half." A risk register that is graded after the fact is a calibration instrument. One
that is written and never revisited is a ritual.

---

## 3. Shipping a document that says the feature does not work

**`[V]`** The concurrency track's fourth document opens with a masthead retraction and is
deliberately **left as written**, to be read in the past tense. Its content, in the record's own
terms:

> this is not a doc about a feature that was not built. It is a doc about a feature that was built,
> passes its tests, and does not work.

And, on its own recommendations:

> its closing design-space section suggests unregistering the dead waiter on `await`'s error path,
> and that turns out to be **unimplementable** once the wrapper is gone… **A correct diagnosis does
> not imply a correct prescription.**

**`[V]`** That last sentence is backed by a number from this project's own history: on the confirmed
defect backlog, **four of six reproduced diagnoses carried wrong prescriptions, and two of those
would have broken the tree.** Reproducing a bug and knowing the fix are separate epistemic
achievements, and the second does not follow from the first.

---

## 4. The green-suite discovery

**`[V]`** The same document establishes that the test suite was green over a feature that never
executed. The acceptance fixture's every `await` was on the root fiber except one that *asserts*
the failure; the case labelled "async/await suspending" **does not suspend anything**; coverage of
"a fiber awaits a pending future and later resumes" is **zero cases**.

This is the negative-control principle from
[`00-provenance-and-citation-discipline.md`](00-provenance-and-citation-discipline.md) §R4 applied
to tests, and it recurs across the project:

- **`[V]`** A garbage-collection regression test passed on the error path *because the exception
  machinery incidentally kept the value reachable* — not because the fix worked on that path.
- **`[V]`** A separate audit found two "load-bearing" tests were vacuous, and six defects had
  shipped green.
- **`[V]`** A defect record's own reproduction case **stopped compiling** after an unrelated
  syntax change, and nobody noticed: "**A crash record whose repro no longer compiles is worse than
  no record.**"

> The question is never "does the test pass." It is "**what would make this test fail, and is that
> the thing I think I am testing.**"

---

## 5. Verified versus assumed, stated per claim

**`[V]`** Several documents carry an explicit section separating what was checked from what was
inferred. Two representative examples:

- "That the `@invariant` weave populates `checking` only from inside a native re-entrant frame is
  *verified*. That *no other* path can populate `checking` is **inferred**."
- On generation wraparound: "It is an absence, not a documented judgment, and **this document is not
  going to manufacture the judgment on the code's behalf**."

The second is the sharper discipline. A documentation author is under constant pressure to make the
system look coherent, and the easiest way to do that is to invent the rationale for an absence —
to write "wraparound is not handled because 2⁶⁴ activations is unreachable in practice," which is
probably true, sounds authoritative, and *has never been decided by anyone*. Manufacturing a
judgment on the code's behalf turns an open question into a settled one without anyone settling it.

**`[V]`** The same restraint appears in citation practice: an exception-safety taxonomy is
attributed to David Abrahams with the note that "the attribution is confident, the exact original
venue is not, so it is offered as attribution rather than citation." And an ABA-problem reference
is marked as practitioner folklore "without one citable first use; stated here as attribution, not
as a citation." A source that could not be extracted was flagged **DOUBTFUL** rather than cited.

That is exactly the discipline the Conway incident violated — and its presence elsewhere in the
same repository is what makes the incident diagnostic rather than characteristic. The house style
knows how to do this. An automated summarizer, writing into a schema whose field is called
`facts`, did not.

---

## 6. Deliberate omission, stated

**`[V]`** One document's comparison section reads, in full:

> **No other language appears in this document, and that is the filter working, not an omission.**
> … Repeating it here would be a survey. **Cut: Lua, Go, Wren, Ruby `Fiber`, CPython generators.**

Naming what was cut, and why, is the difference between a scoped document and an incomplete one.
A reader who wonders "why is Go not here?" gets an answer instead of a doubt about the author's
range. **`[V]`** The practice is consistent across the corpus — cut lists carry reasons, e.g. Java
and C# are cut from an inliner comparison because "booleans are primitive, `if` is grammar —
comparing to them smuggles in the rejected branch as if it were free."

**`[V]`** The house rule for the comparisons that *do* appear is equally sharp:

> **Cite precedent with consequence. Not "Ruby does X" but "Ruby does X, which forces Y."**

A precedent without a consequence is decoration. It signals awareness without transferring
information, and it is the most common failure mode in design writing.

---

## 7. Predict, then check

**`[V]`** A recurring technique with a documented hit rate. Before reading the code, write down what
you expect; then read; then record the delta. Results across the corpus:

- A metaclass document predicted a self-loop apex, checked it against a running VM, and found a
  **two-hop** loop — catching a **defect in two in-source doc comments** in the process. Verdict:
  "in a cyclic kernel the *invariant check* is the ground truth, not the prose next to the struct."
- **`[V]`** An adversarial agent's blind prediction about the inliner was **confirmed and worse
  than predicted**, becoming a filed defect.
- **`[V]`** A reconnaissance document's own assumption about a bootstrap site: "**Wrong.**" Another
  of its findings: "**Wrong twice.**"
- **`[V]`** A near-miss preserved deliberately: "*I nearly wrote this up as a citation error; it is
  not one.*"

That last one is the subtlest and the most worth copying. Recording a **near-miss** — an error you
almost made and caught — preserves the reasoning that made the wrong answer attractive. That
reasoning will recur; the corrected conclusion alone will not warn anyone about it.

---

## 8. The practices, compressed

1. **Number your simplifications and name their creditor.** An untracked simplification is
   indistinguishable from a wrong claim.
2. **Preserve retracted claims struck through, with the mechanism of the error.** Delete the claim
   and you delete the only evidence of why it was tempting.
3. **Track contamination.** A wrong premise handed to someone else has a blast radius; record it.
4. **Grade your risk register afterwards.** Ungraded risk registers are rituals.
5. **Separate verified from inferred, per claim.** Never manufacture a judgment on the code's
   behalf to make the system look coherent.
6. **State what you cut, and why.** Scope is a claim; defend it.
7. **Cite precedent with consequence**, or do not cite it.
8. **Predict, then check** — and record near-misses, not just hits.
9. **Ask what would make this test fail**, never whether it passes.
10. **A correct diagnosis does not imply a correct prescription.** Re-derive the fix from the code,
    then verify it separately.

None of these require tooling. All of them cost a sentence or two at the moment of writing, and
each one saves a later reader from a confident, plausible, wrong belief — which is the only kind
that does real damage.
