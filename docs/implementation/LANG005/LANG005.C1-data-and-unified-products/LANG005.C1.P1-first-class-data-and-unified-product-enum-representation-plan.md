# LANG005.C1.P1 — First-Class `data` and Unified Product/Enum Representation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan checkpoint-by-checkpoint. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement first-class nominal immutable `data` declarations end-to-end and establish the shared, compact product-layout/storage substrate that `data` and general enum payloads use, while preserving existing enum/GADT behavior and leaving scalar replacement/interprocedural optimization to LANG005.C1.P2.

**Architecture:** `DeclarationId` and the canonical `TypeStore` remain semantic type authority; a new `DataInfo` semantic product owns logical data shape/components/constructor identity; a reusable `product` layout/storage subsystem owns physical materialization; a declaration-level runtime behavior `ClassId` remains compatible with Phalcom's nominal reflection/dispatch model without making instances `InstanceObject`s; exact applied data identity is carried by an interned runtime data descriptor; enums keep their existing `VariantId`/`RuntimeVariantId`/`CaseDiscriminant` authority while replacing `Box<[Value]>` payload storage with the shared product storage.

**Tech Stack:** Rust 2024 workspace; `phalcom-ast`; `phalcom-modules`; `phalcom-semantic`; `phalcom-type-meta`; `phalcom-core` compiler/bytecode/VM/GC; `phalcom-lsp`; existing semantic query/fingerprint infrastructure; existing runtime typing metadata/reification registry.

**Spec / governing decision:** PDR-0035, “`data` is a nominal transparent immutable value product,” accepted 2026-09-12. This plan is LANG005.C1.P1 only.

## Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch inspected: main
remote HEAD: 40910f65fed02edc03dbb1ada4a789da98451f1f
HEAD message: docs(stdl): update STDL002.C1 checkpoint path to C1-time
inspection date: 2026-09-12
```

The available repository tools expose remote GitHub state. They do **not** establish local working-tree cleanliness, local uncommitted changes, or a local branch checkout. An implementing agent must perform the drift check below against its actual checkout before editing.

Recent commits after the earlier LANG005 investigation are concurrency/documentation changes; no later data/product/type-system implementation was found at the inspected `main` HEAD.

## Global constraints

- [ ] Treat PDR-0035 semantic rulings as fixed during implementation. Repository drift may change mechanics, not those semantics.
- [ ] Do not implement `data` by expanding to `@data class`.
- [ ] Do not add `TypeData::Data`; named data uses existing `Nominal` / `Applied`.
- [ ] Do not represent data instances as `InstanceObject`.
- [ ] Do not add per-instance component labels.
- [ ] Do not add per-instance arrays of generic type arguments.
- [ ] Do not create one runtime `ClassObject` per applied data specialization.
- [ ] Do not make physical layout or backing `ObjRef` observable through ordinary reflection or `===`.
- [ ] Keep the hot `Value` representation exactly 16 bytes.
- [ ] Do not enlarge every `Object` arena slot by placing variable product payload inline in the `Object` enum.
- [ ] Preserve all current enum semantics: GADT result specialization, exact-case identity, variant-local generics, case behavior classes, root behavior/contracts, matching/refinement, associated constructor lowering, and the `NativeOption` special representation.
- [ ] Keep existing anonymous tuple/record storage unchanged in this plan. The new layout subsystem must be reusable by LANG005.C1.P3, but P1 does not migrate them.
- [ ] Do not implement scalar replacement, escape analysis, interprocedural aggregate elimination, nested-product flattening, specialized collection storage, or generic-code specialization here. Those are C1.P2/P3 consumers of the substrate.
- [ ] Do not silently weaken an unknown/error typing state to `Dynamic`.
- [ ] Do not change generic bound syntax or the existing `<:`, `:>`, `==` constraint implementation.
- [ ] Do not remove the legacy `DataExpander`/`derive_data`; removal is a later LANG005 migration. The new path must simply never call it.
- [ ] Do not change enum source syntax in this plan. Variants-only enum syntax and `impl` migration belong to LANG005.C2.
- [ ] Do not promise stable FFI offsets/layout.

## Architecture map for implementers

The identities below must remain distinct:

```text
DeclarationId
    stable semantic identity of `data Point`

TypeId: Nominal / Applied
    canonical compiler semantic type, e.g. Point or Point<Int>

DataComponentId
    stable logical component identity: declaration + declaration-order index

DataConstructorId
    stable primary product-constructor identity

ClassId
    runtime declaration/behavior identity used by ordinary dispatch and nominal
    reification; one behavior row per data declaration, NOT per specialization

RuntimeDataDescriptorId
    VM-local exact materialized data-type identity; distinguishes Point<Int>
    from Point<Float>, and Signal<Connected> from Signal<Disconnected>

ProductLayoutId
    VM-local physical layout identity; multiple exact data descriptors may share it

VariantId
    stable semantic enum-case identity

RuntimeVariantId
    VM-local enum-case descriptor identity

CaseDiscriminant
    physical dense sum tag

RuntimeTypeRef / stable type metadata
    runtime reflection/reification identity from the existing typing subsystem
