//! Immutable module-layer inputs retained by a completed semantic Universe image.
//!
//! Completed external authority is independent of source-backed build products.
//! Source compilation captures interfaces from the provider cache once.

use crate::authority::ModuleAuthorityMap;
use crate::{LinkedProgram, ModuleComponent, ModuleId, ModulePath, ModuleTopology, ParsedModuleUnit, UniverseSourceProvider, UnlinkedModuleInterface};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Source-free, completed canonical module authority consumed by local workspaces.
#[derive(Debug)]
pub struct CanonicalExternalModuleWorld {
    linked: Arc<LinkedProgram>,
    interfaces: ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>>,
    topology: Arc<ModuleTopology>,
}

impl CanonicalExternalModuleWorld {
    /// Materializes completed authority; does not parse, build interfaces, or link.
    pub fn from_parts(
        linked: Arc<LinkedProgram>,
        interfaces: ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>>,
        topology: Arc<ModuleTopology>,
    ) -> Result<Self, String> {
        if linked.modules.len() != topology.nodes.len()
            || linked.modules.len() != interfaces.len()
            || linked.modules.keys().any(|id| !interfaces.contains_key(id) || !topology.nodes.contains_key(id))
        {
            return Err("canonical linked/interface/topology module sets differ".into());
        }
        Ok(Self { linked, interfaces, topology })
    }

    pub fn linked(&self) -> &Arc<LinkedProgram> { &self.linked }
    pub fn interfaces(&self) -> &ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>> { &self.interfaces }
    pub fn topology(&self) -> &Arc<ModuleTopology> { &self.topology }
}

/// Complete canonical module inputs with deterministic source provenance.
#[derive(Debug)]
pub struct CanonicalModuleProducts {
    external: Arc<CanonicalExternalModuleWorld>,
    sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
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
        let plain_interfaces = interfaces.iter().map(|(id, iface)| (id.clone(), (**iface).clone())).collect();
        let topology = Arc::new(ModuleTopology::from_parts(crate::ResolverGeneration(0), &linked.universe, &plain_interfaces, &BTreeMap::new()));
        let external = Arc::new(CanonicalExternalModuleWorld::from_parts(linked, interfaces.into(), topology)?);
        Ok(Self {
            external,
            sources,
            source_fingerprint: fingerprint,
        })
    }

    /// The foundation's authoritative linked program.
    pub fn linked(&self) -> &Arc<LinkedProgram> {
        self.external.linked()
    }
    /// Source-free dependency authority, without build provenance.
    pub fn external(&self) -> &Arc<CanonicalExternalModuleWorld> { &self.external }
    /// Complete source modules, including modules outside runtime reachability.
    pub fn sources(&self) -> &BTreeMap<ModuleId, Arc<ParsedModuleUnit>> {
        &self.sources
    }
    /// Provider-derived interfaces used to build topology.
    pub fn interfaces(&self) -> &ModuleAuthorityMap<ModuleId, Arc<UnlinkedModuleInterface>> {
        self.external.interfaces()
    }
    /// Deterministic digest of module identities and exact provider source texts.
    pub fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }
}
