//! A7 deterministic module graphs and performance acceptance evidence.

use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceModuleSession, WorkspaceSourceBatchMutation};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

#[derive(Clone, Debug, Eq, PartialEq)]
struct FixtureSource {
    name: String,
    text: String,
}

fn location(root: &Path, name: &str) -> SourceLocation {
    let path = root.join(name);
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path,
    }
}

fn fixture_batch(root: &Path, sources: &[FixtureSource], revision: u64) -> Vec<WorkspaceSourceBatchMutation> {
    sources
        .iter()
        .map(|source| WorkspaceSourceBatchMutation::SetOverlay {
            source: location(root, &source.name),
            text: Arc::from(source.text.clone()),
            revision: SourceRevision(revision),
            recovered_program: None,
        })
        .collect()
}

fn package() -> FixtureSource {
    FixtureSource {
        name: "package.ph".into(),
        text: String::new(),
    }
}

fn linear_chain(count: usize) -> Vec<FixtureSource> {
    let mut sources = vec![package()];
    for index in 0..count {
        let name = format!("m{index:04}.ph");
        let import = (index + 1 < count).then(|| format!("import .m{:04} as Next\n", index + 1)).unwrap_or_default();
        sources.push(FixtureSource {
            name,
            text: format!("{import}class C{index:04} {{}}\nexport C{index:04}\n"),
        });
    }
    sources
}

fn star_provider(consumer_count: usize) -> Vec<FixtureSource> {
    let mut sources = vec![
        package(),
        FixtureSource {
            name: "provider.ph".into(),
            text: "class Provider {}\nexport Provider\n".into(),
        },
    ];
    for index in 0..consumer_count {
        sources.push(FixtureSource {
            name: format!("consumer{index:04}.ph"),
            text: format!("from .provider import Provider\nclass Consumer{index:04} {{}}\nexport Consumer{index:04}\n"),
        });
    }
    sources
}

fn disconnected_components(count: usize) -> Vec<FixtureSource> {
    let mut sources = vec![package()];
    for index in 0..count {
        sources.push(FixtureSource {
            name: format!("isolated{index:04}.ph"),
            text: format!("class Isolated{index:04} {{}}\nexport Isolated{index:04}\n"),
        });
    }
    sources
}

fn import_heavy(import_count: usize) -> Vec<FixtureSource> {
    let mut sources = vec![package()];
    let mut main = String::new();
    for index in 0..import_count {
        main.push_str(&format!("import .dependency{index:04} as D{index:04}\n"));
        sources.push(FixtureSource {
            name: format!("dependency{index:04}.ph"),
            text: format!("class Dependency{index:04} {{}}\nexport Dependency{index:04}\n"),
        });
    }
    main.push_str("class ImportHeavy {}\nexport ImportHeavy\n");
    sources.push(FixtureSource {
        name: "importheavy.ph".into(),
        text: main,
    });
    sources
}

fn negative_import_population(count: usize) -> Vec<FixtureSource> {
    let mut sources = vec![package()];
    for index in 0..count {
        sources.push(FixtureSource {
            name: format!("missing-importer{index:04}.ph"),
            text: format!("from .missing{index:04} import Missing\nclass Importer{index:04} {{}}\nexport Importer{index:04}\n"),
        });
    }
    sources
}

fn reexport_chain(count: usize) -> Vec<FixtureSource> {
    assert!(count > 0);
    let mut sources = vec![package()];
    sources.push(FixtureSource {
        name: "reexport0000.ph".into(),
        text: "class Leaf {}\nexport Leaf\n".into(),
    });
    for index in 1..count {
        sources.push(FixtureSource {
            name: format!("reexport{index:04}.ph"),
            text: format!("export Leaf from .reexport{:04}\n", index - 1),
        });
    }
    sources
}

fn hierarchy_fanout(count: usize) -> Vec<FixtureSource> {
    let mut sources = vec![
        package(),
        FixtureSource {
            name: "hierarchyroot.ph".into(),
            text: "class Root {}\nexport Root\n".into(),
        },
    ];
    for index in 0..count {
        sources.push(FixtureSource {
            name: format!("hierarchychild{index:04}.ph"),
            text: format!("from .hierarchyroot import Root\nclass Child{index:04} is Root {{}}\nexport Child{index:04}\n"),
        });
    }
    sources
}

