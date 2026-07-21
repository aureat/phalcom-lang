# Requirements

## The learner

One reader: a systems-curious language implementer (author of Phalcom, a bytecode VM in
Rust) preparing for expert-level technical conversations. Learning style is systematic
and bottom-up: maximum decomposition, high organization, holistic maps, and a stated
conviction that a well-chosen implementation detail teaches better than any high-level
summary. Pitch is expert — feature-interaction level, never tutorial.

**Known gap, stated directly by the reader:** atomics, and more generally the act of
*connecting the pieces into a full picture*. "I always had trouble understanding atomics
and things like that and connecting everything, putting the pieces together." This makes
the shared-memory ladder (docs 2–5) the pedagogical center of gravity of the whole book.
Consequences are encoded as R11 below.

## Explicit requirements

| ID | Requirement |
|----|-------------|
| R1 | Several markdown files, book-shaped, one topic per file, in `docs/learn/systems/`. |
| R2 | Maximum decomposition: broken to the smallest load-bearing details, highly organized and structured. |
| R3 | Documents sequenced by knowledge-building order — a cover-to-cover read never meets a term before its foundation exists. |
| R4 | Authoritative single source: every claim the reader needs lives inside the book at the depth needed; no external synthesis required. |
| R5 | A holistic map: the reader can always see the whole territory and their position in it. Discharged by `00-map.md` plus per-doc map hooks. |
| R6 | Implementation detail is a first-class teaching instrument, not an appendix. |
| R7 | Concepts defined inline at the moment they become load-bearing, tied into the bigger picture — explicitly not a glossary. Implemented by the concept/recall block system (see `anatomy.md`). |
| R8 | Register: dense full-text paragraphs, visualizations, full code blocks, strategic placement, gradual build. Terse/compressed style never applies to book prose. |
| R9 | Every document has exactly one nameable grip. The cut list in `scope.md` is normative — no doc without a grip, no cut item resurrecting by drift. |
| R10 | **The machine thread:** every doc grounds its theory in at least one experiment runnable on the reader's own laptop (Apple M1 Pro, see `machine.md`), with the actually-measured output captured in the doc at writing time. The theory must be visible on the machine the reader touches every day. |
| R11 | **Full-picture integration:** every doc in Part II ends with a "where you now stand" synthesis that re-places each newly built mechanism on the book's map. The atomics arc (docs 3 and 5) gets the slowest concept ramp in the book — no atomic operation, ordering parameter, or RMW primitive is ever introduced without explicitly connecting it to the cache-coherence substrate below it and the algorithm above it. Doc 5 closes the ladder with a full-chain synthesis: coherence → ordering → atomic RMW → CAS → ABA → reclamation. |

## Implicit requirements (derived from the working method)

| ID | Requirement |
|----|-------------|
| I1 | Expert pitch, interview-grade; the proving-ground rubric (recalled < derived < traded) calibrates the checkpoint questions. |
| I2 | The docs/learn method governs: grip-first; the reader must be able to *re-derive* the mechanism; comparison only when it pays; form follows the topic's theory rather than a fixed skeleton. |
| I3 | Phalcom scars serve as anchor cases where they genuinely fit: ABA ↔ the Map/Set reentrancy corruption; JIT/deopt ↔ the global-slot-cache and guard-identity scars; GC/ownership ↔ the ensure temp-root use-after-free. Scars are re-verified against the tree at writing time and cited by symbol, never by file:line. |
| I4 | This is general-CS material, not Phalcom documentation. It lives beside `proving-ground/`, outside the language design docs. |
| I5 | The visualization builds are a separate, later track. Book docs stand alone as text; at most a five-line appendix parks viz ideas. |

## Quality gates — a doc is not done until all seven pass

- **Q1 — grip test.** The grip is one sentence, nameable, falsifiable. A doc needing two
  grips is two docs or one cut.
- **Q2 — re-derivability.** The closing checkpoint questions are answerable from the doc
  alone. This is the operational meaning of "authoritative."
- **Q3 — no folklore numbers.** Every quantitative claim is measured with the method
  shown, or cited to a named source. (Standing example: the folklore cache line is
  64 bytes; the target machine's is a measured 128.)
- **Q4 — negative control.** Every demo is shown failing *and* fixed under the same
  harness. A demo that only shows the happy path proves nothing.
- **Q5 — tour detector.** Any section that catalogs features without serving the grip is
  cut. Highest-risk docs: memory ordering (3) and JIT (7).
- **Q6 — inline-definition audit.** No term is used before its concept block or recall
  block exists. Mechanical to check; it is the enforcement arm of R7.
- **Q7 — replicability.** Every at-the-machine block can be pasted by the reader and
  produce the same *shape* of result. Extra installs state the install line;
  nondeterministic results (reordering counts, benchmark deltas) state expected ranges,
  never a single number.

## Ruled decisions

| ID | Ruling |
|----|--------|
| D-1 | Location: `docs/learn/systems/`, files `NN-slug.md`, entry point `00-map.md`. |
| D-2 | The three gated docs (JMM happens-before, RAII/GC/ownership, libuv phases) stay out of scope until the core seven land. |
| D-3 | Writing order equals reading order. Pilot is doc 1 (event loop): zero prerequisites, validates the anatomy at lowest cost, becomes the exemplar every later doc is measured against. |
| D-4 | **No SVG anywhere in the book.** Mermaid for state machines, flows, and graphs; fixed-width ASCII for memory layouts (cache lines, struct padding, buffers), where mermaid is the wrong tool anyway. SVG/visualization work is deferred to its own explicitly planned track. |
| D-5 | Viz ideas are confined to the optional five-line appendix hook per doc. |

## Risks

- **Tour risk** (docs 3, 7): the literature is encyclopedic; Q5 is the countermeasure.
- **Anchor rot:** Phalcom scars must be re-verified at writing time; cite symbols.
- **Authority risk:** where the literature disagrees or behavior is
  architecture-dependent (x86-TSO vs ARM weak ordering is the canonical case), the book
  says so instead of flattening disagreement into false confidence. Claims carry their
  warrant (see README provenance discipline).
- **Scope resurrection:** the cut table in `scope.md` is the contract; re-entry requires
  an explicit decision recorded there.
- **Atomics-ramp failure mode:** the personally hardest material (docs 3, 5) is also the
  most tour-prone. If a reader with the stated gap cannot pass doc 5's checkpoint, the
  failure is the book's, not the reader's — R11 exists to make that failure structural,
  not stylistic.
