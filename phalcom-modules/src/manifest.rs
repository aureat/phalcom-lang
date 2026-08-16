//! Project manifest decoding and validation (`project.toml`).

use crate::error::ProjectError;
use crate::identity::ModuleComponent;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn default_source_root() -> PathBuf {
    PathBuf::from("src")
}

/// Raw parsed `project.toml` document structure.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    pub project: ProjectSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectSection {
    pub name: String,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub namespace: Option<String>,
    #[serde(default = "default_source_root")]
    pub source: PathBuf,
    pub entry: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PathDependency {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PackageDependency {
    pub package: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencySpec {
    Path { path: PathBuf },
    Package { package: String, version: String },
}

/// Semantically validated project manifest representation.
#[derive(Debug, Clone)]
pub struct ValidatedProjectManifest {
    /// Exact user-facing project/distribution name.
    pub display_name: String,
    /// Explicit stable programming namespace.
    pub namespace: ModuleComponent,
    pub source: PathBuf,
    pub entry: Option<String>,
    pub dependencies: BTreeMap<ModuleComponent, (String, DependencySpec)>,
}

impl ProjectManifest {
    pub fn parse(toml_str: &str) -> Result<Self, ProjectError> {
        toml::from_str(toml_str).map_err(|e| ProjectError::InvalidProjectManifest(e.to_string()))
    }

    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ProjectError::InvalidProjectManifest(format!("Failed to read {}: {}", path.as_ref().display(), e)))?;
        Self::parse(&content)
    }

    pub fn validate(&self) -> Result<ValidatedProjectManifest, ProjectError> {
        if self.project.name.trim().is_empty() {
            return Err(ProjectError::InvalidProjectManifest("project name cannot be empty".to_string()));
        }

        let display_name = self.project.name.clone();
        let namespace_text = self.project.namespace.as_ref().ok_or(ProjectError::MissingProjectNamespace)?;
        let namespace = ModuleComponent::from_identifier(namespace_text)
            .map_err(|e| ProjectError::InvalidProjectNamespace(namespace_text.clone(), e))?;

        for reserved in ["universe", "std", "core"] {
            if namespace.as_str() == reserved {
                return Err(ProjectError::ImportRootCollision {
                    alias: namespace.as_str().to_string(),
                    reason: format!("reserved builtin root '{reserved}'"),
                });
            }
        }

        let mut validated_deps: BTreeMap<ModuleComponent, (String, DependencySpec)> = BTreeMap::new();
        for (raw_alias, toml_val) in &self.dependencies {
            let component = ModuleComponent::from_identifier(raw_alias)
                .map_err(|e| ProjectError::InvalidDependencyAlias(raw_alias.clone(), e))?;

            if component == namespace {
                return Err(ProjectError::ImportRootCollision {
                    alias: raw_alias.clone(),
                    reason: "collides with self project namespace".to_string(),
                });
            }

            if matches!(component.as_str(), "universe" | "std" | "core") {
                return Err(ProjectError::ImportRootCollision {
                    alias: raw_alias.clone(),
                    reason: format!("reserved builtin root '{}'", component.as_str()),
                });
            }

            if let Some((previous, _)) = validated_deps.get(&component) {
                return Err(ProjectError::ImportRootCollision {
                    alias: raw_alias.clone(),
                    reason: format!("collides with alias '{previous}'"),
                });
            }

            let table = toml_val
                .as_table()
                .ok_or_else(|| ProjectError::InvalidProjectManifest(format!("dependency '{raw_alias}' must be a table")))?;

            let has_path = table.contains_key("path");
            let has_package = table.contains_key("package");
            let has_version = table.contains_key("version");

            if has_path && (has_package || has_version) {
                return Err(ProjectError::InvalidProjectManifest(format!(
                    "dependency '{raw_alias}' cannot specify both 'path' and 'package/version'"
                )));
            }

            let spec = if has_path {
                let path_dep: PathDependency = toml_val
                    .clone()
                    .try_into()
                    .map_err(|e: toml::de::Error| ProjectError::InvalidProjectManifest(format!("invalid path dependency '{raw_alias}': {e}")))?;
                DependencySpec::Path { path: path_dep.path }
            } else if has_package || has_version {
                let pkg_dep: PackageDependency = toml_val
                    .clone()
                    .try_into()
                    .map_err(|e: toml::de::Error| ProjectError::InvalidProjectManifest(format!("invalid package dependency '{raw_alias}': {e}")))?;
                DependencySpec::Package {
                    package: pkg_dep.package,
                    version: pkg_dep.version,
                }
            } else {
                return Err(ProjectError::InvalidProjectManifest(format!(
                    "dependency '{raw_alias}' must specify either 'path' or 'package' + 'version'"
                )));
            };

            validated_deps.insert(component, (raw_alias.clone(), spec));
        }

        if let Some(entry) = &self.project.entry {
            if entry.trim().is_empty() {
                return Err(ProjectError::InvalidEntry(entry.clone(), "entry path cannot be empty".to_string()));
            }
            let parts: Vec<&str> = entry.split('.').collect();
            if parts.is_empty() || parts[0] != namespace.as_str() {
                return Err(ProjectError::InvalidEntry(
                    entry.clone(),
                    format!("entry must be rooted at self namespace '{}'", namespace.as_str()),
                ));
            }
            for part in parts {
                ModuleComponent::from_identifier(part).map_err(|e| ProjectError::InvalidEntry(entry.clone(), e.to_string()))?;
            }
        }

        Ok(ValidatedProjectManifest {
            display_name,
            namespace,
            source: self.project.source.clone(),
            entry: self.project.entry.clone(),
            dependencies: validated_deps,
        })
    }
}

/// Resolved dependency source location from an external dependency provider.
#[derive(Debug, Clone)]
pub struct ResolvedDependencySource {
    pub manifest_path: PathBuf,
}

pub trait DependencyProvider {
    fn resolve_package(&self, package: &str, version_requirement: &str) -> Result<ResolvedDependencySource, ProjectError>;
}

pub struct NullDependencyProvider;

impl DependencyProvider for NullDependencyProvider {
    fn resolve_package(&self, package: &str, version_requirement: &str) -> Result<ResolvedDependencySource, ProjectError> {
        Err(ProjectError::UnresolvedPackageDependency {
            package: package.to_string(),
            version_requirement: version_requirement.to_string(),
        })
    }
}
