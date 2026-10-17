# Slice 3 Report — Import Phase Classification Completeness

## Scope

This slice audited the semantic and compiler paths that can consume an imported declaration or binding and classified them against the P4 lattice:

```rust
DependencyPhase::InterfaceOnly
DependencyPhase::Runtime
```

The central question was not merely “does this syntax look like a type or a value?” It was:

> **Can this use cause the generated executable module to read, materialize, initialize, dispatch through, or reify something owned by the imported module?**

That distinction matters because several syntactically type-oriented constructs ultimately generate runtime class-object reads.

Repository state remains the same: `feat/telos-collections` points at `da436a6c…`, whose parent is the incident code revision `e5988ad…`.

---

# 1. Slice ruling

The phase classifier cannot be a simple syntax-category mapping.

The correct model is:

```text
source semantic use
        │
        ▼
initial requirement
InterfaceOnly / Runtime
        │
        ▼
semantic elaboration
        │
        ├── generated runtime declaration use
        ├── runtime type reification
        ├── selected callable/witness
        ├── runtime class-object requirement
        └── runtime decorator/materialization
        │
        ▼
join all requirements
        │
        ▼
final import phase
```

So, for example:

```telos
class Box {
    value: External
}
```

can contribute an `InterfaceOnly` source use of `External`.

But:

```telos
External
```

as an expression contributes `Runtime`, despite going through the same type resolver.

And:

```telos
class Child is External
```

must be `Runtime`, even though superclass formation initially looks like a declaration/type relation.

This last case is already proven by current code generation.

---

# 2. Current module layer already contains the right lattice—but does not use it correctly

`phalcom-modules/src/graph.rs` already defines:

```rust
pub enum DependencyPhase {
    InterfaceOnly,
    Runtime,
}

impl DependencyPhase {
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Runtime, _) | (_, Self::Runtime) => Self::Runtime,
            _ => Self::InterfaceOnly,
        }
    }
}
```

It also already distinguishes semantic dependency kinds such as:

```text
TypeReference
Superclass
ProtocolReference
ConstraintReference
CallbackSignature
AdtReference
```

and runtime causes:

```rust
pub enum RuntimeDependencyReason {
    WholeModuleImport,
    SelectiveValueImport,
    ReExport,
    RuntimeDeclarationReference,
}
```

This is significant. **`RuntimeDeclarationReference` already exists specifically for the generated-runtime-use problem.**

However, I found no production code that creates a runtime edge with:

```rust
RuntimeDependencyReason::RuntimeDeclarationReference
```

The enum is present and fingerprinted, but apparently unused.

That is a concrete P4 hole rather than merely missing design.

---

# 3. Current selective imports are unconditionally Runtime

`phalcom-modules/src/linker.rs::collect_imports_and_graphs` currently does this for every selective import item:

```rust
self.graphs.runtime.add(RuntimeDependencyEdge {
    importer: module.clone(),
    dependency: target.clone(),
    range: item.range,
    reason: RuntimeDependencyReason::SelectiveValueImport,
});
```

It then creates:

```text
LinkedReadSpec::Binding(...)
ImportBindingId
```

for every imported name.

Likewise:

```rust
pub fn dependency_phase(kind: ReferenceKind) -> DependencyPhase {
    match kind {
        ReferenceKind::InterfaceOnly => DependencyPhase::InterfaceOnly,
        ReferenceKind::WholeModuleImport
        | ReferenceKind::SelectiveImport
        | ReferenceKind::ReExport => DependencyPhase::Runtime,
    }
}
```

So the existing `DependencyPhase` abstraction is ahead of the actual linker architecture. Selective imports are still being classified by **syntactic import kind**, not semantic use.

P4 must invert this:

```text
selective import
    ↓
symbolic import binding
    ↓
semantic evidence
    ↓
final phase
    ↓
only then:
    runtime edge?
    linked-read slot?
```

---

# 4. Classification matrix

The audit now supports the following P4 classification.

