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

/// Semantic products required by one compilation consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticProductRequest {
    /// Requested source presentation coverage.
    pub presentation: PresentationLevel,
    /// Requested advisory-analysis coverage.
    pub advisory: AdvisoryLevel,
}

impl SemanticProductRequest {
    /// Product set for executable lowering and runtime compilation.
    pub const fn runtime_minimal() -> Self {
        Self {
            presentation: PresentationLevel::None,
            advisory: AdvisoryLevel::None,
        }
    }

    /// Product set retained by semantic and tooling callers for compatibility.
    pub const fn full() -> Self {
        Self {
            presentation: PresentationLevel::Full,
            advisory: AdvisoryLevel::Full,
        }
    }
}

impl Default for SemanticProductRequest {
    fn default() -> Self {
        Self::full()
    }
}
