//! Provider-backed builtin project interfaces and canonical virtual source identity.

use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{BuiltinProject, ModuleId, ProjectIdentity, SourceId};
use crate::interface::{DeclarationSurface, ExportSurface, UnlinkedExportTarget, UnlinkedModuleInterface};
use crate::metadata::ModuleMetadata;
use crate::source::ModuleKind;
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

/// Toolchain-owned source/interface authority for one builtin project.
#[derive(Clone, Copy, Debug)]
pub struct BuiltinProjectSourceProvider {
    builtin: BuiltinProject,
}

impl BuiltinProjectSourceProvider {
    pub const fn new(builtin: BuiltinProject) -> Self {
        Self { builtin }
    }

    /// Canonical virtual identity used by diagnostics/LSP for builtin documents.
    pub fn source_id(&self, id: &ModuleId) -> Result<SourceId, ModuleLoadError> {
        self.validate_id(id)?;
        let path = id.path.components().iter().map(|part| part.as_str()).collect::<Vec<_>>().join("/");
        Ok(SourceId(format!("phalcom://{}/{path}", self.builtin).into()))
    }

    /// Loads the immutable public interface of a builtin node.
    ///
    /// The universe root is backed by the VM-free generated native binding
    /// catalog; no filesystem Project or reserved numeric project ID participates.
    pub fn load_interface(&self, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        self.validate_id(id)?;
        if !id.path.is_root() {
            return Err(ModuleResolutionError::ModuleNotFound(format!("builtin module {id} is not available in this runtime floor")).into());
        }

        let mut declarations = BTreeMap::new();
        let mut exports = BTreeMap::new();
        if self.builtin == BuiltinProject::Universe {
            for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|binding| binding.exported) {
                let range = SourceRange::default();
                declarations.insert(
                    binding.name.to_string(),
                    DeclarationSurface {
                        name: binding.name.to_string(),
                        is_const: true,
                        range,
                    },
                );
                exports.insert(
                    binding.name.to_string(),
                    ExportSurface {
                        exported_name: binding.name.to_string(),
                        internal_name: binding.name.to_string(),
                        target: UnlinkedExportTarget::Local(binding.name.to_string()),
                        range,
                    },
                );
            }
        }

        Ok(UnlinkedModuleInterface {
            id: id.clone(),
            kind: ModuleKind::Package,
            declarations,
            exports,
            imports: Vec::new(),
            exposed_children: BTreeSet::new(),
            metadata: ModuleMetadata::default(),
        })
    }

    fn validate_id(&self, id: &ModuleId) -> Result<(), ModuleLoadError> {
        if id.project != ProjectIdentity::Builtin(self.builtin) {
            return Err(ModuleResolutionError::ModuleNotFound(format!("{id} does not belong to builtin project {}", self.builtin)).into());
        }
        Ok(())
    }
}
