use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::module_world::{ArtifactModuleId, ModuleKindV1, ModuleV1, ModuleWorldV1};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};

fn minimal_world() -> ModuleWorldV1 {
    ModuleWorldV1 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![],
        modules: vec![ModuleV1 {
            path: vec![],
            kind: ModuleKindV1::Package,
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            exposed_children: vec![],
            metadata: vec![],
            linked_exports: vec![],
            globals: vec![],
            import_bindings: vec![],
            linked_reads: vec![],
            runtime_dependencies: vec![],
            references: vec![],
            runtime_edges: vec![],
            topology_parent: None,
            topology_children: vec![],
            interface_fingerprint: 0,
            linked_interface_fingerprint: 0,
            dependency_fingerprint: 0,
        }],
        semantic_rows: vec![],
        topology_fingerprint: 0,
    }
}

#[test]
fn legacy_module_world_validation_scratch_is_charged_transactionally() {
    let bytes = minimal_world().encode().expect("encode module world");
    let limits = ValidationLimits::default();

    let mut measuring = AllocationBudget::new(usize::MAX);
    ModuleWorldV1::decode_with_allocation_budget(&bytes, &limits, &mut measuring).expect("decode with ample budget");
    let requested = measuring.requested();
    assert!(requested > 0);

    let mut short = AllocationBudget::new(requested - 1);
    let before = short.remaining();
    let error = ModuleWorldV1::decode_with_allocation_budget(&bytes, &limits, &mut short).unwrap_err();
    assert!(
        matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)),
        "unexpected error: {error:?}"
    );
    assert_eq!(short.remaining(), before, "failed decode must not debit the parent budget");

    let mut retry = AllocationBudget::new(requested);
    ModuleWorldV1::decode_with_allocation_budget(&bytes, &limits, &mut retry).expect("retry with exact budget");
    assert_eq!(retry.remaining(), 0);
}
