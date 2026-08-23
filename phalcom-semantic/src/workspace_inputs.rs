//! Workspace inputs and source overlay change models (Spec 12.3).

use crate::identity::ModuleId;
use phalcom_modules::identity::SourceLocation;
use phalcom_modules::source::ModuleKind;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct WorkspaceRootInput {
    pub root: PathBuf,
}

impl WorkspaceRootInput {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Clone, Debug)]
pub struct SourceOverlayUpdate {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub revision: u64,
    pub text: Arc<str>,
}

impl SourceOverlayUpdate {
    pub fn new(module: ModuleId, kind: ModuleKind, source: SourceLocation, revision: u64, text: Arc<str>) -> Self {
        Self {
            module,
            kind,
            source,
            revision,
            text,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SourceChange {
    Update(SourceOverlayUpdate),
    Remove(ModuleId),
}

#[derive(Debug)]
pub enum WorkspaceSessionError {
    Project(phalcom_modules::ProjectError),
    Resolution(phalcom_modules::ModuleResolutionError),
    Load(phalcom_modules::ModuleLoadError),
    Source(phalcom_modules::SourceError),
    UnknownDocument(PathBuf),
}

impl From<phalcom_modules::ProjectError> for WorkspaceSessionError {
    fn from(err: phalcom_modules::ProjectError) -> Self {
        Self::Project(err)
    }
}

impl From<phalcom_modules::ModuleResolutionError> for WorkspaceSessionError {
    fn from(err: phalcom_modules::ModuleResolutionError) -> Self {
        Self::Resolution(err)
    }
}

impl From<phalcom_modules::ModuleLoadError> for WorkspaceSessionError {
    fn from(err: phalcom_modules::ModuleLoadError) -> Self {
        Self::Load(err)
    }
}

impl From<phalcom_modules::SourceError> for WorkspaceSessionError {
    fn from(err: phalcom_modules::SourceError) -> Self {
        Self::Source(err)
    }
}
