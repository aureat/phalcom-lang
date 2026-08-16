//! Bounded semantic invalidation primitives.

use super::ids::{CallableId, DispatchSide, ModuleId};
use super::snapshot::FileSourceSnapshot;
use super::surface::{FieldKind, MemberKind, MemberVisibility, ModuleSurface};
use phalcom_ast::ast::{Program, Statement};
use std::collections::{BTreeMap, BTreeSet};

/// Classification used to choose the narrowest source invalidation frontier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceChangeKind {
    /// Executable body or initializer changed without changing declarations.
    BodyOnly,
    /// Import declarations changed.
    ImportSurface,
    /// Classes, members, fields, or callable signatures changed.
    DeclarationSurface,
    /// A source module was added or removed.
    FileAddedRemoved,
    /// The logical core module changed.
    CoreSurface,
}

/// Typed declaration identity used to classify source edits without allowing
/// source ranges or debug formatting to affect semantic equality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MemberDeclarationFingerprint {
    pub selector: String,
    pub side: DispatchSide,
    pub kind: MemberKind,
    pub visibility: MemberVisibility,
    pub constructor: bool,
    pub native_return: Option<phalcom_native_surface::NativeReturnShape>,
    pub params: Vec<ParameterDeclarationFingerprint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParameterDeclarationFingerprint {
    pub label: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FieldDeclarationFingerprint {
    name: String,
    side: DispatchSide,
    kind: FieldKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClassDeclarationFingerprint {
    superclass: Option<super::ids::ClassId>,
    members: Vec<MemberDeclarationFingerprint>,
    fields: Vec<FieldDeclarationFingerprint>,
}

type DeclarationFingerprint = BTreeMap<super::ids::ClassId, ClassDeclarationFingerprint>;

/// Complete source replacement delta used to seed semantic recomputation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceDelta {
    pub kind: SourceChangeKind,
    pub changed_callables: BTreeSet<CallableId>,
    pub top_level_changed: bool,
}

/// Classifies source replacement and records exact body-local callable seeds.
pub(crate) fn classify_source_delta(module: &ModuleId, old: Option<&FileSourceSnapshot>, new: Option<&FileSourceSnapshot>) -> SourceDelta {
    let Some(new) = new else {
        return SourceDelta {
            kind: SourceChangeKind::FileAddedRemoved,
            changed_callables: BTreeSet::new(),
            top_level_changed: true,
        };
    };
    let Some(old) = old else {
        return SourceDelta {
            kind: SourceChangeKind::FileAddedRemoved,
            changed_callables: new.callables.keys().cloned().collect(),
            top_level_changed: true,
        };
    };
    let kind = if imports_of(&old.program) != imports_of(&new.program) {
        SourceChangeKind::ImportSurface
    } else if declaration_fingerprint(&old.surface) != declaration_fingerprint(&new.surface) {
        if module.as_str() == super::ids::CORE_MODULE_URI {
            SourceChangeKind::CoreSurface
        } else {
            SourceChangeKind::DeclarationSurface
        }
    } else if module.as_str() == super::ids::CORE_MODULE_URI {
        SourceChangeKind::BodyOnly
    } else {
        SourceChangeKind::BodyOnly
    };

    let changed_callables = if kind == SourceChangeKind::BodyOnly {
        old.callables
            .keys()
            .filter(|callable| {
                let Some(old_member) = old.surface.member_by_id(callable) else { return false };
                let Some(new_member) = new.surface.member_by_id(callable) else { return false };
                if old_member.kind == MemberKind::Field || new_member.kind == MemberKind::Field {
                    return false;
                }
                callable_source(old, old_member) != callable_source(new, new_member)
            })
            .cloned()
            .collect()
    } else {
        BTreeSet::new()
    };

    SourceDelta {
        kind,
        changed_callables,
        top_level_changed: top_level_source(old) != top_level_source(new),
    }
}

/// Classifies a source replacement without considering body text or source
/// ranges as declaration-surface changes.
pub fn classify_source_change(module: &ModuleId, old: Option<&FileSourceSnapshot>, new: Option<&FileSourceSnapshot>) -> SourceChangeKind {
    classify_source_delta(module, old, new).kind
}

fn imports_of(program: &Program) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for dep in &program.preamble.dependencies {
        match dep {
            phalcom_ast::ast::DependencyDecl::Import(imp) => match imp {
                phalcom_ast::ast::ImportDecl::Module(m) => {
                    let binding = if let Some(alias) = &m.alias {
                        alias.name.clone()
                    } else if m.path.segments.is_empty() {
                        match &m.path.root {
                            phalcom_ast::ast::ImportRoot::Absolute(seg) => seg.name.clone(),
                            phalcom_ast::ast::ImportRoot::Relative { .. } => String::new(),
                        }
                    } else {
                        m.path.segments.last().unwrap().name.clone()
                    };
                    out.push((m.path.to_string(), binding));
                }
                phalcom_ast::ast::ImportDecl::Selective(s) => {
                    let path_str = s.path.to_string();
                    for item in &s.items {
                        let binding = if let Some(alias) = &item.alias {
                            alias.name.clone()
                        } else {
                            item.name.clone()
                        };
                        out.push((path_str.clone(), binding));
                    }
                }
            },
            phalcom_ast::ast::DependencyDecl::ReExport(r) => {
                let path_str = r.path.to_string();
                for item in &r.items {
                    out.push((path_str.clone(), item.local_or_remote_name.clone()));
                }
            }
            phalcom_ast::ast::DependencyDecl::Expose(_) => {}
        }
    }
    out
}

