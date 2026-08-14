//! Bounded semantic invalidation primitives.

use std::collections::BTreeSet;

use super::ids::ModuleId;
use super::snapshot::FileSourceSnapshot;
use phalcom_ast::ast::{Program, Statement};

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

/// Classifies a source replacement without considering body text or source
/// ranges as declaration-surface changes.
pub fn classify_source_change(module: &ModuleId, old: Option<&FileSourceSnapshot>, new: Option<&FileSourceSnapshot>) -> SourceChangeKind {
    if module.as_str() == super::ids::CORE_MODULE_URI {
        return SourceChangeKind::CoreSurface;
    }
    let (Some(old), Some(new)) = (old, new) else {
        return SourceChangeKind::FileAddedRemoved;
    };
    if imports_of(&old.program) != imports_of(&new.program) {
        return SourceChangeKind::ImportSurface;
    }
    if declaration_fingerprint(&old.surface) != declaration_fingerprint(&new.surface) {
        return SourceChangeKind::DeclarationSurface;
    }
    SourceChangeKind::BodyOnly
}

fn imports_of(program: &Program) -> Vec<(String, String)> {
    program
        .statements
        .iter()
        .filter_map(|statement| {
            let Statement::Import(import) = statement else { return None };
            Some((import.path.clone(), import.binding.clone()))
        })
        .collect()
}

fn declaration_fingerprint(surface: &super::surface::ModuleSurface) -> Vec<String> {
    let mut fingerprint = Vec::new();
    for (class_id, class) in &surface.classes {
        fingerprint.push(format!("class:{class_id:?}:{:?}", class.superclass));
        for member in class.all_members() {
            let params = member
                .params
                .iter()
                .map(|param| format!("{}:{:?}", param.name, param.label))
                .collect::<Vec<_>>();
            fingerprint.push(format!(
                "member:{}:{:?}:{:?}:{:?}:{:?}:{:?}:{params:?}",
                member.callable.selector, member.side, member.kind, member.visibility, member.is_constructor, member.native_return
            ));
        }
        for name in class.fields.keys() {
            for side in [super::ids::DispatchSide::Instance, super::ids::DispatchSide::Class] {
                let Some(field) = class.field(name, side) else { continue };
                fingerprint.push(format!("field:{name}:{side:?}:{:?}:{:?}", field.kind, field.is_class_side));
            }
        }
    }
    fingerprint.sort();
    fingerprint
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
        FileSourceSnapshot {
            module,
            program,
            surface,
            scopes,
        }
    }

    #[test]
    fn body_edit_preserves_surface_fingerprint() {
        let module = ModuleId::new("file:///tmp/body.ph");
        let old = source(module.clone(), "class A { run() { 1 } }\n");
        let new = source(module.clone(), "class A { run() { 2 } }\n");
        assert_eq!(classify_source_change(&module, Some(&old), Some(&new)), SourceChangeKind::BodyOnly);
    }

    #[test]
    fn import_and_declaration_edits_have_distinct_kinds() {
        let module = ModuleId::new("file:///tmp/change.ph");
        let old = source(module.clone(), "import \"./one\" as One\nclass A { run() {} }\n");
        let import_edit = source(module.clone(), "import \"./two\" as Two\nclass A { run() {} }\n");
        let declaration_edit = source(module.clone(), "import \"./one\" as One\nclass A { run(_) {} }\n");
        assert_eq!(classify_source_change(&module, Some(&old), Some(&import_edit)), SourceChangeKind::ImportSurface);
        assert_eq!(
            classify_source_change(&module, Some(&old), Some(&declaration_edit)),
            SourceChangeKind::DeclarationSurface
        );
    }

    #[test]
    fn add_remove_and_core_keep_dedicated_kinds() {
        let module = ModuleId::new("file:///tmp/change.ph");
        let source = source(module.clone(), "class A {}\n");
        assert_eq!(classify_source_change(&module, None, Some(&source)), SourceChangeKind::FileAddedRemoved);
        assert_eq!(classify_source_change(&module, Some(&source), None), SourceChangeKind::FileAddedRemoved);
        let core = ModuleId::new(super::super::ids::CORE_MODULE_URI);
        assert_eq!(classify_source_change(&core, Some(&source), Some(&source)), SourceChangeKind::CoreSurface);
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
