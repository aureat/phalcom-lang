# 1. Record architecture decisions

- Status: Accepted
- Date: 2026-07-11

## Context

Phalcom is a language implementation with non-obvious design decisions — the
metaclass tower, method-lookup keying, the bootstrap ordering — whose rationale
lives only in conversations and one long spec (`docs/object-model.md`). New
contributors (and AI agents starting fresh sessions) re-derive or, worse,
accidentally violate these decisions because the "why" isn't written down next
to the "what".

## Decision

We will record significant architecture decisions as ADRs in `docs/adr/`, using
Nygard's Context/Decision/Consequences format, one decision per file. An ADR is
warranted when a decision is costly to reverse, affects the public language
semantics, or would otherwise be re-litigated by someone lacking the context.

## Consequences

- Design rationale becomes discoverable and reviewable in the repository,
  independent of chat history or memory tooling.
- A small ongoing cost: decisions must be written up. We accept this for
  decisions that clear the "warranted" bar above; trivial choices need no ADR.
- ADRs complement the layered knowledge system in `CLAUDE.md`: the graphify
  graph answers *structure*, ADRs answer *intent*.
