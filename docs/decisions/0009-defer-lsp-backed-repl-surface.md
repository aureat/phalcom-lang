# PDR-0009 — The LSP-backed REPL surface waits for ADR-0056 to be ratified

- Status: Accepted
- Date: 2026-07-20
- Related: [ADR-0056](../adr/proposed/0056-phalcom-lsp-architecture.md) (**Proposed** — the
  unratified record this deferral is waiting on),
  [U-REPL impl/06-surface.md §1](../forge/units/U-REPL/impl/06-surface.md) (§S5's three
  highlighting layers), [U-REPL surface.md §S2/§S5](../forge/units/U-REPL/surface.md),
  [U-REPL impl/01-wiring.md](../forge/units/U-REPL/impl/01-wiring.md) (the dependency this
  record keeps)

## Context

U-REPL's Stage 6 specifies an LSP-backed REPL surface in two places, with named entry points
and an explicit instruction not to reimplement them:

- `impl/06-surface.md:40-43` — L2 syntactic highlighting via
  `phalcom-lsp/src/semantic_tokens.rs`, *"`tokens_for(text, line_index)` at :340, legend at
  :139… **Call it. Do not port it.**"*
- `surface.md:58-61` — completion kind filtering reuses the LSP's `ReceiverKind` split
  (`phalcom-lsp/src/completion.rs:44`, entry point `completions()` at :376), which *"must not be
  rebuilt."*
- `plan.md:250` — *"`phalcom-repl` depends on both `phalcom-lsp` and `phalcom-core`."*

None of it is built. Verified 2026-07-20 at `c346200`: `grep -rn "phalcom_lsp"
phalcom-repl/src/` returns **zero** matches. The dependency is declared at
`phalcom-repl/Cargo.toml:12` and never imported. `PhalcomHighlighter` implements only Layer 1
(raw `phalcom_ast::lexer`) and Layer 3 (oracle dimming); there is no Layer 2. `PhalcomCompleter`
classifies position with `rfind('.')` string slicing, not an LSP parse.

**The blocking problem is not that Stage 6 is unbuilt. It is what Stage 6 is built on.**

ADR-0056 — the record that authorizes the `phalcom-lsp` crate at all — is **Proposed**, not
Accepted. U-REPL's implementation spec mandates calling into a crate whose existence is an
unratified proposal, and mandates it with no deferral clause, unlike its neighbours (`:reset`
and `:help` are *"specced now and unimplemented on purpose"* at `impl/07-commands.md:16`;
signature hints are *"deferred"* at `surface.md §S4`).

Building Stage 6 now would ratify ADR-0056 by fait accompli — the crate becomes load-bearing for
the REPL, and the proposal is then settled by the fact of the code rather than by anyone
deciding it. That is the failure mode `README.md`'s maintenance rule 5 exists to prevent.

Two secondary defects follow from the same gap:

- `phalcom-repl/src/highlighter.rs:5` documents *"**L2 Syntactic**: `phalcom-lsp` semantic token
  pass"* — a layer the file does not implement. The module doc describes an intent as a fact.
- `CLAUDE.md:60` claims the REPL *"Implements … LSP-backed autocompletion and syntax
  highlighting."* False.

Neither the deferral nor these defects appear in `docs/forge/DEFERRED.md`.

## Decision

### 1. Stage 6 is deferred, explicitly and on the record

The LSP-backed L2 highlighting layer and LSP-backed completion filtering are **not built** until
ADR-0056 is ratified. This is a deferral with a named precondition, not an abandonment.

### 2. The `phalcom-lsp` dependency stays

`phalcom-repl/Cargo.toml:12` is not touched. The dependency is unused today, and removing it
would delete a mandated write-set target and require re-deriving the entry points when Stage 6
is picked up. An unused dependency is a smaller cost than a lost decision.

### 3. Ratifying ADR-0056 is the precondition, and it comes first

If Stage 6 is wanted, ADR-0056 is ratified — or superseded by a PDR — *before* any code calls
into `phalcom-lsp` from the REPL. Not after, and not concurrently.

### 4. Three corrections land now

The deferral is only honest if the tree stops claiming otherwise:

- `phalcom-repl/src/highlighter.rs:5` — the module doc drops L2 or marks it explicitly
  unimplemented.
- `CLAUDE.md:60` — the "LSP-backed" claim is removed; the REPL's completion and highlighting are
  oracle- and lexer-backed.
- `docs/forge/DEFERRED.md` — one entry recording this deferral and its ADR-0056 precondition.
- `docs/forge/UNITS-TRACKER.md` — U-REPL's row stops reading *"fully landed (Stages 0–6)"*, which
  is false while Stage 6 is unbuilt.

## Consequences

- ADR-0056 stays genuinely open, decidable on its own merits, rather than being settled by a
  dependency edge nobody voted on.
- The REPL keeps working: Layer 1 lexical highlighting and oracle-backed completion are real,
  shipped, and adequate. Stage 6 is an improvement, not a prerequisite.
- The tree stops carrying three statements that are not true.
- Whoever ratifies ADR-0056 inherits a well-specified Stage 6 with entry points already
  identified — the deferral costs no design work.

**The cost, named plainly:** the REPL's highlighting stays lexical, so it cannot distinguish a
class name from a local, and completion ranking stays coarser than specified. That is a visible
quality gap for as long as the deferral holds, and `surface.md` will describe a surface the
binary does not have. §4's corrections are what keep that gap honest rather than hidden.

**What this precludes.** Nothing permanently — every entry point named in `impl/06-surface.md`
remains available. It does mean the REPL surface cannot improve past Layer 1 until an ADR is
ratified, so if Stage 6 becomes urgent, the ratification is on the critical path.

## Alternatives rejected

- **Build Stage 6 now.** The spec mandates it with exact entry points, so this is the obvious
  reading of the plan. Rejected because it commits an unratified ADR by implementation. The
  spec's mandate is itself the defect here: U-REPL should not have specced hard against a
  Proposed record.
- **Remove the `phalcom-lsp` dependency.** Tempting — it is genuinely unused, and an unused
  dependency reads as an error. Rejected under §2: it deletes a mandated target and silently
  converts a deferral into an abandonment.
- **Drop the LSP ambition entirely and port the logic into `phalcom-repl`.** Directly
  contradicts *"Call it. Do not port it."* and duplicates a semantic-token implementation that
  would then drift from the LSP's. Needs its own superseding record if ever wanted.
- **Leave it undocumented.** The status quo. Rejected: the tracker currently claims Stage 6
  shipped, so the gap is not merely undocumented but actively misreported — the most expensive
  of the options, because it is the one that stops anyone looking.
