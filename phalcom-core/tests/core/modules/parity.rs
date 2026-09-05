use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer};
use phalcom_modules::{LinkedExportTarget, SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::SemanticWorkspaceSession;
use phalcom_semantic::identity::SemanticTargetId;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

fn location(path: &Path) -> SourceLocation {
    let path = path.canonicalize().unwrap();
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path,
    }
}

fn write_parity_package(root: &Path) {
    fs::write(root.join("package.ph"), "").unwrap();
    fs::write(root.join("main.ph"), "import .either as EitherModule\nfrom .constants import version\n").unwrap();
    fs::write(
        root.join("either.ph"),
        "enum Either<L, R> {\n    @variant\n    Left(_ value: L)\n\n    @variant\n    Right(_ value: R)\n}\nexport Either\n",
    )
    .unwrap();
    fs::write(root.join("constants.ph"), "const version = \"1\"\nexport version\n").unwrap();
}

#[test]
fn strict_compiler_and_workspace_publish_identical_module_products() {
    let temp = TempDir::new().unwrap();
    write_parity_package(temp.path());

    let strict = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Package(temp.path().to_path_buf())).expect("strict package analysis");

    let mut workspace = SemanticWorkspaceSession::new();
    let mutations = ["package.ph", "main.ph", "either.ph", "constants.ph"].into_iter().map(|name| {
        let path = temp.path().join(name);
        WorkspaceSourceBatchMutation::SetDiskSnapshot {
            source: location(&path),
            text: Arc::from(fs::read_to_string(&path).unwrap()),
            revision: SourceRevision(1),
            recovered_program: None,
        }
    });
    let publication = workspace.apply_module_mutations(mutations).expect("workspace package analysis");
    let workspace_modules = workspace.module_session();

    let source_id = |name: &str| SourceId(temp.path().join(name).canonicalize().unwrap().to_string_lossy().into());
    let strict_module = |name: &str| {
        strict
            .sources
            .iter()
            .find(|(_, source)| source.source.as_ref().map(|location| &location.source_id) == Some(&source_id(name)))
            .map(|(module, _)| module.clone())
            .expect("strict source module")
    };
    let strict_main = strict_module("main.ph");
    let strict_either = strict_module("either.ph");
    let strict_constants = strict_module("constants.ph");

    assert_eq!(workspace_modules.module_for_source(&source_id("main.ph")), Some(&strict_main));
    assert_eq!(workspace_modules.module_for_source(&source_id("either.ph")), Some(&strict_either));
    assert_eq!(workspace_modules.module_for_source(&source_id("constants.ph")), Some(&strict_constants));

    let strict_main_linked = &strict.linked.modules[&strict_main];
    let strict_either_read = strict_main_linked
        .linked_reads
        .iter()
        .find_map(|read| match read {
            phalcom_modules::LinkedReadSpec::Module(module) => Some(module),
            _ => None,
        })
        .expect("strict module import read");
    assert_eq!(
        workspace_modules.resolved_imports().get(&(strict_main.clone(), ".either".into())),
        Some(strict_either_read)
    );

    let strict_either_export = &strict.linked.modules[&strict_either].interface.exports["Either"].target;
    let workspace_either_export = &workspace_modules.linked().unwrap().modules[&strict_either].interface.exports["Either"].target;
    assert_eq!(strict_either_export, workspace_either_export);
    assert_eq!(
        strict_either_export,
        &LinkedExportTarget::Binding(phalcom_modules::SymbolId {
            module: strict_either.clone(),
            name: "Either".into(),
        })
    );

    let either_declaration = phalcom_semantic::DeclarationId::new(strict_either.clone(), "Either".into());
    assert!(strict.semantic.declarations().get(&either_declaration).is_some());
    assert!(publication.snapshot.declarations().get(&either_declaration).is_some());

    let version_symbol = phalcom_modules::SymbolId {
        module: strict_constants.clone(),
        name: "version".into(),
    };
    assert!(
        strict_main_linked
            .linked_reads
            .iter()
            .any(|read| matches!(read, phalcom_modules::LinkedReadSpec::Binding(symbol) if symbol == &version_symbol))
    );
    assert!(
        publication
            .snapshot
            .source_index()
            .occurrences_for_target(&SemanticTargetId::ModuleBinding(version_symbol))
            .is_some()
    );
}

#[test]
fn package_less_parity_comparator_rejects_relative_imports() {
    let temp = TempDir::new().unwrap();
    let main = temp.path().join("main.ph");
    fs::write(&main, "import .sibling as Sibling\n").unwrap();
    fs::write(temp.path().join("sibling.ph"), "class Sibling {}\n").unwrap();

    let result = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Module(main));
    assert!(matches!(
        result,
        Err(phalcom_core::modules::compile::ProgramCompileError::StandaloneImportRequiresPackageContext { .. })
    ));
}
