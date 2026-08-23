use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

use crate::{BaselineExpectations, MarkedSource};

#[derive(Debug)]
pub enum GoldenWorkspaceError {
    Io(io::Error),
    Toml(toml::de::Error),
    SourceCount { expected: usize, actual: usize },
    DuplicateMarker(String),
}

impl From<io::Error> for GoldenWorkspaceError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for GoldenWorkspaceError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

pub struct GoldenWorkspace {
    root: PathBuf,
}

impl GoldenWorkspace {
    pub fn repository_fixture() -> Self {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repository_root = crate_root.parent().expect("test-support crate lives at repository root");
        Self {
            root: repository_root.join("examples/ide-golden"),
        }
    }

    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn baseline(&self) -> Result<BaselineExpectations, GoldenWorkspaceError> {
        let raw = fs::read_to_string(self.root.join("expectations/baseline.toml"))?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn validate_integrity(&self) -> Result<(), GoldenWorkspaceError> {
        let baseline = self.baseline()?;
        let mut files = Vec::new();
        collect_ph_files(&self.root, &mut files)?;
        if files.len() != baseline.workspace.phalcom_sources {
            return Err(GoldenWorkspaceError::SourceCount {
                expected: baseline.workspace.phalcom_sources,
                actual: files.len(),
            });
        }

        let mut markers = BTreeSet::new();
        for path in files {
            let raw = fs::read_to_string(path)?;
            let source = MarkedSource::parse(&raw);
            for (marker, _) in source.markers() {
                if !markers.insert(marker.to_string()) {
                    return Err(GoldenWorkspaceError::DuplicateMarker(marker.to_string()));
                }
            }
        }
        Ok(())
    }

    pub fn copy_to_temp(&self) -> Result<TempDir, GoldenWorkspaceError> {
        let temp = tempfile::tempdir()?;
        copy_tree(&self.root, temp.path())?;
        Ok(temp)
    }
}

fn collect_ph_files(dir: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ph_files(&path, output)?;
        } else if path.extension().is_some_and(|ext| ext == "ph") {
            output.push(path);
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_golden_workspace_satisfies_baseline_integrity() {
        GoldenWorkspace::repository_fixture().validate_integrity().unwrap();
    }

    #[test]
    fn copied_workspace_is_isolated() {
        let golden = GoldenWorkspace::repository_fixture();
        let temp = golden.copy_to_temp().unwrap();
        assert!(temp.path().join("project.toml").is_file());
        assert_ne!(golden.root(), temp.path());
    }
}
