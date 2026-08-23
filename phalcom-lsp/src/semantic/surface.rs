//! Source-authored module, class, and member surfaces.

use std::collections::{BTreeMap, BTreeSet};

use phalcom_ast::ast::{
    AttrKind, Attribute, ClassMember, DependencyDecl, ImportDecl, IndexAccessor, ParameterDef, Program, RestMode, Statement, StaticSymbolRef,
};
use phalcom_common::range::SourceRange;
pub use phalcom_common::selector::{Selector, SelectorPattern};
use phalcom_native_surface::{NativeReturnShape, NativeSurfaceId};

use super::ids::{CallableId, ClassId, DispatchSide, ModuleId};

/// One parsed module's class surface.
#[derive(Clone, Debug)]
pub struct ModuleSurface {
    /// Module identity.
    pub module: ModuleId,
    /// Public names exported by this module, including re-export aliases.
    pub exports: BTreeMap<String, ExportSurface>,
    /// Module-scope imported names and their source paths.
    pub imports: BTreeMap<String, ImportSurface>,
    /// Child module names exposed by a package declaration.
    pub exposed_children: BTreeSet<String>,
    /// Header metadata retained for documentation/reflection consumers.
    pub metadata: phalcom_modules::ModuleMetadata,
    /// Classes declared by this module.
    pub classes: BTreeMap<ClassId, ClassSurface>,
}

/// LSP-facing export declaration surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportSurface {
    /// Name visible to importers.
    pub public_name: String,
    /// Local/original name that supplies the value.
    pub local_name: String,
    /// Source span of the export item.
    pub range: SourceRange,
}

/// LSP-facing imported namespace surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportSurface {
    /// Local name introduced by the import.
    pub local_name: String,
    /// Logical source path.
    pub path: String,
    /// Whether this local name denotes a module object.
    pub whole_module: bool,
    /// Source span of the import item.
    pub range: SourceRange,
}

/// Surface of one source-authored class.
#[derive(Clone, Debug)]
pub struct ClassSurface {
    /// Module-qualified class identity.
    pub id: ClassId,
    /// Explicit superclass, if written.
    pub superclass: Option<ClassId>,
    /// Original static superclass reference, before module linking.
    pub superclass_reference: Option<StaticSymbolRef>,
    /// Members keyed by canonical selector, preserving both dispatch sides.
    pub members: BTreeMap<String, MemberSides>,
    /// Fields keyed by source field name, preserving both storage sides.
    pub fields: BTreeMap<String, FieldSides>,
    /// Source span of the class declaration.
    pub source_range: SourceRange,
    /// Source span of the class name.
    pub name_range: SourceRange,
}

/// The two dispatch-side declarations for one selector.
#[derive(Clone, Debug, Default)]
pub struct MemberSides {
    /// Instance-side declaration, if present.
    pub instance: Option<MemberSurface>,
    /// Class-side declaration, if present.
    pub class: Option<MemberSurface>,
}

impl MemberSides {
    /// Returns declaration for one dispatch side.
    pub fn get(&self, side: DispatchSide) -> Option<&MemberSurface> {
        match side {
            DispatchSide::Instance => self.instance.as_ref(),
            DispatchSide::Class => self.class.as_ref(),
        }
    }

    /// Returns mutable declaration slot for one dispatch side.
    pub fn get_mut(&mut self, side: DispatchSide) -> Option<&mut MemberSurface> {
        match side {
            DispatchSide::Instance => self.instance.as_mut(),
            DispatchSide::Class => self.class.as_mut(),
        }
    }

    fn insert(&mut self, side: DispatchSide, member: MemberSurface) {
        match side {
            DispatchSide::Instance => self.instance = Some(member),
            DispatchSide::Class => self.class = Some(member),
        }
    }
}

/// The two storage-side declarations for one field name.
#[derive(Clone, Debug, Default)]
pub struct FieldSides {
    /// Instance-side declaration, if present.
    pub instance: Option<FieldSurface>,
    /// Class-side declaration, if present.
    pub class: Option<FieldSurface>,
}

