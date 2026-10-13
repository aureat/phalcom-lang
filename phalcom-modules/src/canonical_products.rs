//! Immutable module-layer inputs retained by a completed semantic Universe image.
//!
//! This is the bounded P1 dependency adapter, not a replacement module linker or
//! the future external-module image. Interfaces come from the provider cache.

use crate::{LinkedProgram, ModuleComponent, ModuleId, ModulePath, ParsedModuleUnit, UniverseSourceProvider, UnlinkedModuleInterface};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Complete canonical module inputs with deterministic source provenance.
#[derive(Debug)]
pub struct CanonicalModuleProducts {
    linked: Arc<LinkedProgram>,
    sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    interfaces: BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>>,
    source_fingerprint: u64,
}

impl CanonicalModuleProducts {
    /// Captures provider-backed inputs for the already linked foundation.
    /// No interfaces are reconstructed from parsed source at this boundary.
    pub fn capture(provider: &UniverseSourceProvider, linked: Arc<LinkedProgram>) -> Result<Self, String> {
        let mut sources = BTreeMap::new();
        let mut interfaces = BTreeMap::new();
        for node in provider.nodes() {
            let components = node
                .path
                .iter()
                .map(|part| ModuleComponent::from_identifier(part))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?;
            let module = ModuleId::universe(ModulePath::from_components(components));
            let source = provider.load_parsed(&module).map_err(|error| error.to_string())?;
            let interface = provider.load_interface(&module).map_err(|error| error.to_string())?;
            sources.insert(module.clone(), source);
            interfaces.insert(module, Arc::new(interface));
        }
        if linked.modules.len() != sources.len() || linked.modules.keys().any(|module| !sources.contains_key(module)) {
            return Err("canonical linked/source module sets differ".into());
        }
        // FNV-1a with length-delimited inputs; process provenance, not a portable
        // artifact identity. Interface meaning is derived from these exact texts
        // by this binary's module ABI (included in the semantic-world contract).
        let mut fingerprint = 0xcbf29ce484222325_u64;
        for (module, source) in &sources {
            for bytes in [module.to_string().as_bytes(), source.text.as_bytes()] {
                for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
                    fingerprint = (fingerprint ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
                }
            }
        }
        Ok(Self {
            linked,
            sources,
            interfaces,
            source_fingerprint: fingerprint,
        })
    }

    /// The foundation's authoritative linked program.
    pub fn linked(&self) -> &Arc<LinkedProgram> {
        &self.linked
    }
    /// Complete source modules, including modules outside runtime reachability.
    pub fn sources(&self) -> &BTreeMap<ModuleId, Arc<ParsedModuleUnit>> {
        &self.sources
    }
    /// Provider-derived interfaces used to build topology.
    pub fn interfaces(&self) -> &BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>> {
        &self.interfaces
    }
    /// Deterministic digest of module identities and exact provider source texts.
    pub fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }
}
