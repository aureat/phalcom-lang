//! Source provider abstraction and filesystem implementation.

use crate::error::{ModuleResolutionError, SourceError};
use crate::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId, SourceId, SourceLocation};
use crate::project::ResolvedProject;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Module kind: an ordinary `.ph` file or a `package.ph` package descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleKind {
    Module,
    Package,
}

/// A located source unit ready for reading and parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceUnit {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
}

/// Trait abstracting source location and reading.
pub trait SourceProvider {
    fn locate(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError>;

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError>;
}

/// Filesystem source provider with resolution caching and kebab/snake convention handling.
#[derive(Debug)]
pub struct FilesystemSourceProvider {
    cache: Mutex<HashMap<(ResolvedProjectId, ModulePath), Result<SourceUnit, ModuleResolutionError>>>,
    source_cache: Mutex<HashMap<SourceId, Arc<str>>>,
}

impl Default for FilesystemSourceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemSourceProvider {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            source_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Clears the resolution and source caches.
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
        self.source_cache.lock().unwrap().clear();
    }

    fn locate_internal(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        let components = path.components();
        let mut current_dir = project.source_root.clone();

        // 1. Root package case: `[]` -> `<source-root>/package.ph`
        if components.is_empty() {
            let pkg_file = current_dir.join("package.ph");
            if pkg_file.is_file() {
                let canonical = pkg_file.canonicalize().map_err(|e| SourceError::Io(e.to_string()))?;
                let source_id = SourceId(canonical.to_string_lossy().into());
                return Ok(SourceUnit {
                    id: ModuleId {
                        project: project.id,
                        path: path.clone(),
                    },
                    kind: ModuleKind::Package,
                    source: SourceLocation {
                        source_id,
                        display_path: pkg_file,
                    },
                });
            } else {
                return Err(ModuleResolutionError::PackageNotFoundError(format!("{}:<root>", project.name)));
            }
        }

        // 2. Intermediate components: all must be package directories containing `package.ph`
        for comp in &components[..components.len() - 1] {
            let (dir_path, _is_kebab) = self.find_directory(&current_dir, comp)?;

            // Check for nested project boundary: if `project.toml` exists in this subdir and is not project root
            if dir_path != project.root_dir && dir_path.join("project.toml").is_file() {
                return Err(ModuleResolutionError::NestedProjectBoundary(dir_path));
            }

            let pkg_file = dir_path.join("package.ph");
            if !pkg_file.is_file() {
                return Err(ModuleResolutionError::PackageNotFoundError(format!(
                    "Directory '{}' is missing package.ph",
                    dir_path.display()
                )));
            }
            current_dir = dir_path;
        }

        // 3. Final component: can be `<comp>.ph` (Module) or `<comp>/package.ph` (Package)
        let last_comp = components.last().unwrap();
        let (file_candidate, dir_candidate) = self.find_final_candidates(&current_dir, last_comp)?;

        let has_file = file_candidate.as_ref().map(|p| p.is_file()).unwrap_or(false);
        let has_dir_pkg = dir_candidate.as_ref().map(|d| d.join("package.ph").is_file()).unwrap_or(false);

        if has_file && has_dir_pkg {
            return Err(ModuleResolutionError::AmbiguousModule {
                name: last_comp.as_str().to_string(),
                kebab_path: file_candidate.unwrap(),
                snake_path: dir_candidate.unwrap().join("package.ph"),
            });
        }

        if has_file {
            let file_path = file_candidate.unwrap();
            let canonical = file_path.canonicalize().map_err(|e| SourceError::Io(e.to_string()))?;

            // Confinement check: canonical path must start with project source root
            if !canonical.starts_with(&project.source_root) {
                return Err(ModuleResolutionError::ImportOutsideSourceRoot(canonical, project.source_root.clone()));
            }

            let source_id = SourceId(canonical.to_string_lossy().into());
            return Ok(SourceUnit {
                id: ModuleId {
                    project: project.id,
                    path: path.clone(),
                },
                kind: ModuleKind::Module,
                source: SourceLocation {
                    source_id,
                    display_path: file_path,
                },
            });
        }

        if has_dir_pkg {
            let dir_path = dir_candidate.unwrap();
            if dir_path != project.root_dir && dir_path.join("project.toml").is_file() {
                return Err(ModuleResolutionError::NestedProjectBoundary(dir_path));
            }

            let pkg_file = dir_path.join("package.ph");
            let canonical = pkg_file.canonicalize().map_err(|e| SourceError::Io(e.to_string()))?;

            if !canonical.starts_with(&project.source_root) {
                return Err(ModuleResolutionError::ImportOutsideSourceRoot(canonical, project.source_root.clone()));
            }

            let source_id = SourceId(canonical.to_string_lossy().into());
            return Ok(SourceUnit {
                id: ModuleId {
                    project: project.id,
                    path: path.clone(),
                },
                kind: ModuleKind::Package,
                source: SourceLocation {
                    source_id,
                    display_path: pkg_file,
                },
            });
        }

        Err(ModuleResolutionError::ModuleNotFound(format!("{}.{}", project.namespace, path)))
    }

