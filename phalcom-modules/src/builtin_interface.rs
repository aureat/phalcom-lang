//! Source-derived builtin interface generation with native metadata overlay.

use crate::builtin::UniverseSourceProvider;
use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{ModuleId, ProjectIdentity, SourceLocation};
use crate::interface::{DeclarationSurface, ExportSurface, InterfaceBuilder, UnlinkedExportTarget, UnlinkedModuleInterface};
use crate::source::ParsedModuleUnit;
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

type ParsedCache = Mutex<HashMap<ModuleId, Result<Arc<ParsedModuleUnit>, ModuleLoadError>>>;
type InterfaceCache = Mutex<HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>>;

static BUILTIN_PARSED_CACHE: OnceLock<ParsedCache> = OnceLock::new();
static BUILTIN_INTERFACE_CACHE: OnceLock<InterfaceCache> = OnceLock::new();

/// Canonical source declarations available in the Universe provider.
///
/// This catalog records source-owned declaration identity only. Native metadata may
/// associate runtime implementation details with these declarations, but it cannot
/// manufacture a second root-owned declaration.
#[derive(Clone, Debug, Default)]
pub struct UniverseSourceDeclarationCatalog {
    declarations: BTreeMap<ModuleId, BTreeMap<String, DeclarationSurface>>,
}

impl UniverseSourceDeclarationCatalog {
    /// Builds the catalog from each canonical Universe source module.
    pub fn build(provider: &UniverseSourceProvider) -> Result<Self, ModuleLoadError> {
        let mut declarations = BTreeMap::new();
        for node in provider.nodes() {
            let path: Vec<crate::identity::ModuleComponent> = node
                .path
                .iter()
                .map(|component| crate::identity::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                .collect();
            let module = ModuleId::universe(crate::identity::ModulePath::from_components(path));
            let parsed = BuiltinInterfaceBuilder::load_parsed(provider, &module)?;
            let interface = InterfaceBuilder::build(module.clone(), parsed.kind, &parsed.program)
                .map_err(|error| ModuleLoadError::Interface { module: module.clone(), error })?;
            declarations.insert(module, interface.declarations);
        }
        Ok(Self { declarations })
    }

    /// Resolves a native association to the declaration authored by its source module.
    pub fn declaration_for(&self, key: phalcom_native_meta::UniverseKey) -> Result<(ModuleId, String), ModuleLoadError> {
        let path: Vec<crate::identity::ModuleComponent> = key
            .source_path()
            .iter()
            .map(|component| crate::identity::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
            .collect();
        let module = ModuleId::universe(crate::identity::ModulePath::from_components(path));
        let name = key.name().to_string();
        if self.declarations.get(&module).is_some_and(|declarations| declarations.contains_key(&name)) {
            Ok((module, name))
        } else {
            Err(ModuleResolutionError::ModuleNotFound(format!("canonical Universe declaration {module}::{name} is absent from source")).into())
        }
    }
}

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
    pub fn load_parsed(provider: &UniverseSourceProvider, id: &ModuleId) -> Result<Arc<ParsedModuleUnit>, ModuleLoadError> {
        let mut guard = parsed_cache().lock().unwrap();
        if let Some(cached) = guard.get(id) {
            return cached.clone();
        }

        let result = Self::load_parsed_uncached(provider, id);
        guard.insert(id.clone(), result.clone());
        result
    }

    fn load_parsed_uncached(provider: &UniverseSourceProvider, id: &ModuleId) -> Result<Arc<ParsedModuleUnit>, ModuleLoadError> {
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
    pub fn build(provider: &UniverseSourceProvider, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
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
    pub fn build_from_parsed(provider: &UniverseSourceProvider, parsed: &ParsedModuleUnit) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        let mut iface = InterfaceBuilder::build(parsed.id.clone(), parsed.kind, &parsed.program).map_err(|e| ModuleLoadError::Interface {
            module: parsed.id.clone(),
            error: e,
        })?;

        if parsed.id.project == ProjectIdentity::Universe && parsed.id.path.is_root() {
            let catalog = UniverseSourceDeclarationCatalog::build(provider)?;
            for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|binding| binding.exported) {
                let name = binding.name.to_string();
                if !iface.exports.contains_key(&name) {
                    let (module, declaration_name) = catalog.declaration_for(binding.key)?;
                    iface.exports.insert(
                        name.clone(),
                        ExportSurface {
                            exported_name: name.clone(),
                            internal_name: name,
                            target: UnlinkedExportTarget::CanonicalDeclaration {
                                module,
                                name: declaration_name,
                            },
                            range: SourceRange::default(),
                        },
                    );
                }
            }
        }

        Ok(iface)
    }
}
