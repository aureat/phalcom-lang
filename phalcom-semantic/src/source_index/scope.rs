//! Compiler-owned lexical scopes and source binding identity.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::{DeclarationId, ModuleId, SemanticTargetId, SourceSiteId};
use crate::source_index::site::SourceSite;
use phalcom_common::range::SourceRange;

/// Dense lexical scope identity within one source-index snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceScopeId(pub u32);

/// Compiler-owned source binding category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceBindingKind {
    TopLevelLet,
    TopLevelConst,
    LocalLet,
    LocalConst,
    MethodParameter,
    SetterParameter,
    IndexParameter,
    ClosureParameter,
    ForBinding,
    Destructure,
    Import,
}

/// Snapshot-owned metadata for one lexical binding declaration site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceBindingInfo {
    pub declaration_site: SourceSiteId,
    pub scope: SourceScopeId,
    pub name: Box<str>,
    pub kind: SourceBindingKind,
    pub declaration_range: SourceRange,
    pub mutable: bool,
    pub redeclaration_of: Option<SourceSiteId>,
}

/// One lexical scope and its first-binding name map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceScope {
    pub id: SourceScopeId,
    pub parent: Option<SourceScopeId>,
    pub range: SourceRange,
    pub bindings: BTreeMap<Box<str>, SourceSiteId>,
}

/// Result of source-order lexical name resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceNameResolution {
    Binding(SourceSiteId),
    Target(SemanticTargetId),
    ImplicitSelf,
    Unresolved,
}

/// Immutable compiler-owned lexical source index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceScopeIndex {
    pub module: ModuleId,
    pub root: SourceScopeId,
    pub scopes: BTreeMap<SourceScopeId, SourceScope>,
    pub bindings: BTreeMap<SourceSiteId, SourceBindingInfo>,
    pub sites: BTreeMap<SourceSiteId, SourceSite>,
    pub targets: BTreeMap<SourceSiteId, SemanticTargetId>,
    scope_order: Vec<SourceScopeId>,
    scope_max_end_prefix: Vec<usize>,
    declarations: BTreeMap<(usize, usize), SourceSiteId>,
    classes: BTreeMap<Box<str>, DeclarationId>,
    modules: BTreeMap<Box<str>, ModuleId>,
}

impl SourceScopeIndex {
    pub(crate) fn new(module: ModuleId, root_range: SourceRange) -> Self {
        let root = SourceScopeId(0);
        let mut scopes = BTreeMap::new();
        scopes.insert(
            root,
            SourceScope {
                id: root,
                parent: None,
                range: root_range,
                bindings: BTreeMap::new(),
            },
        );
        Self {
            module,
            root,
            scopes,
            bindings: BTreeMap::new(),
            sites: BTreeMap::new(),
            targets: BTreeMap::new(),
            scope_order: vec![root],
            scope_max_end_prefix: vec![root_range.end],
            declarations: BTreeMap::new(),
            classes: BTreeMap::new(),
            modules: BTreeMap::new(),
        }
    }

    pub(crate) fn add_scope(&mut self, id: SourceScopeId, parent: SourceScopeId, range: SourceRange) {
        self.scopes.insert(
            id,
            SourceScope {
                id,
                parent: Some(parent),
                range,
                bindings: BTreeMap::new(),
            },
        );
    }

    pub(crate) fn finish_scope_order(&mut self) {
        self.scope_order = self.scopes.keys().copied().collect();
        self.scope_order.sort_by_key(|id| {
            let scope = &self.scopes[id];
            (scope.range.start, scope.range.end.saturating_sub(scope.range.start), *id)
        });
        self.scope_max_end_prefix.clear();
        let mut max_end = 0;
        for id in &self.scope_order {
            max_end = max_end.max(self.scopes[id].range.end);
            self.scope_max_end_prefix.push(max_end);
        }
    }

    pub(crate) fn register_class(&mut self, name: impl Into<Box<str>>, declaration: DeclarationId) {
        self.classes.entry(name.into()).or_insert(declaration);
    }

