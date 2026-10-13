# Slice 1 Report — Architectural Contract for Phase-Aware Source Dependencies

## Scope

This slice audits the **fundamental architecture**, not individual checker hooks yet: how source imports can simultaneously support canonical semantic resolution, semantic cycles, strict runtime-cycle rejection, executable linked-read layouts, and runtime materialization.

The incident is correctly escalated. The live architecture currently conflates these concerns: ordinary imports create semantic/reference relationships **and** runtime edges/read slots before semantic analysis has a chance to determine whether runtime materialization is actually required. Pasted text

## 1. Primary ruling

**P4 cannot be implemented soundly by adding `DependencyPhase::InterfaceOnly` propagation to the existing linker.**

The required architectural change is a **two-stage linking model**:

```text
Source + interfaces
        │
        ▼
┌───────────────────────────────────┐
│ Semantic / Symbolic Link          │
│                                   │
│ - canonical import identities     │
│ - exports                         │
│ - semantic/interface graph        │
│ - NO executable init-order claim  │
│ - NO requirement for runtime slot │
└───────────────────────────────────┘
        │
        ▼
Canonical semantic analysis
        │
        ├── imported-binding use evidence
        ├── generated/materialization requirements
        └── conservative completeness status
        │
        ▼
┌───────────────────────────────────┐
│ Executable Link Finalization      │
│                                   │
│ - runtime dependency projection   │
│ - runtime linked-read layout      │
│ - runtime materialization needs   │
│ - strict DAG validation           │
│ - initialization order            │
└───────────────────────────────────┘
        │
        ▼
Lowering → materialization → execution
```

This is not merely the cleanest implementation. It follows directly from the ordering contradiction identified in the incident: strict `ModuleLinker::link_all` currently precedes semantic workspace analysis, while runtime-cycle validation occurs inside that link operation. Pasted text

A semantic-derived phase system therefore **cannot use the current `LinkedProgram` as both semantic input and executable output**.

---

# 2. Why the existing architecture is unsound for P4

The live linker currently does three logically independent things in one pass.

For each source import it:

1. resolves a source/reference relationship;
2. establishes semantic accessibility of the imported declaration;
3. creates an eager runtime dependency;
4. assigns a `LinkedReadSpec` / `ImportBindingId`;
5. eventually participates in initialization-order validation.

The incident explicitly identifies this coupling. Pasted text

That produces a circular requirement:

```text
Need semantic analysis
    ↓
to determine whether import is runtime
    ↓
but semantic analysis currently requires LinkedProgram
    ↓
whose construction requires runtime-cycle validation
    ↓
which assumes every import is runtime
```

For the exact case P4 is intended to support—an interface cycle that disappears from the runtime graph—the linker rejects the program **before the evidence that would make it legal can exist**.

Therefore:

> **Runtime DAG validation has to move from semantic-link construction to executable-link finalization.**

It must not be removed or relaxed.

---

# 3. The existing graph types are ahead of the execution architecture

There is an important positive finding.

`phalcom-modules::graph` already contains much of the *conceptual vocabulary* P4 needs:

```rust
enum DependencyPhase {
    InterfaceOnly,
    Runtime,
}
```

and distinct:

```text
ReferenceGraph
SemanticGraph
RuntimeDependencyGraph
```

This is architecturally appropriate.

The problem is that the source linker currently projects ordinary imports immediately into the runtime graph. `InterfaceOnly` exists as a representational possibility, but source linking has no authoritative evidence that permits it. That is exactly why the incident correctly says this is not an enum-value problem. Pasted text

### Consequence

Do **not** redesign the graph model first.

The missing architecture is primarily:

- staged link products;
- semantic evidence ownership;
- a stable import-binding identity;
- executable projection;
- enforcement that compiler reads cannot exceed the projected runtime dependency set.

---

# 4. Required staged types

I recommend making the stage distinction visible in Rust's type system.

Something conceptually equivalent to:

```rust
pub struct SemanticLinkedProgram {
    pub universe: Arc<ProjectUniverse>,
    pub modules: BTreeMap<ModuleId, SemanticLinkedModule>,
    pub graphs: SemanticLinkGraphs,
    pub entry: ModuleId,
}
```