impl FieldSides {
    fn get(&self, side: DispatchSide) -> Option<&FieldSurface> {
        match side {
            DispatchSide::Instance => self.instance.as_ref(),
            DispatchSide::Class => self.class.as_ref(),
        }
    }

    fn get_mut(&mut self, side: DispatchSide) -> &mut Option<FieldSurface> {
        match side {
            DispatchSide::Instance => &mut self.instance,
            DispatchSide::Class => &mut self.class,
        }
    }
}

impl ClassSurface {
    /// Returns declaration for selector and dispatch side.
    pub fn member(&self, selector: &str, side: DispatchSide) -> Option<&MemberSurface> {
        self.members.get(selector).and_then(|members| members.get(side))
    }

    /// Returns declaration identified by its complete callable identity.
    pub fn member_by_id(&self, callable: &CallableId) -> Option<&MemberSurface> {
        (self.id == callable.owner).then(|| self.member(&callable.selector, callable.side)).flatten()
    }

    /// Returns field declaration for name and storage side.
    pub fn field(&self, name: &str, side: DispatchSide) -> Option<&FieldSurface> {
        self.fields.get(name).and_then(|fields| fields.get(side))
    }

    /// Iterates declarations installed on one dispatch side.
    pub fn members_on(&self, side: DispatchSide) -> impl Iterator<Item = &MemberSurface> {
        self.members.values().filter_map(move |members| members.get(side))
    }

    /// Iterates all direct declarations matching a given [`SelectorPattern`].
    pub fn members_matching<'a>(&'a self, side: DispatchSide, pattern: &'a SelectorPattern) -> impl Iterator<Item = &'a MemberSurface> {
        self.members_on(side).filter(move |member| pattern.matches(&member.selector))
    }

    /// Iterates all declarations on both dispatch sides.
    pub fn all_members(&self) -> impl Iterator<Item = &MemberSurface> {
        self.members
            .values()
            .flat_map(|members| [members.instance.as_ref(), members.class.as_ref()].into_iter().flatten())
    }
}

