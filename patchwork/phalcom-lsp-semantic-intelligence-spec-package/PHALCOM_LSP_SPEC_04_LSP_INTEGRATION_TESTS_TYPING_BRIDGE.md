# Phalcom LSP Implementation Spec 4
## LSP Surface Integration, Protocol Correctness, Verification Matrix, and Future Typing Bridge

**Repository baseline:** `e2ec9e5fb6dc362786c9dd9593470feb47c91d94`  
**Depends on:** Specs 1–3  
**Primary scope:** `phalcom-lsp`, `tools/vsphalcom`, integration tests  
**Goal:** expose the consolidated semantic model consistently through standard LSP without unsupported capabilities, and lock the design for future formal typing.

---

# 1. Scope

This final slice wires the semantic core into:

- hover;
- completion;
- inlay hints;
- definition/references;
- semantic tokens;
- VS Code integration.

It also fixes the confirmed `inlayHint/resolve` protocol error and defines the regression suite.

---

# 2. Immediate protocol fix: `inlayHint/resolve`

Current baseline:

- `phalcom-lsp/src/backend.rs:900-1015`

advertises:

```rust
resolve_provider: Some(true)
```

No `inlay_hint_resolve` method exists.

Current inlay hints are fully populated and use `data: None`.

## Required change

Advertise:

```rust
resolve_provider: Some(false)
```

or omit the field.

Do **not** implement a useless resolver merely to match the accidental advertisement.

## Test

In `phalcom-lsp/tests/stage6_inlay_hints.rs`:

```rust
assert_ne!(
    init["result"]["capabilities"]["inlayHintProvider"]["resolveProvider"],
    json!(true)
);
```

Prefer exact expectation `false` if emitted.

This patch can land before the rest of Specs 1–3.

---

# 3. Hover integration

Replace current fallback chain with semantic occurrence dispatch.

Current route:

```text
keyword
class text heuristic
selector index
top-level binding + Phaldoc
```

Target route:

```text
occurrence_at(uri, offset)
  -> binding renderer
  -> class renderer
  -> member renderer
  -> field renderer
  -> operator/member renderer
  -> None
```

## Requirements

- exact `Hover::range` on every successful hover;
- no keyword/literal/symbol-literal hover;
- local/parameter hover in all nested bodies;
- method declarations/calls use exact selector span;
- optional Phaldoc appended after semantic signature/value info;
- unknown inferred value may render `?`, but a valid semantic declaration/reference can still hover.

---

# 4. Member hover content

Recommended display:

```markdown
`Savings.rate`

getter on `Savings`
returns: `Float`
```

For inferred source summary, optionally include confidence/provenance in a secondary section.

For a constructor:

```markdown
`Savings.new(_)`

constructor on `Savings class`
returns: `Savings`
```

For inherited call site, show declaration owner and receiver context separately if useful:

```text
method `toString` declared on Account
receiver: Savings
returns: String
```

Do not pretend the return shape is a formal type annotation.

Use wording such as:

```text
inferred value
```

until formal typing is implemented.

---

# 5. Binding hover content

Examples:

```text
let balance
inferred value: Int
```

```text
const selector
inferred value: Symbol
```

```text
parameter owner
inferred value: String
```

When formal type annotations arrive, planned rendering order:

```text
parameter owner: DeclaredType
inferred runtime value: ConcreteClass   // only if useful and non-redundant
```

Do not overwrite declared type metadata with `ValueShape`.

---

# 6. Completion integration

Keep current member-completion UX and incomplete-dot recovery.

Change semantic source:

- receiver shape from unified expression analyzer;
- visible names from scope graph;
- members from canonical side-aware dispatch surface.

When receiver is a union:

- preserve current coverage ranking;
- never merge instance/class sides accidentally.

For `super.`:

- list members starting above lexical class;
- exclude overridden current-class implementation as lookup target;
- preserve visibility rules.

For top-level/body completion without dot:

- include lexical bindings in correct scope;
- classes/imports;
- implicit-self members where legal.

---

# 7. Inlay hint integration

Current `inlay_hints.rs` already traverses member bodies, but it queries facts by bare name and uses imprecise `for` ranges.

Migrate to:

