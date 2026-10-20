//! Consumer-specific semantic publication requests.

/// Controls whether advisory runtime-shape products are published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvisoryLevel {
    /// Do not run advisory dataflow or publish advisory shards.
    None,
    /// Publish the complete advisory workspace product.
    Full,
}

/// Controls publication of source-navigation and presentation products.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationLevel {
    /// Publish only source shards required to lower workspace runtime modules.
    None,
    /// Publish complete workspace and canonical Universe presentation products.
    Full,
}

/// Controls publication of compiler-required vs editor source semantic indexing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceSemanticLevel {
    /// Do not index source syntax for semantic navigation or lowering attachments.
    None,
    /// Index source shards required by compiler lowering and runtime execution.
    CompilerRequired,
    /// Index complete workspace and canonical Universe source shards for tooling and navigation.
    Full,
}

/// Semantic products required by one compilation consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticProductRequest {
    /// Requested source semantic index coverage.
    pub source_semantics: SourceSemanticLevel,
    /// Requested source presentation coverage.
    pub presentation: PresentationLevel,
    /// Requested advisory-analysis coverage.
    pub advisory: AdvisoryLevel,
    /// Whether checked static bodies must also publish their canonical
    /// operational programs for source-independent compiler images.
    pub canonical_static_programs: bool,
}

impl SemanticProductRequest {
    /// Product set for executable lowering and runtime compilation.
    pub const fn runtime_minimal() -> Self {
        Self {
            source_semantics: SourceSemanticLevel::CompilerRequired,
            presentation: PresentationLevel::None,
            advisory: AdvisoryLevel::None,
            canonical_static_programs: false,
        }
    }

    /// Product set retained by semantic and tooling callers for compatibility.
    pub const fn full() -> Self {
        Self {
            source_semantics: SourceSemanticLevel::Full,
            presentation: PresentationLevel::Full,
            advisory: AdvisoryLevel::Full,
            canonical_static_programs: false,
        }
    }

    /// Explicit canonical compiler-image product request.
    ///
    /// Excludes editor-only presentation text and advisory dataflow while
    /// publishing all compiler-required formal and source semantic products,
    /// including source-independent static programs.
    pub const fn canonical_compiler_image() -> Self {
        Self {
            source_semantics: SourceSemanticLevel::CompilerRequired,
            presentation: PresentationLevel::None,
            advisory: AdvisoryLevel::None,
            canonical_static_programs: true,
        }
    }
}

impl Default for SemanticProductRequest {
    fn default() -> Self {
        Self::full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_request_preserves_full_tooling_compatibility() {
        let default = SemanticProductRequest::default();
        let full = SemanticProductRequest::full();
        assert_eq!(default, full);
        assert_eq!(default.source_semantics, SourceSemanticLevel::Full);
        assert_eq!(default.presentation, PresentationLevel::Full);
        assert_eq!(default.advisory, AdvisoryLevel::Full);
    }

    #[test]
    fn runtime_minimal_retains_compiler_required_source_semantics() {
        let minimal = SemanticProductRequest::runtime_minimal();
        assert_eq!(minimal.source_semantics, SourceSemanticLevel::CompilerRequired);
        assert_eq!(minimal.presentation, PresentationLevel::None);
        assert_eq!(minimal.advisory, AdvisoryLevel::None);
        assert!(!minimal.canonical_static_programs);
    }

    #[test]
    fn canonical_compiler_image_is_explicit_and_omits_editor_products() {
        let canonical = SemanticProductRequest::canonical_compiler_image();
        assert_eq!(canonical.source_semantics, SourceSemanticLevel::CompilerRequired);
        assert_eq!(canonical.presentation, PresentationLevel::None);
        assert_eq!(canonical.advisory, AdvisoryLevel::None);
        assert!(canonical.canonical_static_programs);
        assert_ne!(canonical, SemanticProductRequest::default());
        assert_ne!(canonical, SemanticProductRequest::full());
        assert_ne!(canonical, SemanticProductRequest::runtime_minimal());
    }

    #[test]
    fn canonical_request_is_not_inferred_from_local_request() {
        let canonical = SemanticProductRequest::canonical_compiler_image();
        let mut local = canonical;
        local.advisory = AdvisoryLevel::Full;
        local.presentation = PresentationLevel::Full;
        assert_eq!(canonical.advisory, AdvisoryLevel::None);
        assert_eq!(canonical.presentation, PresentationLevel::None);
        assert_ne!(canonical, local);
    }
}
