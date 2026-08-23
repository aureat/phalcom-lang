use phalcom_native_meta::{NativeAnchorPolicy, NativeDispatch, NativeStability, NativeTrust, NativeVisibility, PrimitiveAbi, UniverseKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveDeclKey {
    pub owner: UniverseKey,
    pub selector: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveDeclField {
    pub name: String,
    pub value: String,
}

/// Owned, VM-free normalized form shared by proc-macro and source tooling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedPrimitiveDecl {
    pub key: PrimitiveDeclKey,
    pub fields: Vec<PrimitiveDeclField>,
    pub params: Option<String>,
    pub returns: Option<String>,
    pub types: Option<String>,
    pub raises: Option<String>,
    pub effects: Option<String>,
    pub side: NativeDispatch,
    pub visibility: Option<NativeVisibility>,
    pub stability: NativeStability,
    pub anchor: NativeAnchorPolicy,
    pub since: Option<String>,
    pub deprecated_since: Option<String>,
    pub replacement: Option<String>,
    pub abi: PrimitiveAbi,
    pub flow: Option<String>,
    pub intrinsic: Option<String>,
    pub trust: NativeTrust,
    pub conceptual: Option<String>,
    pub docs: Vec<String>,
}