    /// Looks for a directory component by checking both kebab-case and snake_case on disk.
    fn find_directory(&self, parent: &Path, comp: &ModuleComponent) -> Result<(PathBuf, bool), ModuleResolutionError> {
        let snake_name = comp.as_str();
        let kebab_name = snake_name.replace('_', "-");

        let snake_dir = parent.join(snake_name);
        let kebab_dir = parent.join(&kebab_name);

        let snake_exists = snake_dir.is_dir();
        let kebab_exists = kebab_name != snake_name && kebab_dir.is_dir();

        if snake_exists && kebab_exists {
            return Err(ModuleResolutionError::AmbiguousModule {
                name: comp.as_str().to_string(),
                kebab_path: kebab_dir,
                snake_path: snake_dir,
            });
        }

        if kebab_exists {
            Ok((kebab_dir, true))
        } else if snake_exists {
            Ok((snake_dir, false))
        } else {
            Err(ModuleResolutionError::PackageNotFoundError(format!(
                "Component '{}' directory not found in {}",
                comp.as_str(),
                parent.display()
            )))
        }
    }

    /// Finds candidate file (`<comp>.ph`) and directory (`<comp>/package.ph`) checking kebab and snake.
    fn find_final_candidates(&self, parent: &Path, comp: &ModuleComponent) -> Result<(Option<PathBuf>, Option<PathBuf>), ModuleResolutionError> {
        let snake_name = comp.as_str();
        let kebab_name = snake_name.replace('_', "-");

        let snake_file = parent.join(format!("{}.ph", snake_name));
        let kebab_file = parent.join(format!("{}.ph", kebab_name));

        let snake_file_exists = snake_file.is_file();
        let kebab_file_exists = kebab_name != snake_name && kebab_file.is_file();

        if snake_file_exists && kebab_file_exists {
            return Err(ModuleResolutionError::AmbiguousModule {
                name: comp.as_str().to_string(),
                kebab_path: kebab_file,
                snake_path: snake_file,
            });
        }

        let resolved_file = if kebab_file_exists {
            Some(kebab_file)
        } else if snake_file_exists {
            Some(snake_file)
        } else {
            None
        };

        let snake_dir = parent.join(snake_name);
        let kebab_dir = parent.join(&kebab_name);

        let snake_dir_exists = snake_dir.is_dir();
        let kebab_dir_exists = kebab_name != snake_name && kebab_dir.is_dir();

        if snake_dir_exists && kebab_dir_exists {
            return Err(ModuleResolutionError::AmbiguousModule {
                name: comp.as_str().to_string(),
                kebab_path: kebab_dir,
                snake_path: snake_dir,
            });
        }

        let resolved_dir = if kebab_dir_exists {
            Some(kebab_dir)
        } else if snake_dir_exists {
            Some(snake_dir)
        } else {
            None
        };

        Ok((resolved_file, resolved_dir))
    }
}

impl SourceProvider for FilesystemSourceProvider {
    fn locate(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        let key = (project.id, path.clone());
        {
            let cache = self.cache.lock().unwrap();
            if let Some(res) = cache.get(&key) {
                return res.clone();
            }
        }

        let result = self.locate_internal(project, path);
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, result.clone());
        result
    }

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        {
            let cache = self.source_cache.lock().unwrap();
            if let Some(content) = cache.get(source) {
                return Ok(content.clone());
            }
        }

        let path = Path::new(&*source.0);
        let content = std::fs::read_to_string(path).map_err(|e| SourceError::Io(format!("Failed to read {}: {}", path.display(), e)))?;

        let arc: Arc<str> = Arc::from(content);
        let mut cache = self.source_cache.lock().unwrap();
        cache.insert(source.clone(), arc.clone());
        Ok(arc)
    }
}
