//! Consumer-specific semantic publication requests.

/// Controls whether advisory runtime-shape products are published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvisoryLevel {
    /// Do not run advisory dataflow or publish advisory shards.
    None,
    /// Publish the complete advisory workspace product.
    Full,
}

/// Semantic products required by one compilation consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticProductRequest {
    pub advisory: AdvisoryLevel,
}

impl SemanticProductRequest {
    /// Product set for executable lowering and runtime compilation.
    pub const fn runtime_minimal() -> Self {
        Self { advisory: AdvisoryLevel::None }
    }

    /// Product set retained by semantic and tooling callers for compatibility.
    pub const fn full() -> Self {
        Self { advisory: AdvisoryLevel::Full }
    }
}

impl Default for SemanticProductRequest {
    fn default() -> Self {
        Self::full()
    }
}