```

Required correspondence:

```text
DataInfo.owner == DeclarationId used by TypeData::Nominal
RuntimeDataDescriptor.declaration == same declaration
RuntimeDataDescriptor.behavior_class == one runtime class/behavior row for declaration
RuntimeDataDescriptor.layout == a ProductLayoutId
```

Required non-equivalence:

```text
RuntimeDataDescriptorId != ClassId != ProductLayoutId
TypeId != RuntimeDataDescriptorId
VariantId != RuntimeVariantId != CaseDiscriminant
```

## Sources of truth

| Concern | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Data declaration identity | `DeclarationId` | `DataInfo`, nominal type, source index, lowering, metadata | declaration name string alone |
| Exact compiler type | canonical `TypeStore` `Nominal` / `Applied` | constructor result, component specialization, type metadata | runtime class name |
| Data logical shape | `DataInfo` / `DataComponentSemantic` | property typing, constructor typing, lowering projection | synthesized class fields/getters |
| Generic constraints | existing `GenericSignature` / canonical constraints | data constructor/type formation | data-specific bounds |
| Physical representation | `ProductLayout` / `ProductLayoutSpec` | materialization, projection, GC | `ClassObject.field_slots` |
| Exact materialized data identity | runtime data descriptor registry | `.class`, exact-type reflection, nullary singleton, value semantics | backing `ObjRef` |
| Enum case identity | existing `VariantId` / `RuntimeAdtRegistry` | construction, matching, behavior | product layout ID |
| Runtime semantic type reflection | existing runtime typing metadata/registry | exact data descriptor metadata | per-instance generic arrays |

## Current repository facts that constrain the patch

1. `phalcom-ast/src/ast.rs::Statement` has `Class`, `Enum`, and `TypeAlias` but no `Data`.
2. `phalcom-ast/src/token.rs` / `lexer.rs` recognize `class` and `enum`, not `data`.
3. `phalcom-ast/src/parser.rs::at_top_level_item_boundary` explicitly names current declaration keywords.
4. `phalcom-modules/src/interface.rs::InterfaceBuilder::build` explicitly publishes class/enum/type-alias/let declarations.
5. `phalcom-modules/src/declaration.rs::DeclarationKind` has `Class`, `Protocol`, `Adt`, `Alias`.
6. `phalcom-semantic/src/types/store.rs::TypeData` already has the correct `Nominal` / `Applied` forms.
7. `phalcom-semantic/src/checker/enum_declaration.rs::build_enum_semantics` is the nearest implementation template for generic declaration-local product components and constructor result templates.
8. `phalcom-semantic/src/enum_semantics.rs` publishes first-class variant fields/constructors rather than disguising them as ordinary class fields.
9. `phalcom-semantic/src/db/key.rs::QueryKey` has `EnumDeclaration`, and `db/product.rs` / `db/query.rs` carry the matching product/query pattern.
10. `phalcom-semantic/src/session.rs` explicitly builds/publishes enum semantic tables/products and is an incremental ownership seam.
11. `phalcom-core/src/product.rs` is already the internal product-construction boundary for anonymous tuples/records.
12. `phalcom-core/src/heap/instance.rs::InstanceObject` is unsuitable: it carries `ClassId + Box<[Value]>` and Nil-initializes slots.
13. `phalcom-core/src/heap/adt.rs::AdtCaseObject` currently carries `RuntimeVariantId + Box<[Value]>`.
14. `phalcom-core/src/heap/object.rs` deliberately boxes large variants to avoid increasing every slotmap arena slot.
15. `phalcom-core/src/heap/trace.rs::trace_object` is exhaustive and must explicitly classify any new object variant/edge.
16. `phalcom-core/src/value/repr.rs::Value` is exactly 16 bytes; `AdtSingleton` proves a zero-allocation VM-local descriptor key is viable.
17. `phalcom-core/src/value/mod.rs::Value::same_as` is representation-bit/handle identity; `Bytecode::Same` currently calls it directly.
18. `phalcom-core/src/typing/reify.rs::reify_type_form` maps nominal forms to existing `ClassObject`s and synthetic/applied forms to typing descriptors.
19. `phalcom-core/src/adt.rs` already separates semantic variant ID, runtime variant ID, physical discriminant, and representation strategy.
20. `phalcom-core/src/modules/semantic_lowering.rs` projects semantic enum products into executable lowering specs; compiler lowering should consume analogous data specs rather than rediscover AST/type facts.
21. `phalcom-core/src/compiler/attributes.rs::DataExpander` / `derive_data` is an ordinary-class derivation path and must remain isolated from first-class `data`.

## Tempting wrong fixes — explicitly forbidden

- Do not make `data` a class with `native_repr = true` and then store instances in `InstanceObject`; `native_repr` is useful for the declaration behavior row, not the value representation.
- Do not add generated class fields/getters to make `p.x` type-check. A component is a component, and getter-shaped syntax must resolve to component projection.
- Do not store a `ClassId` plus `Box<[Value]>` in a new `DataObject` and call the representation complete.
- Do not use `TypeId` as a durable/runtime identifier; it is store-local.
- Do not use `owner.name == "Point"` or other name-based special cases.
- Do not make `RuntimeDataDescriptorId` identical to `ProductLayoutId`; phantom specializations require different exact type IDs with the same layout.
- Do not copy component labels into `DataObject`.
- Do not change `Value::same_as` globally to recurse through the heap; internal code uses representation identity. Add a VM-aware language-level exact comparison seam.
- Do not encode runtime data singleton identity in the currently reserved `Value` meta bits without a separate reason. A new explicit tag is easier to audit and preserves reserved-bit invariants.
- Do not migrate tuple/record storage now merely because `product.rs` is being generalized.
- Do not remove `Object::AdtCase` unless evidence shows a semantic/runtime simplification is safe. Sharing `ProductStorage` does not require sharing heap object variants.
- Do not alter the `NativeOption` fast path to force it through general product storage.
- Do not implement an LSP-only understanding of data components. LSP consumes semantic/source-index products.
- Do not make the runtime metadata serializer a second type checker.
- Do not solve unresolved generic runtime reification by attaching `Box<[RuntimeTypeRef]>` to every data value.
- Do not claim C1.P2 optimizations (scalar replacement, escape analysis) are complete because P1 uses compact materialized storage.

## Drift protocol before every checkpoint

- [ ] Verify the current checkout's `HEAD`.
- [ ] Confirm every Primary working-set file still exists.
- [ ] Confirm the named symbols still own the responsibilities described below.
- [ ] Search for new consumers when changing a public enum/type/query variant.
- [ ] Review effects of completed earlier checkpoints.
- [ ] Adapt mechanical call signatures if repository evolution requires it.
- [ ] Escalate rather than changing semantic design if a source-of-truth assumption is contradicted.

If the checkout has materially moved beyond `40910f65fed02edc03dbb1ada4a789da98451f1f`, record the delta in the implementation state before continuing.

---

# Checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–3 | `data` is a real parsed/bound declaration with its own AST/module kind; no runtime semantics yet | AST parser regressions; module binding/import/export identity; keyword/LSP lexer coverage | semantic type construction, runtime |
| C1 | 4–8 | one canonical `DataInfo` product owns logical shape, generics, components and constructor; cold/incremental publication agrees | semantic declaration/constructor/component tests; hostile nominal-vs-structural and phantom cases; cold/incremental product equivalence | physical layout/runtime |
| C2 | 9–13 | `ProductLayout` is the sole physical authority for materialized named products; data values have compact storage, exact descriptor identity, precise tracing and representation-independent value relations | layout/storage unit tests; GC tests; zero-allocation nullary singleton; `Value` size assertion; exact/structural value semantics | compiler source construction, enum migration |
| C3 | 14–18 | compiler/VM can declare, construct and project first-class data solely from semantic lowering specs; materialized values use C2 representation | end-to-end data programs; generic/phantom constructor typing; component immutability; `.class`/basic type metadata; allocation checks | advanced scalar replacement, unresolved-runtime-generic reification stress |
| C4 | 19–22 | general enum payloads use shared product storage without changing existing enum/GADT/case behavior; `NativeOption` remains special | existing ADT/GADT suites + new packed-payload/GC cases; negative behavior/match regressions | variants-only syntax and `impl` migration |
| C5 | 23–25 | incremental/artifact/source tooling and compatibility boundaries are closed; P1 has delivery/performance evidence | cold/incremental data equivalence; source-index/LSP smoke; negative searches; focused perf/allocation assertions; affected-crate suites | P2 optimizer, P3 anonymous product convergence/dynamic-reification qualification, workspace-wide future program gates |

---

## Checkpoint C0 — First-class source and declaration identity

Tasks:
- Task 1 — Add `data` token and AST declaration shape.
- Task 2 — Parse data declarations and record-shaped construction syntax.
- Task 3 — Publish data declarations through modules and lexical tooling.

Why this is a checkpoint:

Parsing alone is not meaningful evidence for a nominal declaration. C0 is complete only when the AST can represent both data shapes, the module interface binds the declaration exactly once under a `Data` kind, imports/exports see the same canonical declaration, and editor lexical classification recognizes the new keyword. No semantic type product is introduced yet, which keeps syntax/binding failures isolated from type-system work.

Entry conditions:
- PDR-0035 is accepted.
- Existing class/enum/type-alias parsing and module interface tests pass before the patch.
- `DeclarationId` remains the canonical module-owned declaration identity.

Working set:

Primary:
- `phalcom-ast/src/token.rs` — token enum.
- `phalcom-ast/src/lexer.rs` — keyword recognition.
- `phalcom-ast/src/ast.rs` — `Statement`, declaration/component syntax structs, expression variant for record-shaped named construction if required.
- `phalcom-ast/src/parser.rs` — top-level declaration dispatch, recovery boundary, data grammar.
- `phalcom-ast/src/error.rs` — only if a data-specific parse diagnostic is needed.
- `phalcom-ast/tests/data_syntax.rs` — new focused syntax regression file.
- `phalcom-modules/src/declaration.rs` — `DeclarationKind`.
- `phalcom-modules/src/interface.rs` — `InterfaceBuilder::build`.
- `phalcom-semantic/src/semantic_shard.rs` — source declaration-shell projection.
- `phalcom-lsp/src/semantic_tokens.rs` — raw keyword classification.

Secondary — inspect only if evidence requires it:
- `phalcom-ast/tests/parser.rs` — shared parser harness.
- module linker tests that assert class/enum declaration bindings.
- `phalcom-semantic/src/source_index/*` — declaration occurrence work belongs mainly to C1/C5, not syntax.

Out of scope for this checkpoint:
- `DataInfo`.
- constructor inference.
- component property typing.
- runtime class/data object allocation.
- product layouts.
- enum representation.

Semantic contract established by this checkpoint:
- Every `data Name...` declaration receives the same canonical `DeclarationId(module, Name)` that imports/exports and later semantic products use.
- `DeclarationKind::Data` is explicit; data is not reported as `Class` or `Adt`.
- Tuple-shaped and record-shaped logical component order is preserved exactly in the AST.
- No behavior members or mutable field declarations can appear inside a data declaration.
- `data` is a reserved language keyword.
- The legacy `@data` attribute remains independently parseable on classes.

Semantic risks:
- accidentally reusing `ClassDef` and losing the semantic category;
- contextual parsing that changes ordinary call/block syntax;
- record-shape source order being canonicalized too early;
- tuple positional/labeled lanes losing labels;
- module interface treating `data` as a global value rather than a declaration;
- `@data` accidentally becoming ambiguous with the new keyword.

Hostile cases:
- a class and data declaration with the same name in one module must trigger the existing duplicate declaration/binding rule rather than coexist;
- imported `Point` and local `Point` must follow ordinary binding shadow/duplicate rules, not a data-specific fallback;
- `@data class Legacy { ... }` remains the legacy decorator path;
- `data Bad { method() { ... } }` must not be accepted as a behavior-bearing declaration;
- rest/default parameter syntax that is legal for methods must not silently become data component syntax unless explicitly ratified.

Required evidence:
1. `cargo test -p phalcom-ast --test data_syntax` — proves both declaration shapes, generic/where headers, nullary tuple shape, rejected behavior/member syntax, and the named-record construction parse form.
2. `cargo test -p phalcom-modules` with new/extended binding tests — proves `DeclarationKind::Data`, canonical declaration identity, duplicate handling, and import/export publication.
3. `cargo test -p phalcom-lsp semantic_tokens` or the repository's existing semantic-token filter command — proves `data` is tokenized as a keyword; it does **not** prove semantic navigation.
4. Negative search: `rg 'Statement::Data' phalcom-core/src/compiler phalcom-semantic/src/checker` is expected to find only explicit exhaustiveness stubs/errors introduced for future checkpoints, not a class-expansion implementation.

Do not run yet:
- `cargo test -p phalcom-semantic` — no data semantic product exists yet.
- `cargo test -p phalcom-core` — runtime support does not exist yet.
- workspace tests/clippy — deferred to final P1 gate.

Escalate immediately if:
- parser support requires changing the meaning of ordinary `Foo(...)` calls rather than merely resolving them semantically later;
- `DeclarationId` cannot represent data without a new parallel declaration identity;
- module import/export logic branches on only Class/Adt in a way that would require data-specific linker semantics rather than extending `DeclarationKind`;
- the record construction spelling conflicts irreducibly with an existing grammar form.

Checkpoint completion:
- [ ] all tasks implemented
- [ ] required evidence passes
- [ ] hostile cases pass
- [ ] declaration kind is canonical across AST/modules/shard
- [ ] implementation state updated
- [ ] no active incident remains

Suggested commit grouping:
- `feat(ast): add first-class data declaration syntax`
- `feat(modules): bind data as a canonical declaration kind`
- `test(lang): pin data syntax and declaration binding`

### Task 1 — Add `data` token and AST declaration shape

Purpose:
Represent the ratified language construct directly in the AST without reusing `ClassDef`, `EnumDef`, or legacy `BuiltinAttr::Data`.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-ast/src/token.rs` — `Token`.
- `phalcom-ast/src/lexer.rs` — keyword-to-token mapping.
- `phalcom-ast/src/ast.rs` — `Statement`, new `DataDef`, data shape/component syntax.
- exhaustive `Statement`/`Token` matches reported by `cargo check -p phalcom-ast`.

Inspect before editing:
- `ClassDef`, `EnumDef`, `VariantPayloadSyntax`, `ParameterDef`, `TypeAnnotation`.
- `Expr::RecordLiteral` and `RecordLiteralExpr` only as syntax precedent.
- `BuiltinAttr::Data` to ensure it remains an attribute token, not reused.

Do not inspect unless evidence forces expansion:
- VM/runtime.
- trait/protocol AST.
- legacy data derivation builders.

Dependencies:
- PDR-0035 syntax/semantics.
- existing generic parameter and `WhereClauseSyntax`.

Source of truth:
- `DataDef` is source shape.
- later semantic identity remains `DeclarationId`, not an AST-local ID.

Implementation boundary:

Changes:
- Add `Token::Data` and map `"data"` in the lexer.
- Add `Statement::Data(DataDef)`.
- Add dedicated source structs for declaration shape/components.
- Preserve generic parameters and existing `where` syntax.
- Preserve tuple component external-label/local-name distinction.
- Preserve record component declaration order.

Must not:
- embed runtime layout fields in AST;
- synthesize class members;
- create an AST inheritance/superclass slot;
- add methods/fields to the body model.

Current implementation:
`Statement` directly owns `ClassDef` and `EnumDef`; enum payload syntax reuses parameter-like structures but carries enum-specific variant semantics. There is no first-class data declaration.

Target implementation:

STRUCTURAL:

```rust
pub struct DataDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub shape: DataShapeSyntax,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}

pub enum DataShapeSyntax {
    Tuple {
        components: Vec<DataComponentSyntax>,
        range: SourceRange,
    },
    Record {
        components: Vec<DataComponentSyntax>,
        range: SourceRange,
    },
}

pub struct DataComponentSyntax {
    pub local_name: String,
    pub external_label: Option<String>,
    pub annotation: TypeAnnotation,
    pub name_range: SourceRange,
    pub range: SourceRange,
}
```

The exact field names may follow repository conventions, but the information content is required. Do not use `ClassFieldDef`: it carries class storage/mutability semantics that a data component does not own.

Edit operations:
1. [ ] OPEN `phalcom-ast/src/token.rs`; add `Data` beside declaration keywords with documentation.
2. [ ] OPEN `phalcom-ast/src/lexer.rs`; map exact keyword `"data"` to `Token::Data`.
3. [ ] OPEN `phalcom-ast/src/ast.rs`; add `Statement::Data`.
4. [ ] ADD the dedicated data declaration/shape/component syntax structs near `ClassDef`/`EnumDef`.
5. [ ] UPDATE `Statement::range` / source-range helpers and every exhaustive AST match surfaced by compilation.
6. [ ] SEARCH `Token::Class | Token::Enum` style keyword lists and add `Token::Data` only where they mean “declaration keyword”.
7. [ ] DO NOT alter `BuiltinAttr::Data`.
8. [ ] CLEAN imports/docs.

Testing classification:
- No standalone behavioral test until Task 2 completes parser support; validated by checkpoint C0.

Optional compile checkpoint:
`cargo check -p phalcom-ast`

Reason: exhaustive enum match errors cheaply enumerate mechanical caller fanout before parser edits continue.

Checkpoint state update:
Record:
- the final AST struct names;
- whether tuple components reuse any existing parser helper but remain dedicated AST semantics;
- any exhaustive downstream matches intentionally left as temporary “not lowered” branches.

### Task 2 — Parse data declarations and record-shaped construction syntax

Purpose:
Implement the ratified source grammar while reusing existing generic/where/type-annotation parsing and preserving ordinary call parsing for tuple-shaped constructor invocation.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-ast/src/parser.rs` — top-level statement dispatch, `at_top_level_item_boundary`, new `parse_data`.
- `phalcom-ast/src/error.rs` — only dedicated diagnostics actually required.
- `phalcom-ast/tests/data_syntax.rs`.

Inspect before editing:
- `parse_enum`.
- `parse_enum_variant`.
- class generic header / `where` parsing.
- parameter parser used for positional/labeled lanes.
- `parse_record_literal`.
- primary/postfix expression parser around `Expr::UnqualifiedCall`.

Dependencies:
- Task 1 AST.

Source of truth:
- source token stream → `DataDef`; semantic resolution of a call target is **not** parser authority.

Implementation boundary:

Changes:
- Parse tuple declaration shape after the name/generic header with `(...)`.
- Parse record declaration shape with `{ name: Type ... }`.
- Require explicit component type annotations.
- For tuple shape, reject rest parameters, default values, and method-only parameter forms.
- Canonical nullary declaration is `data Name()`.
- Parse `Name { label: expr, ... }` as a nominal record-construction expression without deciding whether `Name` is actually data; semantic analysis decides that.
- Leave `Name(...)` as the existing ordinary invocation AST unless current parser architecture requires a neutral “nominal construct” AST. The semantic call resolver will recognize `DataConstructorId`.

Must not:
- look up declarations from the parser;
- transform record construction to `#{...}`;
- transform tuple construction to `.new`;
- accept method bodies in a data declaration.

Current implementation:
Top-level recovery recognizes `Token::Class | Token::Enum | Token::TypeKw | ...`; enums have a dedicated parser; `#{...}` is the structural record literal; ordinary calls are expression syntax.

Target implementation:
The parser knows only grammatical shape. In particular:

```text
Point(x: 1, y: 2)
    ordinary call-shaped AST
    semantic layer later identifies Point's DataConstructorId

Person { name: "Altun", age: 24 }
    dedicated nominal-record-construction AST
    semantic layer validates Person is record-shaped data
```

If the current postfix parser cannot introduce the record form without an AST target carrying a `StaticSymbolRef`, add the smallest neutral expression node necessary. Do not generalize it into class construction.

Edit operations:
1. [ ] OPEN `Parser` top-level statement dispatch and add `Token::Data => parse_data`.
2. [ ] ADD `Token::Data` to `at_top_level_item_boundary`.
3. [ ] IMPLEMENT `parse_data` by reusing generic parameter / where-clause / type annotation helpers.
4. [ ] ADD a data-component parser that accepts tuple lane labels/local names but rejects rest/default semantics.
5. [ ] PARSE record components as `identifier : TypeAnnotation`; reject behavior syntax.
6. [ ] ADD nominal record-construction postfix/primary parsing at the narrowest expression grammar point.
7. [ ] ADD range/name-range capture matching `ClassDef`/`EnumDef`.
8. [ ] ADD `phalcom-ast/tests/data_syntax.rs` with:
   - `data Pair<A,B>(_ first:A, _ second:B)`;
   - `data Point<T>(x:T,y:T) where ...`;
   - `data Person { name:String age:Int }`;
   - `data Signal<State>()`;
   - `Pair(1, 2)`;
   - `Person { name: "A", age: 1 }`;
   - rejection of method body;
   - rejection of rest component;
   - rejection of component without type;
   - legacy `@data class` still parses.
9. [ ] UPDATE parser recovery tests so a malformed data declaration recovers at the next top-level declaration.
10. [ ] SEARCH all `Statement` exhaustive matches and leave only deliberate future semantic/compiler branches.

Code instructions:
STRUCTURAL. Reuse parser helpers; do not paste enum/class parsers wholesale because their bodies own different semantics.

Testing classification:
- Focused regression required now because grammar is independently meaningful and a malformed grammar would make all later checkpoints noisy.

Checkpoint state update:
Record:
- exact AST node used for `Person { ... }`;
- whether tuple declaration components share a low-level parameter parsing helper;
- exact diagnostic names introduced.

### Task 3 — Publish data through modules, source shells, and lexical tooling

Purpose:
Make `data` a real declaration visible to linker/import/export infrastructure and the semantic declaration-shell graph.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate

Owned files and symbols:
- `phalcom-modules/src/declaration.rs::DeclarationKind`.
- `phalcom-modules/src/interface.rs::InterfaceBuilder::build`.
- `phalcom-semantic/src/semantic_shard.rs::ModuleSemanticStructureShard::from_source`.
- `phalcom-lsp/src/semantic_tokens.rs` keyword classifier.
- module declaration/interface tests.

Inspect before editing:
- class/enum declaration insertion in `InterfaceBuilder::build`.
- duplicate declaration diagnostics.
- `DeclarationBlueprint`.
- `ModuleSemanticStructureShard` class/enum/type-alias projection.

Dependencies:
- Task 2.

Source of truth:
- `DeclarationId(module, name)` plus explicit `DeclarationKind::Data`.

Implementation boundary:

Changes:
- Add `DeclarationKind::Data`.
- Insert data declarations in interface pass 1 exactly like other named declarations.
- Predeclare data shells in semantic structure shard.
- Give data a declaration header fingerprint containing generic header, where clause, and complete logical component signature/order.
- Data has no hierarchy edge.
- Mark `Token::Data` keyword in LSP lexical tokens.

Must not:
- map data to `DeclarationKind::Class`;
- add a superclass;
- expose components as module bindings;
- build data semantic types here.

Edit operations:
1. [ ] ADD `Data` to `DeclarationKind`.
2. [ ] UPDATE all exhaustive `DeclarationKind` matches, selecting behavior based on actual responsibility rather than aliasing Data to Class.
3. [ ] EXTEND `InterfaceBuilder::build` statement pass with `Statement::Data`.
4. [ ] EXTEND semantic shard declaration discovery.
5. [ ] ADD data structural/header fingerprinting that changes when component order/type/labels/generic constraints change.
6. [ ] CONFIRM no hierarchy fingerprint is fabricated.
7. [ ] ADD/extend module tests for local binding, selective export/import, duplicate class/data name.
8. [ ] ADD `Token::Data` to LSP keyword token classification.
9. [ ] SEARCH `DeclarationKind::` matches across workspace and update exhaustively.
10. [ ] RUN the C0 evidence set.

Testing classification:
- Validated at checkpoint C0.

---

## Checkpoint C1 — Canonical data semantic product and incremental publication

Tasks:
- Task 4 — Add stable data component/constructor identities and semantic products.
- Task 5 — Build generic data declaration semantics from the canonical type system.
- Task 6 — Add data query/product/fingerprint/session/snapshot publication.
- Task 7 — Type data construction and component projection without fake fields.
- Task 8 — Publish component/source occurrences from semantic identity.

Why this is a checkpoint:

`DataInfo` is useful only when declaration construction, generic specialization, query publication, expression typing, and source identity agree. Splitting tests earlier would mostly prove Rust plumbing. C1's evidence boundary is the canonical semantic claim: a data declaration has one nominal type identity and one logical product shape, and every consumer reads that product rather than reconstructing it.

Entry conditions:
- C0 COMPLETE.
- `DeclarationId`/`TypeStore` baseline unchanged.
- existing enum/generic inference tests still pass.

Working set:

Primary:
- `phalcom-semantic/src/identity.rs`.
- new `phalcom-semantic/src/data_semantics.rs`.
- `phalcom-semantic/src/checker/enum_declaration.rs` as pattern, plus new data declaration builder module.
- `phalcom-semantic/src/checker/context.rs`.
- `phalcom-semantic/src/checker/expression.rs`.
- `phalcom-semantic/src/db/key.rs`.
- `phalcom-semantic/src/db/product.rs`.
- `phalcom-semantic/src/db/query.rs`.
- `phalcom-semantic/src/db/fingerprint.rs`.
- `phalcom-semantic/src/db/mod.rs`.
- `phalcom-semantic/src/session.rs`.
- `phalcom-semantic/src/snapshot.rs`.
- `phalcom-semantic/src/semantic_shard.rs`.
- `phalcom-semantic/src/source_index/*`.
- new `phalcom-semantic/tests/semantic/data/*`.

Secondary — inspect only if evidence requires it:
- generic callable inference/application helpers.
- `declaration_type.rs`.
- semantic export/type metadata if source component information must be retained now.
- LSP adapters (do not encode semantics there).

Out of scope for this checkpoint:
- runtime ProductLayout.
- bytecode.
- heap representation.
- enum payload changes.

Semantic contract established by this checkpoint:
- `DataInfo` is the single logical-shape authority.
- `DataComponentId` is declaration + index and survives source-name changes only according to normal declaration revision semantics.
- named data types use ordinary nominal/applied `TypeId`.
- generic parameters/constraints are the existing canonical binder machinery.
- constructor application produces the declared nominal/applied result type.
- component projection specializes the declared component type through receiver type arguments.
- component assignment is rejected as immutable.
- data is nominally distinct from an anonymous tuple/record with identical components.
- phantom generic arguments remain type identity.
- cold and incremental analysis publish equivalent `DataInfo`.

Semantic risks:
- accidentally making components `FieldId`s and inheriting mutable field flow rules;
- data-specific generic inference diverging from canonical inference;
- storing raw `TypeId`s in durable artifacts rather than snapshot-local products;
- incremental invalidation missing component type/order changes;
- constructor call resolution treating data as a normal method family;
- record construction label order incorrectly defining record semantic equality instead of declaration component identity;
- imported data declaration resolving to a second local identity.

Hostile cases:
- `data UserId(_ raw:Int)` is not assignable to structurally identical `data OrderId(_ raw:Int)` merely by shape.
- `data Pair<T>(_ x:T)` and anonymous `(Int)` are not the same type.
- `data Id<K>(_ raw:Int)` yields distinct `Id<User>` and `Id<Order>`.
- a component named like an inherited Object method resolves as a data component on a data receiver, not a fabricated mutable field.
- assigning `p.x = ...` is rejected even if a future method with setter-shaped selector exists elsewhere.
- editing only a component type invalidates `DataDeclaration`/dependent expression products; editing an unrelated module does not.

Required evidence:
1. New semantic tests `data_declaration_publishes_nominal_components`, `data_constructor_specializes_generic_result`, `data_phantom_applications_remain_distinct`, `data_component_projection_specializes_receiver`, `data_component_assignment_is_rejected`.
2. Cold/incremental test `data_declaration_cold_incremental_equivalence` comparing declaration ID, component IDs/types, and result type forms.
3. `cargo test -p phalcom-semantic data_` — focused semantic suite.
4. `cargo check -p phalcom-semantic` — proves exhaustive query/product/caller migration compiles.
5. Negative search: `rg 'TypeData::Data|FieldId.*DataComponent|DataComponent.*FieldId' phalcom-semantic/src` should have zero production hits.

Do not run yet:
- core runtime tests.
- full workspace.
- LSP integration suite beyond source-index unit evidence.

Escalate immediately if:
- generic data formation appears to need a new constraint solver;
- component access cannot be represented without mutating `DeclarationSurface` into a storage authority;
- `TypeStore::Nominal` cannot represent a non-class nominal declaration;
- incremental query architecture cannot publish a declaration-local product without a new database authority.

Checkpoint completion:
- [ ] all tasks implemented
- [ ] semantic tests pass
- [ ] hostile nominal/phantom/immutability cases pass
- [ ] cold/incremental equivalence passes
- [ ] no `TypeData::Data`
- [ ] implementation state updated
- [ ] no active incident remains

Suggested commit grouping:
- `feat(semantic): add canonical data declaration products`
- `feat(semantic): type data construction and component projection`
- `test(semantic): pin data nominality and incrementality`

### Task 4 — Add stable data component/constructor identities and semantic products

Purpose:
Introduce the semantic identities and immutable declaration product that every later layer consumes.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/identity.rs`.
- new `phalcom-semantic/src/data_semantics.rs`.
- `phalcom-semantic/src/lib.rs` exports.

Inspect before editing:
- `VariantFieldId`.
- `VariantConstructorId`.
- `VariantFieldSemantic`.
- `VariantConstructorSignature`.
- `EnumInfo` / `EnumSemanticTable`.
- `InvocationTargetId`.
- `SemanticTargetId`.

Dependencies:
- C0 declaration kind.

Source of truth:
- `DeclarationId` + declaration order.

Implementation boundary:

EXACT identity shape:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataComponentId {
    pub owner: DeclarationId,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataConstructorId {
    pub owner: DeclarationId,
}
```

STRUCTURAL semantic product:

```rust
pub enum DataShape {
    Tuple,
    Record,
}

pub struct DataComponentSemantic {
    pub id: DataComponentId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}

pub struct DataConstructorParameter {
    pub component: DataComponentId,
    pub external_label: Option<Box<str>>,
    pub local_name: Box<str>,
    pub declared_type: DeclaredTypeFact,
}

pub struct DataConstructorSignature {
    pub constructor: DataConstructorId,
    pub parameters: Box<[DataConstructorParameter]>,
    pub result_type_template: TypeId,
    pub source: Option<SemanticSourceSpan>,
}

pub struct DataInfo {
    pub owner: DeclarationId,
    pub root_form: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub shape: DataShape,
    pub components: Box<[DataComponentSemantic]>,
    pub constructor: DataConstructorSignature,
    pub source: Option<SemanticSourceSpan>,
}
```

`DataSemanticTable` should follow `EnumSemanticTable` ownership/removal semantics.

Changes:
- add `InvocationTargetId::DataConstructor(DataConstructorId)`;
- add `SemanticTargetId::DataComponent(DataComponentId)`;
- only add a distinct constructor source target if source-index consumers need it; do not invent one preemptively.

Must not:
- use `FieldId` for data components;
- add `CallableId` for the product constructor;
- store a physical slot/offset in `DataComponentSemantic`.

Edit operations:
1. [ ] ADD exact identity structs in `identity.rs`.
2. [ ] EXTEND invocation target enum and helper constructors.
3. [ ] EXTEND semantic source target enum for components.
4. [ ] ADD `data_semantics.rs` modeled on enum products but without exact-case/GADT fields.
5. [ ] ADD table lookup/remove-module methods.
6. [ ] EXPORT through `lib.rs`.
7. [ ] UPDATE exhaustive matches.
8. [ ] `cargo check -p phalcom-semantic`.

Testing classification:
- No standalone behavioral test; validated by C1 semantic products.

Checkpoint state update:
Record the final product field names and invocation-target variant.

### Task 5 — Build generic data declaration semantics from the canonical type system

Purpose:
Construct `DataInfo` using existing declaration generic signatures, type annotation resolution, canonical nominal form, and substitution machinery.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `phalcom-semantic/src/checker/data_declaration.rs`.
- `phalcom-semantic/src/checker/mod.rs`.
- `phalcom-semantic/src/declaration_type.rs` only if data declaration shells need explicit generic-signature registration.
- existing resolver helpers used by `build_enum_semantics`.

Inspect before editing:
- `checker/enum_declaration.rs::build_enum_semantics`.
- declaration generic-signature publication.
- `DeclarationTypeTable::declaration_form`.
- scoped type-parameter resolver.
- `DeclaredTypeFact`.

Dependencies:
- Task 4.

Source of truth:
- existing `GenericSignature` and `TypeStore`.
- `DataDef` supplies logical components.

Implementation boundary:

Changes:
- register data generic parameters exactly as classes/enums do;
- obtain `root_form` from the declaration type table;
- for generic declaration build result template by applying root form to declaration parameters;
- resolve each component annotation in declaration-generic scope;
- create component IDs in source order;
- create one constructor signature with same result template;
- validate duplicate exposed property names/labels according to selector/product rules;
- reject component type formation failures with ordinary typed diagnostics.

Must not:
- create a second generic signature;
- infer component types from initializers (there are none);
- assign runtime storage slots here;
- collapse unknown/error type knowledge to `Dynamic`.

Edit operations:
1. [ ] EXTRACT the reusable “default applied declaration result” logic from enum builder only if it is genuinely identical; otherwise duplicate a small call sequence, not an enum abstraction.
2. [ ] IMPLEMENT `build_data_semantics`.
3. [ ] ADD data declaration generic registration to the same declaration-type pass that handles class/enum generic headers.
4. [ ] ADD tests for generic constraints and kind errors on data parameters/components.
5. [ ] CONFIRM `data Id<K>(_ raw:Int)` constructor result remains `Id<K>`, with K present despite no payload occurrence.

Code instructions:
STRUCTURAL. The enum builder is the repository-grounded template, but do not import GADT `CaseTypeEnvironment` into data.

Testing classification:
- Validated at C1.

### Task 6 — Add data query/product/fingerprint/session/snapshot publication

Purpose:
Make data semantics an ordinary incremental product rather than ad-hoc session state.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate inside semantic subsystem

Owned files and symbols:
- `phalcom-semantic/src/db/key.rs::QueryKey`.
- `phalcom-semantic/src/db/product.rs::SemanticProduct`.
- `phalcom-semantic/src/db/query.rs`.
- `phalcom-semantic/src/db/fingerprint.rs`.
- `phalcom-semantic/src/db/mod.rs::query_module`.
- `phalcom-semantic/src/session.rs`.
- `phalcom-semantic/src/snapshot.rs`.

Inspect before editing:
- `QueryKey::EnumDeclaration`.
- `EnumDeclarationProduct`.
- `query_enum_declaration`.
- enum input/product fingerprint functions.
- session “Compile and publish enum declarations...” phase.
- snapshot enum table/product accessors.

Dependencies:
- Task 5.

Source of truth:
- `QueryKey::DataDeclaration(DeclarationId)` → `DataDeclarationProduct`.

Implementation boundary:

EXACT key shape:

```rust
DataDeclaration(DeclarationId)
```

STRUCTURAL product:

```rust
pub struct DataDeclarationProduct {
    pub info: Arc<DataInfo>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}
```

Use actual repository diagnostic container convention if it differs.

Changes:
- query key + module ownership mapping;
- semantic product variant/accessor;
- input fingerprint must include declaration header, generic constraints, shape, labels/names, component type annotations/order;
- product fingerprint must be stable over semantically equal `DataInfo`;
- session carries previous unaffected data table/products forward and replaces affected module entries;
- snapshot publishes data table/products;
- dependency capture for component/constructor reads records `DataDeclaration`.

Must not:
- fingerprint physical layout here;
- use raw pointer/hash addresses;
- invalidate all data declarations on an unrelated method-body edit.

Edit operations:
1. [ ] ADD query key.
2. [ ] ADD product.
3. [ ] ADD query function patterned on enum declaration query.
4. [ ] ADD module mapping and product materialization arms.
5. [ ] ADD fingerprints.
6. [ ] ADD session base/working maps and publication.
7. [ ] ADD snapshot storage/accessors.
8. [ ] ADD cold/incremental tests changing one component type and an unrelated module.
9. [ ] RUN `cargo check -p phalcom-semantic`.

Testing classification:
- High-risk incremental evidence at C1, not per plumbing edit.

### Task 7 — Type data construction and component projection without fake fields

Purpose:
Connect expressions to `DataConstructorId` and `DataComponentId`, applying existing generic inference/substitution while keeping storage semantics out of `DeclarationSurface`.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/checker/expression.rs::synthesize_get_property`.
- setter/property analysis path.
- unqualified/named-call resolution path.
- checking context data lookup/dependency capture.
- expression semantic/lowering attachment product used by compiler.

Inspect before editing:
- variant constructor semantic application.
- generic callable application/inference.
- `CheckingContext::get_field`.
- `synthesize_set_property`.
- expression occurrence/resolution attachment structures.

Dependencies:
- Tasks 4–6.

Source of truth:
- receiver `TypeId` → data origin/application → `DataInfo`.
- constructor result → `DataConstructorSignature`.

Implementation boundary:

Changes:
- add helper to peel an applied receiver to nominal declaration + substitution;
- if declaration is data, component property lookup precedes ordinary field/getter lookup for declared component names;
- specialize component `DeclaredTypeFact` using receiver substitution;
- attach resolved `DataComponentId` to expression analysis for compiler lowering/source index;
- reject `SetProperty` targeting a component with a dedicated immutable-component diagnostic;
- when a call target resolves to a data declaration, apply `DataConstructorSignature` through canonical generic inference; attach `InvocationTargetId::DataConstructor`;
- record constructor argument order after label matching for lowering;
- record exact inferred result `TypeId` in existing expression semantic result.

Must not:
- insert components into `DeclarationSurface.fields`;
- model constructor as `CallableId`;
- synthesize a getter method;
- allow arbitrary structural record values where the nominal data type is required.

Hostile behavior:
A class getter named `x` and a data component named `x` use their own correct paths; data projection must not accidentally become dynamic getter dispatch.


Edit operations:
1. [ ] OPEN `checker/expression.rs`; locate `synthesize_get_property`, `synthesize_set_property`, and the unqualified/named call path.
2. [ ] ADD a `CheckingContext` helper that resolves a receiver `TypeId` to `(DataInfo, receiver substitution)` while recording a `DataDeclaration` dependency.
3. [ ] BEFORE ordinary field/getter lookup in `synthesize_get_property`, resolve a matching data component and specialize its declared type with the receiver substitution.
4. [ ] ATTACH the resolved `DataComponentId` to the expression's semantic resolution/lowering attachment instead of fabricating a `FieldId`.
5. [ ] IN the setter path, detect a resolved data component and emit the immutable-component diagnostic; never continue to field-write flow state.
6. [ ] IN call resolution, when a binding denotes a data declaration, select `DataConstructorId`, perform canonical argument/label matching, and feed constraints into the existing generic inference/application machinery.
7. [ ] RECORD declaration-order argument mapping and exact inferred result type for compiler lowering.
8. [ ] UPDATE exhaustive `InvocationTargetId`/resolution matches and add the C1 hostile semantic tests.
Testing classification:
- Focused semantic regressions named in C1 evidence.

### Task 8 — Publish data component/source occurrences

Purpose:
Give go-to-definition/rename/hover consumers canonical data/component targets without LSP-specific resolution.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-semantic/src/source_index/builder.rs`.
- `phalcom-semantic/src/source_index/occurrence.rs`.
- source target presentation/adapters.
- source-index tests.

Inspect before editing:
- class field declaration occurrences.
- variant field/constructor occurrences.
- compiler-owned property target attachment.

Dependencies:
- Task 7 resolved component IDs.

Source of truth:
- `SemanticTargetId::DataComponent`.

Changes:
- declaration occurrence for data name targets its `DeclarationId`;
- component declaration occurrences target `DataComponentId`;
- projected property uses target the same component ID when semantic analysis resolved them;
- constructor target uses declaration/constructor identity consistently;
- hover can derive component type from `DataInfo`.

Must not:
- resolve by name in source-index builder when semantic analysis already attached identity;
- add data-specific import resolution to LSP.


Edit operations:
1. [ ] OPEN source-index declaration/occurrence builders and locate existing variant-field and class-field occurrence publication.
2. [ ] PUBLISH the `data` name occurrence as `SemanticTargetId::Declaration(owner)`.
3. [ ] PUBLISH each component declaration range as `SemanticTargetId::DataComponent(id)`.
4. [ ] CONSUME expression semantic attachments so `p.component` occurrences target that same `DataComponentId`; do not re-resolve by name.
5. [ ] WIRE hover/definition presentation through `DataInfo` only where the source-index layer already exposes analogous field/variant metadata.
6. [ ] ADD one source-index test comparing declaration component target and use-site target.
Testing classification:
- source-index focused test now only if existing C1 semantic tests do not expose target IDs; full LSP adapter evidence deferred to C5.

---

## Checkpoint C2 — Shared product layout/storage and materialized data value semantics

Tasks:
- Task 9 — Turn `product.rs` into a reusable layout/storage subsystem.
- Task 10 — Add runtime data descriptor/layout registries.
- Task 11 — Add compact heap-backed data storage and precise GC tracing.
- Task 12 — Add immediate exact nullary data singletons.
- Task 13 — Make language-level exact/equality/hash semantics representation-independent for data.

Why this is a checkpoint:

The runtime representation is coherent only when layout, storage, exact descriptor identity, GC, singleton encoding, and value relations agree. Testing only `DataObject` allocation before tracing/value semantics would give false confidence. C2 proves one claim: a logical product can be materialized compactly and safely without making its allocation identity observable.

Entry conditions:
- C1 COMPLETE.
- `Value` remains 16 bytes.
- existing tuple/record product builder and GC tests pass.

Working set:

Primary:
- `phalcom-core/src/product.rs` plus new `phalcom-core/src/product/layout.rs`, `storage.rs`, `registry.rs` as appropriate.
- `phalcom-core/src/value/repr.rs`.
- `phalcom-core/src/value/mod.rs`.
- `phalcom-core/src/heap/data.rs` — new.
- `phalcom-core/src/heap/object.rs`.
- `phalcom-core/src/heap/mod.rs`.
- `phalcom-core/src/heap/accessors.rs`.
- `phalcom-core/src/heap/trace.rs`.
- `phalcom-core/src/vm/mod.rs` or a dedicated `vm/data.rs`.
- `phalcom-core/src/primitive/object.rs`.
- runtime tests under `phalcom-core/tests/core/language/data/`.

Secondary — inspect only if evidence requires it:
- runtime typing loader/registry for exact type handles.
- memory-management specification when new GC edges are documented.
- `Object::AdtCase` (migration belongs C4).

Out of scope for this checkpoint:
- source compiler data bytecode.
- enum storage migration.
- anonymous tuple/record migration.
- scalar replacement.

Semantic contract established by this checkpoint:
- `ProductLayout` is the sole physical layout authority.
- physical layout has no names/labels.
- known scalar component reps can consume one 64-bit word; unknown reps use two-word `Value`.
- `RuntimeDataDescriptorId` distinguishes exact semantic data applications independently from layout.
- nullary descriptor values materialize without heap allocation.
- positive-arity data materializes as one boxed payload object with no per-instance labels/generic arrays.
- GC follows all `Value` payload handles exactly.
- language-level `===`, `==`, and `hash` for data are independent of backing handle.

Semantic risks:
- unsafe raw-word encoding corrupting `Value`;
- GC missing object handles hidden in universal slots;
- descriptor registry failing to root behavior class/type metadata correctly;
- phantom descriptors collapsing because layout is used as exact identity;
- changing representation-level `same_as` and breaking internal sentinels;
- inflating `Object` arena slot size;
- mixing persisted IDs and VM-local IDs.

Hostile cases:
- two separately boxed `Point<Int>(1,2)` values are `===` under the ratified exact-value rule even though handles differ;
- `Id<User>(42)` and `Id<Order>(42)` are not `===` even if layout/payload bits match;
- repeated `Signal<Connected>()` uses no heap allocation and compares exact;
- `Signal<Connected>()` and `Signal<Disconnected>()` differ;
- a data value holding a heap object survives GC while reachable only through packed product storage;
- dynamic/unproven component uses universal `Value` slot without corrupting GC.

Required evidence:
1. unit tests for layout offsets/word counts and store/load round trips for Int/Float/Bool/Symbol/Value.
2. `value_is_exactly_sixteen_bytes` remains green.
3. `data_nullary_singleton_is_zero_allocation_and_specialization_sensitive`.
4. `data_packed_value_slot_keeps_gc_child_alive`.
5. `data_exact_relation_ignores_backing_handle`.
6. `data_hash_equal_values_equal_hash`.
7. `cargo test -p phalcom-core product` plus `cargo test -p phalcom-core data_`.

Do not run yet:
- ADT suite until C4.
- full core suite until C5.
- workspace.

Escalate immediately if:
- compact storage requires unsafe pointer dereference rather than bounded word encoding;
- exact data type identity can only be preserved by per-instance generic arrays;
- runtime typing metadata cannot provide an internable exact type handle for concrete applied data;
- adding data forces `Value` larger than 16 bytes;
- adding `Object::Data` measurably inflates all arena slots rather than using a boxed payload.

Checkpoint completion:
- [ ] product layout/storage tests pass
- [ ] precise GC test passes
- [ ] singleton allocation test passes
- [ ] exact/equality/hash hostile tests pass
- [ ] Value size invariant passes
- [ ] implementation state updated
- [ ] no active incident remains

Suggested commit grouping:
- `feat(runtime): add shared compact product layout storage`
- `feat(runtime): add data descriptors and materialized values`
- `fix(runtime): make data value relations representation-independent`
- `test(runtime): pin data layout gc and singleton invariants`

### Task 9 — Turn `product.rs` into a reusable layout/storage subsystem

Purpose:
Establish one physical representation language reusable by data now, enum cases in C4, and anonymous products in P3.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-core/src/product.rs`.
- new product submodules.

Inspect before editing:
- `finish_tuple`, `finish_record`.
- `Value` raw layout invariants.
- heap object boxing policy.

Dependencies:
- C1 logical component ordering/types.

Source of truth:
- `ProductLayoutSpec` for physical layout; `DataInfo` remains logical shape authority.

Implementation boundary:

For P1, use a safe bounded **word-packed** baseline rather than a permanently universal `Box<[Value]>`.

Required physical slot kinds:

```rust
pub enum ProductSlotRepr {
    Int64,
    Float64,
    Bool,
    Symbol,
    Value,
}
```

A future checkpoint may add direct object refs, inline nested products, smaller packed bools, or other niches without semantic change.

Required concepts:

```rust
pub struct ProductComponentLayout {
    pub logical_index: u32,
    pub word_offset: u32,
    pub repr: ProductSlotRepr,
}

pub struct ProductLayout {
    pub word_len: u32,
    pub components: Box<[ProductComponentLayout]>,
    // exact Value-slot trace locations or equivalent
}

#[repr(transparent)]
pub struct ProductLayoutId(pub u32);

pub struct ProductStorage {
    pub layout: ProductLayoutId,
    pub words: Box<[u64]>,
}
```

`Value` slots consume two words and use crate-private checked/raw conversion helpers. Native scalar slots consume one word.

Changes:
- keep `finish_tuple`/`finish_record` behavior unchanged;
- add layout validation: dense logical indexes, bounded offsets, no overlap, checked word-size arithmetic;
- add `store_component(layout,index,Value)` with representation validation;
- add `load_component`;
- add trace iterator over Value-slot locations;
- add layout equality/hash for interning;
- store the canonical `ProductLayoutRegistry` as Rust-owned metadata on `Heap`, so precise GC can resolve `ProductStorage.layout` without consulting VM-level semantic registries;
- change the collector/`trace_object` call seam as narrowly as necessary so the exhaustive object tracer receives read-only product-layout metadata while preserving the existing no-wildcard discipline.

Must not:
- put Symbol labels in layout;
- assume every nominal type can be stored as raw `ObjRef`;
- use unchecked transmute for Value;
- migrate tuple/record yet.

Code instructions:
STRUCTURAL. Reuse `Value`'s encapsulation by adding crate-private raw-word encode/decode methods in `value/repr.rs`; do not make raw fields public.


Edit operations:
1. [ ] KEEP `finish_tuple` and `finish_record` in `product.rs` behaviorally unchanged.
2. [ ] ADD `product/layout.rs` with `ProductLayoutId`, slot representations, component layouts, validation, and deterministic equality/hash.
3. [ ] ADD `product/storage.rs` with `ProductStorage { layout, words }` and checked store/load helpers.
4. [ ] ADD `product/registry.rs` with an interned `ProductLayoutRegistry`; make `Heap` own the registry as Rust metadata, not a GC object.
5. [ ] ADD crate-private `Value` raw-word round-trip helpers in `value/repr.rs` that preserve tag/depth/reserved-bit invariants.
6. [ ] IMPLEMENT `Value`-slot tracing by reconstructing only declared two-word Value slots and using `gc_obj_ref`.
7. [ ] ADAPT the collector's internal trace seam to pass read-only layout registry access and keep `trace_object` exhaustive.
8. [ ] ADD layout overlap/overflow/round-trip tests plus the existing empty tuple/record regression.
Testing classification:
- layout/storage unit tests required at C2.

### Task 10 — Add runtime data descriptor/layout registries

Purpose:
Intern exact data-type identity separately from physical layout and behavior class.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `phalcom-core/src/data.rs` or `phalcom-core/src/product/data.rs`.
- `VM` fields/initialization/root enumeration.
- runtime typing registry bridge.

Inspect before editing:
- `RuntimeAdtRegistry`.
- `RuntimeTypingRegistry`.
- `typing::handle::RuntimeTypeRef`.
- VM GC root enumeration for registries containing `ClassId`.

Dependencies:
- Task 9.

Source of truth:
- exact type metadata handle + declaration identifies runtime data descriptor;
- physical layout spec identifies/interns layout.

Target concepts:

```rust
#[repr(transparent)]
pub struct RuntimeDataDescriptorId(pub u32);

pub struct RuntimeDataDescriptor {
    pub semantic_owner: DeclarationId,
    pub runtime_id: RuntimeDataDescriptorId,
    pub behavior_class: ClassId,
    pub exact_type: RuntimeTypeRef, // or existing equivalent stable runtime handle
    pub layout: ProductLayoutId,
}
```

INVESTIGATE-BEFORE-EDIT, narrowly:
Before committing the `exact_type` field type, inspect how the compiled program's semantic metadata pool assigns `RuntimeTypeRef` handles to source/result type forms. Use that existing runtime handle. Do **not** store compiler `TypeId`, and do **not** invent a second serialized type graph.

Changes:
- `Heap` owns/interns `ProductLayout`s and returns `ProductLayoutId`;
- the VM data registry interns exact descriptors by exact runtime semantic type handle;
- multiple descriptors may map to same layout;
- registry roots behavior `ClassId`s;
- descriptor query supports `.class` and value relation logic.

Must not:
- use layout ID as type identity;
- allocate a runtime ClassObject per exact specialization;
- hold a strong heap reflection descriptor merely to identify the type.


Edit operations:
1. [ ] ADD the VM-local `RuntimeDataDescriptorId` and `RuntimeDataDescriptor`/registry near the existing ADT registry rather than in semantic code.
2. [ ] KEY descriptor interning by the existing runtime semantic type handle plus declaration; do not key by layout alone.
3. [ ] RESOLVE/intern physical `ProductLayoutSpec` through `heap.product_layouts` and store only its `ProductLayoutId` in the descriptor.
4. [ ] STORE one declaration behavior `ClassId` in the descriptor and register it as a GC/runtime root through the same VM root enumeration discipline as ADT classes.
5. [ ] INSPECT the metadata loader to select the existing runtime type-handle type (`RuntimeTypeRef` or its current successor); document the final choice in implementation state.
6. [ ] ADD tests where two phantom specializations have different descriptor IDs but the same `ProductLayoutId`.
Testing classification:
- registry identity tests at C2, including phantom layout sharing.

### Task 11 — Add compact heap-backed data storage and precise GC tracing

Purpose:
Provide the conservative materialized positive-arity data representation.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `phalcom-core/src/heap/data.rs`.
- `heap/object.rs::Object`.
- `heap/mod.rs`.
- `heap/accessors.rs`.
- `heap/trace.rs::trace_object`.
- `docs/spec/current/memory-management.md` exact current location if required by repository tracing policy.

Dependencies:
- Tasks 9–10.

Source of truth:
- `RuntimeDataDescriptorId` → layout; `DataObject` stores descriptor + words only.

EXACT conceptual payload:

```rust
pub struct DataObject {
    pub descriptor: RuntimeDataDescriptorId,
    pub storage: ProductStorage, // includes ProductLayoutId for GC/projection
}
```

Use `Object::Data(Box<DataObject>)` to preserve arena slot size.

Changes:
- heap allocator/accessors;
- trace Data by resolving `data.storage.layout` through the Heap-owned `ProductLayoutRegistry` and walking only slots whose representation can contain a GC `Value`;
- extend the collector's internal `trace_object` seam to receive read-only product-layout metadata. Do not duplicate trace maps per value merely to preserve the old two-argument helper signature;
- projection helper loads a `Value` from product storage.

Must not:
- store `ClassId` separately if descriptor already owns it;
- store component names;
- store `Box<[Value]>`.


Edit operations:
1. [ ] ADD `heap/data.rs::DataObject` with only exact descriptor identity plus `ProductStorage`.
2. [ ] ADD boxed `Object::Data(Box<DataObject>)` and update exports/accessors/heap allocator.
3. [ ] UPDATE `trace_object`'s exhaustive match with a Data arm that resolves `storage.layout` from the Heap-owned layout registry and traces declared Value slots.
4. [ ] UPDATE memory-management edge documentation in the same patch if required by the repository's ADR-0050 discipline.
5. [ ] ADD projection helpers that validate logical component index against the layout before decoding.
6. [ ] ADD a GC test whose only live path to a heap child is a universal Value component inside DataObject.
Testing classification:
- GC hostile test mandatory at C2.

### Task 12 — Add immediate exact nullary data singletons

Purpose:
Represent `Signal<State>()` without heap allocation while preserving exact applied specialization.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-core/src/value/repr.rs::ValueTag`.
- `Value` constructors/accessors.
- `Value::class`.
- `Value::gc_obj_ref`.
- runtime data registry.

Inspect before editing:
- `AdtSingleton`.
- Option depth tagging/reserved-bit tests.

Dependencies:
- Task 10 descriptor IDs.

Source of truth:
- `RuntimeDataDescriptorId`, not declaration or layout.

Changes:
- add explicit `DataSingleton` `ValueTag`;
- payload stores VM-local runtime data descriptor ID;
- add checked accessor;
- class lookup resolves descriptor → behavior class;
- `gc_obj_ref` remains none for the immediate itself;
- reserved bits stay zero.

Must not:
- reuse `AdtSingleton` (different registry/semantics);
- encode only declaration ID (would collapse phantom/applied singleton specializations);
- allocate a heap singleton.


Edit operations:
1. [ ] ADD `ValueTag::DataSingleton` without changing the 16-byte struct layout or consuming reserved meta bits.
2. [ ] ADD `Value::data_singleton(RuntimeDataDescriptorId)`, predicate, and checked accessor.
3. [ ] UPDATE `PartialEq`/`Hash` representation-level matching exhaustively for the new tag.
4. [ ] UPDATE `Value::class` to resolve the data descriptor's behavior class.
5. [ ] CONFIRM `gc_obj_ref` returns no heap edge for the immediate.
6. [ ] EXTEND reserved-bit/value-size tests and add zero-allocation specialization-sensitive singleton tests.
Testing classification:
- focused zero-allocation/specialization tests mandatory.

### Task 13 — Make language-level exact/equality/hash semantics representation-independent for data

Purpose:
Prevent a heap allocation strategy from becoming observable through core value relations.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file/runtime hot path

Owned files and symbols:
- `phalcom-core/src/value/mod.rs::Value::same_as`.
- `phalcom-core/src/vm/dispatch.rs::Bytecode::Same`.
- `phalcom-core/src/primitive/object.rs::{object_same, object_eq, object_neq, object_hash}` exact current names.
- data projection/storage helpers.

Dependencies:
- Tasks 10–12.

Source of truth:
- runtime exact data descriptor + logical component sequence.

Target design:
Keep `Value::same_as` as representation-level identity for internal runtime uses.

Add VM-aware language semantic helpers, for example:

```rust
impl VM {
    pub(crate) fn semantic_same(&self, lhs: Value, rhs: Value) -> bool;
    // equality/hash helpers may need the normal send machinery when components
    // override == / hash; follow existing primitive re-entry conventions.
}
```

`Bytecode::Same` and `Object#===` primitive use the VM-aware semantic helper.

For two data values:
- descriptor IDs must denote the same exact semantic type;
- components compare recursively by language `===`;
- backing `ObjRef` does not participate.

For `==` and `hash`, implement representation-independent structural baseline over exact data type + logical components. If the existing primitive calling convention cannot synchronously recurse through overridable `==`/`hash` without dispatch re-entry problems, use the same internal workhorse/pattern already used by Tuple/Record structural operations or install declaration-owned primitive methods. Do not fall back to handle identity.

Must not:
- globally redefine internal `Value::same_as`;
- compare only layout + raw words (phantom types could collide);
- cache mutable hash state in every DataObject.


Edit operations:
1. [ ] ADD a VM-aware language-level exact comparison helper while leaving `Value::same_as` as low-level representation identity.
2. [ ] CHANGE the `Bytecode::Same` handler to call the VM-aware helper.
3. [ ] CHANGE the native `Object#===` primitive to use the same helper.
4. [ ] FOR DataObject/DataSingleton pairs, require exact descriptor semantic-type equality and recursively compare logical components with language `===`.
5. [ ] EXTEND Object `==`/`hash` primitives or install shared data primitives so data values use exact type plus component value semantics rather than backing ObjRef.
6. [ ] ENSURE equal data values hash equally and independently allocated boxes do not affect results.
7. [ ] SEARCH internal `same_as` users and leave sentinel/handle-identity uses unchanged unless they are language-level exact comparison.
Testing classification:
- high-risk C2 hostile relation tests.

---

## Checkpoint C3 — Semantic lowering, bytecode, VM construction, and projection

Tasks:
- Task 14 — Project data semantics into executable lowering specs.
- Task 15 — Add minimal data declaration/construction/projection bytecodes.
- Task 16 — Compile data declaration and constructor sites from semantic authority.
- Task 17 — Execute construction/projection in the VM through product storage.
- Task 18 — Integrate declaration behavior identity and basic runtime type reification.

Why this is a checkpoint:

C3 is the first source-to-runtime vertical slice. Data syntax and runtime storage already exist independently; C3 proves that code generation consumes semantic IDs/layout specs rather than rediscovering AST shape or class fields. Its tests are the first meaningful end-to-end data programs.

Entry conditions:
- C2 COMPLETE.
- semantic lowering infrastructure still owns enum associated construction.
- runtime typing metadata loads for compiled programs.

Working set:

Primary:
- `phalcom-core/src/modules/semantic_lowering.rs`.
- executable semantics table in chunk/compiler.
- `phalcom-core/src/bytecode.rs`.
- new `phalcom-core/src/compiler/lib/data_decl.rs`.
- `phalcom-core/src/compiler/lib/mod.rs`.
- `phalcom-core/src/compiler/lib/expr.rs`.
- `phalcom-core/src/vm/dispatch.rs` and/or `vm/data.rs`.
- `phalcom-core/src/value/mod.rs`.
- `phalcom-core/src/typing/*` registration/reification bridge.
- runtime data tests.

Secondary — inspect only if evidence requires it:
- artifact loader/materialization plan.
- disassembler.
- compiler source mapping.
- native metadata exporter.

Out of scope:
- scalar replacement/escape analysis.
- anonymous product lowering changes.
- generic runtime type-environment completion beyond what is required for correct P1 materialization.

Semantic contract established by this checkpoint:
- compiler construction targets are `DataConstructorId`.
- physical slot choice comes from a projected `ProductLayoutSpec`, never from runtime property names.
- declaration runtime behavior class exists once per `DeclarationId` and refuses generic `InstanceObject` allocation.
- component projection uses `DataComponentId`/logical index.
- materialized nullary data uses immediate singleton.
- materialized positive-arity data uses one DataObject.
- exact known applied type is registered with the data descriptor.
- component mutation is impossible at bytecode/runtime level.

Semantic risks:
- compiler falling back to ordinary `Call`/getter dispatch for data constructor/projection;
- AST shape becoming physical layout authority;
- applied type being erased to declaration class;
- runtime class row accidentally allocatable with `new_`;
- record labels reordered between semantic argument matching and physical component order;
- raw `TypeId` entering chunk/artifact.

Hostile cases:
- labeled constructor arguments supplied in call-site order differing from declaration order still store/project correctly;
- phantom-specialized nullary values produce distinct runtime descriptors;
- imported data constructor uses the same declaration identity;
- attempting class allocator `new_` on the data behavior row is rejected;
- `p.x` does not invoke a dynamically replaced getter.

Required evidence:
1. end-to-end core tests for tuple-shaped, labeled tuple-shaped, record-shaped, nullary, generic concrete, phantom data.
2. allocation assertions: nullary zero; positive materialized one product object, no labels.
3. bytecode/disassembly regression proving data uses dedicated `ConstructData`/`GetDataComponent` (names may adapt) rather than `NewInstance` + field setters.
4. `cargo test -p phalcom-core data_`.
5. `cargo test -p phalcom-semantic data_` re-run because compiler lowering consumes those products.

Do not run yet:
- full ADT suite until C4.
- full core crate until C5.
- workspace.

Escalate immediately if:
- lowering requires reconstructing data shape from AST because `DataInfo` information is missing;
- exact type metadata cannot be attached without raw `TypeId`;
- current bytecode artifact lifecycle cannot resolve the existing stable runtime type handle required by the descriptor; classify as DEPENDENCY/PUBLICATION and fix only that loader seam.

Checkpoint completion:
- [ ] declaration bytecode path works
- [ ] construction/project runtime works
- [ ] end-to-end data programs pass
- [ ] allocation hostile cases pass
- [ ] no `InstanceObject` construction path for data
- [ ] implementation state updated
- [ ] no active incident remains

Suggested commit grouping:
- `feat(lowering): project data constructors and component layouts`
- `feat(vm): execute data construction and projection`
- `test(core): verify data runtime representation`

### Task 14 — Project data semantics into executable lowering specs

Purpose:
Create compiler-consumable data specs from semantic products, analogous to enum lowering, including declaration-order component IDs and physical slot representation.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-core/src/modules/semantic_lowering.rs`.
- executable semantic table types in compiler chunk.
- type metadata export bridge for exact construction result.

Inspect before editing:
- `EnumLoweringSpec`.
- `VariantFieldLoweringSpec`.
- `build_module_lowering_semantics`.
- associated lowering site projection.

Dependencies:
- C1 DataInfo.
- C2 ProductLayoutSpec.

Source of truth:
- DataInfo + expression-resolved constructor result type.

Required spec concepts:

```rust
pub struct DataComponentLoweringSpec {
    pub id: DataComponentId,
    pub logical_index: u32,
}

pub struct DataDeclarationLoweringSpec {
    pub owner: DeclarationId,
    pub components: Box<[DataComponentLoweringSpec]>,
}

pub struct DataConstructionLoweringSpec {
    pub constructor: DataConstructorId,
    pub exact_type: /* existing stable runtime type metadata reference */,
    pub layout: ProductLayoutSpec,
    pub argument_to_component: Box<[u32]>,
}
```

The exact runtime type reference field is STRUCTURAL: use the existing stable metadata export/loader identity. No raw `TypeId`.

Changes:
- project physical slot repr conservatively:
  - proven Int → Int64;
  - proven Float → Float64;
  - proven Bool → Bool;
  - proven Symbol → Symbol;
  - everything else → Value.
- argument-to-component mapping handles labels.
- nullary layout has zero words.
- add deterministic checked overflow.

Must not:
- persist source names in physical layout;
- specialize unknown generic parameter to a guessed representation.


Edit operations:
1. [ ] ADD data declaration/construction/component lowering structs beside enum lowering structs.
2. [ ] PROJECT declaration-order component IDs from `DataInfo` without reading AST fields.
3. [ ] PROJECT conservative physical slot repr from proven semantic component type; unknown/generic/dynamic uses `Value`.
4. [ ] EXPORT/attach the exact construction result through the existing stable runtime type metadata handle, never `TypeId`.
5. [ ] ADD argument-to-component mapping after semantic label matching.
6. [ ] ADD checked layout-size/index conversion errors to `ProjectionError` or the repository's current lowering error type.
7. [ ] ADD deterministic lowering tests for positional, labeled, record, phantom, and mixed-representation data.
Testing classification:
- lowering unit tests at C3.

### Task 15 — Add minimal data declaration/construction/projection bytecodes

Purpose:
Give the VM explicit operations whose contracts match semantic data operations.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-core/src/bytecode.rs`.
- disassembler.
- executable semantics indexes.