```text
binding declarations from ScopeGraph
BindingId -> inferred value at declaration/after initializer
```

Position hint at exact `declaration_range.end`.

Policies remain:

```text
off
stable
all
```

`suppressObvious` remains presentation policy.

Do not hide non-obvious constructor/getter/interpolation/operator results.

---

# 8. Definition and references

Use semantic occurrences when target identity is available.

### Binding

Definition = binding declaration range.

References = all occurrences with same `BindingId`.

This enables method-local rename/navigation later without global name collisions.

### Callable

Definition = resolved `CallableId` declaration name range.

References = resolved semantic callable occurrences where available.

Workspace selector index may continue to serve unresolved/open-world selector references, but receiver-resolved references should prefer semantic identity.

### Class

Definition = `ClassId` declaration.

Keep cross-file/module-qualified behavior.

---

# 9. Semantic tokens

After occurrence integration, refine token classes without replacing the lexer.

Add standard token kinds:

- `PARAMETER`;
- `PROPERTY` if not already present.

AST/semantic overrides:

```text
Binding parameter -> PARAMETER
local binding -> VARIABLE
field/property -> PROPERTY
call/member -> METHOD
class -> CLASS
operator -> OPERATOR
```

Keep keyword/literal coloring.

Do not treat hover suppression as a reason to remove syntax tokens.

---

# 10. Server semantic consistency rule

Every editor feature must obtain semantic identity/value from `SemanticDb`.

Forbidden patterns after migration:

- parsing a receiver fragment and then using a separate handcrafted inference table when a semantic expression query exists;
- looking up callable returns by a fabricated `CallableId`;
- scanning text to decide which local binding is under cursor;
- using side-blind member lookup;
- requiring Phaldoc before rendering a binding hover.

Recovery parsing for an incomplete dangling dot remains allowed because it serves syntactic editor recovery, not semantic identity.

---

# 11. Test fixture matrix

Create a new fixture group, e.g.:

```text
phalcom-lsp/tests/fixtures/semantic2/
```

or extend existing `semantic/`.

Prefer several focused fixtures over one huge file.

Recommended fixtures:

```text
hover_targets.ph
binding_scopes.ph
getter_and_super.ph
operators_and_interpolation.ph
subscripts.ph
constructors_and_fields.ph
callable_flow.ph
class_instance_collision.ph
closure_flow.ph
cross_file_base.ph
cross_file_use.ph
```

Use marker comments supported by existing test helpers when possible.

---

# 12. Hover test matrix

For each success case assert both content and returned range.

Success:

- class declaration/reference;
- method declaration;
- method call;
- getter declaration/call;
- setter declaration/call;
- parameter declaration/use;
- local declaration/read/write;
- const declaration/read;
- closure param/use;
- for binding/use;
- inherited member;
- `super` member;
- subscript;
- operator if operator hover is enabled.

Negative:

- every keyword category;
- string;
- number;
- boolean;
- symbol literal;
- punctuation;
- whitespace;
- method body whitespace;
- braces.

Critical regression:

Hovering arbitrary text inside:

```phalcom
foo(x) {
  let y = x
  y
}
```

must never return a hover whose range is `foo(x) { ... }`.

---

# 13. Inference consistency test matrix

For each expression case, validate at least two surfaces.

Example:

```text
binding inference == inlay hint
receiver inference == completion member set
callable summary == hover return rendering
```

Cases:

- direct constructor instance;
- inherited method result;
- getter;
- `super` getter;
- string interpolation;
- arithmetic;
- comparison;
- chained calls;
- subscript;
- method-return-through-local;
- parameter call-site inference;
- constructor field parameter.

This catches future divergence between consumers.

---

# 14. Protocol transport tests

Add/extend the in-process JSON-RPC harness.

Assert initialize advertises only implemented capabilities.

At minimum:

- `hoverProvider` true;
- `inlayHintProvider` present;
- `resolveProvider` not true;
- semantic token full capability correct;
- completion trigger `.`.

Add a generic future guard if practical:

```text
for each advertised resolveProvider=true capability
    test its resolve method exists
```

tower-lsp may make generic introspection awkward; explicit tests are sufficient.

---

# 15. VS Code E2E tests

`tools/vsphalcom/package.json` already defines:

