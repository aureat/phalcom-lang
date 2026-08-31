//! Source-driven fixtures and structured oracles for ADT/match tests.
//!
//! Keep semantic tests focused on the product they prove. This module owns
//! parsing, standalone analysis, declaration lookup, and match-product
//! navigation so individual scenarios do not fall back to diagnostic-only
//! assertions or reimplement snapshot traversal.

#![allow(dead_code)]

use phalcom_ast::parse_source;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::declarations::DeclarationTypeInfo;
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use phalcom_semantic::enum_semantics::{EnumInfo, VariantInfo};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, VariantFamilyId, VariantFieldId, VariantId};
use phalcom_semantic::match_semantics::{ExhaustivenessResult, MatchArmResolution, MatchResolution, PatternResolution, PatternSpaceSummary, PatternUsefulness};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::types::id::TypeId;
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::workspace::SemanticAnalysis;
use std::collections::HashSet;
use std::sync::Arc;

/// A minimal standalone module containing one ADT test program.
pub struct AdtCase {
    pub module: ModuleId,
    pub source: Arc<str>,
    pub analysis: SemanticAnalysis,
}

/// Analyze one source fixture through the production semantic entry point.
pub fn analyze_adt(source: &str) -> AdtCase {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(source);
    let parsed = parse_source(&source, 0).unwrap_or_else(|error| panic!("source should parse cleanly: {error:#?}\nsource:\n{source}"));
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed));
    assert!(
        analysis.snapshot.internal_incidents.is_empty(),
        "semantic analyzer produced internal incidents: {:#?}",
        analysis.snapshot.internal_incidents
    );
    AdtCase { module, source, analysis }
}

impl AdtCase {
    pub fn diagnostics(&self) -> impl Iterator<Item = &SemanticDiagnostic> {
        self.analysis.snapshot.all_diagnostics()
    }

    pub fn assert_no_diagnostics(&self) {
        let diagnostics = self.diagnostics().collect::<Vec<_>>();
        assert!(diagnostics.is_empty(), "unexpected semantic diagnostics: {diagnostics:#?}");
    }

    pub fn declaration(&self, name: &str) -> &DeclarationTypeInfo {
        let id = DeclarationId::new(self.module.clone(), name.into());
        self.analysis
            .snapshot
            .declarations
            .get(&id)
            .unwrap_or_else(|| panic!("missing declaration `{name}`"))
    }

    pub fn enum_info(&self, name: &str) -> &EnumInfo {
        let owner = DeclarationId::new(self.module.clone(), name.into());
        self.analysis
            .snapshot
            .enum_semantics
            .enum_info(&owner)
            .unwrap_or_else(|| panic!("missing enum metadata for `{name}`"))
    }

    pub fn type_data(&self, ty: TypeId) -> &TypeData {
        self.analysis.snapshot.store.get(ty)
    }

    pub fn associated_family(&self, owner: &str, base: &str) -> &phalcom_semantic::associated::AssociatedFamilyInfo {
        let owner = DeclarationId::new(self.module.clone(), owner.into());
        let surface = self
            .analysis
            .snapshot
            .associated_surfaces
            .surfaces
            .get(&owner)
            .unwrap_or_else(|| panic!("missing associated surface for {owner:?}"));
        surface
            .families
            .get(&phalcom_common::selector::SelectorBase::Named(base.into()))
            .unwrap_or_else(|| panic!("missing associated family `{base}` on {owner:?}"))
    }

    pub fn variant(&self, owner: &str, selector: Selector) -> &VariantInfo {
        let id = self.variant_id(owner, selector);
        self.analysis
            .snapshot
            .enum_semantics
            .variant_info(&id)
            .unwrap_or_else(|| panic!("missing variant {id:?}"))
    }

    pub fn variant_id(&self, owner: &str, selector: Selector) -> VariantId {
        VariantId::new(DeclarationId::new(self.module.clone(), owner.into()), selector)
    }

