//! Module and project semantic identity types.

use crate::error::InvalidModuleNameError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "proj#{}", self.0) }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BuiltinProject { Universe, Std }

impl BuiltinProject {
    pub const fn root_name(self) -> &'static str {
        match self { Self::Universe => "universe", Self::Std => "std" }
    }
}

impl fmt::Display for BuiltinProject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.root_name()) }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SyntheticProjectId(u64);

impl fmt::Display for SyntheticProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "synthetic#{}", self.0) }
}

#[derive(Debug, Default)]
pub struct SyntheticProjectIdAllocator;

static NEXT_SYNTHETIC_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

impl SyntheticProjectIdAllocator {
    pub const fn new() -> Self { Self }

    pub fn allocate(&mut self) -> SyntheticProjectId {
        let id = NEXT_SYNTHETIC_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
        assert!(id != 0, "synthetic project identity space exhausted");
        SyntheticProjectId(id)
    }
}

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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImportRootTarget {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
}

impl ImportRootTarget {
    /// Source-compatibility spelling for the transitional `core` root. It is a
    /// complete alias of Universe, not a separate identity category.
    #[allow(non_upper_case_globals)]
    pub const Core: Self = Self::Builtin(BuiltinProject::Universe);
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModuleComponent(Box<str>);

impl ModuleComponent {
    pub fn from_kebab(s: &str) -> Result<Self, InvalidModuleNameError> {
        if !is_canonical_kebab(s) {
            return Err(if s.is_empty() { InvalidModuleNameError::Empty } else { InvalidModuleNameError::NonCanonicalKebabCase(s.to_string()) });
        }
        Self::from_identifier(&s.replace('-', "_"))
    }

    pub fn from_identifier(s: &str) -> Result<Self, InvalidModuleNameError> {
        if s.is_empty() { return Err(InvalidModuleNameError::Empty); }
        if !is_canonical_snake(s) { return Err(InvalidModuleNameError::NonCanonicalSnakeCase(s.to_string())); }
        Ok(Self(s.into()))
    }

    pub fn as_str(&self) -> &str { &self.0 }
    pub fn to_kebab(&self) -> String { self.0.replace('_', "-") }
}

fn is_canonical_snake(s: &str) -> bool {
    let mut parts = s.split('_');
    let Some(first) = parts.next() else { return false };
    is_word(first) && parts.all(is_word)
}

fn is_canonical_kebab(s: &str) -> bool {
    let mut parts = s.split('-');
    let Some(first) = parts.next() else { return false };
    is_word(first) && parts.all(is_word)
}

fn is_word(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase()) && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

impl fmt::Display for ModuleComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}
impl AsRef<str> for ModuleComponent { fn as_ref(&self) -> &str { &self.0 } }

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModulePath(Box<[ModuleComponent]>);

impl ModulePath {
    pub fn root() -> Self { Self(Box::new([])) }
    pub fn from_components(components: impl Into<Box<[ModuleComponent]>>) -> Self { Self(components.into()) }
    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() { None } else { Some(Self(self.0[..self.0.len() - 1].into())) }
    }
    pub fn join(&self, component: ModuleComponent) -> Self {
        let mut vec = self.0.to_vec();
        vec.push(component);
        Self(vec.into_boxed_slice())
    }
    pub fn components(&self) -> &[ModuleComponent] { &self.0 }
    pub fn is_root(&self) -> bool { self.0.is_empty() }
    pub fn physical_components(&self) -> impl Iterator<Item = String> + '_ { self.0.iter().map(ModuleComponent::to_kebab) }
}

impl fmt::Debug for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() { write!(f, "<root>") } else { write!(f, "{}", self) }
    }
}
impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.iter().map(|c| c.as_str()).collect::<Vec<_>>().join("."))
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleId {
    pub project: ProjectIdentity,
    pub path: ModulePath,
}

impl ModuleId {
    pub fn resolved(project: ResolvedProjectId, path: ModulePath) -> Self {
        Self { project: ProjectIdentity::Resolved(project), path }
    }

    pub fn builtin(project: BuiltinProject, path: ModulePath) -> Self {
        Self { project: ProjectIdentity::Builtin(project), path }
    }

    pub fn synthetic_in(project: SyntheticProjectId, path: ModulePath) -> Self {
        Self { project: ProjectIdentity::Synthetic(project), path }
    }

    /// Compatibility convenience for ad-hoc runtime/test modules. The identity
    /// is fresh on every call; `logical_name` contributes only the logical path,
    /// never the ownership identity.
    pub fn synthetic(logical_name: &str) -> Self {
        let mut allocator = SyntheticProjectIdAllocator::new();
        let project = allocator.allocate();
        let component = ModuleComponent::from_identifier(logical_name)
            .or_else(|_| ModuleComponent::from_kebab(logical_name))
            .ok();
        let path = component.map(|c| ModulePath::from_components(vec![c])).unwrap_or_else(ModulePath::root);
        Self::synthetic_in(project, path)
    }

    pub fn universe() -> Self { Self::builtin(BuiltinProject::Universe, ModulePath::root()) }
    pub fn std() -> Self { Self::builtin(BuiltinProject::Std, ModulePath::root()) }
    pub fn core() -> Self { Self::universe() }

    pub fn resolved_project(&self) -> Option<ResolvedProjectId> {
        match self.project { ProjectIdentity::Resolved(id) => Some(id), _ => None }
    }
    pub fn builtin_project(&self) -> Option<BuiltinProject> {
        match self.project { ProjectIdentity::Builtin(project) => Some(project), _ => None }
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_root() { write!(f, "{}:<root>", self.project) } else { write!(f, "{}:{}", self.project, self.path) }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SourceId(pub Box<str>);
impl fmt::Display for SourceId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) } }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    pub source_id: SourceId,
    pub display_path: PathBuf,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectSourceIdentity(pub PathBuf);
impl ProjectSourceIdentity {
    pub fn from_path(path: impl AsRef<Path>) -> Self { Self(path.as_ref().to_path_buf()) }
}
