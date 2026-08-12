//! Source-authored module, class, and member surfaces.

use std::collections::BTreeMap;

use phalcom_ast::ast::{AttrKind, Attribute, ClassMember, Expr, IndexAccessor, ParameterDef, Program, Statement};
use phalcom_common::range::SourceRange;

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
    /// Members keyed by canonical selector.
    pub members: BTreeMap<String, MemberSurface>,
    /// Members keyed by selector and dispatch side. This preserves distinct
    /// class-side and instance-side declarations that share one selector.
    pub members_by_side: BTreeMap<(String, DispatchSide), MemberSurface>,
    /// Fields keyed by source field name.
    pub fields: BTreeMap<String, FieldSurface>,
    /// Source span of the class declaration.
    pub source_range: SourceRange,
    /// Source span of the class name.
    pub name_range: SourceRange,
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
    /// Whole declaration span.
    pub source_range: SourceRange,
    /// Name or selector span.
    pub name_range: SourceRange,
    /// Parameter surface.
    pub params: Vec<ParamSurface>,
    /// Body retained for later callable-summary inference.
    pub body: Vec<Statement>,
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
    /// Optional source initializer.
    pub initializer: Option<Expr>,
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
    for statement in &program.statements {
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
            members_by_side: BTreeMap::new(),
            fields: BTreeMap::new(),
            source_range: (class_start..class.range.end).into(),
            name_range: class.name_range,
        };
        for member in &class.members {
            let selector = crate::selectors::class_member_selector(member);
            let (kind, side, constructor, params, body, source_range, name_range) = member_parts(member);
            let declaration_start = member_attributes(member)
                .iter()
                .map(|attribute| attribute.range.start)
                .min()
                .unwrap_or(source_range.start);
            if let ClassMember::Field(field) = member {
                class_surface.fields.insert(
                    field.name.clone(),
                    FieldSurface {
                        name: field.name.clone(),
                        kind: if field.name.starts_with("_$") {
                            FieldKind::Implementation
                        } else {
                            FieldKind::Source
                        },
                        is_class_side: field.is_static,
                        source_range: field.range,
                        name_range: field.name_range,
                        initializer: field.default.clone(),
                    },
                );
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
                source_range: (declaration_start..source_range.end).into(),
                name_range,
                params,
                body,
            };
            class_surface.members_by_side.insert((selector.clone(), side), member_surface.clone());
            class_surface.members.entry(selector).or_insert(member_surface);
        }
        surface.classes.insert(id, class_surface);
    }
    surface
}

fn member_parts(member: &ClassMember) -> (MemberKind, DispatchSide, bool, Vec<ParamSurface>, Vec<Statement>, SourceRange, SourceRange) {
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
            method.body.clone(),
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
            getter.body.clone(),
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
            setter.body.clone(),
            setter.range,
            setter.name_range,
        ),
        ClassMember::Field(field) => (
            MemberKind::Field,
            if field.is_static { DispatchSide::Class } else { DispatchSide::Instance },
            false,
            Vec::new(),
            Vec::new(),
            field.range,
            field.name_range,
        ),
        ClassMember::Variant(variant) => (
            MemberKind::Variant,
            DispatchSide::Instance,
            false,
            Vec::new(),
            Vec::new(),
            variant.range,
            variant.name_range,
        ),
        ClassMember::Index(index) => {
            let params = match &index.accessor {
                IndexAccessor::Get => params(&index.params),
                IndexAccessor::Set { put } => index.params.iter().chain(std::iter::once(put)).map(param).collect(),
            };
            (
                MemberKind::Index,
                DispatchSide::Instance,
                false,
                params,
                index.body.clone(),
                index.range,
                index.name_range,
            )
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
        assert_eq!(class.members["new()"].side, DispatchSide::Class);
        assert!(class.members["new()"].is_constructor);
        assert_eq!(class.members["x"].kind, MemberKind::Getter);
    }
}
