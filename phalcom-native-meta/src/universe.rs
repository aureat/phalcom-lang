//! Universe key definitions and binding catalog.

/// Schema version of the generated native surface manifest.
pub const NATIVE_SURFACE_SCHEMA_VERSION: u32 = 1;

/// One VM-free bootstrap class relation owned by the native universe.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UniverseClassRelationSpec {
    pub class: UniverseKey,
    pub superclass: Option<UniverseKey>,
}

/// Stable VM-free enumeration of every canonical built-in class.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum UniverseKey {
    Object,
    Behavior,
    Class,
    Metaclass,

    Number,
    Int,
    Float,
    String,
    Nil,
    Bool,
    True,
    False,
    Symbol,
    Selector,
    SelectorPattern,

    Function,
    Closure,
    BoundMethod,
    Method,
    MethodFamily,
    BoundMethodFamily,
    Family,

    Option,
    Result,
    Ordering,
    Some,
    None,
    Unit,

    Iterable,
    List,
    Map,
    Set,
    Tuple,
    Record,
    Range,
    Bytes,

    Module,
    Package,
    Project,
    System,
    Message,
    Attribute,

    Error,
    MessageNotUnderstood,
    CannotYieldAcrossNativeFrame,
    UseAfterCloseError,

    Fiber,
    Resource,

    ProjectManifest,
    PackageInfo,
    PackageAuthor,
    PackageRequirement,
    ResolvedProjectDependency,
    ModuleDependency,
    ExportTable,
    Export,
    ExportKind,
    ChildModuleTable,
    ModuleIdentity,
    PackageIdentity,
    ProjectIdentity,
    Uri,
}

/// Canonical relations for classes whose existence is owned by the runtime.
/// Source-only helper classes are intentionally absent.
pub const UNIVERSE_CLASS_RELATIONS: &[UniverseClassRelationSpec] = &[
    UniverseClassRelationSpec {
        class: UniverseKey::Object,
        superclass: None,
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Behavior,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Class,
        superclass: Some(UniverseKey::Behavior),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Metaclass,
        superclass: Some(UniverseKey::Behavior),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Number,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Int,
        superclass: Some(UniverseKey::Number),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Float,
        superclass: Some(UniverseKey::Number),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::String,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Nil,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Bool,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::True,
        superclass: Some(UniverseKey::Bool),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::False,
        superclass: Some(UniverseKey::Bool),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Symbol,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Selector,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::SelectorPattern,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Option,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Result,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Ordering,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Some,
        superclass: Some(UniverseKey::Option),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::None,
        superclass: Some(UniverseKey::Option),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Unit,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Function,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Closure,
        superclass: Some(UniverseKey::Function),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::BoundMethod,
        superclass: Some(UniverseKey::Function),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Method,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::MethodFamily,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::BoundMethodFamily,
        superclass: Some(UniverseKey::Function),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Family,
        superclass: Some(UniverseKey::Function),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Iterable,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::List,
        superclass: Some(UniverseKey::Iterable),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Map,
        superclass: Some(UniverseKey::Iterable),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Set,
        superclass: Some(UniverseKey::Iterable),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Tuple,
        superclass: Some(UniverseKey::Iterable),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Record,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Range,
        superclass: Some(UniverseKey::Iterable),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Bytes,
        superclass: Some(UniverseKey::Iterable),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Module,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Package,
        superclass: Some(UniverseKey::Module),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Project,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::System,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Message,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Attribute,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Error,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::MessageNotUnderstood,
        superclass: Some(UniverseKey::Error),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::CannotYieldAcrossNativeFrame,
        superclass: Some(UniverseKey::Error),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::UseAfterCloseError,
        superclass: Some(UniverseKey::Error),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Fiber,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Resource,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ProjectManifest,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::PackageInfo,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::PackageAuthor,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::PackageRequirement,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ResolvedProjectDependency,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ModuleDependency,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ExportTable,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Export,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ExportKind,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ChildModuleTable,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ModuleIdentity,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::PackageIdentity,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::ProjectIdentity,
        superclass: Some(UniverseKey::Object),
    },
    UniverseClassRelationSpec {
        class: UniverseKey::Uri,
        superclass: Some(UniverseKey::Object),
    },
];