fn alias_scc(count: usize) -> Vec<FixtureSource> {
    let mut sources = vec![package()];
    for index in 0..count {
        let next = (index + 1) % count;
        sources.push(FixtureSource {
            name: format!("alias{index:04}.ph"),
            text: format!("type Alias{index:04} = Alias{next:04}\nexport Alias{index:04}\n"),
        });
    }
    sources
}

#[test]
fn checkpoint_a7_fixture_builders_are_deterministic() {
    let builders: [fn(usize) -> Vec<FixtureSource>; 8] = [
        linear_chain,
        star_provider,
        disconnected_components,
        import_heavy,
        negative_import_population,
        reexport_chain,
        hierarchy_fanout,
        alias_scc,
    ];

    for build in builders {
        let first = build(8);
        let second = build(8);
        assert_eq!(first, second);
        assert!(first.len() > 1);
        for source in first {
            let parsed = phalcom_ast::parse(&source.text, 0);
            assert!(parsed.errors.is_empty(), "fixture {} must parse: {:?}", source.name, parsed.errors);
        }
    }
}

#[test]
fn checkpoint_a7_acceptance_matrix_module_work_counts() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("package.ph"), "").unwrap();
    let mut session = WorkspaceModuleSession::new();
    let initial = [
        FixtureSource {
            name: "package.ph".into(),
            text: String::new(),
        },
        FixtureSource {
            name: "provider.ph".into(),
            text: "class Provider { value() -> Int { 1 } }\nexport Provider\n".into(),
        },
        FixtureSource {
            name: "consumer.ph".into(),
            text: "from .provider import Provider\nclass Consumer {}\nexport Consumer\n".into(),
        },
        FixtureSource {
            name: "unused.ph".into(),
            text: "class Unused {}\nexport Unused\n".into(),
        },
        FixtureSource {
            name: "main.ph".into(),
            text: "from .missing import Missing\nclass Main {}\nexport Main\n".into(),
        },
    ];
    session.apply_batch(fixture_batch(root, &initial, 1)).unwrap();

    let body_only = session
        .set_overlay(
            location(root, "provider.ph"),
            Arc::from("class Provider { value() -> Int { 2 } }\nexport Provider\n"),
            SourceRevision(2),
        )
        .unwrap();
    assert_eq!(body_only.stats.imports_resolved, 0);
    assert_eq!(body_only.stats.linked_components_recomputed, 0);
    assert_eq!(body_only.stats.linked_components_considered, 1);
    assert!(body_only.stats.linked_components_reused >= 1);

    let unrelated = session
        .set_overlay(
            location(root, "unrelated.ph"),
            Arc::from("class Unrelated {}\nexport Unrelated\n"),
            SourceRevision(1),
        )
        .unwrap();
    assert_eq!(unrelated.stats.imports_resolved, 0);
    assert!(unrelated.stats.negative_resolutions_reused >= 1);

    let missing = session
        .set_overlay(location(root, "missing.ph"), Arc::from("class Missing {}\nexport Missing\n"), SourceRevision(1))
        .unwrap();
    assert_eq!(missing.stats.imports_resolved, 1);
}

#[test]
fn checkpoint_a7_thousand_module_linear_fixture_has_bounded_body_edit_work() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    fs::write(root.join("package.ph"), "").unwrap();
    let count = 1_000;
    let mut sources = linear_chain(count);
    sources[count].text = format!("class C{:04} {{ value() -> Int {{ 1 }} }}\nexport C{:04}\n", count - 1, count - 1);

    let mut session = WorkspaceModuleSession::new();
    let initial = session.apply_batch(fixture_batch(root, &sources, 1)).unwrap();
    assert_eq!(initial.stats.linked_components_considered, 1);
    assert_eq!(initial.stats.linked_components_recomputed, 1);

    let update = session
        .set_overlay(
            location(root, &format!("m{:04}.ph", count - 1)),
            Arc::from(format!("class C{:04} {{ value() -> Int {{ 2 }} }}\nexport C{:04}\n", count - 1, count - 1)),
            SourceRevision(2),
        )
        .unwrap();
    assert_eq!(update.stats.imports_resolved, 0);
    assert_eq!(update.stats.linked_components_recomputed, 0);
    assert_eq!(update.stats.linked_components_considered, 1);
    assert_eq!(update.stats.linked_modules_reused, count + 1);
}