Dependencies:
- Task 14.

Source of truth:
- lowering-spec indexes stored in chunk executable semantics.

Recommended bytecode family:

```text
Data(spec_index)                         declare/register runtime behavior row
LoadDataSingleton(construction_spec)     nullary exact application
ConstructData(construction_spec)         positive-arity materialization
GetDataComponent(component_spec/index)   immutable projection
FinalizeData(spec_index)                 only if behavior/base-name finalization requires it
```

If `Data`/`FinalizeData` can safely reuse the class declaration allocation helper internally, keep the **bytecode semantic identity distinct** so generic class allocation cannot become the data value constructor.

Must not:
- use `NewInstance`;
- use `GetField`;
- use `SetField`.


Edit operations:
1. [ ] ADD the minimal bytecode variants and stable opcode encoding numbers following current append-only conventions.
2. [ ] ADD executable-semantics indexes for declaration/construction/component specs.
3. [ ] UPDATE bytecode decoder/encoder/disassembler/exhaustive stack-effect matches.
4. [ ] DEFINE stack contracts explicitly for each new opcode; construction pops exactly its normalized arguments and pushes one Value.
5. [ ] DO NOT route any new opcode through class field bytecodes.
6. [ ] ADD bytecode round-trip/disassembly tests if the repository already tests opcode serialization.
Testing classification:
- no standalone test; C3 disassembly/end-to-end evidence.

