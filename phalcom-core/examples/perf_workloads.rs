use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::session::SemanticWorkspaceSession;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

const FEATURE_DENSE_HELPERS: &str = include_str!("../../benchmarks/perf003/fixtures/feature-dense/helpers.ph");
const FEATURE_DENSE_FACADE: &str = include_str!("../../benchmarks/perf003/fixtures/feature-dense/facade.ph");
const FEATURE_DENSE_CLIENT: &str = include_str!("../../benchmarks/perf003/fixtures/feature-dense/client.ph");
const FEATURE_DENSE_SHAPES: &str = include_str!("../../benchmarks/perf003/fixtures/feature-dense/shapes.ph");

fn source_location(path: &Path) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    }
}

fn overlay(source: &SourceLocation, text: &str, revision: u64) -> WorkspaceSourceBatchMutation {
    WorkspaceSourceBatchMutation::SetOverlay {
        source: source.clone(),
        text: Arc::from(text),
        revision: SourceRevision(revision),
        recovered_program: None,
    }
}

fn temporary_package() -> Result<TempDir, String> {
    let root = TempDir::new().map_err(|error| format!("create temporary benchmark package: {error}"))?;
    fs::write(root.path().join("package.ph"), "").map_err(|error| format!("write package marker: {error}"))?;
    Ok(root)
}

fn require_clean_publication(publication: phalcom_semantic::session::SemanticWorkspacePublication, operation: &str) -> Result<(), String> {
    if publication.snapshot.has_errors() {
        return Err(format!("{operation} produced semantic diagnostics: {:#?}", publication.snapshot.diagnostics));
    }
    Ok(())
}

pub fn run_feature_dense() -> Result<(), String> {
    let root = temporary_package()?;
    let modules = [
        ("helpers", FEATURE_DENSE_HELPERS),
        ("facade", FEATURE_DENSE_FACADE),
        ("client", FEATURE_DENSE_CLIENT),
        ("shapes", FEATURE_DENSE_SHAPES),
    ];
    let mutations = modules
        .iter()
        .map(|(name, text)| overlay(&source_location(&root.path().join(format!("{name}.ph"))), text, 1))
        .collect::<Vec<_>>();
    let mut session = SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations(mutations)
        .map_err(|error| format!("publish feature-dense package: {error}"))?;
    require_clean_publication(publication, "feature-dense package")
}

pub struct IncrementalWorkspace {
    _root: TempDir,
    session: SemanticWorkspaceSession,
    main: SourceLocation,
    subject: SourceLocation,
}

impl IncrementalWorkspace {
    pub fn prepare() -> Result<Self, String> {
        let root = temporary_package()?;
        let main = source_location(&root.path().join("main.ph"));
        let subject = source_location(&root.path().join("subject.ph"));
        let dependency_a = source_location(&root.path().join("dependencya.ph"));
        let dependency_b = source_location(&root.path().join("dependencyb.ph"));
        let main_source = "import .dependencya as dep\nlet mut result = dep.calculate(21)\nexport result\n";
        let subject_source = "class PerfSubject { @class transform(_ value: Int) -> Int { value + 1 } }\nexport PerfSubject\n";
        let dependency_source = "let mut calculate = |x| { x * 2 }\nexport calculate\n";
        let mut session = SemanticWorkspaceSession::new();
        let initial = session
            .apply_module_mutations([
                overlay(&main, main_source, 1),
                overlay(&subject, subject_source, 1),
                overlay(&dependency_a, dependency_source, 1),
                overlay(&dependency_b, dependency_source, 1),
            ])
            .map_err(|error| format!("prepare warm incremental workspace: {error}"))?;
        require_clean_publication(initial, "initial incremental workspace")?;
        Ok(Self {
            _root: root,
            session,
            main,
            subject,
        })
    }

    pub fn run_variants(&mut self) -> Result<(), String> {
        let variants = [
            (
                &self.subject,
                "class PerfSubject { @class transform(_ value: Int) -> Int { value + 2 } }\nexport PerfSubject\n",
                2,
                "small body edit",
            ),
            (
                &self.subject,
                "class PerfSubject { @class transform(_ value: Float) -> Float { value + 1.0 } }\nexport PerfSubject\n",
                3,
                "signature edit",
            ),
            (
                &self.subject,
                "class PerfSubject { @class transform(_ value: Float) -> Float { value + 1.0 } }\ndata PerfExtra { count: Int }\nexport PerfSubject, PerfExtra\n",
                4,
                "declaration addition",
            ),
            (
                &self.main,
                "import .dependencyb as dep\nlet mut result = dep.calculate(21)\nexport result\n",
                2,
                "import edit",
            ),
        ];
        for (source, text, revision, label) in variants {
            let publication = self
                .session
                .apply_module_mutations([overlay(source, text, revision)])
                .map_err(|error| format!("{label}: {error}"))?;
            require_clean_publication(publication, label)?;
        }
        Ok(())
    }
}
