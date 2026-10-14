//! Resource limits and validation budgets for untrusted artifact loading.

/// Resource limits for artifact parsing and deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationLimits {
    pub max_artifact_bytes: u64,
    pub max_section_count: usize,
    pub max_encoded_section_bytes: u64,
    pub max_decoded_section_bytes: u64,
    pub max_string_bytes: usize,
    pub max_modules: usize,
    pub max_type_nodes: usize,
    pub max_kind_nodes: usize,
    pub max_parameters: usize,
    pub max_static_values: usize,
    pub max_semantic_entries_per_family: usize,
    pub max_query_roots: usize,
    pub max_query_dependencies_per_root: usize,
    pub max_metadata_bytes: u64,
    pub max_recursion_depth: usize,
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            max_artifact_bytes: 256 * 1024 * 1024, // 256 MiB
            max_section_count: 64,
            max_encoded_section_bytes: 128 * 1024 * 1024, // 128 MiB
            max_decoded_section_bytes: 256 * 1024 * 1024, // 256 MiB
            max_string_bytes: 16 * 1024 * 1024,          // 16 MiB
            max_modules: 10_000,
            max_type_nodes: 1_000_000,
            max_kind_nodes: 1_000_000,
            max_parameters: 1_000_000,
            max_static_values: 1_000_000,
            max_semantic_entries_per_family: 1_000_000,
            max_query_roots: 100_000,
            max_query_dependencies_per_root: 10_000,
            max_metadata_bytes: 64 * 1024 * 1024, // 64 MiB
            max_recursion_depth: 256,
        }
    }
}
