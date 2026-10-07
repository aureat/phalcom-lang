# Whole-Lane Rest Parameter Inference Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement whole-lane rest parameter annotations (`*args: *P`, `**args: **P`, `***args: ***P`) and single-authority parameter contracts in Telos, enabling exact generic rest-call inference for `GoalQuery.of` while preserving `*args: T` as an element contract and eliminating TypePack/Pack conflation.

**Architecture:** Introduce `ParameterAnnotationSyntax` in AST specifically for rest-parameter annotations without global expansion in type positions; replace disparate parameter types with canonical `ParameterContract` (`Fixed` vs `Rest { mode, contract: Unconstrained | Element | Lane }`) and derived `body_type` Pack values; perform single-pass source-order argument checking in `call.rs` for both generic and non-generic calls, relating captured lanes to declared lane contracts via structural equivalence (for generic variables) or pointwise element conformance (for concrete shapes); introduce `ConstraintOrigin::RestLane` to establish sound causality for empty/N-ary lane inference.

**Tech Stack:** Rust (2021 edition), `phalcom-ast` (recursive descent parser), `phalcom-semantic` (type checking, inference engine, semantic database, diagnostics).

---

### File Structure & Responsibility Map

| File | Subsystem | Responsibility |
|---|---|---|
| `phalcom-ast/src/ast.rs` | AST | Introduce `ParameterAnnotationSyntax` (`Type` vs `RestLane(TypePackExpansionSyntax)`) on `ParameterDef`. |
| `phalcom-ast/src/parser.rs` | Parser | Parse contextual `*P`, `**P`, `***P` after `:` in parameter lists into `ParameterAnnotationSyntax::RestLane`. |
| `phalcom-ast/tests/parser.rs` | Tests | Unit tests for positive parsing, element preservation, lane mode preservation, and invalid general expansion rejection inside classes. |
| `phalcom-semantic/src/signature.rs` | Semantic Signature | Define `ParameterContract` and `RestValueContract`; update `CallableParameterSemantic` to single contract authority with derived `body_type`, preserving `rest` and `declared_type` compatibility. |
| `phalcom-semantic/src/dispatch.rs` | Dispatch & Call Signatures | Define `CallableParameterContract` and `RestValueContractKnowledge` on `CallableParameter`. |
| `phalcom-semantic/src/checker/declaration_signature.rs` | Signature Lowering | Lower parameter annotations into `ParameterContract` reusing `store.project_positionals/labeled/complete`; reject `TypePack` on element contracts with `KindExpectedType`. Preserve parameter external labels. Update `project_semantic_signature`. |
| `phalcom-semantic/src/checker/context.rs` | Local Scope | Bind callable parameters in method bodies using canonical derived `body_type` (proper `Pack` values). |
| `phalcom-semantic/src/checker/inference.rs` | Solver | Add `ConstraintOrigin::RestLane`; support whole-pack variable equivalence for empty and non-empty lanes without TypePack subtyping. |
| `phalcom-semantic/src/checker/call.rs` | Call Checker | Single-pass source-order argument analysis, grouping residual arguments into `TypePack([])` or `TypePack([arg1, ...])` and applying `Equivalent` or pointwise conformance across both generic and non-generic call paths. Full `RestMode::Complete` lane assembly and `apply_concrete_lane_conformance` implementation. Preserve data constructor shape handling. |
| `phalcom-semantic/src/db/fingerprint.rs` | Incremental DB | Hash canonical `ParameterContract` in callable signature fingerprinting so contract edits invalidate dependent call sites. Update parameter annotation range access to `a.range()`. |
| `phalcom-semantic/src/checker/declaration.rs` | Inheritance & Override | Compare caller-facing `ParameterContract` in override checks (rejecting Element vs Lane mismatches). |
| `phalcom-semantic/tests/semantic/capabilities/generics.rs` | Acceptance Test | Update `GoalQuery.of(*goals: *Goals)` and verify empty/heterogeneous rest inference. |
| `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs` | Regression Suite | Implement all 15 mandatory anti-regressions and acceptance matrix from `packs-correction.md`. |

---

### Task 1: AST Carrier & Parser for Rest Lane Annotations

**Files:**
- Modify: `phalcom-ast/src/ast.rs`
- Modify: `phalcom-ast/src/parser.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/checker/algebraic_declaration.rs`
- Test: `phalcom-ast/tests/parser.rs`

- [ ] **Step 1: Write the failing parser tests**

Add to `phalcom-ast/tests/parser.rs`:
```rust
#[test]
fn rest_lane_annotations_parse_contextually_on_parameters() {
    use phalcom_ast::ast::{ClassMember, ExpansionMode, ParameterAnnotationSyntax, RestMode, Statement};

    let src = "class C {\n    @class f<*P, **K, ***C>(*a: *P, **b: **K, ***c: ***C, *d: Int, *e: P) {}\n}\n";
    let prog = parse_source(src, 0).expect("source should parse");
    let Statement::Class(c) = &prog.statements[0] else { panic!("expected class"); };
    let ClassMember::Method(m) = &c.members[0] else { panic!("expected method"); };

    assert_eq!(m.params.len(), 5);

    // *a: *P
    assert_eq!(m.params[0].rest_mode, RestMode::Positional);
    match &m.params[0].annotation {
        Some(ParameterAnnotationSyntax::RestLane(lane)) => {
            assert_eq!(lane.mode, ExpansionMode::Positional);
            assert_eq!(lane.operand.origin_symbol_ref().unwrap().root, "P");
        }
        other => panic!("expected RestLane, got {other:?}"),
    }

    // **b: **K
    assert_eq!(m.params[1].rest_mode, RestMode::Labeled);
    match &m.params[1].annotation {
        Some(ParameterAnnotationSyntax::RestLane(lane)) => {
            assert_eq!(lane.mode, ExpansionMode::Labeled);
            assert_eq!(lane.operand.origin_symbol_ref().unwrap().root, "K");
        }
        other => panic!("expected RestLane, got {other:?}"),
    }

    // ***c: ***C
    assert_eq!(m.params[2].rest_mode, RestMode::Complete);
    match &m.params[2].annotation {
        Some(ParameterAnnotationSyntax::RestLane(lane)) => {
            assert_eq!(lane.mode, ExpansionMode::Complete);
            assert_eq!(lane.operand.origin_symbol_ref().unwrap().root, "C");
        }
        other => panic!("expected RestLane, got {other:?}"),
    }

    // *d: Int (remains element type)
    assert_eq!(m.params[3].rest_mode, RestMode::Positional);
    match &m.params[3].annotation {
        Some(ParameterAnnotationSyntax::Type(ty)) => {
            assert_eq!(ty.origin_symbol_ref().unwrap().root, "Int");
        }
        other => panic!("expected Type, got {other:?}"),
    }

    // *e: P (remains plain type reference, not rewritten by parser)
    assert_eq!(m.params[4].rest_mode, RestMode::Positional);
    match &m.params[4].annotation {
        Some(ParameterAnnotationSyntax::Type(ty)) => {
            assert_eq!(ty.origin_symbol_ref().unwrap().root, "P");
        }
        other => panic!("expected Type, got {other:?}"),
    }
}

#[test]
fn general_type_positions_do_not_parse_bare_lane_expansions() {
    let src1 = "class C { @class run() { let x: *P = 1 } }\n";
    assert!(parse_source(src1, 0).is_err());

    let src2 = "class C { @class f() -> *P {} }\n";
    assert!(parse_source(src2, 0).is_err());
}
```