fn declaration_fingerprint(surface: &ModuleSurface) -> DeclarationFingerprint {
    surface
        .classes
        .iter()
        .map(|(class_id, class)| {
            let members = class
                .all_members()
                .map(|member| MemberDeclarationFingerprint {
                    selector: member.callable.selector.clone(),
                    side: member.side,
                    kind: member.kind,
                    visibility: member.visibility,
                    constructor: member.is_constructor,
                    native_return: member.native_return,
                    params: member
                        .params
                        .iter()
                        .map(|param| ParameterDeclarationFingerprint {
                            label: param.label.clone(),
                            name: param.name.clone(),
                        })
                        .collect(),
                })
                .collect();
            let fields = class
                .fields
                .values()
                .flat_map(|sides| [sides.instance.as_ref(), sides.class.as_ref()].into_iter().flatten())
                .map(|field| FieldDeclarationFingerprint {
                    name: field.name.clone(),
                    side: if field.is_class_side { DispatchSide::Class } else { DispatchSide::Instance },
                    kind: field.kind,
                })
                .collect();
            (
                class_id.clone(),
                ClassDeclarationFingerprint {
                    superclass: class.superclass.clone(),
                    members,
                    fields,
                },
            )
        })
        .collect()
}

fn callable_source(source: &FileSourceSnapshot, member: &super::surface::MemberSurface) -> String {
    source
        .text
        .get(member.source_range.start..member.source_range.end)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{:?}", super::source::member_body(source, member.ast)))
}

fn top_level_source(source: &FileSourceSnapshot) -> Vec<String> {
    source
        .program
        .statements
        .iter()
        .filter_map(|statement| {
            if matches!(statement, Statement::Class(_) | Statement::Export(_)) {
                return None;
            }
            let range = statement_range(statement);
            source
                .text
                .get(range.start..range.end)
                .map(str::to_owned)
                .or_else(|| Some(format!("{statement:?}")))
        })
        .collect()
}

fn statement_range(statement: &Statement) -> phalcom_common::range::SourceRange {
    match statement {
        Statement::Class(class) => class.range,
        Statement::Let(binding) => binding.range,
        Statement::Return(returned) => returned.range,
        Statement::Expr { range, .. } | Statement::Break { range } | Statement::Continue { range } | Statement::Throw { range, .. } => *range,
        Statement::For(for_statement) => for_statement.range,
        Statement::Export(export_decl) => export_decl.range,
    }
}

/// A deterministic set of modules awaiting recomputation.
#[derive(Clone, Debug, Default)]
pub struct InvalidationQueue {
    pending: BTreeSet<ModuleId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn source(module: ModuleId, text: &str) -> FileSourceSnapshot {
        let program = Arc::new(phalcom_ast::parser::parse(text, 0).program);
        let surface = super::super::surface::build_module_surface(module.clone(), &program);
        let scopes = super::super::scope::build_scope_graph(module.clone(), &program);
        let callables = surface.callable_index();
        FileSourceSnapshot {
            module,
            text: Arc::from(text),
            program,
            surface,
            scopes,
            callables,
        }
    }

