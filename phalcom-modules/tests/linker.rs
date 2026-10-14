use phalcom_ast::parser::parse;
use phalcom_modules::{
    AvailabilityAuthority, AvailabilityContract, AvailabilityProof, DependencyPhase, ImportBindingId, ImportPhaseSummary, InterfaceBuilder, LinkError,
    LinkedReadSpec, ModuleComponent, ModuleDiagnostic, ModuleDiagnosticKind, ModuleId, ModuleImportPhaseProduct, ModuleKind, ModuleLinker, ModulePath,
    PhaseEvidenceCoverage, ProjectUniverse, ResolvedProjectId, RuntimeDependencyReason, RuntimeDependencyRequirement, RuntimeRequirementProducer,
    RuntimeRequirementProduct, RuntimeRequirementProvenance, RuntimeRequirementTarget, RuntimeRequirementTiming, SymbolId, UnlinkedModuleInterface,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId {
        project: ResolvedProjectId::from_raw(1).into(),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    }
}

fn interfaces(sources: &[(ModuleId, &str)]) -> BTreeMap<ModuleId, UnlinkedModuleInterface> {
    sources
        .iter()
        .map(|(id, source)| {
            let program = parse(source, 0).program;
            let interface = InterfaceBuilder::build(id.clone(), ModuleKind::Module, &program).unwrap();
            (id.clone(), interface)
        })
        .collect()
}

fn requirement(
    importer: &ModuleId,
    dependency: &ModuleId,
    read_target: Option<LinkedReadSpec>,
    producer: RuntimeRequirementProducer,
    product: RuntimeRequirementProduct,
    timing: RuntimeRequirementTiming,
) -> RuntimeDependencyRequirement {
    let target = match read_target.as_ref() {
        Some(LinkedReadSpec::Module(module)) => RuntimeRequirementTarget::Module(module.clone()),
        Some(LinkedReadSpec::Binding(symbol)) => RuntimeRequirementTarget::Binding {
            module: symbol.module.clone(),
            name: symbol.name.clone(),
        },
        None => RuntimeRequirementTarget::Module(dependency.clone()),
    };
    RuntimeDependencyRequirement {
        importer: importer.clone(),
        dependency: dependency.clone(),
        range: Default::default(),
        reason: RuntimeDependencyReason::RuntimeDeclarationReference,
        read_target,
        provenance: RuntimeRequirementProvenance {
            consumer: importer.clone(),
            producer,
            target,
            product,
            timing,
        },
    }
}

fn proof(target: RuntimeRequirementTarget, product: RuntimeRequirementProduct, authority: AvailabilityAuthority) -> AvailabilityProof {
    AvailabilityProof {
        target,
        product,
        authority,
        available_at: RuntimeRequirementTiming::BeforeSourceExecution,
    }
}

/// LINK-01 — Selective import of exported name succeeds and produces LinkedReadSpec::Binding
#[test]
fn link_01_selective_import_produces_binding() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[
        (exporter.clone(), "class Exported {}\nexport Exported\n"),
        (importer.clone(), "from .exporter import Exported\n"),
    ]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(importer.clone(), &resolved).expect("link should succeed");

    let expected_symbol = SymbolId {
        module: exporter.clone(),
        name: "Exported".into(),
    };
    assert_eq!(linked.modules[&importer].linked_reads, vec![LinkedReadSpec::Binding(expected_symbol)]);
    assert_eq!(linked.modules[&importer].bindings.imports["Exported"], ImportBindingId(0));
}

#[test]
fn semantic_link_preserves_item_identity_without_executable_slots() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[
        (exporter.clone(), "class First {}\nclass Second {}\nexport First\nexport Second\n"),
        (importer.clone(), "from .exporter import First as One, Second\n"),
    ]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);

    let linked = linker.link_semantic(importer.clone(), &resolved).expect("symbolic link should succeed");
    let module = &linked.modules[&importer];
    let one = module.imports.by_local_name.get("One").expect("alias should be symbolically resolvable");
    let second = module
        .imports
        .by_local_name
        .get("Second")
        .expect("second item should be symbolically resolvable");

    assert_ne!(one, second);
    assert_eq!(one.site, second.site);
    assert_eq!(one.local.0, 0);
    assert_eq!(second.local.0, 1);
    assert_eq!(
        module.imports.bindings[one].target,
        phalcom_modules::SymbolicImportTarget::Binding(SymbolId {
            module: exporter.clone(),
            name: "First".into(),
        })
    );
    assert_eq!(
        module.imports.bindings[second].target,
        phalcom_modules::SymbolicImportTarget::Binding(SymbolId {
            module: exporter,
            name: "Second".into(),
        })
    );

    let unfinalized = linked.to_unfinalized_program();
    assert_eq!(unfinalized.modules[&importer].linked_reads.len(), 2);
    assert!(unfinalized.initialization_order.is_empty());
}