```text
test
test:lsp:e2e
vsix
```

Add a regression around inlay hints:

1. start extension with built local `phalcom-lsp`;
2. open fixture;
3. wait for inlay hints;
4. ensure no user-facing "Method not found" error/output entry is produced;
5. verify hint appears.

Also verify targeted hover range through VS Code API if API exposure permits.

Run:

```bash
cargo test -p phalcom-lsp
cd tools/vsphalcom
npm test
npm run test:lsp:e2e
```

Do not rely solely on manual VS Code testing.

---

# 16. Performance gates

The semantic refactor adds richer per-file state.

Add simple regression measurements/tests where existing infrastructure permits:

- occurrence lookup should be logarithmic or bounded-linear over sorted local occurrences;
- scope lookup walks lexical parents, not whole workspace;
- affected edit should not rebuild unrelated modules;
- native contract lookup is table/map based;
- hover should not reparse the entire workspace.

A single open-file parse remains acceptable where existing recovery requires it.

---

# 17. Future formal typing bridge

The current typing specification explicitly keeps type metadata out of selector identity/dispatch.

The LSP must preserve that.

## Shared infrastructure

Future checker should reuse:

```text
ModuleId
ClassId
CallableId
ScopeId
BindingId
SemanticOccurrence
DispatchResolver
structured flow graph/state transitions
dependency graph
incremental generations
```

## Separate semantic domains

Do not merge:

```rust
ValueShape
```

with future:

```rust
TypeExpression / TypeDescriptor / TypeConstraint
```

Introduce later:

```rust
pub struct TypeEvidence {
    pub declared: Option<TypeId>,
    pub inferred: Option<TypeId>,
    pub source: TypeEvidenceSource,
}
```

alongside runtime value knowledge.

## Evidence precedence for editor presentation

When typing lands:

1. explicit source annotation is authoritative as declaration metadata;
2. checker-derived formal type is authoritative for type checking;
3. runtime `ValueShape` is supplemental editor knowledge;
4. conflict may become a checker diagnostic according to future checker mode.

Do not mutate runtime shape inference to force it to match annotations.

---

# 18. Recommended module boundaries for extraction later

Keep new analysis modules free from:

- `tower_lsp` types;
- VS Code concepts;
- VM heap objects.

Use `SourceRange`, semantic IDs, AST, and plain Rust structures.

When compiler/type checker becomes a second consumer, candidates for extraction are:

```text
scope
occurrence
dispatch
control_flow
analysis core
```

The LSP-specific database/query/rendering remains in `phalcom-lsp`.

Do not extract early solely to satisfy this spec.

---

# 19. Cleanup order

Only after all new tests pass:

1. remove keyword hover dispatch and unused keyword-doc code;
2. remove top-level-only binding hover helper;
3. remove bare-name `LocalFacts` API;
4. remove `collect_expression_environment`;
5. remove side-blind resolver from semantic inference;
6. remove separate `collect_call_sites_expr`;
7. remove separate dependency walker;
8. remove old `body_summary_value`/parallel summary evaluator;
9. remove duplicate selector-range collector logic made obsolete by occurrence index.

Do cleanup in small commits so regressions are attributable.

---

# 20. Full verification command set

From repository root:

```bash
cargo fmt --all -- --check
cargo test -p phalcom-ast
cargo test -p phalcom-native-surface
cargo test -p phalcom-lsp
cargo test --workspace
```

For VS Code:

```bash
cd tools/vsphalcom
npm ci
npm run lint
npm run compile
npm test
npm run test:lsp:e2e
```

If `npm ci` is inappropriate because lockfile state changed, use the repository's established install command; do not modify dependencies gratuitously.

---

# 21. Acceptance gate

The whole LSP improvement effort is complete when:

- no unsupported inlay resolve capability is advertised;
- exact hover target behavior is stable in nested code;
- no keyword/literal hover remains;
- all binding scopes/parameters are independently addressable;
- completion/inlay/hover agree on common facts;
- getters/operators/subscripts/super participate in inference;
- constructors/fields are semantically correct;
- dependency invalidation covers resolved non-method-call sends;
- future type system can reuse semantic identities/control flow without treating `ValueShape` as `Type`;
- Rust and VS Code E2E suites are green.