impl ModuleSurface {
    /// Builds direct source lookup from canonical callable identity to its AST
    /// reference. Only `MemberOrigin::Source` members carry valid AST refs;
    /// native and generated members are excluded to avoid sentinel propagation.
    pub fn callable_index(&self) -> BTreeMap<CallableId, MemberAstRef> {
        self.classes
            .values()
            .flat_map(|class| {
                class.all_members().filter_map(|member| {
                    if let MemberOrigin::Source(ast_ref) = &member.origin {
                        Some((member.callable.clone(), *ast_ref))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Returns one source member without scanning every class member.
    pub fn member_by_id(&self, callable: &CallableId) -> Option<&MemberSurface> {
        self.classes.get(&callable.owner).and_then(|class| class.member_by_id(callable))
    }
}

/// Normalized rest capture layout on a class member surface.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RestSurface {
    /// Number of leading fixed positional parameters.
    pub fixed_positionals: usize,
    /// Fixed labeled parameters.
    pub fixed_labels: Box<[String]>,
    /// Rest capture mode.
    pub mode: RestSurfaceMode,
}

/// Rest capture mode for variable-arity methods.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RestSurfaceMode {
    /// Captures trailing positionals (`*args`).
    Positional,
    /// Captures keyword arguments (`**kwargs`).
    Labeled,
    /// Captures both positionals and keywords (`*tail, **extra`).
    Split,
    /// Captures all remaining arguments (`***all`).
    Complete,
}

impl RestSurface {
    /// Tests whether this rest signature accepts the given call shape.
    pub fn accepts(&self, positionals: usize, labels: &[String]) -> bool {
        let fixed = self.fixed_positionals;
        match self.mode {
            RestSurfaceMode::Positional => positionals >= fixed && labels == self.fixed_labels.as_ref(),
            RestSurfaceMode::Labeled => positionals == fixed && labels.starts_with(self.fixed_labels.as_ref()),
            RestSurfaceMode::Split | RestSurfaceMode::Complete => positionals >= fixed && labels.starts_with(self.fixed_labels.as_ref()),
        }
    }
}

/// Origin of a member declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberOrigin {
    /// Normal source AST declaration.
    Source(MemberAstRef),
    /// Native primitive declaration.
    Native(NativeSurfaceId),
    /// Synthetic/generated declaration.
    Generated(GeneratedMemberOrigin),
}

/// Stable identity for a generated presentation member without a source AST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedMemberOrigin {
    /// Stable canonical identifier assigned by the generated member catalog.
    pub stable_key: Box<str>,
}

/// One callable or field-like class member.
#[derive(Clone, Debug)]
pub struct MemberSurface {
    /// Canonical callable identity.
    pub callable: CallableId,
    /// Structural exact selector.
    pub selector: Selector,
    /// Rest parameter layout if variable-arity.
    pub rest: Option<RestSurface>,
    /// Source-level member category.
    pub kind: MemberKind,
    /// Source visibility.
    pub visibility: MemberVisibility,
    /// Whether this member is dispatched on the class object.
    pub side: DispatchSide,
    /// Whether this member is a constructor/factory.
    pub is_constructor: bool,
    /// Optional native return contract. Source members leave this absent.
    pub native_return: Option<NativeReturnShape>,
    /// Whole declaration span.
    pub source_range: SourceRange,
    /// Name or selector span.
    pub name_range: SourceRange,
    /// Parameter surface.
    pub params: Vec<ParamSurface>,
    /// Reference to this member's AST in the source program.
    pub ast: Option<MemberAstRef>,
    /// Member implementation origin.
    pub origin: MemberOrigin,
}

/// One declared field.
#[derive(Clone, Debug)]
pub struct FieldSurface {
    /// Field name.
    pub name: String,
    /// Source or implementation field lane.
    pub kind: FieldKind,
    /// Whether field storage is class-side.
    pub is_class_side: bool,
    /// Source span.
    pub source_range: SourceRange,
    /// Exact field-name token span.
    pub name_range: SourceRange,
    /// Reference to the field's AST for initializer lookup.
    pub ast: MemberAstRef,
}

/// Parameter metadata used by callable inference and completion.
#[derive(Clone, Debug)]
pub struct ParamSurface {
    /// Local parameter binding name.
    pub name: String,
    /// External call label, if any.
    pub label: Option<String>,
    /// Rest capture mode.
    pub rest_mode: RestMode,
    /// Parameter source span.
    pub source_range: SourceRange,
    /// Exact local binding token span.
    pub name_range: SourceRange,
    /// Exact external label token span, if written.
    pub label_range: Option<SourceRange>,
}

/// Reference to a member's AST in the parent program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemberAstRef {
    /// Index into `Program::statements`.
    pub class_stmt_idx: usize,
    /// Index into `ClassDef::members`.
    pub member_idx: usize,
}

/// Source-level member category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemberKind {
    /// Ordinary method.
    Method,
    /// Bare-name getter.
    Getter,
    /// Setter member.
    Setter,
    /// Declared field.
    Field,
    /// Bracket subscript member.
    Index,
    /// Sealed variant arm.
    Variant,
}

/// Source visibility used after semantic candidate resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemberVisibility {
    /// Public member.
    Public,
    /// Defining-class-only member.
    Private,
    /// Defining-class-and-subclasses member.
    Protected,
    /// Core/internal member.
    Internal,
}

/// Field storage lane.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FieldKind {
    /// Source-authored field.
    Source,
    /// Implementation field.
    Implementation,
}