### Task 16 — Compile data declaration and constructor sites from semantic authority

Purpose:
Wire AST statements/expressions to dedicated data bytecode, consuming lowering products.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `compiler/lib/data_decl.rs`.
- `compiler/lib/mod.rs::compile_statement`.
- `compiler/lib/expr.rs`.
- compiler errors for missing semantic lowering products.

Dependencies:
- Tasks 14–15.

Source of truth:
- module lowering semantics / expression resolution attachments.

Changes:
- `Statement::Data` invokes `compile_data`.
- compile declaration behavior row once, `native_repr = true`, no fields.
- tuple-shaped constructor call resolved to `DataConstructorId` emits construction bytecode after compiling arguments in semantic normalized order or records mapping in spec.
- record-shaped nominal construction emits the same `ConstructData`.
- `GetProperty` resolved to `DataComponentId` emits `GetDataComponent`.
- setter should never reach bytecode for a component after semantic rejection; fail closed if it does.
- nullary exact application emits singleton load.

Must not:
- look up component names at compile time from AST after semantic resolution;
- invoke `DataExpander`;
- synthesize `new`.


Edit operations:
1. [ ] ADD `compiler/lib/data_decl.rs` and export it from the compiler module.
2. [ ] ADD `Statement::Data` branch in `compile_statement` calling `compile_data`.
3. [ ] MAKE `compile_data` require the projected declaration spec; fail closed with a compiler error if semantic lowering is absent.
4. [ ] COMPILE tuple-shaped ordinary call sites resolved as `DataConstructorId` to the data construction bytecode.
5. [ ] COMPILE nominal record-construction AST to the same construction bytecode.
6. [ ] COMPILE component `GetProperty` attachments to `GetDataComponent`.
7. [ ] EMIT the nullary singleton load for zero-component exact applications.
8. [ ] SEARCH the new compiler file for `DataExpander`, `NewInstance`, `GetField`, and `SetField`; all must be absent.
Testing classification:
- C3 integration.

