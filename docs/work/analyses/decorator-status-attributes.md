# Status attributes: `@native`, `@abstract`, `@unimplemented`, and `@todo`

**Status: ANALYSIS — proposed requirements, not a normative specification and not implementation authorization.**

**Baseline:** Phalcom `4936419cac5b2fe43abac230da3b74eb442f566d` (2026-07-22).

**Scope:** This analysis covers four declaration attributes only. It is constrained to the current decorator specification tree and its Graphify relationships. It does not reopen the general decorator-tier design, class model, error hierarchy, or parser architecture.

Claims marked **[V]** were verified against the cited current decorator specifications in this session. **[R]** is recalled language precedent, not externally reopened. **[O]** is a proposed requirement or recommendation.

---

## 1. Executive decision

**[O]** Give each attribute one non-overlapping semantic job:

| Attribute | Job | Installed callable? | Runtime cost | Main invariant |
|---|---|---:|---:|---|
| `@native` | Source anchor for an implementation installed outside `.ph` | No — source member is dropped | None | Every anchor names an installed native binding with the same selector kind. |
| `@abstract` | Declares an inherited implementation obligation | No executable body | Construction gate only | A concrete class has no unsatisfied abstract selectors. |
| `@unimplemented` | Installs an intentional, deterministic failure placeholder | Yes — canonical throwing stub | Only when called | Placeholder never silently becomes absence or an accidental implementation. |
| `@todo` | Retains a static work-item note on a declaration | Unchanged | None | Metadata is extractable even when another attribute removes the member. |

**[O]** These must not collapse into `@ignore`. `@ignore` already owns one narrow meaning: parse and validate a member, then remove it from compilation. It is unconditional, does not suppress parse errors, and is deliberately the sole general drop mechanism. [V: `@ignore`](../../spec/current/decorators/ignore.md)

The recommended compatibility rule is therefore simple:

```text
native          = external implementation exists
abstract         = implementation is required later
unimplemented    = implementation is intentionally absent now, but callable exists
todo             = work metadata only
ignore           = declaration is not code
```

This line prevents the worst diagnostic regression: an intentional missing implementation becoming `doesNotUnderstand`, or a source anchor accidentally becoming the live implementation.

---

## 2. Evidence and constraints

### 2.1 Existing decorator contract

**[V]** Built decorators are Compile-tier AST transforms today. The registry is the name and target-legality gate; unknown built-in names fail as `attr.unknown`. The current expander interface mutates one member but cannot remove it; subtractive attributes therefore need a driver-owned pass. [V: [decorator index](../../spec/current/decorators/index.md), [native implementation rationale](../../spec/current/decorators/native.md)]

**[V]** `@native` and `@ignore` prove the required ordering rule: check target legality first, then drop before member derivation or body weaving. Otherwise an illegal target can vanish silently, or a later decorator can operate on a member that will disappear. [V: [native ordering](../../spec/current/decorators/native.md)]

**[V]** Method, getter, and setter are different declaration kinds. In particular, `toString` and `toString()` are different selectors. A status attribute whose invariant compares selectors must retain selector kind, labels, and arity rather than compare printed names. [V: [native legality](../../spec/current/decorators/native.md)]

**[V]** Attribute arguments currently have a registry-wide validation hole: existing no-argument built-ins accept and discard arguments. New status attributes must specify argument arity and get a shared validation mechanism; silently accepting `@abstract("why")` is not an acceptable permanent contract. [V: [native legality](../../spec/current/decorators/native.md)]

**[V]** Constructor declarations cannot carry attributes in the present grammar, despite some registry target lists naming `Construct`; the parser rejects the attachment first. New specs must say “unavailable in current syntax,” not claim constructor support from a future-facing target table. [V: [native legality](../../spec/current/decorators/native.md), [ignore legality](../../spec/current/decorators/ignore.md)]

### 2.2 Design axes and hazards

