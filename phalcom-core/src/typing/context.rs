//! Runtime typing context, descriptor weak cache, and capabilities.

use crate::heap::ObjRef;
use crate::typing::capability::{TypingCapabilities, TypingCapability};
use crate::typing::handle::{MetadataPoolId, RuntimeSemanticHandle};
use crate::typing::overlay::RuntimeTypingOverlay;
use phalcom_type_meta::header::MetadataProfile;
use std::collections::HashMap;

/// Runtime typing context data attached to typing context objects.
#[derive(Clone, Debug)]
pub struct TypingContextData {
    pub base_pools: Box<[MetadataPoolId]>,
    pub profile: MetadataProfile,
    pub capabilities: TypingCapabilities,
    pub overlay: RuntimeTypingOverlay,
    /// Weak cache of descriptor objects. NOT traced by the garbage collector!
    pub descriptor_cache: HashMap<RuntimeSemanticHandle, ObjRef>,
}

impl TypingContextData {
    pub fn new(base_pools: Box<[MetadataPoolId]>) -> Self {
        Self::with_profile(base_pools, MetadataProfile::RuntimePublic)
    }

    pub fn with_profile(base_pools: Box<[MetadataPoolId]>, profile: MetadataProfile) -> Self {
        Self {
            base_pools,
            profile,
            capabilities: TypingCapabilities::for_profile(profile),
            overlay: RuntimeTypingOverlay::new(),
            descriptor_cache: HashMap::new(),
        }
    }

    pub fn with_capabilities(base_pools: Box<[MetadataPoolId]>, profile: MetadataProfile, capabilities: TypingCapabilities) -> Self {
        Self {
            base_pools,
            profile,
            capabilities,
            overlay: RuntimeTypingOverlay::new(),
            descriptor_cache: HashMap::new(),
        }
    }

    pub fn can(&self, capability: TypingCapability) -> bool {
        self.capabilities.contains(capability)
    }
}