| Semantic use | Initial phase | Important qualification |
|---|---:|---|
| Parameter annotation | `InterfaceOnly` | Upgrade if separately reified/materialized at runtime |
| Return annotation | `InterfaceOnly` | Same |
| Field annotation | `InterfaceOnly` | Same |
| Type alias RHS | `InterfaceOnly` | Alias use may later produce runtime evidence |
| Generic parameter bound | `InterfaceOnly` | Runtime witness selection is separate evidence |
| `where` constraint | `InterfaceOnly` | Same |
| Trait/protocol reference in constraint | `InterfaceOnly` | Trait declaration itself has no runtime class |
| Conformance declaration's trait reference | `InterfaceOnly` | Runtime witness/impl owners are separate requirements |
| Associated type projection | `InterfaceOnly` | Until materialized as a runtime type-form value |
| Type-member projection | `InterfaceOnly` | Same |
| ADT/data payload annotation | `InterfaceOnly` initially | Runtime metadata/reification may upgrade |
| Callable signature type | `InterfaceOnly` initially | Runtime generic recipe may upgrade |
| Bare imported value read | **`Runtime`** | Direct runtime value |
| Imported callable invocation | **`Runtime`** | Includes unqualified callable path |
| Qualified module exported value | **`Runtime`** | Runtime module/value read |
| Bare imported class/type name as expression | **`Runtime`** | Materializes class object |
| `Expr::TypeForm` over imported nominal | **`Runtime`** | Materializes class object |
| Runtime callable reference | **`Runtime`** | Produces executable bound callable/family |
| Runtime associated invocation/value | **`Runtime`** | Executable operation |
| Imported superclass | **`Runtime`** | Required by `Bytecode::Class` |
| Runtime nominal pattern/class check | **`Runtime`** | Requires runtime class identity |
| Contextual realization requiring imported declaration receiver | **`Runtime`** | Generated declaration reference |
| Runtime decorator/attribute class | **`Runtime`** | Compiler synthesizes constructor call |
| Whole-module import | **`Runtime`** | Fixed policy; creates module value |
| Re-export | **`Runtime`** | Fixed P4 policy |
| Imported static constant used in runtime expression | **`Runtime`** | Runtime materialization/read |
| Imported static constant used only during static evaluation | **`Runtime` for P4** | Could later become compile-time-only optimization |
| Unused selective import | **`Runtime` for P4** | Conservative default |
| Unsupported/unclassified use | **`Runtime`** | Fail closed |

The crucial phrase is **initial phase**. An interface-only source occurrence can still end up Runtime if later semantic projection proves the executable representation requires the imported declaration.

---

# 5. Ordinary type formation is genuinely interface-only

`phalcom-semantic/src/types/annotation.rs` confirms that ordinary declaration type formation is semantic.

The relevant paths eventually perform operations equivalent to:

```rust
resolver.resolve_type_name(...)
resolver.resolve_trait_name(...)
```

and consume declaration forms, generic signatures, aliases, associated projections, constraints, etc.

These paths do not inherently request a runtime value.

That includes ordinary use in:

```text
parameter types
return types
field types
generic arguments
generic constraints
trait constraints
associated type projections
type aliases
ADT component declarations
```

These are legitimate `InterfaceOnly` evidence sources.

This is exactly the class of import P4 is trying to allow without introducing a runtime initialization edge.

---

# 6. Superclasses are the major counterexample

The previous slice warned that a resolver invocation cannot by itself determine phase.

The superclass path proves that concretely.

Semantically:

```telos
class Child is ExternalBase {
    ...
}
```

participates in hierarchy/type formation.

But `phalcom-core/src/compiler/lib/class_decl.rs` later emits:

```rust
if let Some(sc_ref) = class_def.superclass_ref() {
    let sc_sym = self.vm.interner.intern(sc_ref.leaf_name());
    self.emit_global_reference(sc_sym, sc_ref.range)?;
} else {
    ...
}

self.emit(Bytecode::Class(name_idx), range);
```

The comment immediately above explicitly states that the superclass is pushed as an ordinary runtime global so that `Bytecode::Class` can consume it.

Therefore:

```text
ImportedBase used as superclass
```

is unambiguously:

```text
RuntimeDeclarationReference
```

not merely:

```text
SemanticEdgeKind::Superclass
```

Both edges are required:

```text
semantic graph:
    Child declaration ──Superclass──> ExternalBase

runtime graph:
    child module ──RuntimeDeclarationReference──> base module
```

This is probably the clearest test case P4 should add first.

---

# 7. Bare imported type names can be runtime values

`phalcom-semantic/src/checker/expression.rs` has the `Expr::Var` path:

```rust
if let Some(fact) = ctx.lookup_local(value) {
    ...
} else if let Some(decl) = ctx.resolve_type_name(value) {
    ...
}
```

The second branch does **not** mean “type-only reference”.

It creates a class-object value with:

```text
SemanticDenotation::TypeForm(...)
```

That fact is later projected by:

```text
phalcom-core/src/modules/semantic_lowering.rs
    project_type_form_values(...)
```

into:

```rust
RuntimeTypeFormValueSpec::ClassObject {
    declaration
}
```

and finally `phalcom-core/src/compiler/lib/expr.rs` emits:

```rust
self.emit_global_reference(symbol, range)?;
```

So these are different:

```telos
fn f(x: External) { }
```

→ `External`: **InterfaceOnly**

```telos
fn f() {
    External
}
```

→ `External`: **Runtime**

The same declaration identity and the same underlying type resolver participate in both.

This confirms Slice 2's ruling that phase must be assigned by the semantic **consumer**, not the resolver.

---

# 8. `Expr::TypeForm` is also runtime

A second easily misclassified path is explicit type-form expression syntax.

`checker/expression.rs` resolves it semantically, but `semantic_lowering.rs` intentionally projects declaration-backed forms to:

```rust
RuntimeTypeFormValueSpec::ClassObject { declaration }
```

and `compiler/lib/expr.rs` then performs a runtime global read.

Therefore:

```text
type syntax used in a declaration annotation
    → InterfaceOnly

type syntax evaluated as an expression
    → Runtime
```

This distinction must exist explicitly in P4 evidence.

Source-index occurrence role alone cannot reliably infer it.

---

# 9. Imported ordinary value consumption is straightforwardly Runtime

The ordinary runtime paths include:

```text
Expr::Var
unqualified call
module_alias.member
module_alias.member(...)
callable/family values
associated runtime lookup/invoke
```

A subtle completeness issue remains from Slice 2:

`Expr::Var` records:

```rust
binding_read: Some(binding_id)
```

but `synthesize_unqualified_call` can begin directly with:

```rust
ctx.lookup_local(&call.name)
```

and invoke the resulting callable without necessarily creating an independent variable expression carrying `binding_read`.

Therefore:

> **P4 must instrument semantic value consumption, not post-process only `ExpressionAnalysis::binding_read`.**

The recommended body-local provenance map remains:

```rust
BTreeMap<BindingId, SourceImportBindingId>
```

and the consuming operation should record:

```text
SourceImportBindingId → Runtime
```

when the value is actually used.

---

# 10. Runtime associated operations and callable references also require Runtime evidence

The semantic checker already owns exact associated/callable-reference resolution.

`semantic_lowering.rs` then projects these into executable structures such as:

```text
AssociatedLoweringSpec
CallableReferenceLoweringSpec
ExecutableInvocationTarget
RuntimeCallEnvironmentRecipe
```

Accordingly, importing a declaration merely to write:

```telos
T.AssociatedType
```

in a type-forming position can be interface-only.

But constructing or invoking something such as an associated constructor, bound family or callable reference is executable and therefore Runtime whenever its imported binding/declaration must be materialized.

This is another reason not to classify from syntax alone.

---

# 11. Contextual realization produces hidden runtime declaration reads

There is a particularly important generated case in `phalcom-core/src/compiler/lib/expr.rs`:

```rust
fn emit_declaration_class_reference(
    &mut self,
    declaration: &DeclarationId,
    range: SourceRange,
) -> Result<(), CompilerError>
```

For an external declaration it searches imported linked bindings and can emit:

```rust
Bytecode::GetLinked(...)
```

The function is used by contextual algebraic realization when the selected receiver is:

```text
ContextualClassReceiver::Declaration(...)
```

So the semantic checker can select a capability against an imported nominal declaration and later lowering can require the class object **even though no ordinary imported variable read corresponds to that runtime operation**.

This should be recorded as something like:

```text
ImportUseReason::RuntimeDeclarationMaterialization
DependencyPhase::Runtime
RuntimeDependencyReason::RuntimeDeclarationReference
```

The best recording point is the semantic realization product, when the exact declaration receiver becomes known—not the bytecode compiler.

---

# 12. Runtime type reification is another generated upgrade path

`phalcom-core/src/modules/compile.rs` performs a substantial semantic-to-runtime metadata projection.

It constructs:

```text
runtime_root_specs
runtime_call_root_specs
runtime_trait_root_specs
```

and turns semantic type identities into:

```rust
RuntimeTypeRecipe::Closed(...)
RuntimeTypeRecipe::Template(...)
RuntimeTypeRef
```

These are consumed by runtime generic environments and reification.

This has an important consequence for P4:

> A source annotation being `InterfaceOnly` does not prove that its nominal declaration will never be required at runtime.

For example, an imported type may enter a runtime generic specialization or reification recipe after semantic checking.

So generated runtime type roots must participate in the same phase-evidence closure.

Conceptually:

```text
ImportedType
    │
    ├─ source annotation ──────────────> InterfaceOnly
    │
    └─ runtime reification recipe ────> Runtime
                                         │
                                         ▼
                                      final Runtime
```

The phase system must therefore be **proof closure**, not source scanning.

---

# 13. Trait references themselves can remain InterfaceOnly

The current trait architecture provides an unusually clean case.

Actual compiler handling of:

```rust
Statement::Trait(_)
```

is a no-op.

Traits are type-level semantic declarations rather than ordinary runtime classes.

Therefore:

```telos
from traits import Comparable

fn f<T>(x: T) where T: Comparable { ... }
```

does not, by itself, require the module defining `Comparable` to initialize at runtime.

That import can remain `InterfaceOnly`.

However, runtime use of the constraint can select:

```text
conformance evidence
witness callable
conditional implementation
runtime trait environment
```

Those executable products can introduce **other runtime dependencies**.

The dependency should be attributed to the selected executable owner rather than incorrectly upgrading the trait module merely because the trait name appeared in a constraint.

That distinction is important for precise dependency graphs.

---

# 14. Inherent `impl` does not currently introduce a cross-module import problem

I investigated this because an imported type targeted by:

```telos
impl Imported { ... }
```

would clearly create runtime installation concerns.

Current Telos semantics avoid that problem: `resolve_inherent_impl_target` enforces **same-module nominal targets**.

Furthermore:

```rust
Statement::Impl(_)
```

itself is a compile-time no-op in the compiler. Semantically accepted members are attached when the owning class/data/enum behavior object is generated.

Therefore P4 does not need to support cross-module imported inherent-impl targets.

It should preserve the same-module invariant rather than adding phase machinery for a currently illegal case.

---

# 15. Static constants expose a third conceptual phase that P4 should not add yet

`checker/static_body.rs` is quite sophisticated here.

A static expression:

- enters `EvaluationPhase::Static`;
- rejects reads from ordinary runtime bindings;
- permits static constants;
- records exact `SymbolId` reads;
- adds `SemanticDependency::StaticConstant(...)`;
- evaluates them into checked static products.

Architecturally, a remote static constant used only to evaluate another static constant might require the provider at **compile time but not runtime**.

That is neither a perfect semantic fit for:

```text
InterfaceOnly
```

nor genuinely:

```text
Runtime
```

A richer system might eventually use:

```text
InterfaceOnly
CompileTime
Runtime
```

or rename `InterfaceOnly` to a broader `NonRuntime`.

P4 should **not** widen scope this far.

For P4:

```text
imported static constant whose only proven use is static evaluation
    → Runtime conservatively
```

This sacrifices an optimization but preserves correctness and avoids implicitly changing the meaning of the existing two-point lattice.

It can be optimized in a later checkpoint once compile-time artifact dependencies have first-class representation.