/// Builds a module surface from a recovered AST.
pub fn build_module_surface(module: ModuleId, program: &Program) -> ModuleSurface {
    let mut surface = ModuleSurface {
        module: module.clone(),
        exports: BTreeMap::new(),
        imports: BTreeMap::new(),
        exposed_children: BTreeSet::new(),
        metadata: phalcom_modules::ModuleMetadata::from_ast(&program.preamble.metadata, phalcom_modules::ModuleKind::Module).unwrap_or_default(),
        classes: BTreeMap::new(),
    };
    for dependency in &program.preamble.dependencies {
        match dependency {
            DependencyDecl::Import(import_decl) => match import_decl {
                ImportDecl::Module(import) => {
                    let local_name = import
                        .alias
                        .as_ref()
                        .map(|alias| alias.name.clone())
                        .or_else(|| import.path.segments.last().map(|segment| segment.name.clone()))
                        .or_else(|| match &import.path.root {
                            phalcom_ast::ast::ImportRoot::Absolute(segment) => Some(segment.name.clone()),
                            phalcom_ast::ast::ImportRoot::Relative { .. } => None,
                        })
                        .unwrap_or_default();
                    if !local_name.is_empty() {
                        surface.imports.insert(
                            local_name.clone(),
                            ImportSurface {
                                local_name,
                                path: import.path.to_string(),
                                whole_module: true,
                                range: import.range,
                            },
                        );
                    }
                }
                ImportDecl::Selective(import) => {
                    for item in &import.items {
                        let local_name = item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone());
                        surface.imports.insert(
                            local_name.clone(),
                            ImportSurface {
                                local_name,
                                path: import.path.to_string(),
                                whole_module: false,
                                range: item.range,
                            },
                        );
                    }
                }
            },
            DependencyDecl::ReExport(reexport) => {
                for item in &reexport.items {
                    let public_name = item
                        .alias
                        .as_ref()
                        .map(|alias| alias.name.clone())
                        .unwrap_or_else(|| item.local_or_remote_name.clone());
                    surface.exports.insert(
                        public_name.clone(),
                        ExportSurface {
                            public_name,
                            local_name: item.local_or_remote_name.clone(),
                            range: item.range,
                        },
                    );
                }
            }
            DependencyDecl::Expose(expose) => {
                surface.exposed_children.insert(expose.child.name.clone());
            }
        }
    }
    for (class_stmt_idx, statement) in program.statements.iter().enumerate() {
        if let Statement::Export(export) = statement {
            for item in &export.items {
                let public_name = item
                    .alias
                    .as_ref()
                    .map(|alias| alias.name.clone())
                    .unwrap_or_else(|| item.local_or_remote_name.clone());
                surface.exports.insert(
                    public_name.clone(),
                    ExportSurface {
                        public_name,
                        local_name: item.local_or_remote_name.clone(),
                        range: item.range,
                    },
                );
            }
            continue;
        }
        let Statement::Class(class) = statement else { continue };
        let id = ClassId::new(module.clone(), class.name.clone());
        let superclass_reference = class.superclass_ref().cloned();
        let superclass = superclass_reference
            .as_ref()
            .map(|parent| ClassId::new(module.clone(), parent.leaf_name().to_string()));
        let class_start = class
            .attributes
            .iter()
            .map(|attribute| attribute.range.start)
            .min()
            .unwrap_or(class.range.start);
        let mut class_surface = ClassSurface {
            id: id.clone(),
            superclass,
            superclass_reference,
            members: BTreeMap::new(),
            fields: BTreeMap::new(),
            source_range: (class_start..class.range.end).into(),
            name_range: class.name_range,
        };
        for (member_idx, member) in class.members.iter().enumerate() {
            let structural_selector = crate::selectors::selector_from_member(member);
            let selector = structural_selector.encode();
            let (kind, side, constructor, params, source_range, name_range) = member_parts(member);
            let rest = match member {
                ClassMember::Method(m) => rest_surface_from_params(&m.params),
                _ => None,
            };
            let declaration_start = member_attributes(member)
                .iter()
                .map(|attribute| attribute.range.start)
                .min()
                .unwrap_or(source_range.start);
            let ast = MemberAstRef { class_stmt_idx, member_idx };
            if let ClassMember::Field(field) = member {
                let field_surface = FieldSurface {
                    name: field.name.clone(),
                    kind: if field.name.starts_with("__") {
                        FieldKind::Implementation
                    } else {
                        FieldKind::Source
                    },
                    is_class_side: field.is_static,
                    source_range: field.range,
                    name_range: field.name_range,
                    ast,
                };
                class_surface.fields.entry(field.name.clone()).or_default().get_mut(side).replace(field_surface);
            }
            let member_surface = MemberSurface {
                callable: CallableId {
                    owner: id.clone(),
                    selector: selector.clone(),
                    side,
                },
                selector: structural_selector,
                rest,
                kind,
                visibility: member_visibility(member),
                side,
                is_constructor: constructor,
                native_return: None,
                source_range: (declaration_start..source_range.end).into(),
                name_range,
                params,
                ast: Some(ast),
                origin: MemberOrigin::Source(ast),
            };
            class_surface.members.entry(selector).or_default().insert(side, member_surface);
        }
        surface.classes.insert(id, class_surface);
    }
    for statement in &program.statements {
        let Statement::Export(export) = statement else { continue };
        for item in &export.items {
            let public_name = item
                .alias
                .as_ref()
                .map(|alias| alias.name.clone())
                .unwrap_or_else(|| item.local_or_remote_name.clone());
            surface.exports.insert(
                public_name.clone(),
                ExportSurface {
                    public_name,
                    local_name: item.local_or_remote_name.clone(),
                    range: item.range,
                },
            );
        }
    }
    surface
}