    #[test]
    fn body_edit_preserves_surface_fingerprint() {
        let module = ModuleId::new("file:///tmp/body.ph");
        let old = source(module.clone(), "class A { run() { 1 } }\n");
        let new = source(module.clone(), "class A { run() { 2 } }\n");
        let delta = classify_source_delta(&module, Some(&old), Some(&new));
        assert_eq!(delta.kind, SourceChangeKind::BodyOnly);
        assert_eq!(delta.changed_callables.len(), 1);
    }

    #[test]
    fn body_delta_ignores_range_shifts_for_unchanged_bodies() {
        let module = ModuleId::new("file:///tmp/body-shift.ph");
        let old_text = "class A {\n  untouched() { 1 }\n\n  changed() { 2 }\n}\n";
        let new_text = "class A {\n  // moved\n  untouched() { 1 }\n\n  changed() { 3 }\n}\n";
        let old = source(module.clone(), old_text);
        let new = source(module.clone(), new_text);
        let delta = classify_source_delta(&module, Some(&old), Some(&new));
        let changed = delta.changed_callables;
        assert!(changed.iter().any(|id| id.selector == "changed()"));
        assert!(!changed.iter().any(|id| id.selector == "untouched()"));
    }

    #[test]
    fn top_level_executable_change_is_separate_from_callable_body_delta() {
        let module = ModuleId::new("file:///tmp/top-level.ph");
        let old = source(module.clone(), "class A { run() { 1 } }\nlet value = 1\n");
        let new = source(module.clone(), "class A { run() { 1 } }\nlet value = 2\n");
        let delta = classify_source_delta(&module, Some(&old), Some(&new));
        assert_eq!(delta.kind, SourceChangeKind::BodyOnly);
        assert!(delta.changed_callables.is_empty());
        assert!(delta.top_level_changed);
    }

    #[test]
    fn import_and_declaration_edits_have_distinct_kinds() {
        let module = ModuleId::new("file:///tmp/change.ph");
        let old = source(module.clone(), "import .one as One\nclass A { run() {} }\n");
        let import_edit = source(module.clone(), "import .two as Two\nclass A { run() {} }\n");
        let declaration_edit = source(module.clone(), "import .one as One\nclass A { run(_) {} }\n");
        assert_eq!(classify_source_change(&module, Some(&old), Some(&import_edit)), SourceChangeKind::ImportSurface);
        assert_eq!(
            classify_source_change(&module, Some(&old), Some(&declaration_edit)),
            SourceChangeKind::DeclarationSurface
        );
    }

    #[test]
    fn add_remove_and_core_keep_dedicated_kinds() {
        let module = ModuleId::new("file:///tmp/change.ph");
        let source_snapshot = source(module.clone(), "class A {}\n");
        assert_eq!(
            classify_source_change(&module, None, Some(&source_snapshot)),
            SourceChangeKind::FileAddedRemoved
        );
        assert_eq!(
            classify_source_change(&module, Some(&source_snapshot), None),
            SourceChangeKind::FileAddedRemoved
        );
        let core = ModuleId::new(super::super::ids::CORE_MODULE_URI);
        let core_old = source(core.clone(), "class A { run() { 1 } }\n");
        let core_body = source(core.clone(), "class A { run() { 2 } }\n");
        assert_eq!(classify_source_change(&core, Some(&core_old), Some(&core_body)), SourceChangeKind::BodyOnly);
        let core_declaration = source(core.clone(), "class B { run() { 1 } }\n");
        assert_eq!(
            classify_source_change(&core, Some(&core_old), Some(&core_declaration)),
            SourceChangeKind::CoreSurface
        );
    }
}

impl InvalidationQueue {
    /// Adds one changed or dependent module.
    pub fn push(&mut self, module: ModuleId) {
        self.pending.insert(module);
    }

    /// Drains queued modules in module-id order.
    pub fn drain(&mut self) -> impl Iterator<Item = ModuleId> + '_ {
        std::mem::take(&mut self.pending).into_iter()
    }

    /// Returns whether no module is pending.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}