    pub fn family_id(&self, owner: &str, base: &str) -> VariantFamilyId {
        VariantFamilyId::new(DeclarationId::new(self.module.clone(), owner.into()), base)
    }

    pub fn field_id(&self, variant: &VariantId, index: u32) -> VariantFieldId {
        VariantFieldId::new(variant.clone(), index)
    }

    pub fn diagnostic(&self, code: DiagnosticCode) -> &SemanticDiagnostic {
        self.diagnostics_with_code(code)
            .next()
            .unwrap_or_else(|| panic!("missing diagnostic {code}; all diagnostics: {:?}", self.diagnostics().collect::<Vec<_>>()))
    }

    pub fn diagnostics_for(&self, code: DiagnosticCode) -> Vec<&SemanticDiagnostic> {
        self.diagnostics_with_code(code).collect()
    }

    pub fn assert_diagnostic_primary_contains(&self, code: DiagnosticCode, needle: &str) {
        let diagnostic = self.diagnostic(code);
        let range = diagnostic.primary.range;
        let text = &self.source[range.start..range.end];
        assert!(text.contains(needle), "primary range {range:?} does not contain {needle:?}: {text:?}");
    }

    fn diagnostics_with_code(&self, code: DiagnosticCode) -> impl Iterator<Item = &SemanticDiagnostic> {
        self.diagnostics().filter(move |diagnostic| diagnostic.code == code)
    }

    pub fn only_match(&self) -> MatchHandle<'_> {
        let mut matches = self
            .analysis
            .snapshot
            .callable_analyses
            .values()
            .flat_map(|callable| callable.match_resolutions.values());
        let resolution = matches.next().unwrap_or_else(|| panic!("fixture contains no match resolution"));
        assert!(matches.next().is_none(), "fixture contains more than one match resolution");
        MatchHandle::new(resolution, self.analysis.snapshot.as_ref())
    }

    pub fn match_in_callable(&self, owner: &str, selector: Selector, index: usize) -> MatchHandle<'_> {
        let callable_id = CallableId::new(DeclarationId::new(self.module.clone(), owner.into()), selector, DispatchSide::Instance);
        let callable = self
            .analysis
            .snapshot
            .callable_analyses
            .get(&callable_id)
            .unwrap_or_else(|| panic!("missing callable {callable_id:?}"));
        let resolution = callable
            .match_resolutions
            .values()
            .nth(index)
            .unwrap_or_else(|| panic!("missing match {index} in callable {callable_id:?}"));
        MatchHandle::new(resolution, self.analysis.snapshot.as_ref())
    }
}

pub struct MatchHandle<'a> {
    resolution: &'a MatchResolution,
    snapshot: &'a SemanticSnapshot,
}

impl<'a> MatchHandle<'a> {
    fn new(resolution: &'a MatchResolution, snapshot: &'a SemanticSnapshot) -> Self {
        Self { resolution, snapshot }
    }

    pub fn resolution(&self) -> &MatchResolution {
        self.resolution
    }

    pub fn snapshot(&self) -> &SemanticSnapshot {
        self.snapshot
    }

    pub fn arm(&self, index: usize) -> ArmHandle<'a> {
        let arm = self.resolution.arms.get(index).unwrap_or_else(|| panic!("missing match arm {index}"));
        ArmHandle { arm, snapshot: self.snapshot }
    }

    pub fn assert_exhaustive(&self) {
        assert_eq!(
            self.resolution.exhaustiveness,
            ExhaustivenessResult::Proven,
            "match is not exhaustive: {:#?}",
            self.resolution.exhaustiveness
        );
    }

    pub fn assert_initial_space(&self, expected: &PatternSpaceSummary) {
        assert_eq!(&self.resolution.initial_space, expected, "unexpected initial pattern space");
    }

    pub fn assert_result_type(&self, expected: TypeId) {
        assert_eq!(
            self.resolution.result.ty(),
            Some(expected),
            "unexpected match result: {:#?}",
            self.resolution.result
        );
    }

    pub fn assert_not_exhaustive(&self) {
        assert!(!matches!(self.resolution.exhaustiveness, ExhaustivenessResult::Proven), "match unexpectedly proved exhaustive");
    }

    pub fn assert_arm_candidate_variants(&self, arm: usize, expected: &[VariantId]) {
        self.arm(arm).assert_candidate_variants(expected);
    }
}

