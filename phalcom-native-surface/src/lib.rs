//! Canonical native member declarations shared by the runtime and tooling.
//!
//! This crate deliberately contains no VM, AST, or LSP dependency. The runtime
//! validates its primitive registration against this surface, while the LSP
//! uses it to expose native members without linking the runtime.

pub use phalcom_native_meta::*;

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Native member category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeMemberKind {
    /// Ordinary method.
    Method,
    /// Bare-name getter.
    Getter,
    /// Setter member.
    Setter,
}

/// Rich native surface record preserving complete declarative metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSurfaceRecord {
    pub surface: PrimitiveSurfaceSpec,
    pub kind: NativeMemberKind,
    pub abi: PrimitiveAbi,
    pub return_shape: NativeReturnShape,
}

/// Stable identity of a native surface row.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeSurfaceId(pub PrimitiveKey);

impl NativeSurfaceRecord {
    pub const fn owner(&self) -> UniverseKey {
        self.surface.key.owner
    }

    pub const fn id(&self) -> NativeSurfaceId {
        NativeSurfaceId(self.surface.key)
    }

    pub const fn side(&self) -> NativeDispatch {
        self.surface.key.side
    }

    pub const fn selector(&self) -> &'static str {
        self.surface.key.selector
    }

    pub const fn visibility(&self) -> NativeVisibility {
        self.surface.visibility
    }

    pub const fn stability(&self) -> NativeStability {
        self.surface.stability
    }

    pub const fn anchor(&self) -> NativeAnchorPolicy {
        self.surface.anchor
    }

    pub const fn params(&self) -> &'static ParameterTupleSpec {
        self.surface.params
    }

    pub const fn returns(&self) -> &'static TypeExprSpec {
        self.surface.returns
    }

    pub const fn callable(&self) -> &'static CallableTypeSpec {
        self.surface.callable
    }

    pub const fn raises(&self) -> RaisesSpec {
        self.surface.raises
    }

    pub const fn effects(&self) -> EffectSpec {
        self.surface.effects
    }

    pub const fn flow(&self) -> ReturnFlowSpec {
        self.surface.flow
    }

    pub const fn intrinsic(&self) -> Option<NativeIntrinsicId> {
        self.surface.intrinsic
    }

    pub const fn trust(&self) -> NativeTrust {
        self.surface.trust
    }

    pub const fn docs(&self) -> Option<&'static str> {
        self.surface.docs
    }

    pub const fn conceptual(&self) -> Option<&'static str> {
        self.surface.conceptual
    }

    pub const fn lifecycle(&self) -> NativeLifecycleSpec {
        self.surface.lifecycle
    }
}

pub mod generated;
pub use generated::NATIVE_SURFACES;

/// Stable 128-bit structural catalog fingerprint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeCatalogFingerprint(pub [u8; 16]);

/// Indexed VM-free native surface catalog.
pub struct NativeSurfaceCatalog {
    records: &'static [NativeSurfaceRecord],
    index: OnceLock<BTreeMap<PrimitiveKey, usize>>,
    selector_index: OnceLock<BTreeMap<(UniverseKey, NativeDispatch, String), usize>>,
    selector_only_index: OnceLock<BTreeMap<String, usize>>,
}

impl NativeSurfaceCatalog {
    pub const fn new(records: &'static [NativeSurfaceRecord]) -> Self {
        Self {
            records,
            index: OnceLock::new(),
            selector_index: OnceLock::new(),
            selector_only_index: OnceLock::new(),
        }
    }

    pub fn records(&self) -> &'static [NativeSurfaceRecord] {
        self.records
    }

    pub fn iter(&self) -> impl Iterator<Item = &'static NativeSurfaceRecord> {
        self.records.iter()
    }

    pub fn find(&self, key: PrimitiveKey) -> Option<&'static NativeSurfaceRecord> {
        let index = self
            .index
            .get_or_init(|| self.records.iter().enumerate().map(|(index, record)| (record.surface.key, index)).collect());
        index.get(&key).and_then(|index| self.records.get(*index))
    }

    pub fn find_selector(&self, owner: UniverseKey, side: NativeDispatch, selector: &str) -> Option<&'static NativeSurfaceRecord> {
        let index = self.selector_index.get_or_init(|| {
            self.records
                .iter()
                .enumerate()
                .map(|(index, record)| ((record.owner(), record.side(), record.selector().to_owned()), index))
                .collect()
        });
        index.get(&(owner, side, selector.to_owned())).and_then(|index| self.records.get(*index))
    }

    /// Finds the first canonical row for a selector when owner information is
    /// unavailable, using stable key order to keep ambiguous legacy lookups
    /// deterministic.
    pub fn find_selector_any(&self, selector: &str) -> Option<&'static NativeSurfaceRecord> {
        let index = self.selector_only_index.get_or_init(|| {
            let mut ordered = self.records.iter().enumerate().collect::<Vec<_>>();
            ordered.sort_by_key(|(_, record)| record.surface.key.sort_key());
            ordered.into_iter().fold(BTreeMap::new(), |mut index, (position, record)| {
                index.entry(record.selector().to_owned()).or_insert(position);
                index
            })
        });
        index.get(selector).and_then(|index| self.records.get(*index))
    }

    pub fn fingerprint(&self) -> NativeCatalogFingerprint {
        catalog_fingerprint_for(self.records)
    }
}

pub static NATIVE_SURFACE_CATALOG: NativeSurfaceCatalog = NativeSurfaceCatalog::new(NATIVE_SURFACES);

