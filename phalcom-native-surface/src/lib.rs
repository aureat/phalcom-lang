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

/// One canonical native member declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeMember {
    /// Runtime class name owning the member.
    pub class: &'static str,
    /// Runtime dispatch selector, including encoded labels/rest shape.
    pub selector: &'static str,
    /// Native member category.
    pub kind: NativeMemberKind,
    /// Dispatch side.
    pub side: NativeDispatch,
    /// Visibility exposed by runtime dispatch.
    pub visibility: NativeVisibility,
    /// Stable VM-free semantic return contract.
    pub return_shape: NativeReturnShape,
}

/// One runtime-only class relationship needed when source core has no class
/// declaration for a bootstrapped representation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeClass {
    /// Runtime class name.
    pub name: &'static str,
    /// Runtime superclass name, if any.
    pub superclass: Option<&'static str>,
}

/// Bootstrapped classes that own at least one native member.
pub const NATIVE_CLASSES: &[NativeClass] = &[
    NativeClass {
        name: "Object",
        superclass: None,
    },
    NativeClass {
        name: "Behavior",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Class",
        superclass: Some("Behavior"),
    },
    NativeClass {
        name: "Message",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Number",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Int",
        superclass: Some("Number"),
    },
    NativeClass {
        name: "Float",
        superclass: Some("Number"),
    },
    NativeClass {
        name: "String",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Bool",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Symbol",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Selector",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "SelectorPattern",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Option",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Some",
        superclass: Some("Option"),
    },
    NativeClass {
        name: "Method",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "MethodFamily",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Function",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Closure",
        superclass: Some("Function"),
    },
    NativeClass {
        name: "Family",
        superclass: Some("Function"),
    },
    NativeClass {
        name: "System",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Module",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "List",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Bytes",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Map",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Set",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Tuple",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Record",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Range",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Error",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Fiber",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Resource",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Package",
        superclass: Some("Module"),
    },
    NativeClass {
        name: "Project",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "ProjectManifest",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "PackageInfo",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "PackageAuthor",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "PackageRequirement",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "ResolvedProjectDependency",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "ModuleDependency",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "ExportTable",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Export",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "ChildModuleTable",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Uri",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "ModuleIdentity",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "PackageIdentity",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Ordering",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "Ellipsis",
        superclass: Some("Object"),
    },
    NativeClass {
        name: "ProjectIdentity",
        superclass: Some("Object"),
    },
];

macro_rules! native {
    ($class:literal, $selector:literal, $kind:ident, $side:ident, $visibility:ident) => {
        NativeMember {
            class: $class,
            selector: $selector,
            kind: NativeMemberKind::$kind,
            side: NativeDispatch::$side,
            visibility: NativeVisibility::$visibility,
            return_shape: NativeReturnShape::Unknown,
        }
    };
}

macro_rules! native_with_return {
    ($class:literal, $selector:literal, $kind:ident, $side:ident, $visibility:ident, $return_shape:expr) => {
        NativeMember {
            class: $class,
            selector: $selector,
            kind: NativeMemberKind::$kind,
            side: NativeDispatch::$side,
            visibility: NativeVisibility::$visibility,
            return_shape: $return_shape,
        }
    };
}