#[test]
fn semantic_link_resolves_a_cycle_before_executable_validation() {
    let a = module("a");
    let b = module("b");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[
            (a.clone(), "from .b import B\nclass A {}\nexport A\n"),
            (b.clone(), "from .a import A\nclass B {}\nexport B\n"),
        ]),
    );
    let resolved = BTreeMap::from([((a.clone(), ".b".to_string()), b.clone()), ((b.clone(), ".a".to_string()), a.clone())]);
    let symbolic = linker.link_semantic(a.clone(), &resolved).expect("symbolic cycles are not runtime cycles");
    assert_eq!(
        symbolic.modules[&a].imports.get("B").unwrap().target,
        phalcom_modules::SymbolicImportTarget::Binding(SymbolId { module: b, name: "B".into() })
    );
    assert!(matches!(linker.link_all(a, &resolved), Err(LinkError::RuntimeCycle(_))));
}

#[test]
fn finalization_omits_proven_interface_import_and_keeps_unknown_runtime() {
    let exporter = module("exporter");
    let importer = module("importer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[
            (exporter.clone(), "class TypeOnly {}\nclass Unknown {}\nexport TypeOnly\nexport Unknown\n"),
            (importer.clone(), "from .exporter import TypeOnly, Unknown\n"),
        ]),
    );
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter)]);
    let semantic = linker.link_semantic(importer.clone(), &resolved).expect("symbolic link should succeed");
    let type_only = semantic.modules[&importer].imports.by_local_name["TypeOnly"].clone();
    let unknown = semantic.modules[&importer].imports.by_local_name["Unknown"].clone();
    let phase = ModuleImportPhaseProduct {
        module: importer.clone(),
        bindings: BTreeMap::from([(
            type_only,
            ImportPhaseSummary {
                phase: DependencyPhase::InterfaceOnly,
                observed: true,
            },
        )]),
        coverage: PhaseEvidenceCoverage::Complete,
    };
    let executable = linker
        .finalize_executable(semantic, &BTreeMap::from([(importer.clone(), phase)]))
        .expect("runtime projection should succeed");
    let linked = executable.as_program();
    assert!(!linked.modules[&importer].bindings.imports.contains_key("TypeOnly"));
    assert!(linked.modules[&importer].bindings.imports.contains_key("Unknown"));
    assert_eq!(linked.modules[&importer].linked_reads.len(), 1);
    assert_eq!(unknown.local.0, 1);
}

#[test]
fn finalization_includes_generated_runtime_requirements() {
    let exporter = module("exporter");
    let importer = module("importer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[
            (exporter.clone(), "class TypeOnly {}\nexport TypeOnly\n"),
            (importer.clone(), "from .exporter import TypeOnly\n"),
        ]),
    );
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let semantic = linker.link_semantic(importer.clone(), &resolved).expect("symbolic link should succeed");
    let type_only = semantic.modules[&importer].imports.by_local_name["TypeOnly"].clone();
    let phase = ModuleImportPhaseProduct {
        module: importer.clone(),
        bindings: BTreeMap::from([(
            type_only,
            ImportPhaseSummary {
                phase: DependencyPhase::InterfaceOnly,
                observed: true,
            },
        )]),
        coverage: PhaseEvidenceCoverage::Complete,
    };
    let requirement = RuntimeDependencyRequirement {
        importer: importer.clone(),
        dependency: exporter.clone(),
        range: Default::default(),
        reason: RuntimeDependencyReason::RuntimeDeclarationReference,
        read_target: Some(LinkedReadSpec::Binding(SymbolId {
            module: exporter.clone(),
            name: "TypeOnly".into(),
        })),
        provenance: RuntimeRequirementProvenance {
            consumer: importer.clone(),
            producer: RuntimeRequirementProducer::DeclarationRead,
            target: RuntimeRequirementTarget::Binding {
                module: exporter.clone(),
                name: "TypeOnly".into(),
            },
            product: RuntimeRequirementProduct::RuntimeObject,
            timing: RuntimeRequirementTiming::DuringModuleInitializer,
        },
    };

    let executable = linker
        .finalize_executable_with_requirements(semantic, &BTreeMap::from([(importer.clone(), phase)]), &[requirement])
        .expect("generated runtime requirement should be finalized");
    let program = executable.as_program();
    assert!(!program.modules[&importer].bindings.imports.contains_key("TypeOnly"));
    assert_eq!(program.modules[&importer].runtime_dependencies, vec![exporter.clone()]);
    assert!(
        program
            .graphs
            .runtime
            .edges_from(&importer)
            .iter()
            .any(|edge| { edge.reason == RuntimeDependencyReason::RuntimeDeclarationReference && edge.dependency == exporter })
    );
    assert_eq!(
        executable.runtime_read(
            &importer,
            &LinkedReadSpec::Binding(SymbolId {
                module: exporter.clone(),
                name: "TypeOnly".into(),
            })
        ),
        Some(ImportBindingId(0)),
        "generated canonical read must receive a finalized immutable slot"
    );
}