/// Finds a canonical native surface record by owner, side, and selector.
pub fn find_native_surface(owner: UniverseKey, side: NativeDispatch, selector: &str) -> Option<&'static NativeSurfaceRecord> {
    NATIVE_SURFACE_CATALOG.find_selector(owner, side, selector)
}

pub fn find_native_surface_by_selector(selector: &str) -> Option<&'static NativeSurfaceRecord> {
    NATIVE_SURFACE_CATALOG.find_selector_any(selector)
}

/// Returns an iterator over all canonical native surface records for a given owner.
pub fn native_surfaces_for_owner(owner: UniverseKey) -> impl Iterator<Item = &'static NativeSurfaceRecord> {
    NATIVE_SURFACE_CATALOG.iter().filter(move |s| s.owner() == owner)
}

/// Computes a deterministic structural fingerprint. Debug source locations
/// are not part of `PrimitiveSurfaceSpec` and therefore cannot affect this
/// value.
pub fn catalog_fingerprint() -> NativeCatalogFingerprint {
    NATIVE_SURFACE_CATALOG.fingerprint()
}

pub fn catalog_fingerprint_for(records: &[NativeSurfaceRecord]) -> NativeCatalogFingerprint {
    let mut ordered: Vec<&NativeSurfaceRecord> = records.iter().collect();
    ordered.sort_by_key(|record| record.surface.key.sort_key());

    let mut left = 0xcbf29ce484222325_u64;
    let mut right = 0x84222325cbf29ce4_u64;
    for record in ordered {
        let structural = format!(
            "{:?}|{:?}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
            record.surface.key.owner,
            record.surface.key.side,
            record.surface.key.selector,
            record.kind,
            record.surface.visibility,
            record.surface.stability,
            record.surface.callable,
            record.surface.params,
            record.surface.returns,
            record.surface.raises,
            record.surface.effects,
            record.surface.flow,
            record.surface.lifecycle,
            record.surface.intrinsic,
            record.surface.trust,
            record.surface.docs,
            record.surface.conceptual,
            record.abi,
            record.return_shape,
        );
        for byte in structural.as_bytes() {
            left ^= u64::from(*byte);
            left = left.wrapping_mul(0x100000001b3);
            right ^= u64::from(*byte).rotate_left(1);
            right = right.wrapping_mul(0x9e3779b185ebca87);
        }
    }
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&left.to_le_bytes());
    bytes[8..].copy_from_slice(&right.to_le_bytes());
    NativeCatalogFingerprint(bytes)
}

/// Validates intrinsic declarations against their legal fallback selectors.
pub fn validate_intrinsic_expectations(records: &[NativeSurfaceRecord]) -> Result<(), Vec<String>> {
    let mut failures = Vec::new();
    let expected = [
        (NativeIntrinsicId::BoolAnd, UniverseKey::Bool, NativeDispatch::Instance, "and(_)", 1_usize),
        (NativeIntrinsicId::BoolOr, UniverseKey::Bool, NativeDispatch::Instance, "or(_)", 1_usize),
        (NativeIntrinsicId::BoolNot, UniverseKey::Bool, NativeDispatch::Instance, "not", 0_usize),
    ];
    for record in records {
        let Some(intrinsic) = record.intrinsic() else { continue };
        let Some(expectation) = expected.iter().find(|entry| entry.0 == intrinsic) else {
            failures.push(format!("unsupported intrinsic {intrinsic:?} on {}", record.selector()));
            continue;
        };
        let (_, owner, side, selector, arity) = *expectation;
        if record.owner() != owner || record.side() != side || record.selector() != selector {
            failures.push(format!("intrinsic {intrinsic:?} has illegal key {:?}", record.surface.key));
        }
        let actual_arity = record.params().positional.len() + record.params().labeled.len();
        if actual_arity != arity {
            failures.push(format!("intrinsic {intrinsic:?} has arity {actual_arity}, expected {arity}"));
        }
    }
    if failures.is_empty() { Ok(()) } else { Err(failures) }
}

/// Validates duplicate keys and metadata invariants for a catalog.
pub fn validate_native_surface_catalog(records: &[NativeSurfaceRecord]) -> Result<(), Vec<String>> {
    let mut failures = Vec::new();
    let mut keys = std::collections::BTreeSet::new();
    for record in records {
        if !keys.insert(record.surface.key) {
            failures.push(format!("duplicate native surface key {:?}", record.surface.key));
        }
        if !record.lifecycle().is_consistent() {
            failures.push(format!("inconsistent lifecycle metadata for {:?}", record.surface.key));
        }
    }
    if let Err(intrinsic_failures) = validate_intrinsic_expectations(records) {
        failures.extend(intrinsic_failures);
    }
    if failures.is_empty() { Ok(()) } else { Err(failures) }
}

/// VM-free semantic return contract for a native member.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeReturnShape {
    /// No stable source-level contract; semantic consumers remain conservative.
    Unknown,
    /// An instance of a canonical core class.
    Instance(&'static str),
    /// The receiver's runtime shape, preserving instance/class side.
    Receiver,
    /// A canonical core class object.
    ClassObject(&'static str),
    /// One argument, when a native primitive returns it unchanged.
    Argument(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rich_catalog_is_unique_and_intrinsic_safe() {
        validate_native_surface_catalog(NATIVE_SURFACES).unwrap();
        assert_eq!(catalog_fingerprint_for(NATIVE_SURFACES), catalog_fingerprint());
        let mut reordered = NATIVE_SURFACES.to_vec();
        reordered.reverse();
        assert_eq!(catalog_fingerprint_for(&reordered), catalog_fingerprint());
    }
}