/// Normalizes AST parameters into a rest layout surface descriptor if variable-arity.
pub fn rest_surface_from_params(params: &[ParameterDef]) -> Option<RestSurface> {
    let mut fixed_positionals = 0;
    let mut fixed_labels = Vec::new();
    let mut has_pos_rest = false;
    let mut has_lab_rest = false;
    let mut has_comp_rest = false;

    for param in params {
        match param.rest_mode {
            RestMode::None => {
                if let Some(label) = &param.label {
                    fixed_labels.push(label.clone());
                } else {
                    fixed_positionals += 1;
                }
            }
            RestMode::Positional => has_pos_rest = true,
            RestMode::Labeled => has_lab_rest = true,
            RestMode::Complete => has_comp_rest = true,
        }
    }

    let mode = match (has_pos_rest, has_lab_rest, has_comp_rest) {
        (true, true, false) => RestSurfaceMode::Split,
        (true, false, false) => RestSurfaceMode::Positional,
        (false, true, false) => RestSurfaceMode::Labeled,
        (false, false, true) => RestSurfaceMode::Complete,
        _ => return None,
    };

    Some(RestSurface {
        fixed_positionals,
        fixed_labels: fixed_labels.into_boxed_slice(),
        mode,
    })
}

/// Extracts rest layout surface from canonical selector string.
pub fn rest_surface_from_selector_str(selector: &str) -> Option<RestSurface> {
    if !selector.contains('*') {
        return None;
    }
    let open = selector.find('(')?;
    let inner = selector[open + 1..].strip_suffix(')')?;
    let mut fixed_positionals = 0;
    let mut fixed_labels = Vec::new();
    let mut has_pos_rest = false;
    let mut has_lab_rest = false;
    let mut has_comp_rest = false;

    for part in inner.split(',') {
        match part.trim() {
            "*" => has_pos_rest = true,
            "**" => has_lab_rest = true,
            "***" => has_comp_rest = true,
            "_" => fixed_positionals += 1,
            label if !label.is_empty() => fixed_labels.push(crate::selectors::decode_label_component(label)),
            _ => {}
        }
    }

    let mode = match (has_pos_rest, has_lab_rest, has_comp_rest) {
        (true, true, false) => RestSurfaceMode::Split,
        (true, false, false) => RestSurfaceMode::Positional,
        (false, true, false) => RestSurfaceMode::Labeled,
        (false, false, true) => RestSurfaceMode::Complete,
        _ => return None,
    };

    Some(RestSurface {
        fixed_positionals,
        fixed_labels: fixed_labels.into_boxed_slice(),
        mode,
    })
}

