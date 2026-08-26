# Phalcom Semantic Analyzer Specification
## 13 — Semantic Consumers and Request Consistency

**Status:** Normative semantic-analyzer specification.

**Purpose:** Define how compiler, diagnostic, LSP, lint, navigation, refactoring, and presentation consumers query one immutable semantic world without reconstructing competing authority.

---

## 1. One request, one semantic world

A semantic request operates against one pinned immutable snapshot.

```text
request start
    -> pin source/document state
    -> pin SemanticSnapshot
    -> classify source/snapshot relationship
    -> perform read-only queries/adaptation
    -> render response
```

The request must not swap snapshots mid-operation or combine products from incompatible generations.

---

## 2. Source-to-snapshot relationship

Every request classifies its document/source relationship as:

```text
Exact
    request source revision and canonical module mapping match snapshot

Stale
    canonical source exists, but current document text/revision differs

Unmapped
    no canonical source/module attachment exists in snapshot
```

The classification applies before mapping positions into semantic source sites.

---

## 3. Exact requests

For `Exact`, compiler-published semantic products are authoritative.

Request handlers must not:

- re-run formal or advisory analysis;
- redispatch calls or members;
- reconstruct binding, declaration, callable, field, or module identity;
- perform independent module/import/filesystem semantic resolution;
- infer types or runtime shapes from AST when the published product owns them;
- reconcile a second semantic database with the compiler snapshot.

Syntax remains available for presentation structure and protocol adaptation, not competing semantic truth.

---

## 4. Stale requests

A stale request must not apply old semantic ranges or source-site IDs to current text as though they were exact.

It may use:

- syntax recovery;
- lexical classification;
- token/range-local presentation that makes no canonical semantic claim;
- deliberately remapped presentation data only when a verified source-map algorithm proves the mapping.

It must omit definition/reference/type/dispatch claims that require unavailable exact identity. Stale semantic diagnostics must not be rendered at current ranges without verified mapping.

---

## 5. Unmapped requests

An unmapped request uses syntax/lexical behavior until canonical source identity is published.

It may offer purely lexical completion, syntax diagnostics, tokenization, formatting, or document-local structural information. It must not fabricate project/module/declaration identity or claim canonical workspace semantics.

---

## 6. Fallback categories

| Fallback | Exact | Stale | Unmapped |
|---|---:|---:|---:|
| presentation formatting/summarization | allowed | allowed | allowed |
| syntax recovery | allowed for malformed presentation context | allowed | allowed |
| lexical recovery | allowed where semantic claim is unnecessary | allowed | allowed |
| semantic reconstruction | forbidden | forbidden as canonical result | forbidden |
| filesystem semantic discovery | forbidden on request path | forbidden | forbidden |

Fallback must be named by category. “Fallback” alone is not permission to rebuild semantic authority.

---

## 7. Consumer authority matrix

| Consumer | Exact snapshot behavior | Stale/unmapped behavior |
|---|---|---|
| diagnostics | render compiler-published semantic diagnostics and exact ranges | syntax diagnostics; omit/remap semantic diagnostics only with verified mapping |
| hover | query formal/advisory presentation for exact site/target | lexical/syntax hover; omit canonical type/target claim |
| completion | consume canonical scope/member/module/advisory query products | lexical/syntax candidates only, clearly non-canonical |
| inlay hints | enumerate published semantic source sites/products | omit semantic hints or use syntax-only hints |
| definition | follow exact `SemanticTargetId` to declaration source | omit unless canonical target mapping is verified |
| references/rename | use indexed exact reverse occurrences | unavailable; never name/range scan as semantic result |
| document/workspace symbols | consume published declaration/source indexes | document-local syntax symbols may be returned without canonical target claims |
| semantic tokens | refine syntax tokens with exact semantic classification | syntax/lexical tokens only |
| signature help | query canonical call/callee/parameter mapping | syntax-derived selector shape only; no invented callable identity |
| lints/refactoring | consume declared formal/source products | run only checks/edits whose premises remain available and explicit |

---

## 8. Formal and advisory presentation

Consumers may present formal and advisory facts through one visual vocabulary, but internal authority remains separate.

Rules:

- formal established/assumed/unknown/dynamic/status/cause remain unchanged;
- advisory facts may enrich presentation without becoming formal types;
- disagreement does not become a union or hard diagnostic;
- provenance and uncertainty are exposed contextually when useful;
- unavailable advisory coverage is distinct from advisory unknown/non-ready.

Chapter 11 owns the semantic composition rules. Consumers only render/query them.

---

## 9. Request context and pinning

A request context must retain:

- pinned immutable semantic snapshot;
- canonical module identity, when mapped;
- document/source revision used for position mapping;
- exact/stale/unmapped classification;
- cancellation/deadline state;
- protocol/presentation configuration.

Every semantic subquery in that request derives from this context. A helper must not silently fetch a newer snapshot or mutable workspace state.

---

## 10. Query layer

The semantic layer exposes protocol-neutral read queries for:

```text
site_at(module, position)
target_at(site)
occurrences_for(target)
formal_fact(site/expression/binding)
advisory_fact(site/expression/binding/callable)
presentation_for(site/target)
declaration_source(target)
visible_members(scope/receiver)
call_signature(call site)
analysis status / diagnostics / explanations
```

Exact API shape may differ. Queries must preserve canonical identity, snapshot ownership, authority lane, and absence/ambiguity/terminal distinctions.

---

## 11. Request-path restrictions

Request paths are read-only over the pinned snapshot. They must not synchronously:

- scan the filesystem for semantic resolution;
- rebuild project/module graphs;
- mutate the semantic DB or advisory contribution state;
- run fixed-point analysis;
- publish a snapshot;
- perform whole-workspace occurrence scans where an index is required.

Demand-driven compiler queries may exist only when their concurrency, snapshot, cancellation, dependency, and publication semantics preserve the same one-world contract.

---

## 12. Incomplete source and errors

Malformed source may receive partial syntax/lexical behavior. Recovery artifacts do not establish normative semantics for an invalid complete program.

Consumers distinguish:

- syntax recovery;
- unresolved canonical dependency;
- formal semantic invalidity;
- advisory absence/non-readiness;
- stale or unmapped source;
- cancelled request;
- internal semantic incident.

Internal failures are not rendered as plausible user type errors.

---

## 13. Performance and concurrency

Read queries should be bounded by indexed product access and the size of the requested result, not whole-workspace reconstruction.

Publishing a newer snapshot does not invalidate the immutable snapshot pinned by an in-flight request. Cancellation stops unnecessary response work but does not mutate the snapshot.

Consumer caches are presentation caches keyed by snapshot/source identity. They are not alternate semantic databases.

---

## 14. Conformance requirements

Tests must cover:

1. every semantic request pins exactly one snapshot;
2. concurrent publication does not mix generations inside a request;
3. exact hover/completion/definition/references consume canonical compiler products;
4. stale source never receives old semantic ranges as exact;
5. unmapped source offers only syntax/lexical behavior;
6. no exact handler performs semantic redispatch, inference, or filesystem resolution;
7. definition/references/rename use canonical targets and indexed occurrences;
8. formal/advisory disagreement changes presentation only according to chapter 11;
9. cancellation/internal failure remain distinct from user diagnostics;
10. request metrics demonstrate indexed reads and no whole-workspace semantic rebuild.