### Task 17 — Execute construction/projection in the VM through product storage

Purpose:
Implement the bytecode contracts using C2 registries/storage.

Risk:
- Semantic: HIGH
- Implementation fanout: local runtime hot path

Owned files and symbols:
- `vm/dispatch.rs` or dedicated `vm/data.rs`.
- `Heap` data allocators.
- data registry.

Dependencies:
- Tasks 15–16.

Source of truth:
- construction spec → exact descriptor/layout.

Changes:
- declaration opcode allocates/wires a runtime behavior class row and registers nominal `DeclarationId`;
- nullary construction interns/resolves exact descriptor then pushes `Value::data_singleton`;
- positive construction creates storage once at final word length, encodes each argument once, allocates one boxed DataObject;
- projection verifies descriptor/component and loads via layout;
- no mutation bytecode exists.

Must not:
- initialize storage with Nil `Value`s;
- dynamically search labels;
- allocate one heap object per component.


Edit operations:
1. [ ] ADD VM dispatch handlers (or delegate to `vm/data.rs`) for data declaration, singleton load, positive construction, and component projection.
2. [ ] ON declaration, allocate/wire one behavior ClassObject, mark it `native_repr`, and register its nominal declaration association.
3. [ ] ON construction, resolve/intern exact runtime descriptor and layout; allocate final word storage once; encode each argument once in declaration order.
4. [ ] ON singleton load, resolve/intern exact descriptor and push `Value::data_singleton` without heap allocation.
5. [ ] ON projection, resolve the value's layout and decode the logical component; reject malformed bytecode/layout mismatches as internal/runtime validation errors.
6. [ ] ADD live-count and projection end-to-end tests.
Testing classification:
- C3 end-to-end/runtime allocation tests.