impl UniverseKey {
    /// Canonical source module path that owns this declaration or runtime
    /// support class. The path is semantic identity, not a leaf-name lookup.
    pub const fn source_path(&self) -> &'static [&'static str] {
        match self {
            Self::Object => &["object", "object"],
            Self::Behavior => &["object", "behavior"],
            Self::Class => &["object", "class"],
            Self::Metaclass => &["object", "metaclass"],
            Self::Number | Self::Int | Self::Float => &["scalar", "number"],
            Self::String => &["scalar", "string"],
            Self::Nil => &["scalar", "nil"],
            Self::Bool | Self::True | Self::False => &["scalar", "bool"],
            Self::Symbol => &["scalar", "symbol"],
            Self::Selector | Self::SelectorPattern => &["reflection", "selector"],
            Self::Function => &["callable", "function"],
            Self::Closure => &["callable", "closure"],
            Self::BoundMethod | Self::Method => &["callable", "method"],
            Self::MethodFamily | Self::BoundMethodFamily | Self::Family => &["callable", "family"],
            Self::Option | Self::Some | Self::None => &["option", "option"],
            Self::Unit => &["option", "unit"],
            Self::Result => &["errors", "result"],
            Self::Ordering => &["object", "ordering"],
            Self::Iterable => &["collections", "iterable"],
            Self::List => &["collections", "list"],
            Self::Map => &["collections", "map"],
            Self::Set => &["collections", "set"],
            Self::Tuple => &["collections", "tuple"],
            Self::Record => &["collections", "record"],
            Self::Range => &["collections", "range"],
            Self::Bytes | Self::Resource => &["collections", "bytes"],
            Self::Module => &["reflection", "module"],
            Self::Package => &["reflection", "package_object"],
            Self::Project => &["reflection", "project"],
            Self::System | Self::Fiber => &["concurrency", "fiber"],
            Self::Message => &["reflection", "message"],
            Self::Attribute => &["reflection", "attribute"],
            Self::Error | Self::MessageNotUnderstood | Self::CannotYieldAcrossNativeFrame | Self::UseAfterCloseError => {
                &["errors", "error"]
            }
            Self::ProjectManifest => &["reflection", "project_manifest"],
            Self::PackageInfo => &["reflection", "package_info"],
            Self::PackageAuthor => &["reflection", "package_author"],
            Self::PackageRequirement => &["reflection", "package_requirement"],
            Self::ResolvedProjectDependency => &["reflection", "resolved_project_dependency"],
            Self::ModuleDependency => &["reflection", "module_dependency"],
            Self::ExportTable => &["reflection", "export_table"],
            Self::Export => &["reflection", "export"],
            Self::ExportKind => &["reflection", "export_kind"],
            Self::ChildModuleTable => &["reflection", "child_module_table"],
            Self::ModuleIdentity => &["reflection", "module_identity"],
            Self::PackageIdentity => &["reflection", "package_identity"],
            Self::ProjectIdentity => &["reflection", "project_identity"],
            Self::Uri => &["reflection", "uri"],
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Object => "Object",
            Self::Behavior => "Behavior",
            Self::Class => "Class",
            Self::Metaclass => "Metaclass",
            Self::Number => "Number",
            Self::Int => "Int",
            Self::Float => "Float",
            Self::String => "String",
            Self::Nil => "Nil",
            Self::Bool => "Bool",
            Self::True => "True",
            Self::False => "False",
            Self::Symbol => "Symbol",
            Self::Selector => "Selector",
            Self::SelectorPattern => "SelectorPattern",
            Self::Function => "Function",
            Self::Closure => "Closure",
            Self::BoundMethod => "BoundMethod",
            Self::Method => "Method",
            Self::MethodFamily => "MethodFamily",
            Self::BoundMethodFamily => "BoundMethodFamily",
            Self::Family => "Family",
            Self::Option => "Option",
            Self::Result => "Result",
            Self::Ordering => "Ordering",
            Self::Some => "Some",
            Self::None => "None",
            Self::Unit => "Unit",
            Self::Iterable => "Iterable",
            Self::List => "List",
            Self::Map => "Map",
            Self::Set => "Set",
            Self::Tuple => "Tuple",
            Self::Record => "Record",
            Self::Range => "Range",
            Self::Bytes => "Bytes",
            Self::Module => "Module",
            Self::Package => "Package",
            Self::Project => "Project",
            Self::System => "System",
            Self::Message => "Message",
            Self::Attribute => "Attribute",
            Self::Error => "Error",
            Self::MessageNotUnderstood => "MessageNotUnderstood",
            Self::CannotYieldAcrossNativeFrame => "CannotYieldAcrossNativeFrame",
            Self::UseAfterCloseError => "UseAfterCloseError",
            Self::Fiber => "Fiber",
            Self::Resource => "Resource",
            Self::ProjectManifest => "ProjectManifest",
            Self::PackageInfo => "PackageInfo",
            Self::PackageAuthor => "PackageAuthor",
            Self::PackageRequirement => "PackageRequirement",
            Self::ResolvedProjectDependency => "ResolvedProjectDependency",
            Self::ModuleDependency => "ModuleDependency",
            Self::ExportTable => "ExportTable",
            Self::Export => "Export",
            Self::ExportKind => "ExportKind",
            Self::ChildModuleTable => "ChildModuleTable",
            Self::ModuleIdentity => "ModuleIdentity",
            Self::PackageIdentity => "PackageIdentity",
            Self::ProjectIdentity => "ProjectIdentity",
            Self::Uri => "Uri",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Object" => Some(Self::Object),
            "Behavior" => Some(Self::Behavior),
            "Class" => Some(Self::Class),
            "Metaclass" => Some(Self::Metaclass),
            "Number" => Some(Self::Number),
            "Int" => Some(Self::Int),
            "Float" => Some(Self::Float),
            "String" => Some(Self::String),
            "Nil" => Some(Self::Nil),
            "Bool" => Some(Self::Bool),
            "True" => Some(Self::True),
            "False" => Some(Self::False),
            "Symbol" => Some(Self::Symbol),
            "Selector" => Some(Self::Selector),
            "SelectorPattern" => Some(Self::SelectorPattern),
            "Function" => Some(Self::Function),
            "Closure" => Some(Self::Closure),
            "BoundMethod" => Some(Self::BoundMethod),
            "Method" => Some(Self::Method),
            "MethodFamily" => Some(Self::MethodFamily),
            "BoundMethodFamily" => Some(Self::BoundMethodFamily),
            "Family" => Some(Self::Family),
            "Option" => Some(Self::Option),
            "Result" => Some(Self::Result),
            "Ordering" => Some(Self::Ordering),
            "Some" => Some(Self::Some),
            "None" => Some(Self::None),
            "Unit" => Some(Self::Unit),
            "Iterable" => Some(Self::Iterable),
            "List" => Some(Self::List),
            "Map" => Some(Self::Map),
            "Set" => Some(Self::Set),
            "Tuple" => Some(Self::Tuple),
            "Record" => Some(Self::Record),
            "Range" => Some(Self::Range),
            "Bytes" => Some(Self::Bytes),
            "Module" => Some(Self::Module),
            "Package" => Some(Self::Package),
            "Project" => Some(Self::Project),
            "System" => Some(Self::System),
            "Message" => Some(Self::Message),
            "Attribute" => Some(Self::Attribute),
            "Error" => Some(Self::Error),
            "MessageNotUnderstood" => Some(Self::MessageNotUnderstood),
            "CannotYieldAcrossNativeFrame" => Some(Self::CannotYieldAcrossNativeFrame),
            "UseAfterCloseError" => Some(Self::UseAfterCloseError),
            "Fiber" => Some(Self::Fiber),
            "Resource" => Some(Self::Resource),
            "ProjectManifest" => Some(Self::ProjectManifest),
            "PackageInfo" => Some(Self::PackageInfo),
            "PackageAuthor" => Some(Self::PackageAuthor),
            "PackageRequirement" => Some(Self::PackageRequirement),
            "ResolvedProjectDependency" => Some(Self::ResolvedProjectDependency),
            "ModuleDependency" => Some(Self::ModuleDependency),
            "ExportTable" => Some(Self::ExportTable),
            "Export" => Some(Self::Export),
            "ExportKind" => Some(Self::ExportKind),
            "ChildModuleTable" => Some(Self::ChildModuleTable),
            "ModuleIdentity" => Some(Self::ModuleIdentity),
            "PackageIdentity" => Some(Self::PackageIdentity),
            "ProjectIdentity" => Some(Self::ProjectIdentity),
            "Uri" => Some(Self::Uri),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum UniverseBindingKind {
    /// A runtime class that is also a canonical language declaration.
    Class,

    /// A runtime class required by the VM/object model but which must not
    /// create an independent source-semantic declaration.
    RuntimeSupportClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UniverseBindingSpec {
    pub key: UniverseKey,
    pub name: &'static str,
    pub kind: UniverseBindingKind,
    pub exported: bool,
    pub prelude: bool,
}

/// Canonical catalog of built-in universe bindings.
pub const UNIVERSE_BINDINGS: &[UniverseBindingSpec] = &[
    UniverseBindingSpec {
        key: UniverseKey::Object,
        name: "Object",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Behavior,
        name: "Behavior",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Class,
        name: "Class",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Metaclass,
        name: "Metaclass",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Number,
        name: "Number",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Int,
        name: "Int",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Float,
        name: "Float",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::String,
        name: "String",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Nil,
        name: "Nil",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Bool,
        name: "Bool",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::True,
        name: "True",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::False,
        name: "False",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Symbol,
        name: "Symbol",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Selector,
        name: "Selector",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::SelectorPattern,
        name: "SelectorPattern",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Function,
        name: "Function",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Closure,
        name: "Closure",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::BoundMethod,
        name: "BoundMethod",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Method,
        name: "Method",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::MethodFamily,
        name: "MethodFamily",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::BoundMethodFamily,
        name: "BoundMethodFamily",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Family,
        name: "Family",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Option,
        name: "Option",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Result,
        name: "Result",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Ordering,
        name: "Ordering",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Some,
        name: "Some",
        kind: UniverseBindingKind::RuntimeSupportClass,
        exported: false,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::None,
        name: "None",
        kind: UniverseBindingKind::RuntimeSupportClass,
        exported: false,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Unit,
        name: "Unit",
        kind: UniverseBindingKind::RuntimeSupportClass,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Iterable,
        name: "Iterable",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::List,
        name: "List",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Map,
        name: "Map",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Set,
        name: "Set",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Tuple,
        name: "Tuple",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Record,
        name: "Record",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Range,
        name: "Range",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Bytes,
        name: "Bytes",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Module,
        name: "Module",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Package,
        name: "Package",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Project,
        name: "Project",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::System,
        name: "System",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Message,
        name: "Message",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Attribute,
        name: "Attribute",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Error,
        name: "Error",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::MessageNotUnderstood,
        name: "MessageNotUnderstood",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::CannotYieldAcrossNativeFrame,
        name: "CannotYieldAcrossNativeFrame",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::UseAfterCloseError,
        name: "UseAfterCloseError",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Fiber,
        name: "Fiber",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: true,
    },
    UniverseBindingSpec {
        key: UniverseKey::Resource,
        name: "Resource",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ProjectManifest,
        name: "ProjectManifest",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::PackageInfo,
        name: "PackageInfo",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::PackageAuthor,
        name: "PackageAuthor",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::PackageRequirement,
        name: "PackageRequirement",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ResolvedProjectDependency,
        name: "ResolvedProjectDependency",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ModuleDependency,
        name: "ModuleDependency",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ExportTable,
        name: "ExportTable",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Export,
        name: "Export",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ExportKind,
        name: "ExportKind",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ChildModuleTable,
        name: "ChildModuleTable",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ModuleIdentity,
        name: "ModuleIdentity",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::PackageIdentity,
        name: "PackageIdentity",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::ProjectIdentity,
        name: "ProjectIdentity",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
    UniverseBindingSpec {
        key: UniverseKey::Uri,
        name: "Uri",
        kind: UniverseBindingKind::Class,
        exported: true,
        prelude: false,
    },
];

use crate::types::{KindSpec, TypeParameterDeclSpec, UniverseTypeFormSpec};

pub const UNIVERSE_TYPE_FORMS: &[UniverseTypeFormSpec] = &[
    UniverseTypeFormSpec {
        owner: UniverseKey::List,
        parameters: &[TypeParameterDeclSpec {
            name: "T",
            kind: KindSpec::Type,
        }],
    },
    UniverseTypeFormSpec {
        owner: UniverseKey::Set,
        parameters: &[TypeParameterDeclSpec {
            name: "T",
            kind: KindSpec::Type,
        }],
    },
    UniverseTypeFormSpec {
        owner: UniverseKey::Map,
        parameters: &[
            TypeParameterDeclSpec {
                name: "K",
                kind: KindSpec::Type,
            },
            TypeParameterDeclSpec {
                name: "V",
                kind: KindSpec::Type,
            },
        ],
    },
    UniverseTypeFormSpec {
        owner: UniverseKey::Option,
        parameters: &[TypeParameterDeclSpec {
            name: "T",
            kind: KindSpec::Type,
        }],
    },
    UniverseTypeFormSpec {
        owner: UniverseKey::Result,
        parameters: &[
            TypeParameterDeclSpec {
                name: "T",
                kind: KindSpec::Type,
            },
            TypeParameterDeclSpec {
                name: "E",
                kind: KindSpec::Type,
            },
        ],
    },
    UniverseTypeFormSpec {
        owner: UniverseKey::Some,
        parameters: &[TypeParameterDeclSpec {
            name: "T",
            kind: KindSpec::Type,
        }],
    },
];