#[test]
fn exact_availability_proof_removes_edge_but_keeps_canonical_read_slot() {
    let exporter = module("exporter");
    let importer = module("importer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[(exporter.clone(), "class TypeOnly {}\n"), (importer.clone(), "class Consumer {}\n")]),
    );
    let semantic = linker.link_semantic(importer.clone(), &BTreeMap::new()).expect("symbolic link should succeed");
    let read = LinkedReadSpec::Binding(SymbolId {
        module: exporter.clone(),
        name: "TypeOnly".into(),
    });
    let requirement = requirement(
        &importer,
        &exporter,
        Some(read.clone()),
        RuntimeRequirementProducer::DeclarationRead,
        RuntimeRequirementProduct::RuntimeObject,
        RuntimeRequirementTiming::DuringModuleInitializer,
    );
    let availability = AvailabilityContract::from_proofs([proof(
        RuntimeRequirementTarget::Binding {
            module: exporter.clone(),
            name: "TypeOnly".into(),
        },
        RuntimeRequirementProduct::RuntimeObject,
        AvailabilityAuthority::NativeBindingCatalog,
    )]);

    let executable = linker
        .finalize_executable_with_availability(semantic, &BTreeMap::new(), &[requirement.clone()], &availability)
        .expect("exact native-floor proof should finalize");
    assert!(executable.graphs.runtime.edges_from(&importer).is_empty());
    assert_eq!(executable.modules[&importer].linked_reads, vec![read.clone()]);
    assert_eq!(executable.runtime_read(&importer, &read), Some(ImportBindingId(0)));
    assert_eq!(executable.runtime_requirements(), &[requirement]);
}

#[test]
fn mismatched_and_reserved_binding_proofs_keep_conservative_edges() {
    let exporter = module("exporter");
    let importer = module("importer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[(exporter.clone(), "class TypeOnly {}\n"), (importer.clone(), "class Consumer {}\n")]),
    );
    let semantic = linker.link_semantic(importer.clone(), &BTreeMap::new()).expect("symbolic link should succeed");
    let requirement = requirement(
        &importer,
        &exporter,
        Some(LinkedReadSpec::Binding(SymbolId {
            module: exporter.clone(),
            name: "ReservedSourceClass".into(),
        })),
        RuntimeRequirementProducer::DeclarationRead,
        RuntimeRequirementProduct::RuntimeObject,
        RuntimeRequirementTiming::DuringModuleInitializer,
    );
    let availability = AvailabilityContract::from_proofs([
        proof(
            RuntimeRequirementTarget::Binding {
                module: exporter.clone(),
                name: "OtherBinding".into(),
            },
            RuntimeRequirementProduct::RuntimeObject,
            AvailabilityAuthority::NativeBindingCatalog,
        ),
        proof(
            RuntimeRequirementTarget::Module(exporter.clone()),
            RuntimeRequirementProduct::RuntimeObject,
            AvailabilityAuthority::NativeModuleMaterialization,
        ),
    ]);

    let executable = linker
        .finalize_executable_with_availability(semantic, &BTreeMap::new(), &[requirement], &availability)
        .expect("non-exact proofs must leave the conservative requirement");
    let edge = &executable.graphs.runtime.edges_from(&importer)[0];
    assert_eq!(edge.dependency, exporter);
    assert_eq!(edge.provenance.as_ref().unwrap().product, RuntimeRequirementProduct::RuntimeObject);
}