| Axis | Requirement consequence |
|---|---|
| Selector identity | `@native` and abstract-obligation matching use exact selector identity, including getter/setter/method kind. |
| Inheritance completeness | `@abstract` needs a class-finalization view of inherited and locally implemented selectors; a local registry expander cannot decide it alone. |
| Callable observability | `@unimplemented` must install a named throwing callable, while `@ignore`/`@native` must leave no callable. |
| Metadata retention | `@todo` must be extracted before any subtractive pass, or a TODO on an ignored/native anchor is lost. |
| Bootstrap boundary | `@native` validates against bootstrap-installed bindings in an invariant test, not compiler-local lookup. |
| Error semantics | `@unimplemented` and abstract construction need defined, catchable language errors, never compiler panics or DNU accidents. |

**[O]** The important interaction hazard is not decorator ordering; it is conflating *absence*, *obligation*, *failure*, and *external implementation*. They look similar in a source file but require opposite method-table states.

### 2.3 Limited precedent check

| Pattern | Precedent | Consequence |
|---|---|---|
| Abstract member plus non-instantiable incomplete type | Java, C#, Kotlin [R] | Subclass conformance becomes a class-completeness rule; a method-local marker alone is insufficient. |
| Explicit failing placeholder | Rust `todo!` / `unimplemented!` [R] | Failure remains local and intentional, but release policy must stop placeholders from becoming permanent. |
| Externally supplied implementation declaration | C `extern` / FFI declarations [R] | Compilation cannot prove the foreign binding alone; a link/bootstrap-time verification boundary is required. |

No external artifact was opened, by scope. These precedents are design comparisons, not adopted surface syntax.

---

## 3. Common normative requirements

The following should be one shared “status attributes” section, not four disconnected implementations.

### R1. Built-in names and targets

**[O]** Reserve `native`, `abstract`, `unimplemented`, and `todo` as built-in attribute names. A user `Attribute` class cannot shadow them.

**[O]** Current legal targets:

| Attribute | Class | Field | Method | Getter | Setter | Index | Variant | Construct |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `@native` | no | no | yes | yes | yes | no | no | unavailable |
| `@abstract` | yes | no | yes | yes | yes | no | no | unavailable |
| `@unimplemented` | no | no | yes | yes | yes | no | no | unavailable |
| `@todo` | yes | yes | yes | yes | yes | yes | yes | unavailable |

`unavailable` means `attr.dangling` under current parsing, not acceptance. If constructor attributes become syntactically expressible later, their legality must be decided in a separate amendment.

### R2. Arguments are checked centrally

**[O]** Add shared descriptor fields for argument shape and static-argument validation. Each attribute requires:

| Attribute | Arguments | Error |
|---|---|---|
| `@native` | none | `attr.argument_arity` |
| `@abstract` | none | `attr.argument_arity` |
| `@unimplemented` | zero or one non-empty static string | `attr.argument_arity` / `attr.argument_literal` |
| `@todo` | exactly one non-empty static string | `attr.argument_arity` / `attr.argument_literal` |

“Static string” means source string literal after ordinary literal decoding, not an expression evaluated by the program. These attributes must not execute user code while compiling or tooling an item.

### R3. Parse before semantic effect

**[O]** Every marked declaration is lexed, parsed, and checked for attribute legality and argument validity. `@native` retains its special anchor-body rule in §4; `@abstract` and `@unimplemented` use bodyless declarations in §5–6. `@todo` never disables parsing or diagnostics.

### R4. Collision validation is explicit and order-independent

**[O]** Validate the full attribute set before transformation. Source order cannot convert an illegal combination into a legal one.

| Combination | Result | Reason |
|---|---|---|
| `@todo` + any one status attribute | legal | `@todo` is static metadata only. |
| `@native` + `@ignore` | `attr.redundant` | Existing attributes mean different things but currently share a drop; preserve their proposed collision rule. |
| `@native` + `@abstract` / `@unimplemented` | `attr.status_conflict` | External implementation, required implementation, and installed failure are incompatible states. |
| `@abstract` + `@unimplemented` | `attr.status_conflict` | One is a non-callable obligation; one installs a callable failure stub. |
| `@abstract` / `@unimplemented` + `@ignore` | `attr.status_conflict` | Do not erase an obligation or explicit failure. |
| Existing body-changing attribute + `@abstract` / `@unimplemented` | `attr.status_conflict` | Contracts, accessors, and derives must not weave/generate around an absent or canonical placeholder body until a concrete composition design exists. |

The last row is intentionally conservative. It rules out accidental semantics now; a future need can define one composed meaning with tests.