with imports represented canonically but without executable slot assumptions:

```rust
pub struct SemanticImportedBinding {
    pub key: ImportBindingKey,
    pub target: SemanticImportTarget,
    pub symbol: Option<SymbolId>,
}
```

Then:

```rust
pub struct ExecutableLinkedProgram {
    pub semantic: Arc<SemanticLinkedProgram>,
    pub modules: BTreeMap<ModuleId, ExecutableLinkedModule>,
    pub runtime_graph: RuntimeDependencyGraph,
    pub initialization_order: Vec<ModuleId>,
}
```

where only the executable product contains:

```rust
pub struct ExecutableLinkedModule {
    pub linked_reads: Vec<LinkedReadSpec>,
    pub runtime_imports: ...,
    pub runtime_dependencies: Vec<ModuleId>,
}
```

The exact names are mechanically flexible. The important property is:

> **APIs capable of compilation, materialization, or execution must require the finalized type.**

A boolean such as:

```rust
validated: bool
```

would technically work but is considerably weaker. An explicit Rust type boundary makes accidental execution of provisional state structurally difficult.

This directly satisfies the incident's requirement that semantic analysis receive a provisional product while execution can receive only a finalized product. Pasted text

---

# 5. `DependencyPhase` alone is insufficient as semantic evidence

The current P4 draft proposes the lattice:

```text
InterfaceOnly < Runtime
```

That lattice is correct for the **final phase**.

It is not sufficient for the **proof state**.

There are at least three materially different situations:

```text
1. Proven interface-only
2. Proven runtime
3. Not completely classified / unsupported / failed analysis
```

Case 3 must finalize as runtime, but it must not be confused with case 2.

This matters especially for an import with **zero observed uses**.

A naïve implementation might reason:

```rust
let mut phase = InterfaceOnly;

for use_site in uses {
    phase = phase.join(classify(use_site));
}
```

An unused import then incorrectly becomes `InterfaceOnly` even though the architectural rule says absent or incomplete evidence must preserve the runtime dependency. The incident explicitly requires conservative retention when phase evidence is unknown or incomplete. Pasted text

The semantic product therefore needs a notion of completeness, for example:

```rust
pub struct ImportUseSummary {
    pub binding: ImportBindingKey,
    pub target: SymbolId,
    pub strongest_observed_phase: DependencyPhase,
    pub evidence: ImportEvidenceState,
}

pub enum ImportEvidenceState {
    Complete,
    Incomplete,
    Unsupported,
}
```

Finalization can then implement:

```rust
fn effective_phase(summary: &ImportUseSummary) -> DependencyPhase {
    match summary.evidence {
        ImportEvidenceState::Complete => summary.strongest_observed_phase,
        ImportEvidenceState::Incomplete |
        ImportEvidenceState::Unsupported => DependencyPhase::Runtime,
    }
}
```

There are other representations, but **proof completeness must exist somewhere**.

---

# 6. The real semantic question is “does this use require a runtime object?”

The implementation must not reduce phase classification to:

```text
type syntax  => InterfaceOnly
expression   => Runtime
```

That is too simplistic.

The incident already identifies the problem with `declaration_is_type_only`: it tells the checker something about the declaration's static domain, not whether a particular imported use requires runtime representation. Pasted text

The correct semantic question is:

> **Can compilation and execution of this importing module proceed without obtaining or materializing the target module/declaration as a runtime value?**

This means some apparently “type-ish” references may remain runtime.

Likely runtime cases include:

- whole-module import aliases;
- value expressions;
- callable references;
- runtime class objects;
- superclass materialization where class identity is required;
- generated declaration references;
- reflective/reified generic operations;
- re-exports under the current module semantics;
- any unsupported classification.

Potential interface-only cases include, subject to actual checker/lowering audit:

- erased type annotations;
- generic constraints used solely for checking;
- protocol requirements used solely for conformance checking;
- purely semantic declaration signatures;
- recursive ADT/type relationships that do not require runtime object acquisition.

This classification cannot be finalized from syntax alone.

---

# 7. The most important additional hazard: compiler-generated `GetLinked`

The incident correctly calls out generated declaration references. Pasted text

