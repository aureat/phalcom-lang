use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BaselineExpectations {
    pub workspace: WorkspaceExpectation,
    pub diagnostics: DiagnosticsExpectation,
    pub compiler: CompilerExpectation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceExpectation {
    pub root_project: String,
    pub projects: Vec<String>,
    pub phalcom_sources: usize,
    pub entry: String,
    pub steady_phase: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticsExpectation {
    pub parser: usize,
    pub semantic: usize,
    pub module: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CompilerExpectation {
    pub check: String,
    pub compile: String,
}
