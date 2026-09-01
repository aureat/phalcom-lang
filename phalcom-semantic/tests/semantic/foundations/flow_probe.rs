use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::NormalReturnFact;
use phalcom_semantic::checker::context::CheckingContext;
use phalcom_semantic::declarations::DeclarationTypeTable;
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

#[test]
fn flow_probe_isolates_diagnostics_explanations_and_exits() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::default();
    let resolver = SimpleTypeResolver::new();
    let declarations = DeclarationTypeTable::new();
    let module_id = ModuleId::universe_root();

    let int_ty = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let string_ty = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "String".into()));
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module_id);

    let binding_id = ctx.alloc_binding();
    ctx.flow.declare(
        binding_id,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
    );

    let initial_diag_count = ctx.diagnostics.len();
    let initial_exit_count = ctx.normal_return_exits().len();
    let initial_flow = ctx.flow.clone();

    let probe_result = ctx.run_flow_probe(initial_flow.clone(), |probe_ctx| {
        // Emit diagnostic in probe
        probe_ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ModuleId::universe_root(),
            DiagnosticCode::TypeMismatch,
            "probe error",
            SourceRange::default(),
        ));

        // Record normal return in probe
        let summary = probe_ctx.current_flow_summary();
        probe_ctx.record_return_exit(NormalReturnFact {
            knowledge: TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            flow: summary,
            status: phalcom_semantic::checker::analysis::AnalysisStatus::Ready,
            causal_invalidity: phalcom_semantic::checker::causal::CausalInvalidity::Clean,
        });

        // Mutate flow in probe
        probe_ctx.flow.write(
            binding_id,
            TypeKnowledge::established(string_ty, EvidenceOrigin::Flow),
            None,
            phalcom_semantic::checker::BindingConsistency::Unconstrained,
            phalcom_semantic::checker::causal::CausalInvalidity::Clean,
        );

        42
    });

    assert_eq!(probe_result.value, 42);
    assert_eq!(ctx.diagnostics.len(), initial_diag_count, "parent diagnostics must not be polluted");
    assert_eq!(ctx.normal_return_exits().len(), initial_exit_count, "parent return exits must not be polluted");
    assert_eq!(ctx.flow, initial_flow, "parent flow must not be mutated");
    assert_eq!(
        probe_result.flow.get_binding(binding_id).unwrap().current.ty(),
        Some(string_ty),
        "probe flow transfer must be returned"
    );
}
