//! Module and project semantic identity types.

use crate::error::InvalidModuleNameError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A toolchain-owned builtin project identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BuiltinProject {
    Universe,
    Std,
}

impl BuiltinProject {
    pub const fn import_root(self) -> &'static str {
        match self {
            Self::Universe => "universe",
            Self::Std => "std",
        }
    }
}

impl fmt::Display for BuiltinProject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.import_root())
    }
}

/// Opaque graph-node identity for a resolved user project.
///
/// Zero is unrepresentable: builtin and synthetic identities are separate enum
/// variants rather than numeric conventions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolvedProjectId(NonZeroU32);

impl ResolvedProjectId {
    /// Constructs a resolved-project ID. `raw` must be non-zero.
    pub fn from_raw(raw: u32) -> Self {
        Self(NonZeroU32::new(raw).expect("resolved project IDs are non-zero"))
    }

    /// Returns the raw non-zero graph-node number.
    pub const fn raw(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for ResolvedProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "proj#{}", self.raw())
    }
}

/// Process/session-local synthetic project identity used by inline and standalone execution.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SyntheticProjectId(u64);

impl SyntheticProjectId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SyntheticProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "synthetic#{}", self.0)
    }
}

static NEXT_SYNTHETIC_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

/// Process-wide monotonic allocator for synthetic execution identities.
///
/// Allocator values are intentionally stateless: creating another compiler or
/// ProjectUniverse cannot restart the sequence and collide with already loaded
/// standalone/inline modules in the same process.
#[derive(Debug, Default)]
pub struct SyntheticProjectIdAllocator;

impl SyntheticProjectIdAllocator {
    pub fn allocate(&mut self) -> SyntheticProjectId {
        let raw = NEXT_SYNTHETIC_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
        assert!(raw != u64::MAX, "synthetic project identity exhausted");
        SyntheticProjectId(raw)
    }
}

/// Semantic project category. The variants are intentionally disjoint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProjectIdentity {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}

impl ProjectIdentity {
    pub const fn as_resolved(self) -> Option<ResolvedProjectId> {
        match self {
            Self::Resolved(id) => Some(id),
            _ => None,
        }
    }
}

impl From<ResolvedProjectId> for ProjectIdentity {
    fn from(value: ResolvedProjectId) -> Self {
        Self::Resolved(value)
    }
}

impl From<SyntheticProjectId> for ProjectIdentity {
    fn from(value: SyntheticProjectId) -> Self {
        Self::Synthetic(value)
    }
}

impl fmt::Display for ProjectIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin(builtin) => write!(f, "builtin:{builtin}"),
            Self::Resolved(project) => project.fmt(f),
            Self::Synthetic(project) => project.fmt(f),
        }
    }
}

/// Target of an absolute import root in a resolved project's root table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImportRootTarget {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
}

/// A validated snake_case component of a module path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModuleComponent(Box<str>);

impl ModuleComponent {
    /// Creates a component from a canonical physical kebab-case spelling.
    /// Physical underscores and mixed case are rejected before separator conversion.
    pub fn from_kebab(s: &str) -> Result<Self, InvalidModuleNameError> {
        if s.is_empty() {
            return Err(InvalidModuleNameError::Empty);
        }
        let mut chars = s.chars();
        let first = chars.next().expect("checked non-empty");
        if !first.is_ascii_lowercase() {
            return Err(InvalidModuleNameError::InvalidPhysicalForm(s.to_string()));
        }
        if chars.any(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')) || s.contains('_') {
            return Err(InvalidModuleNameError::InvalidPhysicalForm(s.to_string()));
        }
        let converted = s.replace('-', "_");
        Self::from_identifier(&converted)
    }

    /// Creates a component from its canonical logical snake_case spelling.
    pub fn from_identifier(s: &str) -> Result<Self, InvalidModuleNameError> {
        if s.is_empty() {
            return Err(InvalidModuleNameError::Empty);
        }
        let mut chars = s.chars();
        let first = chars.next().expect("checked non-empty");
        if !first.is_ascii_lowercase() && first != '_' {
            return Err(InvalidModuleNameError::InvalidLeadingChar(s.to_string()));
        }
        for c in chars {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' {
                return Err(InvalidModuleNameError::InvalidChar(s.to_string(), c));
            }
        }
        Ok(Self(s.into()))
    }

    /// Canonical physical spelling used for source discovery.
    pub fn to_kebab(&self) -> String {
        self.0.replace('_', "-")
    }

    /// Canonical physical kebab-case representation.
    pub fn physical_kebab(&self) -> String {
        self.to_kebab()
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
/// Combines a disjoint project identity with a project-relative module path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleId {
    pub project: ProjectIdentity,
    pub path: ModulePath,
}

impl ModuleId {
    pub fn resolved(project: ResolvedProjectId, path: ModulePath) -> Self {
        Self {
            project: ProjectIdentity::Resolved(project),
            path,
        }
    }

    pub fn builtin(project: BuiltinProject, path: ModulePath) -> Self {
        Self {
            project: ProjectIdentity::Builtin(project),
            path,
        }
    }

    /// Temporary compatibility identity for the legacy core module.
    /// It remains structurally builtin while staying distinct from `universe/`.
    pub fn core() -> Self {
        Self::builtin(
            BuiltinProject::Universe,
            ModulePath::from_components(vec![ModuleComponent::from_identifier("core").expect("valid identifier")]),
        )
    }

    /// Synthetic module identity. The path does not participate in allocation;
    /// callers must supply an ID from a monotonic allocator.
    pub fn synthetic(project: SyntheticProjectId, path: ModulePath) -> Self {
        Self {
            project: ProjectIdentity::Synthetic(project),
            path,
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
