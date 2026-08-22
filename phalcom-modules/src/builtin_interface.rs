//! Source-derived builtin interface generation with native metadata overlay.

use crate::builtin::BuiltinProjectSourceProvider;
use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{BuiltinProject, ModuleId, ProjectIdentity};
use crate::interface::{DeclarationSurface, ExportSurface, InterfaceBuilder, UnlinkedExportTarget, UnlinkedModuleInterface};
use phalcom_common::range::SourceRange;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static BUILTIN_INTERFACE_CACHE: OnceLock<Mutex<HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>> {
    BUILTIN_INTERFACE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Builder responsible for deriving the authoritative unlinked interface of a builtin module.
#[derive(Debug)]
pub struct BuiltinInterfaceBuilder;

impl BuiltinInterfaceBuilder {
    /// Builds or retrieves the cached unlinked interface for a builtin module identity.
    pub fn build(provider: &BuiltinProjectSourceProvider, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        let mut guard = cache().lock().unwrap();
        if let Some(cached) = guard.get(id) {
            return cached.clone();
        }

        let result = Self::build_uncached(provider, id);
        guard.insert(id.clone(), result.clone());
        result
    }

    fn build_uncached(provider: &BuiltinProjectSourceProvider, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        let source_text = provider.source_text(id)?;
        let source_id = provider.source_id(id)?;
        let source_path = std::path::PathBuf::from(source_id.0.as_ref());
        let parse_result = phalcom_ast::parse(&source_text, 0);
        if !parse_result.errors.is_empty() {
            let err = &parse_result.errors[0];
            return Err(ModuleLoadError::Parse {
                module: id.clone(),
                source: source_path,
                error: err.clone(),
            });
        }

        let kind = provider
            .kind(&id.path)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("builtin module {id} not found")))?;

        let mut iface =
            InterfaceBuilder::build(id.clone(), kind, &parse_result.program).map_err(|e| ModuleLoadError::Interface { module: id.clone(), error: e })?;

        // Overlay primordial native universe bindings for the universe root package or matching submodules
        if id.project == ProjectIdentity::Builtin(BuiltinProject::Universe) {
            let names: Vec<&'static str> = if id.path.is_root() {
                phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|b| b.exported).map(|b| b.name).collect()
            } else {
                let comps: Vec<&str> = id.path.components().iter().map(|c| c.as_str()).collect();
                match comps.as_slice() {
                    ["reflection", "selector"] => vec!["Selector", "SelectorPattern"],
                    ["reflection", "message"] => vec!["Message"],
                    ["reflection", "attribute"] => vec!["Attribute"],
                    ["reflection", "module"] => vec!["Module"],
                    ["reflection", "package_object"] => vec!["Package"],
                    ["reflection", "project"] => vec!["Project"],
                    ["reflection", "uri"] => vec!["Uri"],
                    ["reflection", "module_identity"] => vec!["ModuleIdentity"],
                    ["reflection", "package_identity"] => vec!["PackageIdentity"],
                    ["reflection", "project_identity"] => vec!["ProjectIdentity"],
                    ["concurrency", "fiber"] => vec!["Fiber"],
                    ["errors", "error"] => vec!["Error", "MessageNotUnderstood", "CannotYieldAcrossNativeFrame", "UseAfterCloseError"],
                    ["object", "object"] => vec!["Object"],
                    ["object", "behavior"] => vec!["Behavior"],
                    ["object", "class"] => vec!["Class"],
                    ["object", "metaclass"] => vec!["Metaclass"],
                    ["scalar", "number"] => vec!["Number", "Int", "Float"],
                    ["scalar", "string"] => vec!["String"],
                    ["scalar", "bool"] => vec!["Bool", "True", "False"],
                    ["scalar", "symbol"] => vec!["Symbol"],
                    ["callable", "function"] => vec!["Function"],
                    ["callable", "closure"] => vec!["Closure"],
                    ["callable", "method"] => vec!["Method", "BoundMethod"],
                    ["callable", "family"] => vec!["Family", "MethodFamily", "BoundMethodFamily"],
                    ["option", "option"] => vec!["Option", "Some", "None", "Unit"],
                    ["collections", "list"] => vec!["List"],
                    ["collections", "map"] => vec!["Map"],
                    ["collections", "set"] => vec!["Set"],
                    ["collections", "tuple"] => vec!["Tuple"],
                    ["collections", "record"] => vec!["Record"],
                    ["collections", "range"] => vec!["Range"],
                    ["collections", "bytes"] => vec!["Bytes"],
                    ["collections", "iterable"] => vec!["Iterable"],
                    _ => vec![],
                }
            };

            for name_str in names {
                let range = SourceRange::default();
                let name = name_str.to_string();

                if iface.declarations.contains_key(&name) {
                    continue;
                }

                iface.declarations.insert(
                    name.clone(),
                    DeclarationSurface {
                        name: name.clone(),
                        is_const: true,
                        range,
                    },
                );

                if iface.exports.contains_key(&name) {
                    continue;
                }

                iface.exports.insert(
                    name.clone(),
                    ExportSurface {
                        exported_name: name.clone(),
                        internal_name: name.clone(),
                        target: UnlinkedExportTarget::Local(name),
                        range,
                    },
                );
            }
        }

        Ok(iface)
    }
}
