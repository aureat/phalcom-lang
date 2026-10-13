//! Durable semantic metadata export, reachability, and fingerprinting.

pub mod export;
pub mod reachability;
pub mod stable_identity;

pub use export::{MetadataExportError, MetadataExporter};
pub use reachability::{
    compute_reachability_closure, MetadataReachabilityClosure, MetadataRetentionRequest, MetadataRootSet, ReachabilityMetrics,
};