- [ ] **Step 2: Run parser test to verify failure**

Run: `RUSTFLAGS='' cargo test -p phalcom-ast --test parser rest_lane_annotations_parse_contextually_on_parameters`
Expected: FAIL with compilation error: `ParameterAnnotationSyntax` not found.

- [ ] **Step 3: Implement `ParameterAnnotationSyntax` and contextual parser support**

1. In `phalcom-ast/src/ast.rs`:
```rust
/// Parameter annotation syntax distinguishing ordinary element types from rest lane contracts.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterAnnotationSyntax {
    Type(TypeAnnotation),
    RestLane(TypePackExpansionSyntax),
}

impl ParameterAnnotationSyntax {
    pub fn range(&self) -> SourceRange {
        match self {
            Self::Type(ty) => ty.range,
            Self::RestLane(lane) => lane.range,
        }
    }

    pub fn as_type(&self) -> Option<&TypeAnnotation> {
        match self {
            Self::Type(ty) => Some(ty),
            Self::RestLane(_) => None,
        }
    }

    pub fn as_rest_lane(&self) -> Option<&TypePackExpansionSyntax> {
        match self {
            Self::Type(_) => None,
            Self::RestLane(lane) => Some(lane),
        }
    }
}
```
Update `ParameterDef`:
```rust
pub struct ParameterDef {
    pub name: String,
    pub name_range: SourceRange,
    pub label: Option<String>,
    pub label_range: Option<SourceRange>,
    pub rest_mode: RestMode,
    pub annotation: Option<ParameterAnnotationSyntax>,
    pub range: SourceRange,
}

impl ParameterDef {
    pub fn is_rest(&self) -> bool {
        self.rest_mode != RestMode::None
    }

    pub fn type_annotation(&self) -> Option<&TypeAnnotation> {
        self.annotation.as_ref().and_then(|a| a.as_type())
    }

    pub fn rest_lane_annotation(&self) -> Option<&TypePackExpansionSyntax> {
        self.annotation.as_ref().and_then(|a| a.as_rest_lane())
    }
}
```
2. In `phalcom-ast/src/parser.rs`:
In `parse_selector_params` around lines 6495, 6540, 6570:
```rust
let annotation = if self.eat(&Token::Colon) {
    if let Some(mode) = self.expansion_mode_at_cursor() {
        let op_start = self.cur_start();
        self.advance();
        let operator_range = (op_start..self.prev_end).into();
        let operand = self.parse_type_annotation()?;
        let range = (op_start..self.prev_end).into();
        Some(ParameterAnnotationSyntax::RestLane(TypePackExpansionSyntax {
            mode,
            operator_range,
            operand,
            range,
        }))
    } else {
        Some(ParameterAnnotationSyntax::Type(self.parse_type_annotation()?))
    }
} else {
    None
};
```
Wrap parsed `TypeAnnotation` in `ParameterAnnotationSyntax::Type` at other `ParameterDef` construction sites (setters, index methods).
3. In `phalcom-semantic/src/session.rs` lines 8149, 9721 and `phalcom-semantic/src/checker/algebraic_declaration.rs` lines 525, 816: update direct `parameter.annotation.as_ref()` expecting `&TypeAnnotation` to call `parameter.type_annotation()`.

- [ ] **Step 4: Run parser tests to verify PASS**