Inspection of `phalcom-core/src/compiler/lib/expr.rs` exposes an especially important risk.

`emit_declaration_class_reference` can produce:

```rust
Bytecode::GetLinked(...)
```

for a cross-module declaration even though the reference is not necessarily a direct source-level imported-value occurrence.

More importantly, one fallback path can dynamically append a `RuntimeLinkedRead::Binding` from a canonical Universe binding.

Architecturally, that means this invariant:

> every runtime read has a corresponding runtime dependency

cannot merely be checked by analyzing ordinary imported-name occurrences.

### Strong recommendation

For analyzed executable programs, lowering should **not independently discover new cross-module runtime dependencies**.

Instead:

```text
semantic analysis / lowering requirements
                ↓
     executable finalizer
                ↓
 complete runtime import/read layout
                ↓
             compiler
```

The compiler should consume a pre-authorized runtime binding:

```rust
RuntimeImportBindingId
```

or equivalent.

If compiler lowering discovers that it needs a runtime declaration but no finalized runtime mapping exists, that should be a compiler/link-contract error—not a reason to silently append a new runtime read.

Otherwise this failure is possible:

```text
A imports B::T
semantic classifier → InterfaceOnly
runtime graph       → A has no B dependency

later lowering:
    generated reference to B::T
    emits GetLinked(B::T)

runtime graph says B unnecessary
bytecode says B required
```

That is exactly the split-brain condition P4 must prevent.

---

# 8. Runtime edges and runtime slots must be one projection

This produces one of the most important P4 design rules:

> **Runtime dependency edges and executable linked-read slots must be generated together from the same finalized requirements.**

Not sequentially by different subsystems.

Bad architecture:

```text
semantic phases → runtime graph

compiler        → linked reads
```

Correct architecture:

```text
semantic/runtime requirements
          │
          ▼
 executable finalizer
       /       \
runtime graph   runtime read layout
       \       /
    same authority
```

Then:

```rust
ExecutableLinkedModule {
    linked_reads,
    runtime_dependencies,
}
```

is internally coherent by construction.

A structural validation should additionally assert:

```text
for every linked runtime read R:
    target_module(R) ∈ transitive/direct required runtime dependencies
```

depending on the exact initialization contract.

---

# 9. Canonical identity must remain independent of runtime projection

The incident's identity invariant is critical: an interface-only import must resolve to the **same canonical declaration**, not a parallel “type-only” declaration representation. Pasted text

Therefore semantic linking must preserve:

```text
imported local binding
    ↓
canonical SymbolId / DeclarationId
```

even when:

```text
runtime slot = None
```

Conceptually:

```rust
struct SemanticImportedBinding {
    key: ImportBindingKey,
    target: SymbolId,
}

struct FinalizedImportedBinding {
    semantic: SemanticImportedBinding,
    runtime: Option<ImportBindingId>,
}
```

This is much cleaner than overloading `ImportBindingId`, because `ImportBindingId` currently implies position in an executable linked-read array.

An interface-only semantic binding should have **no fake runtime slot**.

---

# 10. Crate ownership should remain acyclic

The P4 design must not make:

```text
phalcom-modules → phalcom-semantic
```

because semantic analysis already depends on module products.

A workable ownership direction is:

```text
phalcom-modules
    defines:
      ImportBindingKey
      DependencyPhase
      phase-summary input types
      SemanticLinkedProgram
      ExecutableLinkedProgram
      finalizer

phalcom-semantic
    consumes SemanticLinkedProgram
    produces ImportPhaseEvidence

phalcom-core
    orchestrates:
      semantic_link
      semantic_analysis
      executable_finalize
      compile/materialize
```

This preserves a sensible dependency topology:

```text
modules ← semantic
   ↑
   └──── core orchestration
```

The phase *fact* is semantically owned, while its transport representation can still live in the lower-level modules crate.

---

# 11. Whole-module imports and re-exports

I agree with the blocked P4 plan's conservative fixed rule:

```text
whole-module import → Runtime
direct re-export    → Runtime
```

for this increment.

This is important scope containment.

A whole-module alias explicitly introduces a module value into the importing namespace under current semantics. Making that interface-only would require a substantially different module-value model.