### R5. One ordered pipeline

**[O]** Required processing order:

1. parse declarations and attributes;
2. resolve attribute names, targets, and argument shape;
3. reject collision sets;
4. collect `TodoRecord`s from original declarations;
5. collect abstract declarations and inherited-obligation information;
6. validate native-anchor invariants at the bootstrap/integration boundary;
7. remove `@native` and `@ignore` members before derive/weave passes;
8. lower `@unimplemented` declarations to canonical throwing members;
9. finalize class abstractness/completeness and its construction guard;
10. run ordinary later generation and weaving only on surviving compatible members.

Steps 4–9 are semantic phases, not a promise about one Rust function. The ordering is the contract.

---

## 4. `@native`: ratify and close its existing gaps

**[V]** Existing meaning is sound: a `.ph` member is a navigation/documentation anchor for a Rust primitive; the compiler drops it and does not install or protect a binding. Bootstrap owns the live binding. [V: [native semantics](../../spec/current/decorators/native.md)]

### Required contract

**[O]** Retain the existing meaning with these mandatory amendments:

1. `@native` takes no arguments; enforce this through R2.
2. It is legal only on a method, getter, or setter. Do not expose a fictional constructor target.
3. Its body is required and is parsed, but is not name-resolved, type-checked, lowered, or emitted. It is an anchor, not executable reference code.
4. A bootstrap/integration invariant checks every core anchor against an installed native binding using `(holder, full selector identity)`; selector kind is mandatory.
5. `@native` is incompatible with every other status attribute except `@todo`.
6. Tooling must index the original anchor span before the member is dropped, or the implementation must retain an anchor record. This resolves existing question N-2 rather than normalizing a non-working go-to-definition promise.

### What this does not solve

**[O]** The invariant proves anchor-to-binding existence, not body truth. The Rust primitive remains source of record. Removing `@native` can still make its `.ph` body go live; a bootstrap test cannot detect an attribute that no longer exists. Preserve this as a documented hazard rather than pretending a selector-presence test closes it.

---

## 5. `@abstract`: an obligation and class-completeness feature

### Recommended surface

```phalcom
@abstract class Stream {
  @abstract next()
  @abstract at(index)
}

class ArrayStream < Stream {
  next() { /* real implementation */ }
  at(index) { /* real implementation */ }
}
```

**[O]** A bodyless member is legal only when marked `@abstract` or `@unimplemented`. This should be represented as `body: Option<Block>` in the declaration AST and validated after attributes resolve; do not use an empty block as a fake abstract body.

### Required semantics

1. **[O]** `@abstract` on a member introduces its exact selector into the class’s unresolved-obligation set. It installs no executable member.
2. **[O]** A concrete member with the same full selector satisfies an inherited obligation. Getter, setter, and method forms never satisfy one another merely because their printed base name matches.
3. **[O]** `@unimplemented` does not satisfy an abstract obligation.
4. **[O]** `@abstract class` declares that construction of that class is forbidden. A class with any unresolved abstract selector must be declared abstract; otherwise class finalization fails with `attr.abstract_unfulfilled` and lists the selectors.
5. **[O]** An abstract class can inherit concrete methods and can have zero locally declared abstract members. It remains abstract until its inherited obligation set is empty *and* its class declaration is made concrete.
6. **[O]** Every construction path, including reflective/dynamic construction, must reject an abstract class with a catchable `AbstractInstantiationError`. Compile-time checks alone are insufficient in a dynamic object system.
7. **[O]** A send that reaches an abstract declaration due to malformed reflective state must raise `AbstractMethodError`, never produce DNU. This is a backstop, not normal control flow.

### Why the class marker is necessary

**[O]** A member-only `@abstract` would leave an impossible question: can a class with unresolved requirements still be instantiated? Making every such class implicitly abstract hides a public API change. Requiring `@abstract class` makes intent visible and lets finalization point at a missing declaration or missing override.

### Implementation boundary

**[O]** This is not merely a new registry row. It requires an inherited-selector collection phase plus a construction gate visible to all allocation routes. No implementation should start until a PDR defines the class-finalization and allocation-hook seam. A registry-only no-op would create documentation without enforcement.

---