Run: `RUSTFLAGS='' cargo test -p phalcom-ast --test parser rest_lane_annotations_parse_contextually_on_parameters`
Expected: PASS.
Run: `RUSTFLAGS='' cargo test -p phalcom-ast --test parser general_type_positions_do_not_parse_bare_lane_expansions`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-ast/src/ast.rs phalcom-ast/src/parser.rs phalcom-ast/tests/parser.rs phalcom-semantic/src/session.rs phalcom-semantic/src/checker/algebraic_declaration.rs
git commit -m "feat(ast): introduce contextual rest lane parameter annotations"
```

---

### Task 2: Canonical Parameter Contract Representation in Semantic Layer

**Files:**
- Modify: `phalcom-semantic/src/signature.rs`
- Modify: `phalcom-semantic/src/dispatch.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/pack_types.rs`

- [ ] **Step 1: Write unit test for ParameterContract and body-capture Pack derivation**

Add to `phalcom-semantic/tests/semantic/foundations/pack_types.rs`:
```rust
#[test]
fn parameter_contract_derives_canonical_body_pack_types() {
    use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, TypeTerm};
    use phalcom_semantic::signature::{ParameterContract, RestValueContract};
    use phalcom_semantic::types::parameter::TypeParameterOwner;

    let mut store = TypeStore::new();
    let int_ty = store.intern(TypeData::Nominal { declaration: DeclarationId::new(ModuleId::universe(), "Int".into()) });
    let elem_fact = DeclaredTypeFact::known(TypeTerm::Canonical(int_ty), DeclaredTypeBasis::DeclarationSemantics);

    // *args: Int derives Pack with Typed(Int) positional remainder
    let element_contract = ParameterContract::Rest {
        mode: RestMode::Positional,
        contract: RestValueContract::Element(elem_fact),
    };
    let body_ty = element_contract.derive_body_type(&mut store);
    let TypeData::Pack(pack) = store.get(body_ty.canonical_type().unwrap()) else { panic!("expected Pack"); };
    assert_eq!(pack.positional_remainder, AnonymousTupleRemainder::Typed(int_ty));
    assert_eq!(pack.labeled_remainder, AnonymousTupleRemainder::None);

    // *args: *P derives PackFromPacks(P, EmptyLabeled)
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe(), "Test".into()));
    let p_id = store.intern_type_parameter(TypeParameterData::new(owner, 0, "P", KindId::TYPE_PACK).with_mode(GenericParameterMode::PositionalPack));
    let p_form = store.parameter_form(p_id);
    let lane_contract = ParameterContract::Rest {
        mode: RestMode::Positional,
        contract: RestValueContract::Lane(p_form),
    };
    let lane_body_ty = lane_contract.derive_body_type(&mut store);
    let TypeData::PackFromPacks { positionals, labeled } = store.get(lane_body_ty.canonical_type().unwrap()) else { panic!("expected PackFromPacks"); };
    assert_eq!(*positionals, p_form);
    let TypeData::LabeledTypePack(empty_labeled) = store.get(*labeled) else { panic!("expected empty labeled"); };
    assert!(empty_labeled.is_empty());

    // *args (unconstrained) derives Pack with Unconstrained positional remainder
    let unconstrained_contract = ParameterContract::Rest {
        mode: RestMode::Positional,
        contract: RestValueContract::Unconstrained,
    };
    let unconstrained_body_ty = unconstrained_contract.derive_body_type(&mut store);
    let TypeData::Pack(u_pack) = store.get(unconstrained_body_ty.canonical_type().unwrap()) else { panic!("expected Pack"); };
    assert_eq!(u_pack.positional_remainder, AnonymousTupleRemainder::Unconstrained);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic parameter_contract_derives_canonical_body_pack_types`
Expected: FAIL with compilation error: `ParameterContract` not found.

- [ ] **Step 3: Implement `ParameterContract` and `derive_body_type`**

1. In `phalcom-semantic/src/signature.rs`:
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterContract {
    Fixed(DeclaredTypeFact),
    Rest {
        mode: RestMode,
        contract: RestValueContract,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestValueContract {
    Unconstrained,
    Element(DeclaredTypeFact),
    Lane(TypeId),
}

impl ParameterContract {
    pub fn rest_mode(&self) -> RestMode {
        match self {
            Self::Fixed(_) => RestMode::None,
            Self::Rest { mode, .. } => *mode,
        }
    }

    pub fn declared_type(&self) -> DeclaredTypeFact {
        match self {
            Self::Fixed(fact) => fact.clone(),
            Self::Rest { contract: RestValueContract::Element(fact), .. } => fact.clone(),
            Self::Rest { contract: RestValueContract::Unconstrained, .. } => {
                DeclaredTypeFact::unknown(UnknownReason::OmittedParameterAnnotation)
            }
            Self::Rest { contract: RestValueContract::Lane(_), .. } => {
                DeclaredTypeFact::unknown(UnknownReason::NoTypeEvidence)
            }
        }
    }

    pub fn derive_body_type(&self, store: &mut TypeStore) -> DeclaredTypeFact {
        match self {
            Self::Fixed(fact) => fact.clone(),
            Self::Rest { mode, contract } => match contract {
                RestValueContract::Unconstrained => {
                    let pack = match mode {
                        RestMode::Positional => PackType {
                            elements: Box::default(),
                            positional_remainder: AnonymousTupleRemainder::Unconstrained,
                            labeled_remainder: AnonymousTupleRemainder::None,
                        },
                        RestMode::Labeled => PackType {
                            elements: Box::default(),
                            positional_remainder: AnonymousTupleRemainder::None,
                            labeled_remainder: AnonymousTupleRemainder::Unconstrained,
                        },
                        RestMode::Complete => PackType {
                            elements: Box::default(),
                            positional_remainder: AnonymousTupleRemainder::Unconstrained,
                            labeled_remainder: AnonymousTupleRemainder::Unconstrained,
                        },
                        RestMode::None => unreachable!(),
                    };
                    DeclaredTypeFact::known(TypeTerm::Canonical(store.pack(pack)), DeclaredTypeBasis::DeclarationSemantics)
                }
                RestValueContract::Element(fact) => {
                    let elem_ty = fact.canonical_type().unwrap_or_else(|| store.unit());
                    let pack = match mode {
                        RestMode::Positional => PackType {
                            elements: Box::default(),
                            positional_remainder: AnonymousTupleRemainder::Typed(elem_ty),
                            labeled_remainder: AnonymousTupleRemainder::None,
                        },
                        RestMode::Labeled => PackType {
                            elements: Box::default(),
                            positional_remainder: AnonymousTupleRemainder::None,
                            labeled_remainder: AnonymousTupleRemainder::Typed(elem_ty),
                        },
                        RestMode::Complete => PackType {
                            elements: Box::default(),
                            positional_remainder: AnonymousTupleRemainder::Typed(elem_ty),
                            labeled_remainder: AnonymousTupleRemainder::Typed(elem_ty),
                        },
                        RestMode::None => unreachable!(),
                    };
                    DeclaredTypeFact::known(TypeTerm::Canonical(store.pack(pack)), DeclaredTypeBasis::DeclarationSemantics)
                }
                RestValueContract::Lane(lane_id) => {
                    match mode {
                        RestMode::Positional => {
                            let empty_labeled = store.labeled_type_pack(Box::default()).unwrap_or_else(|_| store.unit());
                            match store.pack_from_packs(*lane_id, empty_labeled) {
                                Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                                Err(_) => DeclaredTypeFact::unknown(UnknownReason::PackFormation),
                            }
                        }
                        RestMode::Labeled => {
                            let empty_pos = store.type_pack(Box::default()).unwrap_or_else(|_| store.unit());
                            match store.pack_from_packs(empty_pos, *lane_id) {
                                Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                                Err(_) => DeclaredTypeFact::unknown(UnknownReason::PackFormation),
                            }
                        }
                        RestMode::Complete => {
                            let pos = store.project_positionals(*lane_id).unwrap_or_else(|_| store.unit());
                            let lab = store.project_labeled(*lane_id).unwrap_or_else(|_| store.unit());
                            match store.pack_from_packs(pos, lab) {
                                Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                                Err(_) => DeclaredTypeFact::unknown(UnknownReason::PackFormation),
                            }
                        }
                        RestMode::None => unreachable!(),
                    }
                }
            },
        }
    }
}
```
Update `CallableParameterSemantic`:
```rust
pub struct CallableParameterSemantic {
    pub id: CallableParameterId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestMode,
    pub declared_type: DeclaredTypeFact,
    pub contract: ParameterContract,
    pub source: Option<SemanticSourceSpan>,
}

impl CallableParameterSemantic {
    pub fn new(id: CallableParameterId, local_name: impl Into<Box<str>>, declared_type: DeclaredTypeFact) -> Self {
        Self {
            id,
            local_name: local_name.into(),
            external_label: None,
            rest: RestMode::None,
            contract: ParameterContract::Fixed(declared_type.clone()),
            declared_type,
            source: None,
        }
    }

    pub fn with_contract(mut self, contract: ParameterContract) -> Self {
        self.rest = contract.rest_mode();
        self.declared_type = contract.declared_type();
        self.contract = contract;
        self
    }

    pub fn with_label(mut self, label: impl Into<Box<str>>) -> Self {
        self.external_label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: RestMode) -> Self {
        self.rest = rest;
        self
    }

    pub fn with_source(mut self, source: SemanticSourceSpan) -> Self {
        self.source = Some(source);
        self
    }
}
```
2. In `phalcom-semantic/src/dispatch.rs`:
Define `CallableParameterContract` and `RestValueContractKnowledge` mirroring `ParameterContract` with `TypeKnowledge`.
Update `CallableParameter`:
```rust
pub struct CallableParameter {
    pub external_label: Option<String>,
    pub local_name: String,
    pub ty: TypeKnowledge,
    pub rest: RestMode,
    pub contract: CallableParameterContract,
}

impl CallableParameter {
    pub fn new(local_name: impl Into<String>, ty: TypeKnowledge) -> Self {
        Self {
            external_label: None,
            local_name: local_name.into(),
            contract: CallableParameterContract::Fixed(ty.clone()),
            ty,
            rest: RestMode::None,
        }
    }

    pub fn with_contract(mut self, contract: CallableParameterContract) -> Self {
        self.rest = contract.rest_mode();
        self.ty = contract.ty();
        self.contract = contract;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.external_label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: RestMode) -> Self {
        self.rest = rest;
        self
    }
}
```