### Task 18 — Integrate declaration behavior identity and basic runtime type reification

Purpose:
Make `.class`/nominal type reflection compatible with data without conflating behavior class, exact applied type, and physical layout.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-runtime

Owned files and symbols:
- runtime typing registry nominal registration.
- `typing/reify.rs`.
- `Value::class`.
- data registry.

Dependencies:
- Task 17.

Source of truth:
- declaration behavior `ClassId` for `.class`;
- exact runtime semantic type handle for applied-type reflection.

Changes:
- register data declaration's behavior row as the nominal runtime association for its `DeclarationId`;
- mark row `native_repr = true` so generic allocator refuses `InstanceObject`;
- `Value::class`:
  - DataObject → descriptor.behavior_class;
  - DataSingleton → descriptor.behavior_class;
- nominal reification continues returning the behavior class object;
- concrete applied data type reflection resolves through existing typing descriptor machinery and descriptor's exact type handle.

Boundary:
C1.P3 owns exhaustive unresolved-runtime-generic, Dynamic roundtrip, and reflection stress. P1 must preserve exact known specialization identity and must not encode a false erased type when exact metadata is available.


Edit operations:
1. [ ] REGISTER the data declaration behavior class with `RuntimeTypingRegistry` through the same nominal association used by `reify_type_form`.
2. [ ] UPDATE `Value::class` DataObject/DataSingleton paths to use the descriptor behavior class.
3. [ ] VERIFY generic class allocator refuses the data behavior row because `native_repr` is set.
4. [ ] WIRE concrete applied data descriptor exact-type handles into the existing weak reification path; do not create applied ClassObjects.
5. [ ] ADD tests for nominal reification, `.class`, and concrete applied type descriptor identity.
6. [ ] DOCUMENT in implementation state any unresolved runtime-generic type-environment case and leave its exhaustive qualification to C1.P3; do not erase it falsely.
Testing classification:
- basic reflection `.class`, nominal reify, concrete applied descriptor checks at C3.

---

## Checkpoint C4 — Enum payload convergence without semantic migration

Tasks:
- Task 19 — Add product layouts to enum lowering specs.
- Task 20 — Replace general `AdtCaseObject` payload storage with `ProductStorage`.
- Task 21 — Make construct/project/GC paths layout-aware.
- Task 22 — Prove GADT/behavior/NativeOption preservation.

Why this is a checkpoint:

Changing enum storage is safe only if construction, payload projection, GC, matching, behavior-class dispatch, and native Option remain consistent. C4 intentionally changes no enum source or semantic identity; it replaces only the physical payload authority.

Entry conditions:
- C3 COMPLETE.
- all existing focused ADT tests pass before this checkpoint.
- current `RuntimeAdtRegistry` identities remain intact.

Working set:

Primary:
- `phalcom-core/src/modules/semantic_lowering.rs::{VariantLoweringSpec, VariantFieldLoweringSpec}`.
- `phalcom-core/src/heap/adt.rs::AdtCaseObject`.
- `phalcom-core/src/heap/trace.rs`.
- `phalcom-core/src/adt.rs`.
- `phalcom-core/src/vm/adt.rs`.
- `phalcom-core/src/vm/dispatch.rs`.
- compiler associated/variant construction lowering.
- ADT tests in `phalcom-core/tests/core/language/algebraic_data`.
- semantic ADT tests only where a representation change could reveal an accidental semantic regression.

Secondary:
- match compiler/pattern projection.
- variant behavior compiler.
- reflection tests.

Out of scope:
- enum syntax simplification.
- moving behavior to `impl`.
- changing GADT semantic products.
- changing `NativeOption` representation.