## 6. `@unimplemented`: installed, intentional failure

### Recommended surface

```phalcom
class RemoteCatalog {
  @todo("implement after transport protocol is stable")
  @unimplemented("remote lookup is not available in this build")
  find(id)
}
```

### Required semantics

1. **[O]** `@unimplemented` requires a bodyless method/getter/setter declaration and lowers it to a canonical member that raises `UnimplementedError` when sent.
2. **[O]** The error carries receiver class, full selector, and optional reason. It must be catchable through the ordinary language error path.
3. **[O]** The canonical stub is installed in the method table. Reflection, tooling, and a caller distinguish it from a missing selector (`doesNotUnderstand`).
4. **[O]** A declared body is `attr.unimplemented_has_body`. Unlike `@native`, a dead reference body has no truthful role here and invites false confidence.
5. **[O]** `@unimplemented` may appear in a concrete class when no abstract obligation is involved. If it purports to override an abstract selector, that obligation remains unresolved; the class must be abstract or finalization fails.
6. **[O]** It has no release-mode disappearance rule. Any release policy belongs to a separate lint/build tool and must not change callable semantics.

### Why this is not `@ignore`

**[O]** `@ignore` produces absence; callers receive DNU and cannot tell whether the author forgot a selector, selected a wrong selector kind, or intentionally deferred work. `@unimplemented` preserves identity and produces one reliable, actionable failure.

---

## 7. `@todo`: retained static work metadata

### Recommended surface

```phalcom
@todo("remove legacy equality bridge after migration")
class LegacyEquality {
  @todo("anchor must be validated against bootstrap binding")
  @native toString => "anchor prose only"
}
```

### Required semantics

1. **[O]** `@todo` accepts exactly one non-empty static string reason.
2. **[O]** It creates a `TodoRecord { owner, target, selector?, reason, source_span }` in compiler/tooling metadata. It adds no bytecode, method-table entry, layout state, dispatch hook, or runtime reflection object.
3. **[O]** Records are collected before `@native` or `@ignore` removal; a TODO never disappears because its declaration is deliberately non-live.
4. **[O]** TODOs are declaration-local and do not inherit with methods, fields, variants, or classes.
5. **[O]** The compiler provides a deterministic machine-readable extraction mode. It does not emit a warning or fail ordinary compilation: current decorator specs have no warning tier, and turning TODOs into build failure would smuggle policy into language semantics.
6. **[O]** No deadline, assignee, network issue synchronization, conditional compilation, or automatic expiration belongs in the language attribute. Reason text plus source location is the complete v1 contract.

### Extraction acceptance shape

```json
{
  "owner": "LegacyEquality",
  "target": "getter",
  "selector": "toString",
  "reason": "anchor must be validated against bootstrap binding",
  "span": { "file": "core.ph", "start": 42, "end": 43 }
}
```

The wire format is illustrative; stable field names and sort order need the owning tooling spec.

---

## 8. Diagnostics and acceptance requirements

### Required diagnostics

| Code | Trigger | Required useful data |
|---|---|---|
| `attr.argument_arity` | Wrong number of status-attribute arguments | Attribute name, expected shape, received count. |
| `attr.argument_literal` | Dynamic/non-string/empty reason | Attribute name and literal-only requirement. |
| `attr.status_conflict` | Incompatible status attributes | Both attributes and one-sentence conflicting-state explanation. |
| `attr.unimplemented_has_body` | Placeholder declaration has a body | Explain that `@native` is the anchor-body attribute. |
| `attr.abstract_unfulfilled` | Concrete class leaves obligations unresolved | Class and all full selectors. |
| `AbstractInstantiationError` | Any construction of abstract class | Class identity and unresolved selectors. |
| `AbstractMethodError` | Abstract declaration reached at runtime | Receiver class and full selector. |
| `UnimplementedError` | Unimplemented stub sent | Receiver class, full selector, optional reason. |

### Minimum tests

