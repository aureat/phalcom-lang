//! Canonical native member declarations shared by the runtime and tooling.
//!
//! This crate deliberately contains no VM, AST, or LSP dependency. The runtime
//! validates its primitive registration against this surface, while the LSP
//! uses it to expose native members without linking the runtime.

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

/// Native dispatch side.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeDispatch {
    /// Instance-side dispatch.
    Instance,
    /// Class-side dispatch.
    Class,
}

/// Native visibility.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NativeVisibility {
    /// Public protocol member.
    Public,
    /// Runtime implementation member.
    Internal,
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
    native!("Symbol", "new(_)", Method, Class, Public),
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
}