/// Canonical native primitive surface.
pub const NATIVE_MEMBERS: &[NativeMember] = &[
    native!("Object", "name", Getter, Instance, Public),
    native!("Object", "class", Getter, Instance, Public),
    native!("Object", "class=(put)", Setter, Instance, Public),
    native!("Object", "toString", Getter, Instance, Public),
    native_with_return!("Object", "hash", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native_with_return!("Object", "==(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Object", "!=(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Object", "===(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Object", "matches(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Object", "understands(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native!("Object", "perform(_,***)", Method, Instance, Public),
    native!("Object", "respondsTo(_)", Method, Instance, Public),
    native!("Object", "doesNotUnderstand(_)", Method, Instance, Public),
    native!("Object", "methodFor(_)", Method, Instance, Public),
    native!("Object", "_$invariantEnter()", Method, Instance, Internal),
    native!("Object", "_$invariantExit()", Method, Instance, Internal),
    native!("Object", "_$attributes", Getter, Instance, Public),
    native!("Object", "_$attach(_)", Method, Instance, Public),
    native!("Object", "_$freezeAttributes()", Method, Instance, Public),
    native!("Message", "selector", Getter, Instance, Public),
    native!("Message", "name", Getter, Instance, Public),
    native!("Message", "labels", Getter, Instance, Public),
    native!("Message", "args", Getter, Instance, Public),
    native!("Behavior", "superclass", Getter, Instance, Public),
    native!("Behavior", "superclass=(put)", Setter, Instance, Public),
    native!("Behavior", "name", Getter, Instance, Public),
    native!("Behavior", "methods", Getter, Instance, Public),
    native!("Behavior", ">>(_)", Method, Instance, Public),
    native!("Class", "+(_)", Method, Instance, Public),
    native!("Class", "_$new()", Method, Instance, Internal),
    native!("Number", "+(_)", Method, Instance, Public),
    native!("Number", "-(_)", Method, Instance, Public),
    native!("Number", "*(_)", Method, Instance, Public),
    native!("Number", "/(_)", Method, Instance, Public),
    native!("Number", "%(_)", Method, Instance, Public),
    native!("Number", "~/(_)", Method, Instance, Public),
    native!("Number", "**(_)", Method, Instance, Public),
    native_with_return!("Number", "<(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Number", "<=(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Number", ">(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Number", ">=(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Number", "compare(_)", Method, Instance, Public, NativeReturnShape::Instance("Ordering")),
    native!("Number", "+", Getter, Instance, Public),
    native!("Number", "-", Getter, Instance, Public),
    native_with_return!("Number", "hash", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native_with_return!("Number", "toString", Getter, Instance, Public, NativeReturnShape::Instance("String")),
    native!("Number", "new()", Method, Class, Public),
    native!("Number", "new(_)", Method, Class, Public),
    native!("Int", "&(_)", Method, Instance, Public),
    native!("Int", "|(_)", Method, Instance, Public),
    native!("Int", "^(_)", Method, Instance, Public),
    native!("Int", "~", Getter, Instance, Public),
    native!("Int", "<<(_)", Method, Instance, Public),
    native!("Int", ">>(_)", Method, Instance, Public),
    native!("Int", "bitAt(_)", Method, Instance, Public),
    native_with_return!("Int", "bitCount", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native_with_return!("Int", "bitLength", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native_with_return!("Int", "trailingZeros", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native!("Int", "new()", Method, Class, Public),
    native!("Int", "new(_)", Method, Class, Public),
    native!("Float", "new()", Method, Class, Public),
    native!("Float", "new(_)", Method, Class, Public),
    native!("Float", "abs", Getter, Instance, Public),
    native!("Float", "sign", Getter, Instance, Public),
    native!("Float", "floor", Getter, Instance, Public),
    native!("Float", "ceil", Getter, Instance, Public),
    native!("Float", "truncated", Getter, Instance, Public),
    native!("Float", "rounded", Getter, Instance, Public),
    native!("Float", "toIntExact", Getter, Instance, Public),
    native_with_return!("Float", "isInteger", Getter, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Float", "isNaN", Getter, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Float", "isFinite", Getter, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Float", "isInfinite", Getter, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("String", "+(_)", Method, Instance, Public, NativeReturnShape::Instance("String")),
    native_with_return!("String", "hash", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native!("String", "new()", Method, Class, Public),
    native!("String", "new(_)", Method, Class, Public),
    native!("String", "_$byteCount", Getter, Instance, Internal),
    native!("String", "_$byteAt(_)", Method, Instance, Internal),
    native!("String", "_$slice(_,_)", Method, Instance, Internal),
    native!("Bool", "new()", Method, Class, Public),
    native!("Bool", "new(_)", Method, Class, Public),
    native!("Bool", "and(_)", Method, Instance, Public),
    native!("Bool", "or(_)", Method, Instance, Public),
    native_with_return!("Bool", "not", Getter, Instance, Public, NativeReturnShape::Instance("Bool")),
    native!("Bool", "ifTrue(_)", Method, Instance, Public),
    native!("Bool", "ifFalse(_)", Method, Instance, Public),
    native!("Bool", "ifTrue(_,ifFalse)", Method, Instance, Public),
    native_with_return!("Bool", "hash", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native_with_return!("Symbol", "toString", Getter, Instance, Public, NativeReturnShape::Instance("String")),
    native_with_return!("Symbol", "hash", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native_with_return!("Symbol", "isSelector", Getter, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Symbol", "isSelectorPattern", Getter, Instance, Public, NativeReturnShape::Instance("Bool")),
    native!("Symbol", "new(_)", Method, Class, Public),
    native!("Selector", "call(_)", Method, Class, Public),
    native!("Selector", "from(_)", Method, Class, Public),
    native!("Selector", "new(_)", Method, Class, Public),
    native!("Selector", "base", Getter, Instance, Public),
    native!("Selector", "kind", Getter, Instance, Public),
    native!("Selector", "slots", Getter, Instance, Public),
    native_with_return!("Selector", "toString", Getter, Instance, Public, NativeReturnShape::Instance("String")),
    native_with_return!("Selector", "==(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("Selector", "hash", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native!("SelectorPattern", "call(_)", Method, Class, Public),
    native!("SelectorPattern", "from(_)", Method, Class, Public),
    native!("SelectorPattern", "new(_)", Method, Class, Public),
    native!("SelectorPattern", "base", Getter, Instance, Public),
    native_with_return!("SelectorPattern", "matches(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("SelectorPattern", "toString", Getter, Instance, Public, NativeReturnShape::Instance("String")),
    native_with_return!("SelectorPattern", "==(_)", Method, Instance, Public, NativeReturnShape::Instance("Bool")),
    native_with_return!("SelectorPattern", "hash", Getter, Instance, Public, NativeReturnShape::Instance("Int")),
    native!("Some", "call(_)", Method, Class, Public),
    native!("Some", "new(_)", Method, Class, Public),
    native!("Option", "match(some,none)", Method, Instance, Public),
    native!("Method", "new(_)", Method, Class, Public),
    native!("Method", "arity", Getter, Instance, Public),
    native!("Method", "name", Getter, Instance, Public),
    native!("Method", "invokeOn(_,***)", Method, Instance, Public),
    native!("Method", "bind(_)", Method, Instance, Public),
    native!("Method", "selector", Getter, Instance, Public),
    native!("Method", "holder", Getter, Instance, Public),
    native!("Family", "receiver", Getter, Instance, Public),
    native!("Family", "selector", Getter, Instance, Public),
    native!("Family", "pattern", Getter, Instance, Public),
    native!("Family", "isExact", Getter, Instance, Public),
    native!("Family", "get()", Method, Instance, Public),
    native!("Family", "set(_)", Method, Instance, Public),
    native!("MethodFamily", "bind(_)", Method, Instance, Public),
    native!("MethodFamily", "selectors", Getter, Instance, Public),
    native!("MethodFamily", "size", Getter, Instance, Public),
    native!("MethodFamily", "methodFor(_)", Method, Instance, Public),
    native!("Function", "arity", Getter, Instance, Public),
    native!("Function", "name", Getter, Instance, Public),
    native!("Function", "callWith(_)", Method, Instance, Public),
    native!("Function", "call(***)", Method, Instance, Public),
    native!("Closure", "arity", Getter, Instance, Public),
    native!("Closure", "name", Getter, Instance, Public),
    native!("Closure", "whileTrue(_)", Method, Instance, Public),
    native!("Closure", "on(_,_)", Method, Instance, Public),
    native!("Closure", "ensure(_)", Method, Instance, Public),
    native!("System", "print(_)", Method, Class, Public),
    native!("System", "new()", Method, Class, Public),
    native!("System", "schedule(_)", Method, Class, Public),
    native!("System", "nextScheduled", Getter, Class, Public),
    native!("System", "gc", Getter, Class, Public),
    native!("System", "_$write(_)", Method, Class, Internal),
    native!("List", "new()", Method, Class, Public),
    native!("List", "_$length", Getter, Instance, Internal),
    native!("List", "_$at(_)", Method, Instance, Internal),
    native!("List", "_$set(_,_)", Method, Instance, Internal),
    native!("List", "_$push(_)", Method, Instance, Internal),
    native!("List", "_$replaceSlice(_,_,_)", Method, Instance, Internal),
    native!("List", "toString", Getter, Instance, Public),
    native!("Bytes", "new(_)", Method, Class, Public),
    native!("Bytes", "_$fromString(_)", Method, Class, Internal),
    native!("Bytes", "_$size", Getter, Instance, Internal),
    native!("Bytes", "_$at(_)", Method, Instance, Internal),
    native!("Bytes", "_$set(_,_)", Method, Instance, Internal),
    native!("Bytes", "_$fill(_)", Method, Instance, Internal),
    native!("Bytes", "_$slice(_,_)", Method, Instance, Internal),
    native!("Bytes", "_$copyInto(_,_)", Method, Instance, Internal),
    native!("Bytes", "_$utf8", Getter, Instance, Internal),
    native!("Bytes", "_$utf8Lossy", Getter, Instance, Internal),
    native!("Bytes", "_$equalsConstantTime(_)", Method, Instance, Internal),
    native!("Map", "new()", Method, Class, Public),
    native!("Map", "_$size", Getter, Instance, Internal),
    native!("Map", "_$get(_)", Method, Instance, Internal),
    native!("Map", "_$put(_,_)", Method, Instance, Internal),
    native!("Map", "_$has(_)", Method, Instance, Internal),
    native!("Map", "_$remove(_)", Method, Instance, Internal),
    native!("Map", "_$keyAt(_)", Method, Instance, Internal),
    native!("Map", "_$valueAt(_)", Method, Instance, Internal),
    native!("Set", "new()", Method, Class, Public),
    native!("Set", "_$size", Getter, Instance, Internal),
    native!("Set", "_$add(_)", Method, Instance, Internal),
    native!("Set", "_$has(_)", Method, Instance, Internal),
    native!("Set", "_$remove(_)", Method, Instance, Internal),
    native!("Set", "_$at(_)", Method, Instance, Internal),
    native!("Tuple", "_$fromList(_)", Method, Class, Internal),
    native!("Tuple", "_$size", Getter, Instance, Internal),
    native!("Tuple", "_$at(_)", Method, Instance, Internal),
    native!("Tuple", "_$positionalSize", Getter, Instance, Internal),
    native!("Tuple", "_$labelAt(_)", Method, Instance, Internal),
    native!("Tuple", "_$positionals", Getter, Instance, Internal),
    native!("Tuple", "_$labeled", Getter, Instance, Internal),
    native!("Tuple", "_$slice(_,_)", Method, Instance, Internal),
    native!("Record", "_$size", Getter, Instance, Internal),
    native!("Record", "_$labelAt(_)", Method, Instance, Internal),
    native!("Record", "_$valueAt(_)", Method, Instance, Internal),
    native!("Range", "_$lower", Getter, Instance, Internal),
    native!("Range", "_$upper", Getter, Instance, Internal),
    native!("Range", "_$upperInclusive", Getter, Instance, Internal),
    native!("Error", "message", Getter, Instance, Public),
    native!("Error", "raise()", Method, Instance, Public),
    native!("Fiber", "new(_)", Method, Class, Public),
    native!("Fiber", "call()", Method, Instance, Public),
    native!("Fiber", "call(_)", Method, Instance, Public),
    native!("Fiber", "try()", Method, Instance, Public),
    native!("Fiber", "try(_)", Method, Instance, Public),
    native!("Fiber", "yield()", Method, Class, Public),
    native!("Fiber", "yield(_)", Method, Class, Public),
    native!("Fiber", "current", Getter, Class, Public),
    native!("Fiber", "abort(_)", Method, Class, Public),
    native!("Fiber", "isDone", Getter, Instance, Public),
    native!("Fiber", "isRoot", Getter, Instance, Public),
    native!("Fiber", "error", Getter, Instance, Public),
    native!("Resource", "_$register(_)", Method, Class, Internal),
    native!("Resource", "_$close()", Method, Instance, Internal),
    native!("Resource", "_$isClosed", Getter, Instance, Internal),
    native!("System", "_$leakReport", Getter, Class, Internal),
    native!("System", "_$strictResources(_)", Method, Class, Internal),
    // Module
    native!("Module", "new()", Method, Class, Public),
    native!("Module", "doesNotUnderstand(_)", Method, Instance, Public),
    native!("Module", "name", Getter, Instance, Public),
    native!("Module", "namespace", Getter, Instance, Public),
    native!("Module", "package", Getter, Instance, Public),
    native!("Module", "rootPackage", Getter, Instance, Public),
    native!("Module", "packageInfo", Getter, Instance, Public),
    native!("Module", "exports", Getter, Instance, Public),
    native!("Module", "metadata", Getter, Instance, Public),
    native!("Module", "dependencies", Getter, Instance, Public),
    native!("Module", "uri", Getter, Instance, Public),
    native!("Module", "identity", Getter, Instance, Public),
    native!("Module", "__exports__", Getter, Instance, Public),
    native!("Module", "__export__(_)", Method, Instance, Public),
    native!("Module", "__understands__(_)", Method, Instance, Public),
    native!("Module", "__metadata__", Getter, Instance, Public),
    native!("Module", "__dependencies__", Getter, Instance, Public),
    native!("Module", "__uri__", Getter, Instance, Public),
    native!("Module", "__name__", Getter, Instance, Public),
    native!("Module", "__id__", Getter, Instance, Public),
    native!("Module", "__path__", Getter, Instance, Public),
    native!("Module", "toString", Getter, Instance, Public),
    // Package
    native!("Package", "package", Getter, Instance, Public),
    native!("Package", "parentPackage", Getter, Instance, Public),
    native!("Package", "rootPackage", Getter, Instance, Public),
    native!("Package", "packageInfo", Getter, Instance, Public),
    native!("Package", "children", Getter, Instance, Public),
    native!("Package", "isRoot", Getter, Instance, Public),
    native!("Package", "__parent__", Getter, Instance, Public),
    native!("Package", "__children__", Getter, Instance, Public),
    native!("Package", "__version__", Getter, Instance, Public),
    native!("Package", "__namespace__", Getter, Instance, Public),
    native!("Package", "toString", Getter, Instance, Public),
    // Project
    native!("Project", "name", Getter, Instance, Public),
    native!("Project", "namespace", Getter, Instance, Public),
    native!("Project", "manifest", Getter, Instance, Public),
    native!("Project", "rootPackage", Getter, Instance, Public),
    native!("Project", "dependencies", Getter, Instance, Public),
    native!("Project", "developmentEntry", Getter, Instance, Public),
    native!("Project", "identity", Getter, Instance, Public),
    native!("Project", "toString", Getter, Instance, Public),
    // ProjectManifest
    native!("ProjectManifest", "name", Getter, Instance, Public),
    native!("ProjectManifest", "namespace", Getter, Instance, Public),
    native!("ProjectManifest", "version", Getter, Instance, Public),
    native!("ProjectManifest", "authors", Getter, Instance, Public),
    native!("ProjectManifest", "description", Getter, Instance, Public),
    native!("ProjectManifest", "license", Getter, Instance, Public),
    native!("ProjectManifest", "homepage", Getter, Instance, Public),
    native!("ProjectManifest", "repository", Getter, Instance, Public),
    native!("ProjectManifest", "source", Getter, Instance, Public),
    native!("ProjectManifest", "entry", Getter, Instance, Public),
    native!("ProjectManifest", "defaultEntry", Getter, Instance, Public),
    native!("ProjectManifest", "dependencyDeclarations", Getter, Instance, Public),
    native!("ProjectManifest", "dependencies", Getter, Instance, Public),
    native!("ProjectManifest", "toString", Getter, Instance, Public),
    // PackageInfo
    native!("PackageInfo", "name", Getter, Instance, Public),
    native!("PackageInfo", "namespace", Getter, Instance, Public),
    native!("PackageInfo", "version", Getter, Instance, Public),
    native!("PackageInfo", "authors", Getter, Instance, Public),
    native!("PackageInfo", "description", Getter, Instance, Public),
    native!("PackageInfo", "license", Getter, Instance, Public),
    native!("PackageInfo", "homepage", Getter, Instance, Public),
    native!("PackageInfo", "repository", Getter, Instance, Public),
    native!("PackageInfo", "requirements", Getter, Instance, Public),
    native!("PackageInfo", "defaultEntry", Getter, Instance, Public),
    native!("PackageInfo", "identity", Getter, Instance, Public),
    native!("PackageInfo", "toString", Getter, Instance, Public),
    // PackageAuthor
    native!("PackageAuthor", "name", Getter, Instance, Public),
    native!("PackageAuthor", "email", Getter, Instance, Public),
    native!("PackageAuthor", "url", Getter, Instance, Public),
    // PackageRequirement
    native!("PackageRequirement", "alias", Getter, Instance, Public),
    native!("PackageRequirement", "package", Getter, Instance, Public),
    native!("PackageRequirement", "versionRequirement", Getter, Instance, Public),
    native!("PackageRequirement", "optional", Getter, Instance, Public),
    // ResolvedProjectDependency
    native!("ResolvedProjectDependency", "alias", Getter, Instance, Public),
    native!("ResolvedProjectDependency", "requirement", Getter, Instance, Public),
    native!("ResolvedProjectDependency", "packageInfo", Getter, Instance, Public),
    native!("ResolvedProjectDependency", "rootPackage", Getter, Instance, Public),
    native!("ResolvedProjectDependency", "origin", Getter, Instance, Public),
    // ModuleDependency
    native!("ModuleDependency", "module", Getter, Instance, Public),
    native!("ModuleDependency", "phase", Getter, Instance, Public),
    native!("ModuleDependency", "reason", Getter, Instance, Public),
    // ExportTable
    native!("ExportTable", "names", Getter, Instance, Public),
    native!("ExportTable", "keys", Getter, Instance, Public),
    native!("ExportTable", "size", Getter, Instance, Public),
    native!("ExportTable", "contains(_)", Method, Instance, Public),
    native!("ExportTable", "descriptor(_)", Method, Instance, Public),
    native!("ExportTable", "get(_)", Method, Instance, Public),
    // Export
    native!("Export", "name", Getter, Instance, Public),
    native!("Export", "kind", Getter, Instance, Public),
    native!("Export", "module", Getter, Instance, Public),
    native!("Export", "value", Getter, Instance, Public),
    native!("Export", "isModule", Getter, Instance, Public),
    native!("Export", "isBinding", Getter, Instance, Public),
    // ChildModuleTable
    native!("ChildModuleTable", "names", Getter, Instance, Public),
    native!("ChildModuleTable", "size", Getter, Instance, Public),
    native!("ChildModuleTable", "contains(_)", Method, Instance, Public),
    native!("ChildModuleTable", "get(_)", Method, Instance, Public),
    // Uri
    native!("Uri", "toString", Getter, Instance, Public),
    native!("Uri", "==(_)", Method, Instance, Public),
    // ModuleIdentity
    native!("ModuleIdentity", "uri", Getter, Instance, Public),
    native!("ModuleIdentity", "toString", Getter, Instance, Public),
    // PackageIdentity
    native!("PackageIdentity", "toString", Getter, Instance, Public),
    // ProjectIdentity
    native!("ProjectIdentity", "toString", Getter, Instance, Public),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn native_rows_have_unique_runtime_keys_and_canonical_contracts() {
        let classes = NATIVE_CLASSES.iter().map(|class| class.name).collect::<BTreeSet<_>>();
        let mut keys = BTreeSet::new();

        for member in NATIVE_MEMBERS {
            assert!(classes.contains(member.class), "native member references unknown class {}", member.class);
            assert!(
                keys.insert((member.class, member.selector, member.side, member.kind, member.visibility)),
                "duplicate native member row: {member:?}"
            );
            match member.return_shape {
                NativeReturnShape::Instance(name) | NativeReturnShape::ClassObject(name) => {
                    assert!(classes.contains(name), "native return contract references unknown class {name}");
                }
                NativeReturnShape::Unknown | NativeReturnShape::Receiver | NativeReturnShape::Argument(_) => {}
            }
        }
    }

    #[test]
    fn rich_catalog_is_unique_and_intrinsic_safe() {
        validate_native_surface_catalog(NATIVE_SURFACES).unwrap();
        assert_eq!(catalog_fingerprint_for(NATIVE_SURFACES), catalog_fingerprint());
        let mut reordered = NATIVE_SURFACES.to_vec();
        reordered.reverse();
        assert_eq!(catalog_fingerprint_for(&reordered), catalog_fingerprint());
    }
}
