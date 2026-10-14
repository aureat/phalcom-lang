//! Durable semantic metadata export, reachability, and fingerprinting.

pub mod export;
pub mod import;
pub mod stable_identity;
pub mod static_value;

pub use export::{MetadataExportError, MetadataExporter};
pub use import::SemanticMetadataImporter;