fn member_parts(member: &ClassMember) -> (MemberKind, DispatchSide, bool, Vec<ParamSurface>, SourceRange, SourceRange) {
    match member {
        ClassMember::Method(method) => (
            MemberKind::Method,
            if method.is_static || method.is_constructor || has_builtin(&method.attributes, "class") || has_builtin(&method.attributes, "constructor") {
                DispatchSide::Class
            } else {
                DispatchSide::Instance
            },
            method.is_constructor || has_builtin(&method.attributes, "constructor"),
            params(&method.params),
            method.range,
            method.name_range,
        ),
        ClassMember::Getter(getter) => (
            MemberKind::Getter,
            if getter.is_static || has_builtin(&getter.attributes, "class") {
                DispatchSide::Class
            } else {
                DispatchSide::Instance
            },
            false,
            Vec::new(),
            getter.range,
            getter.name_range,
        ),
        ClassMember::Setter(setter) => (
            MemberKind::Setter,
            if setter.is_static || has_builtin(&setter.attributes, "class") {
                DispatchSide::Class
            } else {
                DispatchSide::Instance
            },
            false,
            vec![ParamSurface {
                name: setter.param.name.clone(),
                label: setter.param.label.clone(),
                rest_mode: RestMode::None,
                source_range: setter.param.range,
                name_range: setter.param.name_range,
                label_range: setter.param.label_range,
            }],
            setter.range,
            setter.name_range,
        ),
        ClassMember::Field(field) => (
            MemberKind::Field,
            if field.is_static { DispatchSide::Class } else { DispatchSide::Instance },
            false,
            Vec::new(),
            field.range,
            field.name_range,
        ),
        ClassMember::Variant(variant) => (
            MemberKind::Variant,
            DispatchSide::Instance,
            false,
            Vec::new(),
            variant.range,
            variant.name_range,
        ),
        ClassMember::Index(index) => {
            let params = match &index.accessor {
                IndexAccessor::Get => params(&index.params),
                IndexAccessor::Set { put } => index.params.iter().chain(std::iter::once(put.as_ref())).map(param).collect(),
            };
            (MemberKind::Index, DispatchSide::Instance, false, params, index.range, index.name_range)
        }
    }
}

fn params(parameters: &[ParameterDef]) -> Vec<ParamSurface> {
    parameters.iter().map(param).collect()
}

fn member_attributes(member: &ClassMember) -> &[Attribute] {
    match member {
        ClassMember::Method(item) => &item.attributes,
        ClassMember::Getter(item) => &item.attributes,
        ClassMember::Setter(item) => &item.attributes,
        ClassMember::Field(item) => &item.attributes,
        ClassMember::Variant(item) => &item.attributes,
        ClassMember::Index(item) => &item.attributes,
    }
}

fn param(parameter: &ParameterDef) -> ParamSurface {
    ParamSurface {
        name: parameter.name.clone(),
        label: parameter.label.clone(),
        rest_mode: parameter.rest_mode,
        source_range: parameter.range,
        name_range: parameter.name_range,
        label_range: parameter.label_range,
    }
}

fn has_builtin(attributes: &[Attribute], name: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| matches!(&attribute.kind, AttrKind::Builtin(kind) if kind.name() == name))
}

