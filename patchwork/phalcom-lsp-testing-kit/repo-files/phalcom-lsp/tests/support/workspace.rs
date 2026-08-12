use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use tower_lsp::lsp_types::Url;

use super::fixture_path;

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(1);

pub struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    pub fn from_fixture_dir(relative: impl AsRef<Path>) -> Self {
        let id = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "phalcom-lsp-test-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temp LSP workspace");

        let source = fixture_path(relative);
        copy_tree(&source, &root);

        Self { root }
    }

    pub fn uri(&self) -> String {
        Url::from_directory_path(&self.root)
            .expect("workspace path can become file URL")
            .to_string()
    }

    pub fn file_uri(&self, relative: impl AsRef<Path>) -> String {
        Url::from_file_path(self.root.join(relative))
            .expect("workspace file can become URL")
            .to_string()
    }

    pub fn read(&self, relative: impl AsRef<Path>) -> String {
        let path = self.root.join(relative);
        fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
    }

    pub fn write(&self, relative: impl AsRef<Path>, text: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(&path, text)
            .unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in fs::read_dir(source)
        .unwrap_or_else(|err| {
            panic!(
                "read fixture directory {}: {err}",
                source.display()
            )
        })
    {
        let entry = entry.expect("read fixture entry");
        let file_type = entry.file_type().expect("fixture file type");
        let target = destination.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&target).expect("create copied fixture directory");
            copy_tree(&entry.path(), &target);
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target).expect("copy fixture file");
        }
    }
}
