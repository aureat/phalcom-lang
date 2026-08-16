//! Module and project semantic identity types.

use crate::error::InvalidModuleNameError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Opaque graph-node identity for a resolved project within a [`ProjectUniverse`](crate::project::ProjectUniverse).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ResolvedProjectId(pub u32);

impl fmt::Display for ResolvedProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "proj#{}", self.0)
    }
}

/// A validated snake_case component of a module path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModuleComponent(Box<str>);

impl ModuleComponent {
    /// Creates a component from a kebab-case string (e.g. from filesystem or manifest).
    /// Converts `-` to `_` then validates.
    pub fn from_kebab(s: &str) -> Result<Self, InvalidModuleNameError> {
        let converted = s.replace('-', "_");
        Self::from_identifier(&converted)
    }

    /// Creates a component directly from an identifier string (must be valid snake_case / identifier).
    pub fn from_identifier(s: &str) -> Result<Self, InvalidModuleNameError> {
        if s.is_empty() {
            return Err(InvalidModuleNameError::Empty);
        }
        let mut chars = s.chars();
        let first = chars.next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(InvalidModuleNameError::InvalidLeadingChar(s.to_string()));
        }
        for c in chars {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return Err(InvalidModuleNameError::InvalidChar(s.to_string(), c));
            }
        }
        Ok(Self(s.into()))
    }

    /// Returns string slice of the component.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ModuleComponent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A project-relative logical module path.
/// Root package is represented by an empty slice.
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModulePath(Box<[ModuleComponent]>);

impl ModulePath {
    /// The root package path (`[]`).
    pub fn root() -> Self {
        Self(Box::new([]))
    }

    /// Creates a module path from a slice of components.
    pub fn from_components(components: impl Into<Box<[ModuleComponent]>>) -> Self {
        Self(components.into())
    }

    /// Returns the parent path if not root.
    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            let parent_slice = &self.0[..self.0.len() - 1];
            Some(Self(parent_slice.into()))
        }
    }

    /// Joins a child component to this path.
    pub fn join(&self, component: ModuleComponent) -> Self {
        let mut vec = self.0.to_vec();
        vec.push(component);
        Self(vec.into_boxed_slice())
    }

    /// Slice of path components.
    pub fn components(&self) -> &[ModuleComponent] {
        &self.0
    }

    /// Whether this is the root package path.
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "<root>")
        } else {
            let joined = self.0.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(".");
            write!(f, "{joined}")
        }
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = self.0.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(".");
        f.write_str(&joined)
    }
}

/// Canonical toolchain identity for a module.
/// Combines a resolved project graph node with a project-relative module path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModuleId {
    pub project: ResolvedProjectId,
    pub path: ModulePath,
}

impl ModuleId {
    /// Canonical identity for the core module.
    pub fn core() -> Self {
        Self {
            project: ResolvedProjectId(0),
            path: ModulePath::from_components(vec![ModuleComponent::from_identifier("core").expect("valid identifier")]),
        }
    }

    /// Synthetic module identity for standalone or REPL code.
    pub fn synthetic(name: &str) -> Self {
        Self {
            project: ResolvedProjectId(0),
            path: ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap_or_else(|_| ModuleComponent(name.into()))]),
        }
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_root() {
            write!(f, "{}:<root>", self.project)
        } else {
            write!(f, "{}:{}", self.project, self.path)
        }
    }
}

/// Source-provider identity, distinct from semantic `ModuleId`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SourceId(pub Box<str>);

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Source location for diagnostics and UI mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    pub source_id: SourceId,
    pub display_path: PathBuf,
}

/// Physical or logical identity used to deduplicate resolved project graph nodes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectSourceIdentity(pub PathBuf);

impl ProjectSourceIdentity {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self(path.as_ref().to_path_buf())
    }
}