- [ ] **Step 4: Run test to verify PASS**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic parameter_contract_derives_canonical_body_pack_types`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/src/signature.rs phalcom-semantic/src/dispatch.rs phalcom-semantic/tests/semantic/foundations/pack_types.rs
git commit -m "feat(semantic): define ParameterContract with canonical derived body Pack types"
```

---

### Task 3: Declaration Signature Lowering & Lane Projection

**Files:**
- Modify: `phalcom-semantic/src/checker/declaration_signature.rs`
- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`

- [ ] **Step 1: Write tests for parameter contract lowering and rejection of bare TypePack in element position**

Add to `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`:
```rust
#[test]
fn rest_parameter_lane_projection_and_kind_enforcement() {
    // 1. Explicit lane *P lowers to Lane(P)
    let f1 = Fixture::new(r#"
class C {
    @class f<*P>(*args: *P) {}
}
"#);
    f1.assert_no_error_diagnostics();

    // 2. Bare *args: P where P: TypePack is rejected with KindExpectedType
    let f2 = Fixture::new(r#"
class C {
    @class f<*P>(*args: P) {}
}
"#);
    assert!(f2.has_diagnostic(DiagnosticCode::KindExpectedType));

    // 3. Concrete tuple lane *args: *(Int, String) projects positionals
    let f3 = Fixture::new(r#"
class C {
    @class f(*args: *(Int, String)) {}
}
"#);
    f3.assert_no_error_diagnostics();
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_parameter_lane_projection_and_kind_enforcement`
Expected: FAIL.

- [ ] **Step 3: Implement parameter contract lowering and `project_semantic_signature`**

In `phalcom-semantic/src/checker/declaration_signature.rs`:
1. In `parameter_fact`:
```rust
fn parameter_fact(
    ctx: &mut CheckingContext<'_>,
    callable: &CallableId,
    index: usize,
    parameter: &ParameterDef,
    resolver: &dyn crate::types::annotation::TypeResolver,
    site: &TypeFormationSite,
    missing: UnknownReason,
) -> CallableParameterSemantic {
    let contract = if parameter.rest_mode == RestMode::None {
        let declared = match &parameter.annotation {
            Some(ParameterAnnotationSyntax::Type(ty)) => annotation_fact(ctx, resolver, site, Some(ty), missing),
            Some(ParameterAnnotationSyntax::RestLane(lane)) => {
                ctx.publish_diagnostics(vec![SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::AnnotationUnsupported,
                    "rest lane annotation is only permitted on rest parameters",
                    lane.range,
                )]);
                DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause)
            }
            None => DeclaredTypeFact::unknown(missing),
        };
        ParameterContract::Fixed(declared)
    } else {
        match &parameter.annotation {
            None => ParameterContract::Rest {
                mode: parameter.rest_mode,
                contract: RestValueContract::Unconstrained,
            },
            Some(ParameterAnnotationSyntax::Type(ann)) => {
                let mut diagnostics = Vec::new();
                let form = crate::types::annotation::resolve_type_form(ctx.store, ctx.declarations, resolver, site, ann, &mut diagnostics);
                let declared = match form {
                    TypeFormResolution::Ready(ty) => {
                        let kind = ctx.store.kind_of(ty);
                        if kind != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                format!("rest element contract requires a proper Type; `{}` has domain {kind:?}", ann.origin_symbol_ref().map(|s| s.root.as_str()).unwrap_or("operand")),
                                ann.range,
                            ));
                            DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause)
                        } else {
                            DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::SourceAnnotation)
                        }
                    }
                    _ => DeclaredTypeFact::unknown(missing),
                };
                ctx.publish_diagnostics(diagnostics);
                ParameterContract::Rest {
                    mode: parameter.rest_mode,
                    contract: RestValueContract::Element(declared),
                }
            }
            Some(ParameterAnnotationSyntax::RestLane(lane)) => {
                let expansion_mode = match lane.mode {
                    ExpansionMode::Positional => RestMode::Positional,
                    ExpansionMode::Labeled => RestMode::Labeled,
                    ExpansionMode::Complete => RestMode::Complete,
                };
                if expansion_mode != parameter.rest_mode {
                    ctx.publish_diagnostics(vec![SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::AnnotationUnsupported,
                        "rest parameter mode and lane annotation mode do not match",
                        lane.range,
                    )]);
                    ParameterContract::Rest {
                        mode: parameter.rest_mode,
                        contract: RestValueContract::Unconstrained,
                    }
                } else {
                    let mut diagnostics = Vec::new();
                    let form = crate::types::annotation::resolve_type_form(ctx.store, ctx.declarations, resolver, site, &lane.operand, &mut diagnostics);
                    let lane_id = match form {
                        TypeFormResolution::Ready(operand_ty) => {
                            let projected = match parameter.rest_mode {
                                RestMode::Positional => ctx.store.project_positionals(operand_ty),
                                RestMode::Labeled => ctx.store.project_labeled(operand_ty),
                                RestMode::Complete => ctx.store.project_complete(operand_ty),
                                RestMode::None => unreachable!(),
                            };
                            match projected {
                                Ok(p) => Some(p),
                                Err(err) => {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        ctx.current_module.clone(),
                                        DiagnosticCode::AnnotationUnsupported,
                                        format!("failed to project {}: {err}", parameter.rest_mode),
                                        lane.range,
                                    ));
                                    None
                                }
                            }
                        }
                        _ => None,
                    };
                    ctx.publish_diagnostics(diagnostics);
                    match lane_id {
                        Some(lane_ty) => ParameterContract::Rest {
                            mode: parameter.rest_mode,
                            contract: RestValueContract::Lane(lane_ty),
                        },
                        None => ParameterContract::Rest {
                            mode: parameter.rest_mode,
                            contract: RestValueContract::Unconstrained,
                        },
                    }
                }
            }
        }
    };

    let mut semantic = CallableParameterSemantic::new(
        CallableParameterId::new(callable.clone(), index as u32),
        parameter.name.clone(),
        contract.declared_type(),
    )
    .with_contract(contract)
    .with_source(crate::diagnostic::SemanticSourceSpan::new(ctx.current_module.clone(), parameter.name_range));

    // CRITICAL: Preserve external label
    if let Some(label) = &parameter.label {
        semantic = semantic.with_label(label.clone());
    }

    semantic
}
```
2. In `project_semantic_signature` (lines 877–888):
```rust
pub(crate) fn project_semantic_signature(signature: &CallableSemanticSignature) -> CallableSignature {
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            let contract_knowledge = match &parameter.contract {
                ParameterContract::Fixed(fact) => CallableParameterContract::Fixed(fact.to_knowledge()),
                ParameterContract::Rest { mode, contract } => CallableParameterContract::Rest {
                    mode: *mode,
                    contract: match contract {
                        RestValueContract::Unconstrained => RestValueContractKnowledge::Unconstrained,
                        RestValueContract::Element(fact) => RestValueContractKnowledge::Element(fact.to_knowledge()),
                        RestValueContract::Lane(lane_id) => RestValueContractKnowledge::Lane(*lane_id),
                    },
                },
            };
            let mut projected = CallableParameter::new(parameter.local_name.to_string(), parameter.declared_type.to_knowledge())
                .with_contract(contract_knowledge)
                .with_rest(parameter.rest);
            if let Some(label) = &parameter.external_label {
                projected = projected.with_label(label.to_string());
            }
            projected
        })
        .collect::<Vec<_>>();
    ...