pub struct ArmHandle<'a> {
    arm: &'a MatchArmResolution,
    snapshot: &'a SemanticSnapshot,
}

impl ArmHandle<'_> {
    pub fn resolution(&self) -> &MatchArmResolution {
        self.arm
    }

    pub fn assert_usefulness(&self, expected: PatternUsefulness) {
        assert_eq!(self.arm.usefulness, expected, "unexpected arm usefulness: {:#?}", self.arm);
    }

    pub fn assert_candidate_variants(&self, expected: &[VariantId]) {
        let PatternResolution::Variant(pattern) = &self.arm.pattern else {
            panic!("expected variant pattern, got {:#?}", self.arm.pattern);
        };
        let actual = pattern.candidates.iter().map(|candidate| candidate.variant.clone()).collect::<Vec<_>>();
        assert_eq!(actual, expected, "unexpected resolved candidate variants");
    }

    pub fn find_binding(&self, name: &str) -> Option<&phalcom_semantic::match_semantics::PatternBindingResolution> {
        self.arm.bindings.iter().find(|binding| binding.name.as_ref() == name)
    }

    pub fn assert_binding_names(&self, expected: &[&str]) {
        let actual = self.arm.bindings.iter().map(|binding| binding.name.as_ref()).collect::<Vec<_>>();
        assert_eq!(actual, expected, "unexpected arm binding names");
    }

    pub fn assert_no_binding(&self, name: &str) {
        assert!(self.find_binding(name).is_none(), "unexpected arm binding `{name}`: {:#?}", self.arm.bindings);
    }

    pub fn assert_binding_type(&self, name: &str, expected: TypeId) {
        let binding = self.find_binding(name).unwrap_or_else(|| panic!("missing arm binding `{name}`"));
        assert_eq!(binding.knowledge.ty(), Some(expected), "unexpected type for binding `{name}`");
    }

    pub fn assert_binding_union_members(&self, name: &str, expected: &[TypeId]) {
        let binding = self.find_binding(name).unwrap_or_else(|| panic!("missing arm binding `{name}`"));
        let actual_ty = binding.knowledge.ty().unwrap_or_else(|| panic!("binding `{name}` has no known type"));
        let actual = match self.snapshot.store.get(actual_ty) {
            TypeData::Union(members) => members.iter().copied().collect::<HashSet<_>>(),
            _ => [actual_ty].into_iter().collect(),
        };
        let expected = expected.iter().copied().collect::<HashSet<_>>();
        assert_eq!(actual, expected, "unexpected joined type for binding `{name}`");
    }

    pub fn assert_unique_binding_ids(&self) {
        let mut ids = self.arm.bindings.iter().map(|binding| binding.binding).collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), self.arm.bindings.len(), "arm publishes duplicate binding identities");
    }

    pub fn assert_binding_ids_equal(&self, left: &str, right: &str) {
        let left = self.find_binding(left).unwrap_or_else(|| panic!("missing binding `{left}`"));
        let right = self.find_binding(right).unwrap_or_else(|| panic!("missing binding `{right}`"));
        assert_eq!(left.binding, right.binding, "bindings `{}` and `{}` must share identity", left.name, right.name);
    }

    pub fn assert_proof_empty(&self) {
        assert!(self.arm.proof.is_empty(), "unexpected branch proof: {:#?}", self.arm.proof);
    }
}
