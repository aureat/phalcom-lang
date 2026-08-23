//! Source-derived builtin interface generation with native metadata overlay.

use crate::builtin::BuiltinProjectSourceProvider;
use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{BuiltinProject, ModuleId, ProjectIdentity, SourceLocation};
use crate::interface::{DeclarationSurface, ExportSurface, InterfaceBuilder, UnlinkedExportTarget, UnlinkedModuleInterface};
use crate::source::ParsedModuleUnit;
use phalcom_common::range::SourceRange;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

type ParsedCache = Mutex<HashMap<ModuleId, Result<Arc<ParsedModuleUnit>, ModuleLoadError>>>;
type InterfaceCache = Mutex<HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>>;

static BUILTIN_PARSED_CACHE: OnceLock<ParsedCache> = OnceLock::new();
static BUILTIN_INTERFACE_CACHE: OnceLock<InterfaceCache> = OnceLock::new();

fn parsed_cache() -> &'static Mutex<HashMap<ModuleId, Result<Arc<ParsedModuleUnit>, ModuleLoadError>>> {
    BUILTIN_PARSED_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn interface_cache() -> &'static Mutex<HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>> {
    BUILTIN_INTERFACE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Builder responsible for deriving the authoritative unlinked interface and parsed artifact of a builtin module.
#[derive(Debug)]
pub struct BuiltinInterfaceBuilder;

impl BuiltinInterfaceBuilder {
    /// Loads or retrieves the cached parsed unit for a builtin module identity.
    pub fn load_parsed(provider: &BuiltinProjectSourceProvider, id: &ModuleId) -> Result<Arc<ParsedModuleUnit>, ModuleLoadError> {
        let mut guard = parsed_cache().lock().unwrap();
        if let Some(cached) = guard.get(id) {
            return cached.clone();
        }

        let result = Self::load_parsed_uncached(provider, id);
        guard.insert(id.clone(), result.clone());
        result
    }

    fn load_parsed_uncached(provider: &BuiltinProjectSourceProvider, id: &ModuleId) -> Result<Arc<ParsedModuleUnit>, ModuleLoadError> {
        let source_text = provider.source_text(id)?;
        let source_id = provider.source_id(id)?;
        let display_path = std::path::PathBuf::from(source_id.0.as_ref());
        let parse_result = phalcom_ast::parse(&source_text, 0);
        if !parse_result.errors.is_empty() {
            let err = &parse_result.errors[0];
            return Err(ModuleLoadError::Parse {
                module: id.clone(),
                source: display_path,
                error: err.clone(),
            });
        }

        let kind = provider
            .kind(&id.path)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("builtin module {id} not found")))?;

        Ok(Arc::new(ParsedModuleUnit {
            id: id.clone(),
            kind,
            source: Some(SourceLocation { source_id, display_path }),
            text: source_text,
            program: Arc::new(parse_result.program),
        }))
    }

    /// Builds or retrieves the cached unlinked interface for a builtin module identity.
    pub fn build(provider: &BuiltinProjectSourceProvider, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        let mut guard = interface_cache().lock().unwrap();
        if let Some(cached) = guard.get(id) {
            return cached.clone();
        }

        let parsed = Self::load_parsed(provider, id)?;
        let result = Self::build_from_parsed(provider, &parsed);
        guard.insert(id.clone(), result.clone());
        result
    }

    /// Derives unlinked module interface from the canonical parsed source.
    pub fn build_from_parsed(_provider: &BuiltinProjectSourceProvider, parsed: &ParsedModuleUnit) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        let mut iface = InterfaceBuilder::build(parsed.id.clone(), parsed.kind, &parsed.program).map_err(|e| ModuleLoadError::Interface {
            module: parsed.id.clone(),
            error: e,
        })?;

        if parsed.id.project == ProjectIdentity::Builtin(BuiltinProject::Universe) {
            if parsed.id.path.is_root() {
                // Root/prelude bindings are policy data, not source declarations.
                // Module and class presentation data comes exclusively from the
                // parsed universe modules above.
                for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|binding| binding.exported) {
                    let name = binding.name.to_string();
                    let range = SourceRange::default();
                    if !iface.declarations.contains_key(&name) {
                        iface.declarations.insert(
                            name.clone(),
                            DeclarationSurface {
                                name: name.clone(),
                                is_const: true,
                                range,
                            },
                        );
                    }
                    if !iface.exports.contains_key(&name) {
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
            } else {
                // In non-root canonical universe modules, all declared classes are public exports of that module.
                let decl_names: Vec<(String, SourceRange)> = iface.declarations.iter().map(|(n, d)| (n.clone(), d.range)).collect();
                for (name, range) in decl_names {
                    if !iface.exports.contains_key(&name) {
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
            }
        }

        Ok(iface)
    }
}