```

- [ ] **Step 4: Run tests to verify PASS**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_parameter_lane_projection_and_kind_enforcement`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/src/checker/declaration_signature.rs phalcom-semantic/src/source_index/builder.rs phalcom-semantic/tests/semantic/foundations/type_annotations.rs
git commit -m "feat(semantic): lower rest lane parameter contracts using canonical lane projection"
```

---

### Task 4: Local Parameter Binding in Method Bodies

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/body.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`

- [ ] **Step 1: Write test for body-local Pack binding types**

Create `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`:
```rust
use crate::support::fixture::Fixture;
use phalcom_ast::ast::DispatchSide;
use phalcom_semantic::types::store::TypeData;

#[test]
fn rest_capture_local_is_proper_pack_not_element_type() {
    let f = Fixture::new(r#"
class C {
    @class test_elem(*xs: Int) {
        const local_xs = xs
    }
    @class test_lane<*P>(*ys: *P) {
        const local_ys = ys
    }
}
"#);
    f.assert_no_error_diagnostics();
    let callable_elem = f.callable("C", "test_elem", DispatchSide::Class);
    let ty_xs = f.binding(callable_elem, "local_xs").current.ty().expect("inferred local_xs");
    let TypeData::Pack(pack) = f.analysis.snapshot.store.get(ty_xs) else {
        panic!("local_xs must be a Pack, not an Int");
    };
    assert!(matches!(pack.positional_remainder, phalcom_semantic::types::store::AnonymousTupleRemainder::Typed(_)));

    let callable_lane = f.callable("C", "test_lane", DispatchSide::Class);
    let ty_ys = f.binding(callable_lane, "local_ys").current.ty().expect("inferred local_ys");
    assert!(matches!(f.analysis.snapshot.store.get(ty_ys), TypeData::PackFromPacks { .. }));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_capture_local_is_proper_pack_not_element_type`
Expected: FAIL.

- [ ] **Step 3: Update `bind_canonical_callable_parameter` in `checker/context.rs`**

In `checker/context.rs`:
```rust
pub fn bind_canonical_callable_parameter(
    &mut self,
    parameter: &crate::signature::CallableParameterSemantic,
    fallback_range: SourceRange,
) -> BindingDeclarationResult {
    let range = parameter.source.as_ref().map_or(fallback_range, |source| source.range);
    let body_type_fact = parameter.contract.derive_body_type(self.store);
    self.bind_callable_parameter_with_identity(
        parameter.local_name.to_string(),
        body_type_fact.to_knowledge(),
        range,
        crate::checker::causal::CausalInvalidity::Clean,
        Some(parameter.id.clone()),
    )
}
```