Semantic contract established by this checkpoint:
- `VariantId`, `RuntimeVariantId`, `CaseDiscriminant`, behavior class remain unchanged authorities.
- general constructor payload storage is ProductStorage/layout-driven.
- field logical identity/order remains semantic declaration order.
- singleton variants remain immediate.
- NativeOption remains its own representation strategy.
- case behavior dispatch and matching produce identical observable results.

Semantic risks:
- layout discriminant mistaken for variant identity;
- GADT field types specialized incorrectly;
- pattern payload indexes diverging from product logical indexes;
- behavior class lost when AdtCase changes;
- Option Some-depth accidentally routed into Data/Product general box;
- GC regression.

Hostile cases:
- enum variants with same physical payload layout remain different cases;
- generic variants with phantom type distinctions preserve semantic typing;
- case-local method dispatch still selects hidden case behavior class;
- match projection works after payload storage change;
- nested heap object in enum payload survives GC;
- `Option::Some`/`None` retains current immediate rules.

Required evidence:
1. `cargo test -p phalcom-core algebraic_data`.
2. `cargo test -p phalcom-semantic adts`.
3. new runtime test `general_adt_payload_uses_product_layout_without_changing_case_identity`.
4. new GC test with packed Value slot.
5. negative search `rg 'payload: Box<\\[Value\\]>' phalcom-core/src/heap/adt.rs` → zero.
6. verify `RuntimeAdtRepresentation::NativeOption` still has explicit branch and tests.

Do not run yet:
- full workspace; C5.
- enum parser migration; LANG005.C2.

Escalate immediately if:
- existing GADT tests fail and proposed repair changes semantic enum tables rather than storage/lowering;
- NativeOption would have to lose immediate representation;
- case behavior class lookup currently depends on payload being `Box<[Value]>` in an undocumented way.

Checkpoint completion:
- [ ] shared product storage used for general enum payloads
- [ ] ADT core suite passes
- [ ] semantic ADT suite passes
- [ ] GC hostile case passes
- [ ] NativeOption preservation proven
- [ ] negative storage search passes
- [ ] implementation state updated
- [ ] no active incident remains

Suggested commit grouping:
- `refactor(adt): project variant product layouts`
- `refactor(runtime): store adt payloads in shared product storage`
- `test(adt): prove representation migration preserves semantics`

### Task 19 — Add product layouts to enum lowering specs

Purpose:
Make enum construction use semantic field types to choose the same conservative/native slot representations as data.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `modules/semantic_lowering.rs`.
- `VariantLoweringSpec`.
- `VariantFieldLoweringSpec`.

Inspect before editing:
- current field slot assignment.
- variant `DeclaredTypeFact` in `VariantInfo`.
- NativeOption representation branch.

Dependencies:
- C2 ProductLayoutSpec.

Source of truth:
- `VariantInfo.fields` in declaration order.

Changes:
- retain stable `VariantFieldId`;
- replace/augment raw u16 `slot` with logical index + product component layout;
- produce product layout only for `RuntimeAdtRepresentation::General`;
- native Option follows existing special lowering.

Must not:
- read field annotations from AST here;
- change exact-case/result template semantics.


Edit operations:
1. [ ] EXTEND `VariantLoweringSpec` with a general-product layout spec (or internable equivalent) while retaining `VariantId`, shape, and field identities.
2. [ ] DERIVE each physical component representation from `VariantInfo.fields[*].declared_type`, not AST.
3. [ ] KEEP logical variant field index distinct from physical word offset.
4. [ ] BYPASS general product layout generation for `RuntimeAdtRepresentation::NativeOption` except metadata needed by existing code.
5. [ ] UPDATE lowering projection errors/tests for overflow and missing field metadata.
Testing classification:
- C4.

### Task 20 — Replace general `AdtCaseObject` payload storage with ProductStorage

Purpose:
Remove unconditional `Box<[Value]>` enum payload storage.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local + callers

Owned files and symbols:
- `heap/adt.rs::AdtCaseObject`.
- `Heap::alloc_adt_case`.

Dependencies:
- Task 19.

Source of truth:
- `RuntimeVariantId` selects the variant; its projected `ProductLayoutId` selects physical payload storage.

Target implementation:

```rust
pub struct AdtCaseObject {
    pub variant: RuntimeVariantId,
    pub storage: ProductStorage,
}
```

The layout remains reachable via runtime variant descriptor or explicit compact layout ID.

Must not:
- merge `AdtCaseObject` with `DataObject`;
- store labels.


Edit operations:
1. [ ] CHANGE `AdtCaseObject.payload: Box<[Value]>` to `storage: ProductStorage`.
2. [ ] CHANGE `Heap::alloc_adt_case` signature to accept completed ProductStorage (or layout + normalized values if construction is centralized there).
3. [ ] UPDATE direct struct construction/accessors/callers surfaced by `cargo check -p phalcom-core`.
4. [ ] DO NOT change RuntimeVariantId, behavior class, or enum registry identity.
5. [ ] ADD a unit test confirming no labels/type arguments are stored on the case object.
Testing classification:
- C4.

### Task 21 — Make enum construct/project/GC paths layout-aware

Purpose:
Use ProductStorage for all general variant payload execution.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `Bytecode::ConstructVariant` handler.
- `Bytecode::GetVariantPayload`.
- match/pattern helpers.
- `trace_object` ADT arm.

Dependencies:
- Task 20.

Source of truth:
- `RuntimeVariantDescriptor`/`VariantLoweringSpec` for case identity and logical fields; `ProductLayout` only for physical encode/decode/trace.

Changes:
- encode arguments once using variant layout;
- project by logical field index;
- trace only universal Value slots according to layout;
- keep variant behavior class resolution through RuntimeVariantDescriptor.


Edit operations:
1. [ ] CHANGE `ConstructVariant` general-representation handler to allocate ProductStorage with the projected layout.
2. [ ] CHANGE `GetVariantPayload` to decode logical field index through the product layout.
3. [ ] UPDATE pattern/match helper code only where it directly assumed `case.payload[index]`.
4. [ ] CHANGE the `Object::AdtCase` trace arm to resolve ProductStorage.layout from Heap product layouts and trace Value slots.
5. [ ] LEAVE behavior-class lookup, variant tests, and case discriminant logic on RuntimeVariantId.
6. [ ] ADD packed payload and GC tests.
Testing classification:
- C4 focused + existing suite.

### Task 22 — Prove GADT, behavior, singleton, and NativeOption preservation

Purpose:
Close the storage migration with evidence that semantic identity and execution were not changed.

Risk:
- Semantic: HIGH
- Implementation fanout: tests + only fixes supported by evidence

Owned tests:
- `phalcom-core/tests/core/language/algebraic_data/{execution,behavior,construction_primitives,gc,gc_scenarios,associated_runtime,associated_reification,...}.rs`.
- `phalcom-semantic/tests/semantic/adts/{declarations,constructors,exact_cases,generics,behavior,matching,...}`.

Dependencies:
- Tasks 19–21.

Source of truth:
- existing semantic ADT/GADT products and their pre-existing regression suites; representation code is the only permitted repair boundary unless evidence proves baseline/plan drift.

Failure protocol:
If a semantic ADT test fails, first classify whether the failure is PRODUCT (layout), DEPENDENCY/PUBLICATION, BACKEND, or BASELINE. Do not edit GADT semantic theory merely because representation changed.


Edit operations:
1. [ ] RUN the existing focused semantic ADT suites before making semantic repairs.
2. [ ] RUN the existing core algebraic-data suites after storage migration.
3. [ ] ADD one case-identity hostile test with two variants sharing identical physical payload layout.
4. [ ] ADD one behavior dispatch test and one GADT match/projection test that exercise the new storage path.
5. [ ] ADD/extend NativeOption test asserting the special representation branch and immediate behavior remain unchanged.
6. [ ] RUN the negative `Box<[Value]>` payload search only after all callers have migrated.
Testing classification:
- checkpoint evidence task.

---

## Checkpoint C5 — Incremental, tooling, compatibility, and P1 delivery closure

Tasks:
- Task 23 — Close layout/semantic fingerprint and artifact publication boundaries.
- Task 24 — Finish source tooling and explicitly isolate legacy `@data`.
- Task 25 — Run focused performance/allocation qualification and delivery gates.

Why this is a checkpoint:

C0–C4 prove individual semantic/runtime boundaries. C5 proves that repository consumers agree, that edits invalidate the right products, that legacy mechanisms cannot silently become authority, and that the representation actually satisfies P1's efficiency claims.

Entry conditions:
- C4 COMPLETE.
- no active incident in state file.

Working set:

Primary:
- semantic incremental tests.
- module/compiler artifact/lowering serialization boundary actually used by compiled program.
- source-index/LSP adapters.
- legacy attribute compiler.
- runtime allocation/layout tests.
- PDR/spec/status documents when landing.

Secondary:
- broad affected crate test manifests.
- docs references to data syntax.

Out of scope:
- P2 optimizer.
- P3 anonymous product migration and exhaustive Dynamic/generic runtime reification.
- C2 `impl`.
- traits.

Semantic contract established by this checkpoint:
- semantic component changes invalidate semantic/lowering products.
- behavior-unrelated changes do not create spurious layout identity changes.
- cold/incremental outputs agree.
- compiler/LSP source targets agree on declaration/component identity.
- legacy `@data` remains isolated.
- P1's measurable representation invariants hold.
- no deferred P1 evidence is forgotten.

Semantic risks:
- stale layout after incremental edit;
- runtime artifact retaining raw store-local TypeId;
- LSP resolving by name differently from semantic source index;
- legacy DataExpander accidentally running on first-class data;
- broad test pass hiding allocation regression.

Hostile cases:
- change component `Int` → `Float`: layout/fingerprint changes and stale compiled product is not reused;
- rename an unrelated method body: data layout remains stable;
- user `@data class Legacy` and `data Modern` coexist through separate paths;
- a data declaration imported under alias still navigates to canonical declaration;
- physical representation checks prove no label/generic arrays per value.

Required evidence:
1. cold/incremental equivalence tests over `DataDeclarationProduct` and affected lowering/layout fingerprint.
2. source-index/LSP navigation test for data declaration and component.
3. allocation/layout assertions.
4. affected-crate tests in smallest-first order.
5. final negative searches.
6. format/check/test/clippy gates listed below.

Do not run yet:
- nothing within P1 remains deferred after final gate; P2/P3 are scope exclusions, not forgotten P1 tests.

Escalate immediately if:
- artifact/lowering persistence uses raw `TypeId`;
- an LSP fix requires a second data resolver;
- a performance invariant can only be met by changing PDR semantics.

Checkpoint completion:
- [ ] incremental equivalence passes
- [ ] source-tooling identity passes
- [ ] legacy/new authority isolation proven
- [ ] performance/allocation assertions pass
- [ ] final broad gates pass
- [ ] negative gates pass
- [ ] no deferred P1 evidence remains
- [ ] state file has no INCIDENT
- [ ] PDR/status/spec landing updates prepared
- [ ] P1 is COMPLETE

Suggested commit grouping:
- `fix(incremental): fingerprint data semantic and layout products`
- `feat(tooling): surface data declaration and component identities`
- `docs(lang005): land data PDR and P1 state`
- `test(lang005): close first-class data representation gates`

### Task 23 — Close layout/semantic fingerprint and artifact publication boundaries

Purpose:
Ensure incremental compilation never reuses stale logical or physical data products and never persists store-local semantic handles.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate

Owned files and symbols:
- semantic fingerprint/query inputs from C1.
- `modules/semantic_lowering.rs` fingerprints if lowering is cached.
- compiled artifact/executable semantics persistence path actually used by current compiler.
- runtime typing metadata loader references used by `DataConstructionLoweringSpec`.

Inspect before editing:
- exact compiled-program retention path.
- existing enum executable semantics persistence.
- stable metadata type references.

Dependencies:
- C4.

Source of truth:
- semantic data fingerprint separate from physical layout fingerprint.

Required invalidation:
- component type/order/label;
- data generic signature/constraint;
- representation-relevant exact applied type at construction site.

Must **not** invalidate data layout for:
- future method body edits;
- unrelated module changes.

Changes:
- add deterministic layout spec fingerprint;
- ensure no raw `TypeId` crosses durable artifact boundary;
- add cold/edit tests.


Edit operations:
1. [ ] OPEN semantic query fingerprinting and record the DataDeclaration input/product fingerprints established in C1.
2. [ ] ADD a separate deterministic physical `ProductLayoutSpec` fingerprint wherever executable lowering/artifacts cache layout assumptions.
3. [ ] TRACE the exact compiled artifact/executable-semantics retention path and replace any accidental `TypeId` carriage with existing stable metadata refs.
4. [ ] ADD an edit test changing component Int→Float and asserting semantic + lowering/layout products invalidate.
5. [ ] ADD a comparator edit changing an unrelated method body and asserting data layout fingerprint remains stable.
6. [ ] COMPARE cold and incremental DataInfo/component IDs/type forms, not only diagnostics text.
Testing classification:
- focused incremental evidence required at C5.

### Task 24 — Finish source tooling and explicitly isolate legacy `@data`

