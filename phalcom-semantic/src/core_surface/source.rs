//! Source core declaration surface extraction.

use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use phalcom_ast::ast::{ClassMember, Program, Statement};
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
    pub range: SourceRange,
    pub doc_comment: Option<String>,
    pub binding_role: SourceNativeBindingRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceClassRecord {
    pub declaration_id: DeclarationId,
    pub name: String,
    pub superclass: Option<String>,
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
                                    range: m.range,
                                    doc_comment: None,
                                    binding_role: source_native_binding_role(&m.attributes),
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
                                    range: g.range,
                                    doc_comment: None,
                                    binding_role: source_native_binding_role(&g.attributes),
                                });
                            }
                        }
                        ClassMember::Setter(s) => {
                            if let Ok(sel) = Selector::setter(&s.name) {
                                members.push(SourceMemberRecord {
                                    selector: sel,
                                    selector_raw: format!("{}=(put)", s.name),
                                    side: if s.is_static { DispatchSide::Class } else { DispatchSide::Instance },
                                    is_getter: false,
                                    is_setter: true,
                                    range: s.range,
                                    doc_comment: None,
                                    binding_role: source_native_binding_role(&s.attributes),
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
                    members,
                    range: class_def.range,
                    doc_comment: None,
                }));
            }
            Statement::Enum(enum_def) => {
                let enum_decl = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                let enum_binding_role = source_native_binding_role(&enum_def.attributes);
                let mut variants = Vec::new();
                let mut members = Vec::new();

                for member in &enum_def.members {
                    match member {
                        phalcom_ast::ast::EnumMember::Variant(v) => {
                            let sel = phalcom_ast::selector::selector_from_variant(v);
                            let v_binding_role = match source_native_binding_role(&v.attributes) {
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
                        phalcom_ast::ast::EnumMember::Behavior(b) => {
                            match b {
                                phalcom_ast::ast::EnumBehaviorMember::Method(m) => {
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
                                            range: m.range,
                                            doc_comment: None,
                                            binding_role: match source_native_binding_role(&m.attributes) {
                                                SourceNativeBindingRole::None => enum_binding_role,
                                                role => role,
                                            },
                                        });
                                    }
                                }
                                phalcom_ast::ast::EnumBehaviorMember::Getter(g) => {
                                    if let Ok(sel) = Selector::getter(&g.name) {
                                        members.push(SourceMemberRecord {
                                            selector: sel,
                                            selector_raw: g.name.clone(),
                                            side: if g.is_static { DispatchSide::Class } else { DispatchSide::Instance },
                                            is_getter: true,
                                            is_setter: false,
                                            range: g.range,
                                            doc_comment: None,
                                            binding_role: match source_native_binding_role(&g.attributes) {
                                                SourceNativeBindingRole::None => enum_binding_role,
                                                role => role,
                                            },
                                        });
                                    }
                                }
                                phalcom_ast::ast::EnumBehaviorMember::Setter(s) => {
                                    if let Ok(sel) = Selector::setter(&s.name) {
                                        members.push(SourceMemberRecord {
                                            selector: sel,
                                            selector_raw: format!("{}=(put)", s.name),
                                            side: if s.is_static { DispatchSide::Class } else { DispatchSide::Instance },
                                            is_getter: false,
                                            is_setter: true,
                                            range: s.range,
                                            doc_comment: None,
                                            binding_role: match source_native_binding_role(&s.attributes) {
                                                SourceNativeBindingRole::None => enum_binding_role,
                                                role => role,
                                            },
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                declarations.push(SourceDeclarationRecord::Enum(SourceEnumRecord {
                    declaration_id: enum_decl,
                    name: enum_def.name.clone(),
                    variants,
                    members,
                    range: enum_def.range,
                    doc_comment: None,
                    binding_role: enum_binding_role,
                }));
            }
            _ => {}
        }
    }
    declarations
}

pub fn source_native_binding_role(attributes: &[phalcom_ast::ast::Attribute]) -> SourceNativeBindingRole {
    if attributes
        .iter()
        .any(|attribute| matches!(attribute.name.as_str(), "wrapsNative" | "wrap_native" | "nativeWrapper"))
    {
        SourceNativeBindingRole::WrapperOverNative
    } else if attributes.iter().any(|attribute| attribute.name == "native") {
        SourceNativeBindingRole::DeclarationImplementation
    } else {
        SourceNativeBindingRole::None
    }
}