- [ ] **Step 4: Run test to verify PASS**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_capture_local_is_proper_pack_not_element_type`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/src/checker/context.rs phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs
git commit -m "feat(checker): bind callable body rest parameters with derived Pack types"
```

---

### Task 5: Constraint Origin `RestLane` & Solver Whole-Lane Unification

**Files:**
- Modify: `phalcom-semantic/src/checker/inference.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/inference.rs`

- [ ] **Step 1: Write solver unit test for whole type-pack variable unification using live solver APIs**

Add to `phalcom-semantic/tests/semantic/foundations/inference.rs`:
```rust
#[test]
fn whole_type_pack_variable_unifies_with_empty_and_concrete_lanes() {
    let mut store = TypeStore::new();
    let int_ty = store.intern(TypeData::Nominal { declaration: DeclarationId::new(ModuleId::universe(), "Int".into()) });
    let str_ty = store.intern(TypeData::Nominal { declaration: DeclarationId::new(ModuleId::universe(), "String".into()) });
    let hier = MapTypeHierarchy::new();
    let control = phalcom_semantic::checker::inference::InferenceSolveControl::unbounded();

    // 1. Unify with empty lane TypePack([])
    let mut session = InferenceSession::new();
    let var_p = session.fresh_variable(KindId::TYPE_PACK);
    let empty_pack = store.type_pack(Box::default()).unwrap();
    session.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(var_p), InferenceTerm::Canonical(empty_pack)),
        ConstraintOrigin::Explicit,
        None,
    );
    session.propagate_with_control(&mut store, &hier, &control).unwrap();
    let outcome = session.finish_root();
    assert!(outcome.is_solved());
    if let InferenceOutcome::Solved(solution) = outcome {
        assert_eq!(session.solved_type_for(&solution, var_p), Some(empty_pack));
    }

    // 2. Unify with heterogeneous lane TypePack([Int, String])
    let mut session2 = InferenceSession::new();
    let var_p2 = session2.fresh_variable(KindId::TYPE_PACK);
    let concrete_p = store.type_pack(vec![int_ty, str_ty].into_boxed_slice()).unwrap();
    session2.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(var_p2), InferenceTerm::Canonical(concrete_p)),
        ConstraintOrigin::Explicit,
        None,
    );
    session2.propagate_with_control(&mut store, &hier, &control).unwrap();
    let outcome2 = session2.finish_root();
    assert!(outcome2.is_solved());
    if let InferenceOutcome::Solved(solution2) = outcome2 {
        assert_eq!(session2.solved_type_for(&solution2, var_p2), Some(concrete_p));
    }
}
```

- [ ] **Step 2: Run test to verify status**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic whole_type_pack_variable_unifies_with_empty_and_concrete_lanes`
Verify that `ConstraintOrigin::RestLane` is added cleanly.

- [ ] **Step 3: Add `ConstraintOrigin::RestLane` in `checker/inference.rs`**

```rust
pub enum ConstraintOrigin {
    ...
    RestLane {
        call: ExpressionId,
        parameter_index: u16,
    },
    ...
}
```

- [ ] **Step 4: Run test to verify PASS**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic whole_type_pack_variable_unifies_with_empty_and_concrete_lanes`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/src/checker/inference.rs phalcom-semantic/tests/semantic/foundations/inference.rs
git commit -m "feat(inference): introduce ConstraintOrigin::RestLane and whole-pack unification"
```

---

### Task 6: Call Checker Whole-Lane Rest Solving

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`

- [ ] **Step 1: Write tests for single-pass evaluation order and lane rest inference**

Add to `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`:
```rust
#[test]
fn rest_argument_analysis_preserves_source_order() {
    let f = Fixture::new(r#"
class Trace {
    @class log(_ n: Int) -> Int { n }
    @class run() {
        Trace.target(Trace.log(1), Trace.log(2), label: Trace.log(3))
    }
    @class target(*args: *P, label: Int) {}
}
"#);
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_concrete_lane_uses_pointwise_value_conformance() {
    let f = Fixture::new(r#"
class Number {}
class Int: Number {}
class C {
    @class accept(*args: *(Number, String)) {}
    @class run() {
        C.accept(Int(), "ok")
    }
}
"#);
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_concrete_lane_rejects_arity_mismatch() {
    let f = Fixture::new(r#"
class C {
    @class accept(*args: *(Int, String)) {}
    @class run() {
        C.accept(1)
    }
}
"#);
    assert!(f.has_diagnostic(DiagnosticCode::CallShapeMismatch));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_concrete_lane_uses_pointwise_value_conformance`
Expected: FAIL.

- [ ] **Step 3: Implement single-pass source-order lane rest solving in `checker/call.rs`**

