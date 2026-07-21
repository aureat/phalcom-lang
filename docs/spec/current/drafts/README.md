# `docs/spec/current/drafts/` — the exploration tier

**Nothing in this folder is decided.** These are growing documents: places to accumulate
findings, cost-benefit analyses, precedent, and hazards for ideas that are **not
implemented, not planned, and not ratified**. They exist so that when a question is
finally put, the reasoning is already assembled instead of re-derived from scratch.

## What this folder is for

Phalcom has four tiers of design document. Know which one you are reading:

| Tier | Where | Means |
|---|---|---|
| **Ratified decision** | `docs/adr/accepted/` | Committed. Changing it needs a superseding ADR. |
| **Normative spec** | `docs/spec/current/*.md` | The designed surface. Cites the ADR that ratified it. |
| **As-built** | `docs/spec/current/decorators/`, `docs/forge/units/*/as-built.md` | What actually shipped, with `file:line` evidence. |
| **Draft** ← *you are here* | `docs/spec/current/drafts/` | **Exploration.** No authority. May contradict itself. May be wrong. |

A draft **may not** be cited as a plan, a commitment, or a reason to build something. If
a draft's idea gets taken up, it graduates by growing an ADR — and the ADR, not the
draft, becomes the citation.

## The house rules for a draft

1. **Status banner is mandatory and must say `Draft`.** No exceptions. A reader arriving
   from a search result must learn in the first line that this is not decided.
2. **Never mark a draft `Accepted` or `Proposed`.** Those words have process meaning in
   this repo (`docs/adr/STATUS.md`) and using them here forges a decision that was never
   made. This exact defect class — a doc's status line disagreeing with reality — has
   already cost this project several reconciliation passes; see the overlay's *Known
   documentation defects*.
3. **Verify or flag.** Every claim about the tree carries a `file:line`. Every claim
   about a committed position cites an ADR or spec §. Anything you did not check gets
   marked explicitly as unverified. **Guessing is worse than a gap** — a gap invites
   someone to look, a wrong assertion stops them looking.
4. **Precedent must carry its consequence.** "Rust does X" is trivia. "Rust does X, which
   forces Y, and cost them Z" is an argument. Never cite a language without naming what
   the choice cost it.
5. **Say what it precludes.** Every proposal forecloses something — a future feature, an
   optimization, an invariant. A draft that lists only benefits is not finished.
6. **Append, don't rewrite.** These files grow. When a later session learns something,
   add it with its date and evidence; contradict an earlier section explicitly rather
   than silently editing it away. The record of *why we changed our minds* is worth more
   than a tidy document.
7. **Number your open questions** (`F-1`, `B-2`, `C-3`, …) and never renumber them —
   other docs will start citing them.

## Current drafts

| File | Explores | Notes |
|---|---|---|
| [`stdlib-catalog.md`](stdlib-catalog.md) | The whole missing standard library, and the order it must be built in | **Start here for "what does Phalcom still need?"** Catalogs Tier 0 (language mechanisms) → Tier 6 (ecosystem) with illustrative signatures. Its deliverable is the dependency order, not the list: 10 items block everything else. Three forks need a user ruling (S-1 resource lifetime, S-2 IO shape, S-3 threads). |
| [`ffi.md`](ffi.md) | Foreign function interface; native/extension modules | Argues FFI, not the ADR-0019 floor, is the door for native capability. Dependency for `bytes`/`crypto`/`native-math`. |
| [`bytes.md`](bytes.md) | A `Bytes` type — native arm vs `.ph` | Follows ADR-0020's `List` template. Zeroization is unfixable under a GC. |
| [`native-math.md`](native-math.md) | numpy-shaped native math; do we need `f32`/`u32`? | Answer: dtype-on-array, not new scalar classes. Collides with ADR-0019 head-on. |
| [`crypto.md`](crypto.md) | A crypto standard library | Bind audited crates; never implement. Surveys JS/Node/WebCrypto/wasm/noble/Rust. |
| [`default-arguments.md`](default-arguments.md) | Default args — **a REJECTED feature** | Exploration only. [ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) rejects them. Records what they'd buy and what a superseding ADR must prove. |
| [`sealed-classes.md`](sealed-classes.md) | Sealing + exhaustiveness | NB: `@sealed`/`@variant` are **already built** (`compiler/attributes.rs:648`); the missing half is a `match` checker. |
| [`reactivity.md`](reactivity.md) | Signals/`Computed`/`Effect` | ADR-0058 ratified the *design*; `Reactive` is unbuilt (zero hits in `phalcom-core/src`) and unowned. |
| [`decorators-observable.md`](decorators-observable.md) | `@observable`/`@computed` | Gated on `reactivity.md`'s substrate landing. |
| [`decorators-web.md`](decorators-web.md) | `@resource`/routing/param binding | No HTTP surface exists; no owning unit. |
| [`decorators-persistence.md`](decorators-persistence.md) | `@entity` / ORM tier | No DB surface exists; no owning unit. |
| [`decorators-behavioral.md`](decorators-behavioral.md) | `@retry`/`@cached`/… | Support classes shipped; the decorators did not. |
| [`decorators-dispatch-observability.md`](decorators-dispatch-observability.md) | `@traced`/`@delegate`/`@featureFlag` | Needs the unbuilt Install/Dispatch/Runtime tiers. |
| [`proxy.md`](proxy.md) | Object-granularity proxies, capability membranes | Unratified. Blocks reactivity's R-3/R-5 upgrade path. |
| [`callables.md`](callables.md) | `Family` vs `Method.bind` unification | Unratified; open per functions.md §3. |
| [`implicit-self.md`](implicit-self.md) | Implicit-`self` elision | Unratified. Hazard: implicit-self elision varies effective arity → collides with selector identity (ADR-0012/0043). |

## Graduating a draft

A draft leaves this folder by **growing an ADR**, not by being promoted in place:

1. The open questions get answered — by the user for a design fork, or by measurement
   where [ADR-0051](../../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)'s
   measure-first rule applies.
2. An ADR is written and ratified. It cites the draft for its reasoning.
3. The normative surface moves to `docs/spec/current/`; the as-built record follows once a
   unit lands it.
4. The draft is deleted or reduced to a pointer. **Two live documents describing one
   feature is the defect this tiering exists to prevent** — see `docs/forge/README.md` on
   why the `phase-next/` convention was retired.
