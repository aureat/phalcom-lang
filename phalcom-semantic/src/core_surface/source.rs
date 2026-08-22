//! Source core declaration surface extraction.

use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use phalcom_ast::ast::{ClassMember, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMemberRecord {
    pub selector: Selector,
    pub selector_raw: String,
    pub side: DispatchSide,
    pub is_getter: bool,
    pub is_setter: bool,
    pub range: SourceRange,
    pub doc_comment: Option<String>,
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

/// Extracts source declaration records from a parsed Program.
pub fn extract_source_surface(module_id: &ModuleId, program: &Program) -> Vec<SourceClassRecord> {
    let mut classes = Vec::new();
    for stmt in &program.statements {
        if let Statement::Class(class_def) = stmt {
            let class_decl = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
            let superclass = class_def.superclass_ref().map(|s| s.root.clone());
            let mut members = Vec::new();

            for member in &class_def.members {
                match member {
                    ClassMember::Method(m) => {
                        let mut slots = Vec::new();
                        for p in &m.params {
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
                            });
                        }
                    }
                    _ => {}
                }
            }

            classes.push(SourceClassRecord {
                declaration_id: class_decl,
                name: class_def.name.clone(),
                superclass,
                members,
                range: class_def.range,
                doc_comment: None,
            });
        }
    }
    classes
}
