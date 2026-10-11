//! Source core declaration surface extraction.

use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use phalcom_ast::ast::{ClassMember, ImplementationKind, IndexAccessor, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

/// Explicit authority for a source/native collision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceNativeBindingRole {
    None,
    DeclarationImplementation,
    WrapperOverNative,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMemberRecord {
    pub selector: Selector,
    pub selector_raw: String,
    pub side: DispatchSide,
    pub is_getter: bool,
    pub is_setter: bool,
    pub implementation: ImplementationKind,
    pub is_primitive: bool,
    pub range: SourceRange,
    pub doc_comment: Option<String>,
    pub binding_role: SourceNativeBindingRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceClassRecord {
    pub declaration_id: DeclarationId,
    pub name: String,
    pub superclass: Option<String>,
    pub is_primitive: bool,
    pub members: Vec<SourceMemberRecord>,
    pub range: SourceRange,
    pub doc_comment: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceEnumVariantRecord {
    pub name: String,
    pub selector: Selector,
    pub range: SourceRange,
    pub binding_role: SourceNativeBindingRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceEnumRecord {
    pub declaration_id: DeclarationId,
    pub name: String,
    pub variants: Vec<SourceEnumVariantRecord>,
    pub members: Vec<SourceMemberRecord>,
    pub range: SourceRange,
    pub doc_comment: Option<String>,
    pub binding_role: SourceNativeBindingRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceDeclarationRecord {
    Class(SourceClassRecord),
    Enum(SourceEnumRecord),
}

/// Extracts source class declaration records from a parsed Program.
pub fn extract_source_surface(module_id: &ModuleId, program: &Program) -> Vec<SourceClassRecord> {
    let mut classes = Vec::new();
    for decl in extract_source_declarations(module_id, program) {
        if let SourceDeclarationRecord::Class(c) = decl {
            classes.push(c);
        }
    }
    classes
}

/// Extracts all source declaration records (classes and enums) from a parsed Program.
pub fn extract_source_declarations(module_id: &ModuleId, program: &Program) -> Vec<SourceDeclarationRecord> {
    let mut declarations = Vec::new();
    for stmt in &program.statements {
        match stmt {
            Statement::Class(class_def) => {
                let class_decl = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                let superclass = class_def.superclass_ref().map(|s| s.root.clone());
                let mut members = Vec::new();

                for member in &class_def.members {
                    match member {
                        ClassMember::Method(m) => {
                            let mut slots = Vec::new();
                            for p in &m.params {
                                if p.rest_mode != phalcom_ast::ast::RestMode::None {
                                    continue;
                                }
                                if let Some(ref l) = p.label {
                                    slots.push(SelectorSlot::Label(l.clone()));
                                } else {
                                    slots.push(SelectorSlot::Positional);
                                }
                            }
                            if let Ok(sel) = Selector::method(&m.name, slots) {
                                let encoded = sel.encode();
                                members.push(SourceMemberRecord {
                                    selector: sel,
                                    selector_raw: encoded,
                                    side: if m.is_static { DispatchSide::Class } else { DispatchSide::Instance },
                                    is_getter: false,
                                    is_setter: false,
                                    implementation: m.implementation,
                                    is_primitive: m.is_primitive,
                                    range: m.range,
                                    doc_comment: None,
                                    binding_role: source_native_binding_role(&m.attributes, m.implementation),
                                });
                            }
                        }
                        ClassMember::Getter(g) => {
                            if let Ok(sel) = Selector::getter(&g.name) {
                                members.push(SourceMemberRecord {
                                    selector: sel,
                                    selector_raw: g.name.clone(),
                                    side: if g.is_static { DispatchSide::Class } else { DispatchSide::Instance },
                                    is_getter: true,
                                    is_setter: false,
                                    implementation: g.implementation,
                                    is_primitive: g.is_primitive,
                                    range: g.range,
                                    doc_comment: None,
                                    binding_role: source_native_binding_role(&g.attributes, g.implementation),
                                });
                            }
                        }
                        ClassMember::Setter(s) => {
                            if let Ok(sel) = Selector::setter(&s.name) {
                                members.push(SourceMemberRecord {
                                    selector: sel,
                                    selector_raw: format!("{}=(_)", s.name),
                                    side: if s.is_static { DispatchSide::Class } else { DispatchSide::Instance },
                                    is_getter: false,
                                    is_setter: true,
                                    implementation: s.implementation,
                                    is_primitive: s.is_primitive,
                                    range: s.range,
                                    doc_comment: None,
                                    binding_role: source_native_binding_role(&s.attributes, s.implementation),
                                });
                            }
                        }
                        ClassMember::Index(index) => {
                            let slots = index
                                .params
                                .iter()
                                .filter(|parameter| parameter.rest_mode == phalcom_ast::ast::RestMode::None)
                                .map(|parameter| {
                                    parameter
                                        .label
                                        .as_ref()
                                        .map_or(SelectorSlot::Positional, |label| SelectorSlot::Label(label.clone()))
                                })
                                .collect::<Vec<_>>();
                            let selector = match &index.accessor {
                                IndexAccessor::Get => Selector::subscript_get(slots),
                                IndexAccessor::Set { .. } => Selector::subscript_set(slots),
                            };
                            if let Ok(selector) = selector {
                                members.push(SourceMemberRecord {
                                    selector_raw: selector.encode(),
                                    selector,
                                    side: DispatchSide::Instance,
                                    is_getter: matches!(&index.accessor, IndexAccessor::Get),
                                    is_setter: matches!(&index.accessor, IndexAccessor::Set { .. }),
                                    implementation: index.implementation,
                                    is_primitive: index.is_primitive,
                                    range: index.range,
                                    doc_comment: None,
                                    binding_role: source_native_binding_role(&index.attributes, index.implementation),
                                });
                            }
                        }
                        _ => {}
                    }
                }

                declarations.push(SourceDeclarationRecord::Class(SourceClassRecord {
                    declaration_id: class_decl,
                    name: class_def.name.clone(),
                    superclass,
                    is_primitive: class_def.is_primitive,
                    members,
                    range: class_def.range,
                    doc_comment: None,
                }));
            }
            Statement::Data(data_def) if data_def.is_sum() => {
                let cases = data_def.cases().unwrap_or_default();
                let enum_decl = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                let enum_binding_role = source_native_binding_role(&data_def.attributes, data_def.implementation);
                let mut variants = Vec::new();

                for v in cases {
                    let sel = phalcom_ast::selector::selector_from_variant(v);
                    let v_binding_role = match source_native_binding_role(&v.attributes, ImplementationKind::Phalcom) {
                        SourceNativeBindingRole::None => enum_binding_role,
                        role => role,
                    };
                    variants.push(SourceEnumVariantRecord {
                        name: v.name.clone(),
                        selector: sel,
                        range: v.range,
                        binding_role: v_binding_role,
                    });
                }

                declarations.push(SourceDeclarationRecord::Enum(SourceEnumRecord {
                    declaration_id: enum_decl,
                    name: data_def.name.clone(),
                    variants,
                    members: Vec::new(),
                    range: data_def.range,
                    doc_comment: None,
                    binding_role: enum_binding_role,
                }));
            }
            _ => {}
        }
    }
    declarations
}

pub fn source_native_binding_role(attributes: &[phalcom_ast::ast::Attribute], implementation: ImplementationKind) -> SourceNativeBindingRole {
    if attributes
        .iter()
        .any(|attribute| matches!(attribute.name.as_str(), "wrapsNative" | "wrap_native" | "nativeWrapper"))
    {
        SourceNativeBindingRole::WrapperOverNative
    } else if implementation == ImplementationKind::Native {
        SourceNativeBindingRole::DeclarationImplementation
    } else {
        SourceNativeBindingRole::None
    }
}