#[test]
fn metadata_descriptor_requirement_uses_exact_metadata_proof_without_a_read_slot() {
    let dependency = module("metadata_owner");
    let importer = module("consumer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[(dependency.clone(), "class Owner {}\n"), (importer.clone(), "class Consumer {}\n")]),
    );
    let semantic = linker.link_semantic(importer.clone(), &BTreeMap::new()).expect("symbolic link should succeed");
    let requirement = requirement(
        &importer,
        &dependency,
        None,
        RuntimeRequirementProducer::MetadataTypeOwner,
        RuntimeRequirementProduct::MetadataDescriptor,
        RuntimeRequirementTiming::BeforeSourceExecution,
    );
    let availability = AvailabilityContract::from_proofs([proof(
        RuntimeRequirementTarget::Module(dependency.clone()),
        RuntimeRequirementProduct::MetadataDescriptor,
        AvailabilityAuthority::SemanticMetadataBundle,
    )]);

    let executable = linker
        .finalize_executable_with_availability(semantic, &BTreeMap::new(), &[requirement], &availability)
        .expect("exact metadata proof should remove only the metadata edge");
    assert!(executable.graphs.runtime.edges_from(&importer).is_empty());
    assert!(executable.modules[&importer].linked_reads.is_empty());
}

#[test]
fn module_proof_can_satisfy_whole_module_import_but_binding_proof_cannot() {
    let exporter = module("exporter");
    let importer = module("importer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[
            (exporter.clone(), "class Exported {}\nexport Exported\n"),
            (importer.clone(), "import .exporter as exp\n"),
        ]),
    );
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let semantic = linker.link_semantic(importer.clone(), &resolved).expect("symbolic link should succeed");
    let module_proof = AvailabilityContract::from_proofs([AvailabilityProof {
        target: RuntimeRequirementTarget::Module(exporter.clone()),
        product: RuntimeRequirementProduct::SourceInitializedValue,
        authority: AvailabilityAuthority::FullyInitializedUniverse,
        available_at: RuntimeRequirementTiming::AfterBootstrap,
    }]);
    let executable = linker
        .finalize_executable_with_availability(semantic.clone(), &BTreeMap::new(), &[], &module_proof)
        .expect("module proof should satisfy whole-module object availability");
    assert!(executable.graphs.runtime.edges_from(&importer).is_empty());
    assert_eq!(executable.modules[&importer].linked_reads, vec![LinkedReadSpec::Module(exporter.clone())]);

    let binding_proof = AvailabilityContract::from_proofs([proof(
        RuntimeRequirementTarget::Binding {
            module: exporter.clone(),
            name: "Exported".into(),
        },
        RuntimeRequirementProduct::RuntimeObject,
        AvailabilityAuthority::NativeBindingCatalog,
    )]);
    let executable = linker
        .finalize_executable_with_availability(semantic, &BTreeMap::new(), &[], &binding_proof)
        .expect("binding proof cannot prove a whole-module import");
    assert_eq!(executable.graphs.runtime.edges_from(&importer).len(), 1);
}

#[test]
fn exact_binding_identity_proof_satisfies_authored_import_and_keeps_its_read_slot() {
    let exporter = module("exporter");
    let importer = module("importer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[
            (exporter.clone(), "trait Exported {}\nexport Exported\n"),
            (importer.clone(), "from .exporter import Exported\n"),
        ]),
    );
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let semantic = linker.link_semantic(importer.clone(), &resolved).expect("symbolic link should succeed");
    let target = RuntimeRequirementTarget::Binding {
        module: exporter.clone(),
        name: "Exported".into(),
    };
    let requirement = requirement(
        &importer,
        &exporter,
        Some(LinkedReadSpec::Binding(SymbolId {
            module: exporter.clone(),
            name: "Exported".into(),
        })),
        RuntimeRequirementProducer::ImportRead,
        RuntimeRequirementProduct::CanonicalIdentity,
        RuntimeRequirementTiming::BeforeSourceExecution,
    );
    let availability = AvailabilityContract::from_proofs([proof(
        target,
        RuntimeRequirementProduct::CanonicalIdentity,
        AvailabilityAuthority::SemanticIdentityCatalog,
    )]);

    let executable = linker
        .finalize_executable_with_availability(semantic, &BTreeMap::new(), &[requirement.clone()], &availability)
        .expect("exact declaration identity proof should satisfy authored import ordering");
    assert!(executable.graphs.runtime.edges_from(&importer).is_empty());
    assert_eq!(
        executable.modules[&importer].linked_reads,
        vec![LinkedReadSpec::Binding(SymbolId {
            module: exporter,
            name: "Exported".into(),
        })],
        "identity-only proof must not remove the canonical read slot"
    );
    assert_eq!(executable.runtime_requirements(), &[requirement]);
}