Purpose:
Ensure user-facing navigation consumes semantic identities and the old decorator cannot become a hidden implementation path.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files/symbols:
- source-index adapters from Task 8.
- `phalcom-lsp` hover/definition/completion consumers that already read source index.
- `compiler/attributes.rs::DataExpander` / `derive_data` only for isolation assertions/comments.
- docs syntax page where first-class declarations are listed.

Dependencies:
- stable DataInfo/source targets.

Source of truth:
- semantic source index.

Changes:
- add data/component token classification based on occurrences;
- go-to-definition/hover use same semantic target;
- document legacy `@data` as compatibility distinct from first-class `data`;
- no changes to legacy derivation behavior.

Negative gates:
- new `compiler/lib/data_decl.rs` must contain no reference to `DataExpander`, `derive_data`, or `BuiltinAttr::Data`;
- first-class `DataDef` must not enter `expand_class_attributes`.


Edit operations:
1. [ ] OPEN source-index and LSP consumers for definition/hover/semantic tokens; confirm they consume compiler semantic occurrences.
2. [ ] ADD data declaration/component presentation using those targets, with no new LSP resolver.
3. [ ] ADD an alias-import navigation regression and component hover regression.
4. [ ] ADD a comment/documentation boundary in legacy `DataExpander` explaining it is `@data class` compatibility only.
5. [ ] RUN negative searches proving `data_decl.rs` and `data_semantics.rs` do not call `derive_data`.
6. [ ] UPDATE current language syntax docs/PDR references needed by shipped first-class data.
Testing classification:
- LSP/source-index integration smoke at C5.

### Task 25 — Performance/allocation qualification and delivery gates

Purpose:
Prove P1 has actually delivered the promised materialized representation and repository compatibility.

Risk:
- Semantic: MEDIUM
- Implementation fanout: tests/docs

Owned files and symbols:
- focused data/product/ADT runtime tests added by prior checkpoints;
- existing `value/repr.rs` size tests;
- LANG005 implementation-state document;
- PDR/status/docs landing files.

Inspect before editing:
- existing heap `live_count` test conventions;
- existing product empty-allocation test;
- existing ADT construction/GC test harness.

Dependencies:
- Tasks 23–24 and all prior checkpoints COMPLETE.

Source of truth:
- deterministic runtime allocation/word-count/identity assertions plus the checkpoint evidence ledger; microbenchmark timing is supplementary only.

Required deterministic representation assertions:
- `size_of::<Value>() == 16`.
- `Signal<Connected>()` does not increase heap live count.
- two exact Signal specializations differ.
- positive-arity data allocates one DataObject, not `InstanceObject`.
- DataObject contains no component label vector.
- layout for two Int components consumes two payload words, not four `Value` words.
- mixed `Int + Dynamic + Bool` layout is `1 + 2 + 1` words.
- general enum `Ok(Int)` payload consumes one scalar word plus object/header, not one 16-byte Value slot.
- GC reaches a heap child stored in a universal Value component.
- no `InstanceObject::new` occurs on the data construction path.

Add microbenchmarks only if the repo already has an appropriate harness. Do not introduce a benchmark framework solely for P1; deterministic allocation/word-count tests are stronger release gates for this checkpoint.


Edit operations:
1. [ ] ADD/CONSOLIDATE deterministic allocation and word-count assertions listed above into focused product/data/ADT tests.
2. [ ] RUN focused data/product tests first, then affected crate suites, then workspace delivery gates in the documented order.
3. [ ] RUN every final negative search and record intentional legacy tuple/record/@data exceptions.
4. [ ] RUN format, workspace check, workspace tests, and clippy; record exact PASS/FAIL in the state file.
5. [ ] COPY PDR-0035 into repository path and update `docs/pdr/STATUS.md` in the same commit when implementation work is actually landing.
6. [ ] MARK P1 COMPLETE only after the deferred-evidence audit has no remaining P1 item and no INCIDENT remains.
Testing classification:
- final P1 evidence.

---

# Failure protocol for all checkpoints

When required evidence fails, the checkpoint becomes `INCIDENT`, not “mostly complete”.

Before changing additional code:

1. record exact command and failing test/check;
2. trace the direct path from fixture → semantic/lowering/runtime owner → failure;
3. find a nearby passing comparator;
4. classify:
   - `PRODUCT`;
   - `FIXTURE`;
   - `DEPENDENCY/PUBLICATION`;
   - `BACKEND/HARNESS`;
   - `BASELINE`;
   - `PLAN DRIFT`;
5. state the narrow subsystem/symbol allowed to change;
6. reject broad repairs explicitly.

Do not:
- turn an error/unknown into Dynamic;
- restore a class-expansion implementation;
- special-case LSP;
- compare declarations by string name;
- expose layout to fix reflection;
- weaken GADT/exact-case tests after enum storage migration;
- disable GC assertions;
- increase `Value` size to make data encoding easier.

Later checkpoints may not build on an unresolved incident.

---

# Implementation state file protocol

Maintain:

```text
docs/implementation/LANG005-data-impl-traits/
  C1-optimized-immutable-products/
    implementation-state.md
```

If the repository establishes a different LANG005 directory before implementation begins, use that canonical path and record the move.

After every checkpoint append/update:

```markdown
# LANG005.C1 implementation state

## Repository revision
- base:
- current:

## Checkpoint status
- C0 — COMPLETE | INCIDENT | NOT STARTED
...

## Established invariants
- I-P1-01: ...

## Decisions
- D-P1-01: ...

## Evidence ledger
| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Negative/deletion gates
- command → result → expected remaining occurrences

## Deferred gates
- command/evidence → destination checkpoint

## Unexpected repository findings
- concise fact + code anchor

## Active incident
None.

## Next resume action
Begin C<N> Task <N>.
```

Do not store chain-of-thought or implementation diaries. Store facts, decisions, code anchors and evidence.

---

# Final delivery

## Checkpoint evidence summary

At delivery, fill this table from actual executed evidence:

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | first-class parsed/bound data declaration identity | AST + modules + keyword tests | NOT RUN by this plan |
| C1 | canonical data semantic product and incremental typing | focused semantic + cold/incremental tests | NOT RUN by this plan |
| C2 | compact shared product storage and value semantics | layout/GC/singleton/relation tests | NOT RUN by this plan |
| C3 | source-to-runtime data construction/projection | lowering/disassembly/end-to-end tests | NOT RUN by this plan |
| C4 | enum payload convergence without semantic regression | core ADT + semantic ADT + GC tests | NOT RUN by this plan |
| C5 | incremental/tooling/performance/delivery closure | source/LSP + allocation + broad gates | NOT RUN by this plan |

No row may be changed to COMPLETE until its named evidence has actually passed.

## Final broad gates

Run smallest-first after C5 focused evidence is green:

```bash
cargo +stable fmt --all -- --check
```

Proves:
- repository formatting compliance.

Does not prove:
- semantic data correctness.

```bash
cargo +stable check --workspace --all-targets
```

Proves:
- exhaustive enum/API caller migration compiles throughout the workspace;
- test/bench/binary targets see the new AST/query/heap variants.

```bash
cargo +stable test -p phalcom-ast
cargo +stable test -p phalcom-modules
cargo +stable test -p phalcom-semantic
cargo +stable test -p phalcom-core
cargo +stable test -p phalcom-lsp
```

Proves:
- affected ownership layers remain compatible beyond focused tests.
- Each crate suite should be run once here after focused checkpoint evidence, not repeatedly after every task.

```bash
cargo +stable test --workspace --all-targets
```

Proves:
- cross-workspace delivery compatibility after all affected crates pass.

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
```

Proves:
- no warning-level integration debt remains in Rust targets.

If the repository's current CI uses explicit feature matrices or native-metadata validation commands, add the same commands recorded by CI at implementation time. Do not invent a feature matrix from this plan.

## Final negative/deletion gates

Run:

```bash
rg 'TypeData::Data' phalcom-semantic phalcom-core
```

Expected: zero production hits.

```bash
rg 'DataExpander|derive_data' \
  phalcom-core/src/compiler/lib/data_decl.rs \
  phalcom-semantic/src/data_semantics.rs \
  phalcom-semantic/src/checker/data_declaration.rs
```

Expected: zero hits. Legacy occurrences in `compiler/attributes.rs` are intentional.

```bash
rg 'InstanceObject::new|Bytecode::NewInstance|Bytecode::SetField' \
  phalcom-core/src/compiler/lib/data_decl.rs \
  phalcom-core/src/product \
  phalcom-core/src/heap/data.rs
```

Expected: zero hits in the new data path.

```bash
rg 'payload: Box<\[Value\]>' phalcom-core/src/heap/adt.rs
```

Expected: zero after C4.

```bash
rg 'labels: Box<\[Symbol\]>' phalcom-core/src/heap/data.rs phalcom-core/src/product
```

Expected: zero. Tuple/Record legacy representations may still intentionally contain label arrays until C1.P3.

```bash
rg 'generic_arguments|type_arguments: Box' phalcom-core/src/heap/data.rs
```

Expected: zero per-instance generic argument arrays.

```bash
rg 'Statement::Data' phalcom-core/src/compiler/attributes.rs
```

Expected: zero first-class-data routing through the legacy attribute expander.

Also inspect every remaining `Object::AdtCase` match after C4 and confirm it consumes `ProductStorage`, not an old `Box<[Value]>`.

## Deferred-evidence audit

Before P1 is declared complete:

```text
No deferred P1 test/check may remain without:
- having executed successfully;
- being explicitly removed from P1 scope with a written justification;
- or being recorded as a known release blocker/INCIDENT.
```

The following are **scope exclusions**, not deferred P1 evidence:

- escape analysis/scalar replacement;
- interprocedural unboxing;
- nested product flattening;
- anonymous tuple/record representation convergence;
- specialized inline collections;
- exhaustive Dynamic box/unbox/re-narrow qualification;
- full runtime generic-environment optimization;
- `impl`/trait behavior;
- enum source syntax migration.

Those have named later LANG005 owners.

## Staged commit groups

Recommended order:

1. `feat(ast): add first-class data declaration syntax`
2. `feat(modules): bind data declarations canonically`
3. `feat(semantic): publish canonical data products`
4. `feat(semantic): type data construction and component projection`
5. `feat(runtime): add compact shared product layout storage`
6. `feat(runtime): add exact data descriptors and singleton values`
7. `feat(lowering): lower first-class data construction`
8. `refactor(adt): migrate general variant payloads to product storage`
9. `feat(tooling): publish data/component source identities`
10. `test(lang005): close data representation and incremental gates`
11. `docs(lang005): land PDR-0035 and implementation state`

Combine adjacent commits if the actual patch is cleaner, but do not mix C4 enum storage migration into the initial AST/semantic commit; that separation is valuable for diagnosis.

## Known scope exclusions

This plan does not implement:

- LANG005.C1.P2 representation-aware optimizer/virtual aggregates/scalar replacement;
- LANG005.C1.P3 anonymous product convergence or exhaustive Dynamic/reification qualification;
- inherent `impl`;
- enum variants-only syntax or moving enum behavior into impl blocks;
- traits or trait conformances;
- associated types/projections;
- trait generic evidence;
- final full reflection API;
- removal of legacy `@data`;
- final public derivation policy for `toString`/update helpers;
- stable C/FFI layout;
- inline `List<Data>` representation;
- JIT optimization.

## PDR landing requirements

When PDR-0035 is copied into `docs/pdr/`:

- add it as `docs/pdr/0035-data-is-a-nominal-transparent-immutable-value-product.md`;
- update `docs/pdr/STATUS.md` in the same commit;
- keep status `Accepted`;
- mark Shipped as unimplemented until this plan has actual completion evidence;
- update syntax/type docs touched by the implemented behavior rather than leaving PDR as the sole user-facing specification.

## State-file completion requirements

At P1 completion the implementation state must contain:

- exact final repository revision;
- all C0–C5 statuses as COMPLETE;
- final established semantic/runtime invariants;
- final chosen product-layout/storage API symbols;
- final exact-runtime-data descriptor identity mechanism;
- all focused and broad commands with PASS results;
- negative search results and intentional exceptions;
- no unresolved INCIDENT;
- no forgotten deferred P1 gates;
- explicit next action: begin LANG005.C1.P2.

## Release-complete criteria

LANG005.C1.P1 is complete only when:

- every checkpoint C0–C5 is COMPLETE;
- all checkpoint semantic evidence passes;
- all hostile cases pass;
- `data` is a real nominal declaration across parser/modules/semantic/runtime/tooling;
- components are first-class immutable semantic components, not fake fields/getters;
- generic/phantom type identity is preserved semantically;
- nullary exact data applications materialize without heap allocation;
- materialized positive-arity data uses compact ProductStorage and no per-instance labels/generic arrays;
- data language-level `===`, `==`, and `hash` do not reveal backing allocation identity;
- general enum payloads use the same product storage while all current ADT/GADT/behavior tests pass;
- `NativeOption` remains valid and immediate;
- precise GC tests pass;
- incremental cold/edit equivalence passes;
- source-index/LSP identity agrees with compiler semantics;
- all required obsolete-path negative gates pass;
- final format/check/test/clippy gates pass;
- PDR-0035 and STATUS tracker are synchronized when committed;
- no unresolved state-file incident exists.

At that point C1.P2 may rely on one crucial invariant: **a data/enum product can remain purely logical in the compiler and can be materialized through one representation-independent ProductLayout/ProductStorage boundary only when execution requires a runtime value.**
