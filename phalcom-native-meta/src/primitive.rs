//! Semantic primitive metadata and descriptor types.

use crate::types::{CallableTypeSpec, ParameterTupleSpec, TypeExprSpec};
use crate::universe::UniverseKey;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum NativeDispatch {
    Instance,
    Class,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum NativeVisibility {
    Public,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum NativeStability {
    Unspecified,
    Experimental,
    Stable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RaisesSpec {
    Unknown,
    Known(&'static [TypeExprSpec]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum NativeEffect {
    Mutation,
    Io,
    Scheduling,
    Reflection,
    Nondeterminism,
    Blocking,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EffectSpec {
    Unknown,
    Pure,
    Known(&'static [NativeEffect]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ReturnFlowSpec {
    Value,
    Receiver,
    Argument(usize),
    Never,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum NativeIntrinsicId {
    BoolAnd,
    BoolOr,
    BoolNot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum NativeTrust {
    Ordinary,
    Privileged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PrimitiveAbi {
    Value,
    Shape,
}

/// Release metadata attached to one native declaration.
///
/// Kept as one value so every projection can preserve lifecycle information
/// without reinterpreting the individual optional fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NativeLifecycleSpec {
    pub since: Option<&'static str>,
    pub deprecated_since: Option<&'static str>,
    pub replacement: Option<&'static str>,
}

impl NativeLifecycleSpec {
    pub const UNKNOWN: Self = Self {
        since: None,
        deprecated_since: None,
        replacement: None,
    };

    pub const fn is_consistent(self) -> bool {
        match self.replacement {
            Some(_) => self.deprecated_since.is_some(),
            None => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PrimitiveKey {
    pub owner: UniverseKey,
    pub side: NativeDispatch,
    pub selector: &'static str,
}

impl PrimitiveKey {
    pub const fn sort_key(&self) -> (UniverseKey, NativeDispatch, &'static str) {
        (self.owner, self.side, self.selector)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ImplementationKind {
    Source,
    NativePrimitive,
    Generated,
    Abstract,
    External,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSourceSpec {
    pub module_path: &'static str,
    pub rust_name: &'static str,
    pub file: &'static str,
    pub line: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TerminationSpec {
    #[default]
    Unknown,
    Terminates,
    MayDiverge,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeAnchorPolicy {
    #[default]
    Required,
    Hidden,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimitiveSurfaceSpec {
    pub key: PrimitiveKey,
    pub visibility: NativeVisibility,
    pub stability: NativeStability,
    pub anchor: NativeAnchorPolicy,

    pub params: &'static ParameterTupleSpec,
    pub returns: &'static TypeExprSpec,
    pub callable: &'static CallableTypeSpec,

    pub raises: RaisesSpec,
    pub effects: EffectSpec,
    pub flow: ReturnFlowSpec,
    pub termination: TerminationSpec,

    pub since: Option<&'static str>,
    pub deprecated_since: Option<&'static str>,
    pub replacement: Option<&'static str>,

    pub lifecycle: NativeLifecycleSpec,

    pub intrinsic: Option<NativeIntrinsicId>,
    pub trust: NativeTrust,
    pub docs: Option<&'static str>,
    pub conceptual: Option<&'static str>,
}