---

# 16. Runtime decorators/attributes are another non-body hole

This audit also found an important compiler-generated runtime path in:

```text
phalcom-core/src/compiler/lib/class_decl.rs
```

For non-compiler-only attributes, `emit_attribute_attach` synthesizes:

```rust
Expr::MethodCall(Box::new(MethodCallExpr {
    object: Expr::Var {
        value: attr.name.clone(),
        ...
    },
    method: "new".to_string(),
    ...
}))
```

and compiles that synthetic expression.

In other words:

```telos
@ExternalAttribute(...)
class C { ... }
```

ultimately executes something equivalent to:

```telos
ExternalAttribute.new(...)
```

at runtime.

This is **Runtime**.

More importantly, this use does not naturally belong to an ordinary user-authored callable-body `CallableAnalysis`.

So P4 cannot limit evidence collection to callable bodies.

The semantic declaration/runtime-materialization layer must account for executable declaration syntax such as runtime attributes.

Compiler-only attributes remain non-runtime by definition.

---

# 17. Runtime patterns need the same treatment

`phalcom-core/src/compiler/lib/patterns.rs` contains paths that load class/global identities to perform runtime pattern checks.

Therefore an imported nominal declaration used by an executable pattern can also require:

```text
RuntimeDeclarationReference
```

even if the source occurrence looks structurally similar to a type reference.

P4 should not ask the bytecode compiler to infer this dependency itself.

Pattern semantic resolution should identify the canonical declaration; executable pattern projection should contribute Runtime evidence for that declaration.

---

# 18. Source-use classification and executable-requirement classification should be separate APIs

The cleanest model after this slice is no longer one generic:

```rust
record_import_use(...)
```

called everywhere.

I recommend two conceptual lanes:

```rust
record_source_import_use(
    binding: SourceImportBindingId,
    requirement: ImportUseRequirement,
    reason: SourceImportUseReason,
)
```

and:

```rust
record_runtime_declaration_requirement(
    declaration: DeclarationId,
    reason: RuntimeDeclarationUseReason,
)
```

The second operation then maps the canonical declaration back to an importing binding **only when that declaration was supplied through one**.

Why separate them?

Because generated runtime requirements often start from:

```text
DeclarationId
CallableId
ImplId
runtime type recipe
```

rather than a source name.

This avoids forcing semantic lowering to reconstruct source spelling merely to express a runtime dependency.

---

# 19. Runtime closure must include more than imported-binding evidence

This reveals a broader architectural requirement.

Suppose module `A` semantically selects a callable owned by module `B`, even if no direct imported source binding in `A` names that callable.

Then executable correctness requires:

```text
A →runtime B
```

regardless of source-import evidence.

So executable dependency finalization should combine:

```text
source import phase evidence
        +
runtime declaration requirements
        +
selected external callable owners
        +
selected conformance/witness owners
        +
runtime type/reification requirements
        +
fixed whole-module/re-export rules
```

The result—not the import statement alone—is the final runtime module graph.

This is a stronger and more correct formulation of P4.

---

# 20. Recommended evidence categories

P4 does not need dozens of lattice values. Keep the lattice binary and make the reasons expressive.

A suitable reason taxonomy is approximately:

```rust
pub enum ImportUseRequirement {
    InterfaceOnly,
    Runtime,
}

pub enum ImportUseReason {
    // Interface-only semantic formation
    TypeAnnotation,
    TypeAlias,
    GenericConstraint,
    TraitConstraint,
    AssociatedTypeProjection,
    AdtTypeReference,
    CallableSignature,

    // Direct runtime use
    ValueRead,
    CallableInvocation,
    ModuleValue,
    ClassObjectValue,
    ExecutableTypeForm,
    CallableReference,
    AssociatedRuntimeOperation,

    // Generated/runtime materialization
    SuperclassMaterialization,
    PatternTypeCheck,
    ContextualRealization,
    RuntimeTypeReification,
    RuntimeAttribute,
    SelectedExternalCallable,
    SelectedConformanceWitness,

    // Fixed/conservative
    ReExport,
    StaticConstantDependency,
    Unsupported,
}
```

