//! Source-authored module, class, and member surfaces.

use std::collections::BTreeMap;

use phalcom_ast::ast::{AttrKind, Attribute, ClassMember, IndexAccessor, ParameterDef, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_native_surface::NativeReturnShape;

use super::ids::{CallableId, ClassId, DispatchSide, ModuleId};

/// One parsed module's class surface.
#[derive(Clone, Debug)]
pub struct ModuleSurface {
    /// Module identity.
    pub module: ModuleId,
    /// Classes declared by this module.
    pub classes: BTreeMap<ClassId, ClassSurface>,
}

/// Surface of one source-authored class.
#[derive(Clone, Debug)]
pub struct ClassSurface {
    /// Module-qualified class identity.
    pub id: ClassId,
    /// Explicit superclass, if written.
    pub superclass: Option<ClassId>,
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

    /// Iterates all declarations on both dispatch sides.
    pub fn all_members(&self) -> impl Iterator<Item = &MemberSurface> {
        self.members
            .values()
            .flat_map(|members| [members.instance.as_ref(), members.class.as_ref()].into_iter().flatten())
    }
}

/// One callable or field-like class member.
#[derive(Clone, Debug)]
pub struct MemberSurface {
    /// Canonical callable identity.
    pub callable: CallableId,
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
    pub ast: MemberAstRef,
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
        classes: BTreeMap::new(),
    };
    for (class_stmt_idx, statement) in program.statements.iter().enumerate() {
        let Statement::Class(class) = statement else { continue };
        let id = ClassId::new(module.clone(), class.name.clone());
        let superclass = class.superclass.as_ref().map(|parent| ClassId::new(module.clone(), parent.name.clone()));
        let class_start = class
            .attributes
            .iter()
            .map(|attribute| attribute.range.start)
            .min()
            .unwrap_or(class.range.start);
        let mut class_surface = ClassSurface {
            id: id.clone(),
            superclass,
            members: BTreeMap::new(),
            fields: BTreeMap::new(),
            source_range: (class_start..class.range.end).into(),
            name_range: class.name_range,
        };
        for (member_idx, member) in class.members.iter().enumerate() {
            let selector = crate::selectors::class_member_selector(member);
            let (kind, side, constructor, params, source_range, name_range) = member_parts(member);
            let declaration_start = member_attributes(member)
                .iter()
                .map(|attribute| attribute.range.start)
                .min()
                .unwrap_or(source_range.start);
            let ast = MemberAstRef { class_stmt_idx, member_idx };
            if let ClassMember::Field(field) = member {
                let field_surface = FieldSurface {
                    name: field.name.clone(),
                    kind: if field.name.starts_with("_$") {
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
                kind,
                visibility: member_visibility(member),
                side,
                is_constructor: constructor,
                native_return: None,
                source_range: (declaration_start..source_range.end).into(),
                name_range,
                params,
                ast,
            };
            class_surface.members.entry(selector).or_default().insert(side, member_surface);
        }
        surface.classes.insert(id, class_surface);
    }
    surface
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
                IndexAccessor::Set { put } => index.params.iter().chain(std::iter::once(put)).map(param).collect(),
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

        assert_eq!(class.field("_count", DispatchSide::Instance).unwrap().is_class_side, false);
        assert_eq!(class.field("_count", DispatchSide::Class).unwrap().is_class_side, true);
    }
}
