//! Module and project semantic identity types.

use crate::error::InvalidModuleNameError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Opaque graph-node identity for a persistent resolved project within a
/// [`ProjectUniverse`](crate::project::ProjectUniverse).
///
/// The numeric payload is intentionally private and non-zero. Builtin and
/// synthetic identity are represented by [`ProjectIdentity`] variants rather
/// than by numeric conventions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolvedProjectId(NonZeroU32);

impl ResolvedProjectId {
    pub(crate) fn from_index(index: usize) -> Self {
        let raw = u32::try_from(index + 1).expect("project universe cannot contain more than u32::MAX projects");
        Self(NonZeroU32::new(raw).expect("index + 1 is non-zero"))
    }

    pub(crate) fn index(self) -> usize {
        usize::try_from(self.0.get() - 1).expect("u32 fits usize on supported targets")
    }
}

impl fmt::Display for ResolvedProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "proj#{}", self.0)
    }
}

/// Builtin toolchain-owned project roots.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BuiltinProject {
    Universe,
    Std,
}

impl BuiltinProject {
    pub const fn root_name(self) -> &'static str {
        match self {
            Self::Universe => "universe",
            Self::Std => "std",
        }
    }
}

impl fmt::Display for BuiltinProject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.root_name())
    }
}

/// Opaque identity for one synthetic project/unit family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SyntheticProjectId(u64);

impl fmt::Display for SyntheticProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "synthetic#{}", self.0)
    }
}

/// Allocator used by compile/runtime sessions when they need fresh synthetic
/// identities. IDs come from one process-wide monotonic sequence so distinct
/// session allocators cannot alias one another.
#[derive(Debug, Default)]
pub struct SyntheticProjectIdAllocator;

static NEXT_SYNTHETIC_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

impl SyntheticProjectIdAllocator {
    pub const fn new() -> Self {
        Self
    }

    pub fn allocate(&mut self) -> SyntheticProjectId {
        let id = NEXT_SYNTHETIC_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
        assert!(id != 0, "synthetic project identity space exhausted");
        SyntheticProjectId(id)
    }
}

/// Semantic category and identity of the project/root owning a module.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProjectIdentity {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}

impl fmt::Display for ProjectIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin(project) => write!(f, "builtin:{project}"),
            Self::Resolved(project) => project.fmt(f),
            Self::Synthetic(project) => project.fmt(f),
        }
    }
}

/// Target of an absolute import root in a project's root table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImportRootTarget {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
}

/// A validated canonical snake_case component of a logical module path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModuleComponent(Box<str>);

impl ModuleComponent {
    /// Creates a logical component from canonical physical kebab-case.
    pub fn from_kebab(s: &str) -> Result<Self, InvalidModuleNameError> {
        if !is_canonical_kebab(s) {
            return Err(if s.is_empty() {
                InvalidModuleNameError::Empty
            } else {
                InvalidModuleNameError::NonCanonicalKebabCase(s.to_string())
            });
        }
        Self::from_identifier(&s.replace('-', "_"))
    }

    /// Creates a logical component directly from canonical snake_case.
    pub fn from_identifier(s: &str) -> Result<Self, InvalidModuleNameError> {
        if s.is_empty() {
            return Err(InvalidModuleNameError::Empty);
        }
        if !is_canonical_snake(s) {
            return Err(InvalidModuleNameError::NonCanonicalSnakeCase(s.to_string()));
        }
        Ok(Self(s.into()))
    }

    /// Returns the logical snake_case spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the unique physical kebab-case spelling for this component.
    pub fn to_kebab(&self) -> String {
        self.0.replace('_', "-")
    }
}

fn is_canonical_snake(s: &str) -> bool {
    let mut parts = s.split('_');
    let Some(first) = parts.next() else { return false };
    if !is_word(first) {
        return false;
    }
    parts.all(is_word)
}

fn is_canonical_kebab(s: &str) -> bool {
    let mut parts = s.split('-');
    let Some(first) = parts.next() else { return false };
    if !is_word(first) {
        return false;
    }
    parts.all(is_word)
}

fn is_word(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase()) && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
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

/// A project-relative logical module path. Root package is represented by an
/// empty slice.
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModulePath(Box<[ModuleComponent]>);

impl ModulePath {
    pub fn root() -> Self {
        Self(Box::new([]))
    }

    pub fn from_components(components: impl Into<Box<[ModuleComponent]>>) -> Self {
        Self(components.into())
    }

    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            Some(Self(self.0[..self.0.len() - 1].into()))
        }
    }

    pub fn join(&self, component: ModuleComponent) -> Self {
        let mut vec = self.0.to_vec();
        vec.push(component);
        Self(vec.into_boxed_slice())
    }

    pub fn components(&self) -> &[ModuleComponent] {
        &self.0
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// Deterministic physical path components for this logical path.
    pub fn physical_components(&self) -> impl Iterator<Item = String> + '_ {
        self.0.iter().map(ModuleComponent::to_kebab)
    }
}

impl fmt::Debug for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "<root>")
        } else {
            write!(f, "{}", self)
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

    pub fn synthetic(project: SyntheticProjectId, path: ModulePath) -> Self {
        Self {
            project: ProjectIdentity::Synthetic(project),
            path,
        }
    }

    pub fn universe() -> Self {
        Self::builtin(BuiltinProject::Universe, ModulePath::root())
    }

    pub fn std() -> Self {
        Self::builtin(BuiltinProject::Std, ModulePath::root())
    }

    /// Transitional compatibility shim. Public `core` is the builtin Universe
    /// root identity; it is not a third identity category.
    pub fn core() -> Self {
        Self::universe()
    }

    pub fn resolved_project(&self) -> Option<ResolvedProjectId> {
        match self.project {
            ProjectIdentity::Resolved(id) => Some(id),
            _ => None,
        }
    }

    pub fn builtin_project(&self) -> Option<BuiltinProject> {
        match self.project {
            ProjectIdentity::Builtin(project) => Some(project),
            _ => None,
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