| Area | Fixtures/invariants required |
|---|---|
| Common | All target rows, every argument shape, every collision, and source-order invariance. |
| `@native` | No bytecode/member installed; syntax error in anchor body still fails; exact selector-kind bootstrap invariant; LSP/source-anchor retention. |
| `@abstract` | Same-name getter vs zero-arg method does not satisfy; inherited obligation is satisfied by exact override; concrete incomplete class diagnostic; all construction routes reject abstract class; malformed reflective send raises `AbstractMethodError`. |
| `@unimplemented` | Stub is discoverable and raises `UnimplementedError`; it is not DNU; body is rejected; it cannot falsely satisfy an abstract obligation. |
| `@todo` | Stable extraction ordering; class/member target records; records survive `@ignore`/`@native`; identical runtime output/bytecode with and without TODO. |

Every test lane should include a mutation check proving the harness notices a wrong expected result, following current decorator-fixture practice. [V: [native test strategy](../../spec/current/decorators/native.md), [ignore test strategy](../../spec/current/decorators/ignore.md)]

---

## 9. Expressibility versus value

| Attribute | Expressible with current decorator mechanisms? | Worth doing now? | Gate |
|---|---|---|---|
| `@native` | Partly built [V] | Yes, after anchor indexing, argument validation, and binding invariant land together. | Existing spec amendment + integration test. |
| `@todo` | Yes: Compile-tier metadata collection [O] | Yes, low semantic surface and improves explicit work tracking. | Define metadata/extraction owner. |
| `@unimplemented` | Not wholly: bodyless declaration grammar plus canonical error lowering needed [O] | Yes, if projects need callable placeholders; otherwise defer rather than add duplicate TODO syntax. | Parser/AST and defined error surface. |
| `@abstract` | No: registry alone cannot enforce inheritance or all construction paths [O] | Valuable, but only as a class-model feature. | PDR for finalization, inheritance obligations, and construction guard. |

**[O]** Recommended delivery order: first close `@native` correctness gaps; then `@todo`; then `@unimplemented` with bodyless declarations and error semantics; finally `@abstract` after its PDR. Do not ship `@abstract` as inert metadata to make the order look shorter.

---

## 10. What this design precludes

- **[O]** No second generic drop attribute. `@ignore` remains the only “this declaration is not code” meaning.
- **[O]** No use of `@native` as FFI installation, sealing, or dispatch protection. Bootstrap owns binding installation; reopening behavior is unrelated.
- **[O]** No selector-name-only matching for native bindings or abstract overrides. Full selector identity remains load-bearing.
- **[O]** No “abstract but instantiable” half-state. A class with unresolved obligations cannot become a normal instance through an alternate construction path.
- **[O]** No `@unimplemented` lowering to DNU, `nil`/absence, or a host panic.
- **[O]** No TODO-controlled compilation, network tracker integration, deadline semantics, or warning policy hidden in the language feature.
- **[O]** No user-defined Compile-tier substitute for these built-ins. Their effects depend on compiler-controlled method-table, class-finalization, and bootstrap boundaries.

---

## 11. Decisions required before normative specs

1. **[O]** Approve bodyless member declarations restricted to `@abstract` and `@unimplemented`, or choose a different explicit declaration syntax.
2. **[O]** Name/locate `AbstractInstantiationError`, `AbstractMethodError`, and `UnimplementedError` in the language error hierarchy.
3. **[O]** Define every construction route covered by the abstract-class gate, including reflection and any bootstrap allocation path.
4. **[O]** Choose the owner and stable output contract for TODO extraction.
5. **[O]** Amend `@native` with the exact-selector invariant, anchor retention rule, and central argument validation before adding the first real core anchor.

Until these are answered, this document recommends no code changes.

---

## 12. Provenance ledger

| Source | Use | First-hand? |
|---|---|---:|
| Graphify query/explain over `graphify-out/graph.json` | Located decorator surface, `@native` sections, attribute compiler relationships, and current source nodes without broad repository exploration. | Yes |
| [Decorators index](../../spec/current/decorators/index.md) | Built tiers, order, registry limits, and subtractive-attribute boundary. | Yes |
| [`@native`](../../spec/current/decorators/native.md) | Existing anchor semantics, legality, ordering, bootstrap invariant gap, hazards, and open questions. | Yes |
| [`@ignore`](../../spec/current/decorators/ignore.md) | Sole sanctioned drop semantics, legality-first rule, and collision precedent. | Yes |

No external source, implementation file, or unrelated specification was used. Recalled precedents in §2.3 are marked `[R]`.