fn member_visibility(member: &ClassMember) -> MemberVisibility {
    let (name, attributes, is_field) = match member {
        ClassMember::Method(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Getter(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Setter(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Field(item) => (Some(item.name.as_str()), item.attributes.as_slice(), true),
        ClassMember::Variant(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Index(item) => (None, item.attributes.as_slice(), false),
    };
    if name.is_some_and(|name| name.starts_with("_$")) {
        MemberVisibility::Internal
    } else if is_field || has_builtin(attributes, "private") {
        MemberVisibility::Private
    } else if has_builtin(attributes, "protected") {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    #[test]
    fn same_named_classes_keep_module_qualified_ids() {
        let program = parse("class Point { move() { } }", 0).program;
        let one = build_module_surface(ModuleId::new("file:///one.ph"), &program);
        let two = build_module_surface(ModuleId::new("file:///two.ph"), &program);
        assert_ne!(one.classes.keys().next(), two.classes.keys().next());
    }

    #[test]
    fn constructor_and_member_surface_preserve_dispatch_metadata() {
        let program = parse("class Point { @constructor new() { } x { } }", 0).program;
        let surface = build_module_surface(ModuleId::new("file:///point.ph"), &program);
        let class = surface.classes.values().next().unwrap();
        let constructor = class.member("new()", DispatchSide::Class).unwrap();
        assert_eq!(constructor.side, DispatchSide::Class);
        assert!(constructor.is_constructor);
        assert_eq!(class.member("x", DispatchSide::Instance).unwrap().kind, MemberKind::Getter);
        assert_eq!(constructor.selector.encode(), "new()");
    }

    #[test]
    fn instance_and_class_members_with_same_selector_remain_distinct() {
        let program = parse(
            r#"
class Widget {
  make() { 1 }

  @class
  make() { 2 }
}
"#,
            0,
        )
        .program;

        let module = ModuleId::new("file:///widget.ph");
        let surface = build_module_surface(module.clone(), &program);
        let class = &surface.classes[&ClassId::new(module, "Widget")];

        let instance = class.member("make()", DispatchSide::Instance).expect("instance make");
        let class_side = class.member("make()", DispatchSide::Class).expect("class make");

        assert_ne!(instance.callable, class_side.callable);
        assert_eq!(class.members_on(DispatchSide::Instance).count(), 1);
        assert_eq!(class.members_on(DispatchSide::Class).count(), 1);
        assert_eq!(class.all_members().count(), 2);
    }

    #[test]
    fn field_sides_preserve_same_name_storage_lanes() {
        let program = parse("class Widget { _count\n _count\n }", 0).program;
        let mut program = program;
        let phalcom_ast::ast::Statement::Class(class) = &mut program.statements[0] else {
            panic!("expected class");
        };
        let phalcom_ast::ast::ClassMember::Field(class_field) = &mut class.members[1] else {
            panic!("expected field");
        };
        class_field.is_static = true;

        let module = ModuleId::new("file:///widget-fields.ph");
        let surface = build_module_surface(module.clone(), &program);
        let class = &surface.classes[&ClassId::new(module, "Widget")];

        assert!(!class.field("_count", DispatchSide::Instance).unwrap().is_class_side);
        assert!(class.field("_count", DispatchSide::Class).unwrap().is_class_side);
    }

    #[test]
    fn rest_surfaces_preserve_structure_and_matching() {
        let program = parse(
            r#"
class C {
  sum(*numbers) { }
  format(_ fmt, *args) { }
  options(timeout, **kwargs) { }
  split(_ first, *tail, mode, **extra) { }
  complete(_ first, ***all) { }
}
"#,
            0,
        )
        .program;

        let module = ModuleId::new("file:///rest.ph");
        let surface = build_module_surface(module.clone(), &program);
        let class = &surface.classes[&ClassId::new(module, "C")];

        let sum = class.member("sum(_)", DispatchSide::Instance).unwrap();
        assert_eq!(sum.rest.as_ref().unwrap().mode, RestSurfaceMode::Positional);
        assert!(sum.rest.as_ref().unwrap().accepts(3, &[]));
        assert!(!sum.rest.as_ref().unwrap().accepts(0, &["a".into()]));

        let pattern = SelectorPattern::named_method("sum", [], [], true).unwrap();
        let matches = class.members_matching(DispatchSide::Instance, &pattern).collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].callable.selector, "sum(_)");
    }

    #[test]
    fn module_surface_preserves_logical_module_declarations() {
        let program = parse(
            "import .provider as Provider\nfrom ..shared import (Thing as T)\nexpose .child\nexport T as Public\n",
            0,
        )
        .program;
        let surface = build_module_surface(ModuleId::new("file:///package/main.ph"), &program);

        assert_eq!(surface.imports["Provider"].path, ".provider");
        assert_eq!(surface.imports["T"].path, "..shared");
        assert_eq!(surface.exports["Public"].local_name, "T");
        assert!(surface.exposed_children.contains("child"));
    }
}