1. Define `apply_concrete_lane_conformance` covering `Positional`, `Labeled`, and `Complete` rest modes:
```rust
fn apply_concrete_lane_conformance(
    ctx: &mut CheckingContext<'_>,
    lane_args: &[&(usize, ApplicationArgument<'_>, AnalyzedApplicationArgument)],
    lane_id: TypeId,
    mode: RestMode,
    call_range: SourceRange,
    origin: ConstraintOrigin,
    session: &Rc<RefCell<InferenceSession>>,
) {
    match mode {
        RestMode::Positional => {
            let Ok(expected_pos) = ctx.store.project_positionals(lane_id) else { return; };
            let TypeData::TypePack(expected_elems) = ctx.store.get(expected_pos) else { return; };
            if lane_args.len() != expected_elems.len() {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    format!("rest lane expects {} arguments, found {}", expected_elems.len(), lane_args.len()),
                    call_range,
                ));
                return;
            }
            for (idx, (_, _, typed)) in lane_args.iter().enumerate() {
                if let Some(arg_ty) = typed.knowledge.ty() {
                    let expected_elem = expected_elems[idx];
                    session.borrow_mut().add_constraint(
                        InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), InferenceTerm::Canonical(expected_elem)),
                        origin.clone(),
                        None,
                    );
                }
            }
        }
        RestMode::Labeled => {
            let Ok(expected_lab) = ctx.store.project_labeled(lane_id) else { return; };
            let TypeData::LabeledTypePack(expected_elems) = ctx.store.get(expected_lab) else { return; };
            if lane_args.len() != expected_elems.len() {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    format!("labeled rest lane expects {} arguments, found {}", expected_elems.len(), lane_args.len()),
                    call_range,
                ));
                return;
            }
            for (idx, (_, arg, typed)) in lane_args.iter().enumerate() {
                let expected = &expected_elems[idx];
                if arg.label().as_deref() != Some(expected.label.as_ref()) {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::CallShapeMismatch,
                        format!("labeled rest lane expected `{}`, found `{}`", expected.label, arg.label().unwrap_or("_")),
                        call_range,
                    ));
                    continue;
                }
                if let Some(arg_ty) = typed.knowledge.ty() {
                    session.borrow_mut().add_constraint(
                        InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), InferenceTerm::Canonical(expected.element)),
                        origin.clone(),
                        None,
                    );
                }
            }
        }
        RestMode::Complete => {
            let Ok(expected_pos) = ctx.store.project_positionals(lane_id) else { return; };
            let Ok(expected_lab) = ctx.store.project_labeled(lane_id) else { return; };
            let TypeData::TypePack(pos_elems) = ctx.store.get(expected_pos) else { return; };
            let TypeData::LabeledTypePack(lab_elems) = ctx.store.get(expected_lab) else { return; };

            let pos_args: Vec<_> = lane_args.iter().filter(|(_, arg, _)| arg.label().is_none()).copied().collect();
            let lab_args: Vec<_> = lane_args.iter().filter(|(_, arg, _)| arg.label().is_some()).copied().collect();

            if pos_args.len() != pos_elems.len() || lab_args.len() != lab_elems.len() {
                ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    format!("complete rest lane shape mismatch: expected {} positionals and {} labeled", pos_elems.len(), lab_elems.len()),
                    call_range,
                ));
                return;
            }
            for (idx, (_, _, typed)) in pos_args.iter().enumerate() {
                if let Some(arg_ty) = typed.knowledge.ty() {
                    session.borrow_mut().add_constraint(
                        InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), InferenceTerm::Canonical(pos_elems[idx])),
                        origin.clone(),
                        None,
                    );
                }
            }
            for (idx, (_, arg, typed)) in lab_args.iter().enumerate() {
                let expected = &lab_elems[idx];
                if arg.label().as_deref() != Some(expected.label.as_ref()) {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::CallShapeMismatch,
                        format!("complete rest labeled mismatch: expected `{}`, found `{}`", expected.label, arg.label().unwrap_or("_")),
                        call_range,
                    ));
                    continue;
                }
                if let Some(arg_ty) = typed.knowledge.ty() {
                    session.borrow_mut().add_constraint(
                        InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), InferenceTerm::Canonical(expected.element)),
                        origin.clone(),
                        None,
                    );
                }
            }
        }
        RestMode::None => unreachable!(),
    }
}
```

2. In `apply_generic_callable_in_context`:
Preserve data constructor shape handling (`target.data_constructor_shape`).
For non-constructor callable targets:
- Analyze every argument in strict source order exactly once.
- Pointwise constraints for Fixed and Element-rest parameters.
- For Lane-rest parameters: assemble captured lane terms:
  - `Positional`: `InferenceTerm::TypePack(...)`
  - `Labeled`: `InferenceTerm::LabeledTypePack(...)`
  - `Complete`:
    ```rust
    let mut pos_terms = Vec::new();
    let mut lab_terms = Vec::new();
    for (_, arg, typed) in &lane_args {
        let term = typed.knowledge.ty().map(InferenceTerm::Canonical).unwrap_or(InferenceTerm::Canonical(ctx.store.unit()));
        if let Some(label) = arg.label() {
            lab_terms.push(InferenceLabeledTypePackElement { label: label.into(), term });
        } else {
            pos_terms.push(term);
        }
    }
    InferenceTerm::CompleteTypePack {
        positionals: Box::new(InferenceTerm::TypePack(pos_terms.into_boxed_slice())),
        labeled: Box::new(InferenceTerm::LabeledTypePack(lab_terms.into_boxed_slice())),
    }
    ```
  - For generic pack variables: emit `InferenceRelation::Equivalent(captured_lane_term, lane_param_term)`.
  - For concrete lane contracts: call `apply_concrete_lane_conformance`.

3. In `apply_non_generic_callable`:
For parameters with `RestValueContractKnowledge::Lane(lane_id)`:
After source-order argument analysis, invoke `apply_concrete_lane_conformance` against `lane_id`.

- [ ] **Step 4: Run tests to verify PASS**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_concrete_lane_uses_pointwise_value_conformance`
Expected: PASS.
Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_concrete_lane_rejects_arity_mismatch`
Expected: PASS.
Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_argument_analysis_preserves_source_order`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/src/checker/call.rs phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs
git commit -m "feat(checker): implement whole-lane rest call solving and source-order argument checking"
```

---

### Task 7: Signature Fingerprinting, Incremental Recomputation & Override Compatibility