Re-export is more nuanced theoretically, but the current runtime export/materialization system makes it unsafe to classify away without a separate re-export architecture.

Therefore **P4 should only optimize selected imported declarations initially**.

That dramatically narrows the proof surface.

---

# 12. Semantic cycles vs runtime cycles

This distinction should become explicit in tests.

A legal future program may contain:

```text
A --interface--> B
B --interface--> A
```

because semantic/type relationships can be solved as an SCC/fixed point.

But this remains illegal:

```text
A --runtime--> B
B --runtime--> A
```

And a mixed cycle:

```text
A --interface--> B
B --runtime--> A
```

does **not** itself form a runtime cycle if the interface-only edge is genuinely absent from the runtime projection.

That is the capability P4 is trying to create.

The incident is therefore right to insist that strict runtime-cycle rejection remains unchanged rather than making runtime SCCs tolerant. Pasted text

---

# 13. Assessment of the four alternatives from the incident

The incident presents four possible directions. Pasted text

| Alternative | Assessment |
|---|---|
| Semantic-derived phase product | **Correct long-term architecture** |
| Authored `interface-only` syntax | Not required; shifts proof burden to users and creates language/spec work |
| Keep modules cohesive | Correct temporary containment, but does not solve the architectural deficiency |
| Different module representation | Possible locally, but risks avoiding rather than solving the source/runtime dependency problem |

I would therefore accept **Alternative 1**, while retaining Alternative 3 as the containment rule until Alternative 1 is implemented and proven.

I would **not** introduce `import type`, `interface import`, or similar source syntax for P4.

Telos can infer this more reliably from its semantic pipeline, provided the pipeline records complete evidence.

---

# 14. Revised architectural invariants

I would sharpen the P4 invariants to the following.

```text
INV-A — Semantic identity
Every source import resolves to one canonical declaration/module identity
independent of whether a runtime slot exists.

INV-B — Evidence completeness
An import may finalize InterfaceOnly only after semantic analysis has
successfully and completely classified all uses and generated requirements
relevant to that imported binding.

INV-C — Runtime dominance
Any runtime requirement upgrades the imported binding to Runtime.

INV-D — Conservative failure
Missing, stale, failed, unsupported, or ambiguous evidence finalizes Runtime.

INV-E — Single executable projection
Runtime graph edges, runtime import slots, declaration-materialization
requirements, and initialization order derive from one finalization operation.

INV-F — No compiler dependency discovery
Executable lowering may consume finalized runtime dependencies but must not
silently invent cross-module executable dependencies.

INV-G — Strict executable DAG
Only the finalized runtime graph determines initialization order, and every
cycle in that graph remains a hard executable-link failure.

INV-H — Stage isolation
Semantic/provisional linked state cannot be passed to executable lowering,
materialization, or initialization APIs.

INV-I — Incremental equivalence
Cold and incremental semantic analysis must produce the same finalized
runtime projection for the same program.
```

These are stronger than the current draft in two places: **evidence completeness** and **compiler dependency discovery**.

---

# 15. Slice 1 conclusion

The architecture incident is valid and P4 was correctly blocked. Pasted text

The architectural decision I would make is:

> **Adopt semantic-derived dependency phases, but only through an explicit two-stage linker architecture. Semantic linking establishes canonical identity without claiming executability. Semantic analysis produces complete per-import use/runtime evidence. A single executable finalizer then constructs runtime slots, runtime edges, validates the DAG, and produces the only program type accepted by lowering/materialization/execution.**

Two refinements are essential beyond the existing draft:

1. **Do not represent proof state with `DependencyPhase` alone.** There must be a distinction between *proven interface-only* and *incomplete/unknown evidence*.
2. **Do not allow compiler lowering to invent linked runtime reads after finalization.** Generated declaration/runtime requirements must enter the executable dependency projection before DAG validation.

Those two points are where I see the largest risk of a superficially passing but semantically unsound P4 implementation.

### Next slice

The next useful slice is the **semantic evidence-owner audit**: inspect `ImportSiteId`/`ImportSiteLocalId`, module binding catalogs, source-index target attachment, checker resolution paths, semantic dependency tracking, and incremental products to determine the exact stable key and recording hooks for `ImportPhaseEvidence`.