#[test]
fn generated_requirement_read_slots_are_ordered_independently_of_input_order() {
    let first = module("first");
    let second = module("second");
    let consumer = module("consumer");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[
            (first.clone(), "class First {}\n"),
            (second.clone(), "class Second {}\n"),
            (consumer.clone(), "class Consumer {}\n"),
        ]),
    );
    let semantic = linker.link_semantic(consumer.clone(), &BTreeMap::new()).expect("symbolic link should succeed");
    let first_requirement = requirement(
        &consumer,
        &first,
        Some(LinkedReadSpec::Binding(SymbolId {
            module: first.clone(),
            name: "First".into(),
        })),
        RuntimeRequirementProducer::DeclarationRead,
        RuntimeRequirementProduct::RuntimeObject,
        RuntimeRequirementTiming::DuringModuleInitializer,
    );
    let second_requirement = requirement(
        &consumer,
        &second,
        Some(LinkedReadSpec::Binding(SymbolId {
            module: second.clone(),
            name: "Second".into(),
        })),
        RuntimeRequirementProducer::DeclarationRead,
        RuntimeRequirementProduct::RuntimeObject,
        RuntimeRequirementTiming::DuringModuleInitializer,
    );
    let executable = linker
        .finalize_executable_with_requirements(semantic, &BTreeMap::new(), &[second_requirement, first_requirement])
        .expect("requirements form an acyclic graph");
    assert_eq!(
        executable.modules[&consumer].linked_reads,
        vec![
            LinkedReadSpec::Binding(SymbolId {
                module: first,
                name: "First".into()
            }),
            LinkedReadSpec::Binding(SymbolId {
                module: second,
                name: "Second".into()
            }),
        ]
    );
}

#[test]
fn finalization_rejects_cycles_created_by_generated_runtime_requirements() {
    let a = module("a");
    let b = module("b");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[(a.clone(), "class A {}\n"), (b.clone(), "class B {}\n")]),
    );
    let semantic = linker.link_semantic(a.clone(), &BTreeMap::new()).expect("symbolic link should succeed");
    let requirements = [
        RuntimeDependencyRequirement {
            importer: a.clone(),
            dependency: b.clone(),
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            read_target: None,
            provenance: RuntimeRequirementProvenance {
                consumer: a.clone(),
                producer: RuntimeRequirementProducer::MetadataTypeOwner,
                target: RuntimeRequirementTarget::Module(b.clone()),
                product: RuntimeRequirementProduct::MetadataDescriptor,
                timing: RuntimeRequirementTiming::BeforeSourceExecution,
            },
        },
        RuntimeDependencyRequirement {
            importer: b.clone(),
            dependency: a.clone(),
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            read_target: None,
            provenance: RuntimeRequirementProvenance {
                consumer: b.clone(),
                producer: RuntimeRequirementProducer::MetadataTypeOwner,
                target: RuntimeRequirementTarget::Module(a.clone()),
                product: RuntimeRequirementProduct::MetadataDescriptor,
                timing: RuntimeRequirementTiming::BeforeSourceExecution,
            },
        },
    ];

    let error = linker
        .finalize_executable_with_requirements(semantic, &BTreeMap::new(), &requirements)
        .expect_err("genuine runtime initialization cycle remains rejected");
    let LinkError::RuntimeCycle(phalcom_modules::ModuleGraphError::RuntimeCycle { components, path, .. }) = error else {
        panic!("expected strict runtime-cycle error");
    };
    assert_eq!(components, vec![vec![a.clone(), b.clone()]]);
    assert_eq!(path.len(), 2, "reported path must show the deterministic cycle");
    assert!(
        path.iter().all(|edge| edge.provenance.is_some()),
        "cycle path preserves generated-requirement provenance"
    );
    assert_eq!(path[0].importer, a);
    assert_eq!(path[0].dependency, b);
}

