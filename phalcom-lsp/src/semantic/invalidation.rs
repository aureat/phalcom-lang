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
        for ((selector, side), member) in &class.members_by_side {
            let params = member
                .params
                .iter()
                .map(|param| format!("{}:{:?}", param.name, param.label))
                .collect::<Vec<_>>();
            fingerprint.push(format!(
                "member:{selector}:{side:?}:{:?}:{:?}:{:?}:{:?}:{params:?}",
                member.kind, member.visibility, member.is_constructor, member.native_return
            ));
        }
        for (name, field) in &class.fields {
            fingerprint.push(format!("field:{name}:{:?}:{:?}", field.kind, field.is_class_side));
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