    pub(crate) fn register_module(&mut self, name: impl Into<Box<str>>, module: ModuleId) {
        self.modules.entry(name.into()).or_insert(module);
    }

    pub(crate) fn register_site(&mut self, site: SourceSite) {
        self.sites.insert(site.id.clone(), site);
    }

    pub(crate) fn register_binding(&mut self, info: SourceBindingInfo) {
        self.declarations
            .entry((info.declaration_range.start, info.declaration_range.end))
            .or_insert_with(|| info.declaration_site.clone());
        self.bindings.insert(info.declaration_site.clone(), info);
    }

    pub(crate) fn register_target(&mut self, site: SourceSiteId, target: SemanticTargetId) {
        self.targets.insert(site, target);
    }

    /// Finds the innermost lexical scope containing `offset`.
    pub fn scope_at(&self, offset: usize) -> SourceScopeId {
        let mut low = 0;
        let mut high = self.scope_order.len();
        while low < high {
            let middle = (low + high) / 2;
            if self.scopes[&self.scope_order[middle]].range.start <= offset {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        let mut best = None;
        let mut index = low;
        while index > 0 {
            index -= 1;
            if self.scope_max_end_prefix[index] <= offset {
                break;
            }
            let scope = &self.scopes[&self.scope_order[index]];
            if scope.range.contains(offset) && best.is_none_or(|best: &SourceScope| scope.range.len() < best.range.len()) {
                best = Some(scope);
            }
        }
        best.map_or(self.root, |scope| scope.id)
    }

    /// Resolves one name according to source order and lexical parent scopes.
    pub fn resolve_name(&self, scope: SourceScopeId, name: &str, offset: usize) -> SourceNameResolution {
        let mut current = Some(scope);
        while let Some(scope_id) = current {
            let Some(scope_info) = self.scopes.get(&scope_id) else { break };
            if let Some(site) = scope_info.bindings.get(name)
                && let Some(binding) = self.bindings.get(site)
                && binding.declaration_range.start <= offset
            {
                return SourceNameResolution::Binding(site.clone());
            }
            current = scope_info.parent;
        }
        if let Some(class) = self.classes.get(name) {
            return SourceNameResolution::Target(SemanticTargetId::Declaration(class.clone()));
        }
        if let Some(module) = self.modules.get(name) {
            return SourceNameResolution::Target(SemanticTargetId::Module(module.clone()));
        }
        if name == "self" {
            return SourceNameResolution::ImplicitSelf;
        }
        SourceNameResolution::Unresolved
    }

    /// Returns the first binding recorded for an exact declaration range.
    pub fn binding_for_declaration(&self, range: SourceRange) -> Option<&SourceBindingInfo> {
        self.declarations.get(&(range.start, range.end)).and_then(|site| self.bindings.get(site))
    }

    /// Returns bindings visible at `offset`, nearest scope first, with shadowed
    /// spellings removed.
    pub fn visible_bindings_at(&self, offset: usize) -> Vec<&SourceBindingInfo> {
        let mut seen = BTreeSet::new();
        let mut visible = Vec::new();
        let mut current = Some(self.scope_at(offset));
        while let Some(scope_id) = current {
            let Some(scope) = self.scopes.get(&scope_id) else { break };
            for (name, site) in &scope.bindings {
                if !seen.insert(name.clone()) {
                    continue;
                }
                if let Some(binding) = self.bindings.get(site)
                    && binding.declaration_range.start <= offset
                {
                    visible.push(binding);
                }
            }
            current = scope.parent;
        }
        visible
    }

    /// Returns the immutable source site for an identity.
    pub fn site(&self, site: &SourceSiteId) -> Option<&SourceSite> {
        self.sites.get(site)
    }

    /// Returns the exact canonical target attached to a source site, if any.
    pub fn target_for(&self, site: &SourceSiteId) -> Option<&SemanticTargetId> {
        self.targets.get(site)
    }
}
