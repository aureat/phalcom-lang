# 25. Separate external labels from internal parameter names

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0012](0012-selector-signature-encoding-and-dispatch.md) (label-encoded
  selectors — the label, not the binding name, is part of selector identity),
  `docs/spec/v0.2/messages-and-selectors.md` §2–3, `docs/spec/v0.2/selectors.md`,
  `docs/spec/v0.2/open-questions.md` Q3, `phalcom-ast` (parser/param list), `frame.rs`

## Context

A Phalcom keyword parameter today forces the **call-site label** and the
**method-body binding** to be the same word. In `at(i, put:)`
([`core.ph`](../../phalcom-core/core/core.ph)) the label is `put`, so the body
*must* also call the value `put` — even where a real variable name
(`newValue`) would read better. Every keyword parameter is this compromise: the
word that reads well at the call site (`to`, `by`, `with`, `into`) is often a poor
local variable, and vice-versa.

[ADR-0012](0012-selector-signature-encoding-and-dispatch.md) encodes a selector as
**name + labels** (`at(_,put)`, `move(_,to,duration)`); the internal binding name is
**not** part of the selector symbol. So allowing the two to differ changes nothing
about dispatch — it is a parser + frame-binding concern only. Open-question Q3 asks
whether to take that ergonomic split; this ADR answers yes.

## Decision

**Allow a keyword parameter to declare a separate internal binding name.** The
external label participates in the selector; the internal name is a frame-local slot
only.

- **Spelling.** In a labeled parameter, the word(s) before the final identifier are
  the **label**, the final identifier is the **binding**, and the trailing colon
  marks the slot labeled:
  ```phalcom
  move(to target:) {        // label `to`, internal binding `target`
    _position = target
  }
  p.move(to: origin)        // call site is unchanged: labels with `to`
  ```
- **Single-word form is sugar for `x x:`.** When label and binding coincide, write
  one word — exactly today's syntax, fully backward-compatible:
  ```phalcom
  resize(width:) { _w = width }   // label `width`, binding `width`
  ```
- **Selector identity is unchanged.** `move(to target:)` and `move(to:)` both encode
  the selector `move(to)`; they are the *same* method and dispatch identically. Only
  the frame slot the argument binds to differs.
  ```phalcom
  at(i, put value:) { self.rawSet(i, value) }   // selector still at(_,put)
  grid.at(0, put: x)                             // call unchanged
  ```

## Consequences

- **Call sites read like prose while bodies get real names.** `grid.at(2, put: v)`
  at the call site, `value` (not `put`) inside the method; `move(to: p)` outside,
  `target` inside. The `at(_,put)`-style "name the variable against its will" problem
  goes away.
- **Zero dispatch impact.** No change to `encode_selector`, the method dictionary,
  inline caches, or selector identity — the internal name never entered the selector.
  The change is localized to the `phalcom-ast` param-list grammar and how a frame
  binds the incoming argument (to the internal slot name instead of the label).
- **One extra grammar form.** The parser must accept `label binding:` in addition to
  `binding:`. Small, and disambiguated by counting identifiers before the colon.
- **Backward-compatible.** Every existing single-word labeled parameter keeps working
  as the `label == binding` case; no existing method changes.
- **A style question, not a correctness one.** Nothing forces the split; it is used
  where the two good names differ. Convention (which parameters deserve a distinct
  internal name) is a docs/style matter, not enforced by the language.

## Alternatives considered

- **Keep label == binding (status quo).** Minimal grammar, but bakes in the
  call-label-vs-variable-name compromise on every keyword parameter — the exact pain
  `at(_,put)` already exhibits. Rejected for a language built around fluent keyword
  messages.
- **Types-carry-the-split (Swift's `to target: Point`).** Phalcom parameters are
  untyped, so there is no type token to hang the second name off; the `label binding:`
  form is the untyped analogue.
- **A punctuation sigil between label and binding (`to->target:`, `to=target:`).**
  Noisier than juxtaposition and buys nothing; whitespace-separated `label binding:`
  is the lowest-ceremony reading.
