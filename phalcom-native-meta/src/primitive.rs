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

#[derive(Clone, Copy, Debug)]
pub struct NativeSourceSpec {
    pub module_path: &'static str,
    pub rust_name: &'static str,
    pub file: &'static str,
    pub line: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PrimitiveSurfaceSpec {
    pub key: PrimitiveKey,
    pub visibility: NativeVisibility,
    pub stability: NativeStability,

    pub params: &'static ParameterTupleSpec,
    pub returns: &'static TypeExprSpec,
    pub callable: &'static CallableTypeSpec,

    pub raises: RaisesSpec,
    pub effects: EffectSpec,
    pub flow: ReturnFlowSpec,

    pub since: Option<&'static str>,
    pub deprecated_since: Option<&'static str>,
    pub replacement: Option<&'static str>,

    pub intrinsic: Option<NativeIntrinsicId>,
    pub trust: NativeTrust,
}