**Files:**
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`

- [ ] **Step 1: Write tests for fingerprint sensitivity and override checking**

Add to `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`:
```rust
#[test]
fn rest_contract_change_invalidates_dependent_calls() {
    let mut f = Fixture::new(r#"
class Callee {
    @class run(*xs: Int) -> Int { 1 }
}
class Caller {
    @class call() { Callee.run(1) }
}
"#);
    f.assert_no_error_diagnostics();
    let initial_fp = f.callable_product_fingerprint("Callee", "run", DispatchSide::Class);

    // Edit Callee to *xs: *P
    f.update_source(r#"
class Callee {
    @class run<*P>(*xs: *P) -> Int { 1 }
}
class Caller {
    @class call() { Callee.run(1) }
}
"#);
    f.assert_no_error_diagnostics();
    let edited_fp = f.callable_product_fingerprint("Callee", "run", DispatchSide::Class);
    assert_ne!(initial_fp, edited_fp, "changing rest contract must alter callable product fingerprint");
}

#[test]
fn override_checking_rejects_element_vs_lane_mismatch() {
    let f = Fixture::new(r#"
class Base {
    m(*args: Animal) {}
}
class Sub: Base {
    m<*P>(*args: *P) {}
}
class Animal {}
"#);
    assert!(f.has_diagnostic(DiagnosticCode::IncompatibleOverride));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic override_checking_rejects_element_vs_lane_mismatch`
Expected: FAIL.

- [ ] **Step 3: Update `fingerprint.rs` and `checker/declaration.rs`**

1. In `fingerprint.rs`:
Update line 128 to `parameter.annotation.as_ref().map(|a| a.range())`.
Hash `ParameterContract`:
```rust
fn hash_parameter_contract(contract: &ParameterContract, hasher: &mut impl Hasher) {
    match contract {
        ParameterContract::Fixed(fact) => {
            0u8.hash(hasher);
            fact.hash(hasher);
        }
        ParameterContract::Rest { mode, contract } => {
            1u8.hash(hasher);
            hash_rest_mode(*mode, hasher);
            match contract {
                RestValueContract::Unconstrained => 0u8.hash(hasher),
                RestValueContract::Element(fact) => {
                    1u8.hash(hasher);
                    fact.hash(hasher);
                }
                RestValueContract::Lane(lane_id) => {
                    2u8.hash(hasher);
                    lane_id.hash(hasher);
                }
            }
        }
    }
}
```
2. In `checker/declaration.rs`:
In method override compatibility:
```rust
let contract_compatible = match (&candidate.contract, &inherited.contract) {
    (CallableParameterContract::Fixed(c), CallableParameterContract::Fixed(i)) => {
        // contravariant
        ctx.check_knowledge_against_type(i, c.canonical_type().unwrap()).is_proven()
    }
    (CallableParameterContract::Rest { mode: m1, contract: c }, CallableParameterContract::Rest { mode: m2, contract: i }) if m1 == m2 => {
        match (c, i) {
            (RestValueContractKnowledge::Element(c_ty), RestValueContractKnowledge::Element(i_ty)) => {
                ctx.check_knowledge_against_type(i_ty, c_ty.canonical_type().unwrap()).is_proven()
            }
            (RestValueContractKnowledge::Lane(c_lane), RestValueContractKnowledge::Lane(i_lane)) => {
                c_lane == i_lane
            }
            (RestValueContractKnowledge::Unconstrained, RestValueContractKnowledge::Unconstrained) => true,
            _ => false, // Element vs Lane is incompatible
        }
    }
    _ => false,
};
```

- [ ] **Step 4: Run tests to verify PASS**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic override_checking_rejects_element_vs_lane_mismatch`
Expected: PASS.
Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic rest_contract_change_invalidates_dependent_calls`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/src/db/fingerprint.rs phalcom-semantic/src/checker/declaration.rs phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs
git commit -m "feat(semantic): fingerprint parameter contracts and enforce override compatibility"
```

---

### Task 8: GoalQuery Acceptance & Mandatory Anti-Regressions

**Files:**
- Modify: `phalcom-semantic/tests/semantic/capabilities/generics.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`

- [ ] **Step 1: Update GoalQuery fixture in `generics.rs`**

In `phalcom-semantic/tests/semantic/capabilities/generics.rs:728`:
Change:
```telos
impl<*Goals> GoalQuery<*Goals> {
  @class of(*goals: Pack<*Goals>) -> Self { GoalQuery { goals: goals } }
}
```
to:
```telos
impl<*Goals> GoalQuery<*Goals> {
  @class of(*goals: *Goals) -> Self { GoalQuery { goals: goals } }
}
```
Remove `/// RED-CAPABILITY` comment.

- [ ] **Step 2: Run `generic_pack_query_infers_empty_and_heterogeneous_goal_lanes`**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic generic_pack_query_infers_empty_and_heterogeneous_goal_lanes`
Expected: PASS (0 errors, `empty` infers `TypePack[]`, `mixed` infers `TypePack[Int, String]`).

- [ ] **Step 3: Implement all 15 anti-regressions mapped to Sections 5–17 of `packs-correction.md`**

In `phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs`:
1. `rest_plain_annotation_remains_element_contract` (Section 5, REST-02)
2. `rest_typepack_name_is_not_implicitly_expanded` (Section 3.3, REST-03)
3. `rest_typepack_element_contract_rejects_non_type_kind` (Section 6.1, REST-03)
4. `rest_explicit_positional_lane_infers_whole_typepack` (Section 7, REST-06)
5. `rest_empty_lane_infers_empty_typepack` (Section 7.1, REST-06)
6. `rest_capture_local_is_proper_pack_not_element_type` (Section 5.5, REST-08)
7. `rest_lane_capture_local_is_pack_from_inferred_lane` (Section 8.1, REST-08)
8. `rest_pack_of_expansion_remains_proper_pack_element_contract` (Section 9, REST-05 - `consume(*packs: Pack<*P>)` accepts `Pack(1, "x")` but rejects scalar `1, "x"`)
9. `rest_concrete_lane_uses_pointwise_value_conformance` (Section 10, REST-07)
10. `rest_whole_pack_inference_does_not_create_typepack_subtyping` (Section 14.5, REST-06)
11. `rest_lane_inference_does_not_partition_pack_concat` (Section 14.6)
12. `rest_argument_analysis_preserves_source_order` (Section 17)
13. `rest_complete_capture_preserves_both_lanes` (Section 12)
14. `rest_labeled_lane_preserves_labels_and_order` (Section 11)
15. `rest_contract_change_invalidates_dependent_calls` (Section 24)

- [ ] **Step 4: Run the complete rest contracts suite**

Run: `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::rest_contracts`
Expected: ALL PASS.

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/tests/semantic/capabilities/generics.rs phalcom-semantic/tests/semantic/capabilities/rest_contracts.rs
git commit -m "test(semantic): certify GoalQuery acceptance and complete rest contract anti-regressions"
```

---

### Task 9: Checkpoint, Walkthrough & Handoff Records

**Files:**
- Modify: `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md`
- Create: `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/LANG007.C4.P4.R1-TEL-1-whole-lane-walkthrough.md`
- Create: `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/LANG007.C4.P4.R1-TEL-1-whole-lane-handoff.md`

- [ ] **Step 1: Write the walkthrough and handoff records**
Document what was implemented, exact test outcomes, and verified invariants (including D classification of Future and unsaturated constructor tests).

- [ ] **Step 2: Update the checkpoint ledger**
Mark TEL-1 as COMPLETE / IMPLEMENTED / FOCUSED_TESTED.

- [ ] **Step 3: Commit**

```bash
git add docs/implementation/LANG007/LANG007.C4/
git commit -m "docs(implementation): record TEL-1 whole-lane rest inference walkthrough and handoff"
```