#[test]
fn individually_acyclic_generated_requirement_families_can_form_combined_cycle() {
    let a = module("a");
    let b = module("b");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[(a.clone(), "class A {}\n"), (b.clone(), "class B {}\n")]),
    );
    let semantic = linker
        .link_semantic(a.clone(), &BTreeMap::new())
        .expect("symbolic linking accepts both modules");
    let semantic_generated = requirement(
        &a,
        &b,
        None,
        RuntimeRequirementProducer::CallableOwner,
        RuntimeRequirementProduct::SourceInitializedValue,
        RuntimeRequirementTiming::DuringModuleInitializer,
    );
    let metadata_generated = requirement(
        &b,
        &a,
        None,
        RuntimeRequirementProducer::MetadataTypeOwner,
        RuntimeRequirementProduct::SourceInitializedValue,
        RuntimeRequirementTiming::DuringModuleInitializer,
    );

    for family in [&semantic_generated, &metadata_generated] {
        linker
            .finalize_executable_with_requirements(semantic.clone(), &BTreeMap::new(), std::slice::from_ref(family))
            .expect("one directed dependency cannot create a cycle");
    }

    let error = linker
        .finalize_executable_with_requirements(semantic, &BTreeMap::new(), &[semantic_generated, metadata_generated])
        .expect_err("combined producer families must not evade strict DAG rejection");
    let LinkError::RuntimeCycle(phalcom_modules::ModuleGraphError::RuntimeCycle { components, path, .. }) = error else {
        panic!("expected a provenance-bearing runtime-cycle diagnostic");
    };
    assert_eq!(components, vec![vec![a, b]]);
    assert_eq!(path.len(), 2, "combined cycle must retain a complete edge path");
    assert!(path.iter().all(|edge| edge.provenance.is_some()));
}

/// LINK-02 — Selective import of non-exported name produces LinkError::MissingExport
#[test]
fn link_02_selective_import_missing_export() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[(exporter.clone(), "class Private {}\n"), (importer.clone(), "from .exporter import Private\n")]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let result = linker.link(importer.clone(), &resolved);

    assert!(
        matches!(result, Err(LinkError::MissingExport { ref module, importer: ref error_importer, ref name, .. })
            if module == &exporter && error_importer == &importer && name == "Private"),
        "expected MissingExport for Private, got {:?}",
        result
    );
}

/// LINK-02b — Missing-export diagnostics retain importer ownership and target provenance.
#[test]
fn link_02b_missing_export_diagnostic_uses_importer_range_owner() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[(exporter.clone(), "class Private {}\n"), (importer.clone(), "from .exporter import Private\n")]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map.clone());
    let error = linker.link(importer.clone(), &resolved).expect_err("link should report a missing export");
    let LinkError::MissingExport { range, .. } = error.clone() else {
        panic!("expected MissingExport, got {error:?}");
    };

    let diagnostic = ModuleDiagnostic::from_link_error(error, Some(&iface_map[&exporter]));
    assert_eq!(diagnostic.module, importer);
    assert_eq!(diagnostic.range, range);
    assert_eq!(
        diagnostic.kind,
        ModuleDiagnosticKind::NonExportedImport {
            module: exporter.to_string(),
            name: "Private".to_string(),
        }
    );
}

/// LINK-03 — Module-import produces LinkedReadSpec::Module
#[test]
fn link_03_module_import_produces_module_spec() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[(exporter.clone(), "class Widget {}\nexport Widget\n"), (importer.clone(), "import .exporter\n")]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(importer.clone(), &resolved).expect("link should succeed");

    assert_eq!(linked.modules[&importer].linked_reads, vec![LinkedReadSpec::Module(exporter.clone())]);
    assert_eq!(linked.modules[&importer].bindings.imports["exporter"], ImportBindingId(0));
}

/// LINK-04 — Import alias rebinds the local name
#[test]
fn link_04_import_alias_rebinds_local_name() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[
        (exporter.clone(), "class Widget {}\nexport Widget\n"),
        (importer.clone(), "import .exporter as exp\n"),
    ]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(importer.clone(), &resolved).expect("link should succeed");

    assert_eq!(linked.modules[&importer].bindings.imports.get("exp"), Some(&ImportBindingId(0)));
    assert_eq!(linked.modules[&importer].bindings.imports.get("exporter"), None);
}