The reason should remain evidence/debug information.

The lattice stays:

```text
InterfaceOnly < Runtime
```

---

# 21. The most important P4 verification invariant

Once executable finalization is implemented, add a mechanical audit invariant:

> **Every emitted external runtime read must have a corresponding Runtime-classified dependency authority.**

For example, any compiler path capable of emitting:

```text
GetLinked
external class/global resolution
external declaration materialization
runtime metadata nominal resolution
```

must be explainable by a retained semantic/runtime-requirement fact.

Conversely:

> An `InterfaceOnly` import must have no executable linked-read slot and no runtime dependency edge solely because the import exists.

That gives P4 a much stronger test than merely checking graph counts.

---

# 22. Highest-risk completeness holes found in Slice 3

| Priority | Hole | Consequence |
|---|---|---|
| **Critical** | `RuntimeDeclarationReference` exists but apparently has no producer | Superclasses and other generated declaration reads can be omitted from a semantic phase model |
| **Critical** | Superclass is semantically type-like but compiled as runtime class-object read | Naive type-reference classification produces broken executable graph |
| **Critical** | Runtime type-form values reuse `resolve_type_name` | Resolver-level `InterfaceOnly` recording is unsound |
| **High** | Runtime generic/reification recipes can introduce declaration requirements after source analysis | Source-only evidence is incomplete |
| **High** | Runtime attributes synthesize executable expressions outside normal callable-body analysis | Body-only instrumentation misses runtime imports |
| **High** | Contextual realization can call `emit_declaration_class_reference` | Generated external read may lack source binding read |
| **High** | Unqualified imported calls bypass simple `Expr::Var` `binding_read` pattern | Post-hoc binding-read scans are incomplete |
| **High** | Runtime patterns can load class identities | Type-looking pattern target can actually require runtime class |
| **Medium** | Static cross-module constants occupy an implicit compile-time-only category | Do not over-optimize in P4 |

---

# 23. Revised P4 phase algorithm

The classification stage should effectively compute:

```text
for each SourceImportBindingId:
    phase = InterfaceOnly

    join all explicit semantic source uses

    join all generated executable requirements
        that resolve back to this imported target

    if evidence incomplete:
        phase = Runtime

    if no evidence:
        phase = Runtime
```

Separately, runtime closure must collect executable owners not necessarily associated with an explicit local imported binding.

Then:

```text
Runtime module graph
    =
runtime import bindings
    ∪ whole-module imports
    ∪ re-exports
    ∪ runtime declaration requirements
    ∪ selected external callable owners
    ∪ selected runtime conformance/witness owners
    ∪ runtime reification owners
```

Only after this closure should the linker assign:

```text
ImportBindingId
linked_reads
runtime_dependencies
initialization_order
```

---

# 24. Slice 3 conclusion

Slice 2 established **who should own the evidence**.

Slice 3 establishes **what must count as evidence**.

The principal architectural result is:

> **`InterfaceOnly` is a property of a proven semantic use, not of an imported declaration category. Runtime is the join of direct value use plus every executable requirement generated by semantic elaboration and lowering.**

The largest correction to the preliminary model is superclass handling:

```text
superclass type formation
    ≠ InterfaceOnly

because

superclass declaration
    → runtime ClassObject read
    → Bytecode::Class
```

The existing `RuntimeDependencyReason::RuntimeDeclarationReference` is already the correct vocabulary; what is missing is the semantic producer and finalization pipeline.

The other major conclusion is that **callable-body import-use recording alone is insufficient**. P4 must cover declaration materialization, runtime type-form projection, reification, attributes, patterns, contextual realization, and selected executable owners.

## Next slice

The logical next slice is **Slice 4: executable finalization and runtime-graph construction**. It should trace `LinkedProgram → runtime_modules → ModuleLoweringSemantics → CompiledModule → ModuleMaterializationPlan`, determine exactly where provisional symbolic linking must split from executable linking, and design the invariant-preserving algorithm that removes `InterfaceOnly` bindings from `linked_reads` while still producing complete runtime closure, initialization order, fingerprints, and incremental reuse.