/// LINK-05 — Re-export chains correctly to the original symbol
#[test]
fn link_05_reexport_chains_to_original_symbol() {
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let mod_c = module("mod_c");
    let iface_map = interfaces(&[
        (mod_a.clone(), "class Foo {}\nexport Foo\n"),
        (mod_b.clone(), "export Foo from .mod_a\n"),
        (mod_c.clone(), "from .mod_b import Foo\n"),
    ]);
    let resolved = BTreeMap::from([
        ((mod_b.clone(), ".mod_a".to_string()), mod_a.clone()),
        ((mod_c.clone(), ".mod_b".to_string()), mod_b.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(mod_c.clone(), &resolved).expect("link should succeed");

    let original_symbol = SymbolId {
        module: mod_a.clone(),
        name: "Foo".into(),
    };
    assert_eq!(linked.modules[&mod_c].linked_reads, vec![LinkedReadSpec::Binding(original_symbol)]);
    let executable =
        phalcom_modules::ExecutableLinkedProgram::from_conservative_link(linked).expect("canonical re-export reads must be covered by the runtime DAG");
    assert!(executable.graphs.runtime.edges_from(&mod_c).iter().any(|edge| edge.dependency == mod_a));
    let order = &executable.initialization_order;
    assert!(order.iter().position(|id| id == &mod_a).unwrap() < order.iter().position(|id| id == &mod_c).unwrap());
}

/// LINK-06 — Cyclic re-export produces LinkError::CyclicReExport
#[test]
fn link_06_cyclic_reexport_rejected() {
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let iface_map = interfaces(&[(mod_a.clone(), "export Foo from .mod_b\n"), (mod_b.clone(), "export Foo from .mod_a\n")]);
    let resolved = BTreeMap::from([
        ((mod_a.clone(), ".mod_b".to_string()), mod_b.clone()),
        ((mod_b.clone(), ".mod_a".to_string()), mod_a.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let result = linker.link(mod_a, &resolved);

    assert!(
        matches!(result, Err(LinkError::CyclicReExport { .. })),
        "expected CyclicReExport, got {:?}",
        result
    );
}

/// LINK-07 — Diamond dependency is deduplicated in initialization order
#[test]
fn link_07_diamond_dependency_deduplicated_order() {
    let main_mod = module("main_mod");
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let base_mod = module("base_mod");

    let iface_map = interfaces(&[
        (base_mod.clone(), "class Base {}\nexport Base\n"),
        (mod_a.clone(), "from .base_mod import Base\nclass A {}\nexport A\n"),
        (mod_b.clone(), "from .base_mod import Base\nclass B {}\nexport B\n"),
        (main_mod.clone(), "from .mod_a import A\nfrom .mod_b import B\n"),
    ]);
    let resolved = BTreeMap::from([
        ((mod_a.clone(), ".base_mod".to_string()), base_mod.clone()),
        ((mod_b.clone(), ".base_mod".to_string()), base_mod.clone()),
        ((main_mod.clone(), ".mod_a".to_string()), mod_a.clone()),
        ((main_mod.clone(), ".mod_b".to_string()), mod_b.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(main_mod.clone(), &resolved).expect("link should succeed");

    let order = &linked.initialization_order;
    // base_mod must appear exactly once
    assert_eq!(order.iter().filter(|m| **m == base_mod).count(), 1);

    let pos_base = order.iter().position(|m| *m == base_mod).unwrap();
    let pos_a = order.iter().position(|m| *m == mod_a).unwrap();
    let pos_b = order.iter().position(|m| *m == mod_b).unwrap();
    let pos_main = order.iter().position(|m| *m == main_mod).unwrap();

    assert!(pos_base < pos_a, "base must precede a");
    assert!(pos_base < pos_b, "base must precede b");
    assert!(pos_a < pos_main, "a must precede main");
    assert!(pos_b < pos_main, "b must precede main");
}

/// LINK-08 — Missing module in link universe produces LinkError::MissingModule
#[test]
fn link_08_missing_module_in_universe() {
    let entry = module("entry");
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), BTreeMap::new());
    let result = linker.link(entry.clone(), &BTreeMap::new());

    assert!(
        matches!(result, Err(LinkError::MissingModule { ref module }) if *module == entry),
        "expected MissingModule, got {:?}",
        result
    );
}

/// LINK-09 — Binding collision (same local name imported twice) produces LinkError::BindingCollision
#[test]
fn link_09_binding_collision_rejected() {
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let importer = module("importer");

    let iface_a = interfaces(&[(mod_a.clone(), "class Foo {}\nexport Foo\n")]).remove(&mod_a).unwrap();
    let iface_b = interfaces(&[(mod_b.clone(), "class Foo {}\nexport Foo\n")]).remove(&mod_b).unwrap();

    let mut importer_iface = interfaces(&[(importer.clone(), "from .mod_a import Foo\n")]).remove(&importer).unwrap();
    let second_iface = interfaces(&[(importer.clone(), "from .mod_b import Foo\n")]).remove(&importer).unwrap();
    importer_iface.imports.extend(second_iface.imports);

    let iface_map = BTreeMap::from([(mod_a.clone(), iface_a), (mod_b.clone(), iface_b), (importer.clone(), importer_iface)]);
    let resolved = BTreeMap::from([
        ((importer.clone(), ".mod_a".to_string()), mod_a.clone()),
        ((importer.clone(), ".mod_b".to_string()), mod_b.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let result = linker.link(importer, &resolved);

    assert!(
        matches!(result, Err(LinkError::BindingCollision { ref name, .. }) if name == "Foo"),
        "expected BindingCollision for Foo, got {:?}",
        result
    );
}

#[test]
fn unreachable_interface_is_not_in_linked_program() {
    let entry = module("entry");
    let unused = module("unused");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[(entry.clone(), ""), (unused.clone(), "class Never {}\n")]),
    );
    let linked = linker.link(entry.clone(), &BTreeMap::new()).unwrap();
    assert_eq!(linked.modules.keys().cloned().collect::<BTreeSet<_>>(), BTreeSet::from([entry]));
}

#[test]
fn tolerant_linker_accumulates_diagnostics_and_preserves_valid_modules() {
    let mod_good = module("good");
    let mod_broken = module("broken");
    let mod_dep = module("dep_on_broken");
    let iface_map = interfaces(&[
        (mod_good.clone(), "class Good {}\nexport Good\n"),
        (mod_broken.clone(), "from .nonexistent import Missing\n"),
        (mod_dep.clone(), "from .broken import Missing\n"),
    ]);
    let resolved = BTreeMap::from([
        ((mod_broken.clone(), ".nonexistent".to_string()), module("nonexistent")),
        ((mod_dep.clone(), ".broken".to_string()), mod_broken.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);

    // Linking mod_good: standalone component, succeeds completely
    let res_good = linker.link_component_tolerant(mod_good.clone(), &resolved);
    assert!(res_good.program.modules.contains_key(&mod_good));
    assert!(res_good.diagnostics.is_empty());
    assert!(res_good.blocked_modules.is_empty());

    // Linking mod_dep: imports broken module which has unresolvable import
    let res_broken = linker.link_component_tolerant(mod_dep.clone(), &resolved);
    assert!(res_broken.blocked_modules.contains(&mod_dep) || res_broken.blocked_modules.contains(&mod_broken));
    assert!(!res_broken.diagnostics.is_empty());
}

#[test]
fn tolerant_runtime_cycle_preserves_independent_survivor_order() {
    let mod_x = module("x");
    let mod_y = module("y");
    let mod_z = module("z");
    let mod_w = module("w");

    // Cycle between X and Y: X imports Y, Y imports X
    // Independent pair Z -> W: Z imports W
    let iface_map = interfaces(&[
        (mod_x.clone(), "import .y as Y\n"),
        (mod_y.clone(), "import .x as X\n"),
        (mod_w.clone(), "class W {}\nexport W\n"),
        (mod_z.clone(), "import .w as W\n"),
    ]);

    let resolved = BTreeMap::from([
        ((mod_x.clone(), ".y".to_string()), mod_y.clone()),
        ((mod_y.clone(), ".x".to_string()), mod_x.clone()),
        ((mod_z.clone(), ".w".to_string()), mod_w.clone()),
    ]);

    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);

    // Link cyclic component X
    let res_x = linker.link_component_tolerant(mod_x.clone(), &resolved);
    // Link independent component Z
    let res_z = linker.link_component_tolerant(mod_z.clone(), &resolved);

    // Z -> W produces valid topological order [W, Z]
    let order_z = res_z.program.graphs.runtime.initialization_order().expect("valid topological order for Z-W");
    let pos_w = order_z.iter().position(|m| m == &mod_w);
    let pos_z = order_z.iter().position(|m| m == &mod_z);
    assert!(pos_w.is_some() && pos_z.is_some());
    assert!(pos_w.unwrap() < pos_z.unwrap(), "W must precede Z in topological order");

    // X/Y cycle is blocked in res_x, diagnostic reported
    assert!(!res_x.diagnostics.is_empty() || !res_x.blocked_modules.is_empty());
}
