//! Compiler-owned, protocol-neutral semantic queries for editor features.

use crate::advisory::ValueShape;
use crate::advisory::advisory_shape_from_formal;
use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId};
use crate::snapshot::SemanticSnapshot;
use crate::source_index::{OccurrenceHint, OccurrenceRole, SourceBindingInfo};
use crate::surface::MemberVisibility;
use crate::types::evidence::TypeKnowledge;
use crate::types::relation::TypeHierarchy;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use std::collections::BTreeSet;
use std::sync::Arc;

/// Dispatch side used while resolving an editor receiver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiverMode {
    Instance,
    Class,
}

/// One canonical declaration alternative for a possibly-union receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverAlternative {
    pub declaration: DeclarationId,
    pub mode: ReceiverMode,
}

/// Conservative receiver result. Unknown or unsupported shapes produce no
/// alternatives instead of guessed members.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedReceiver {
    pub alternatives: Arc<[ReceiverAlternative]>,
}

/// Lexical access context for canonical member visibility checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessContext {
    pub enclosing_declaration: Option<DeclarationId>,
    pub enclosing_callable: Option<CallableId>,
}

/// Canonical member target returned to protocol adapters.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EditorMemberTarget {
    Callable(CallableId),
    Field(FieldId),
}

/// One visible canonical member, without protocol-specific labels or ranges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorMember {
    pub target: EditorMemberTarget,
    pub owner: DeclarationId,
    pub visibility: MemberVisibility,
}

/// One visible lexical symbol and its canonical target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibleSymbol {
    pub name: Box<str>,
    pub declaration_site: SourceSiteId,
    pub target: SemanticTargetId,
}

/// Read-only editor query facade over one immutable semantic snapshot.
#[derive(Clone, Copy, Debug)]
pub struct EditorSemanticQuery<'a> {
    snapshot: &'a SemanticSnapshot,
}

impl<'a> EditorSemanticQuery<'a> {
    pub(crate) fn new(snapshot: &'a SemanticSnapshot) -> Self {
        Self { snapshot }
    }

    /// Returns exact canonical target at a source position.
    pub fn target_at(&self, module: &ModuleId, offset: usize) -> Option<SemanticTargetId> {
        let occurrence = self.snapshot.occurrence_at(module, offset)?;
        occurrence
            .target
            .cloned()
            .or_else(|| self.snapshot.advisory.target(&occurrence.occurrence.site).map(|target| target.target.clone()))
    }

    /// Returns declaration sites for one canonical target.
    pub fn definition_sites(&self, target: &SemanticTargetId) -> Vec<SourceSiteId> {
        self.sites_for_target(target, true)
    }

    /// Returns non-declaration reference sites for one canonical target.
    pub fn reference_sites(&self, target: &SemanticTargetId) -> Vec<SourceSiteId> {
        self.sites_for_target(target, false)
    }

    fn sites_for_target(&self, target: &SemanticTargetId, definitions: bool) -> Vec<SourceSiteId> {
        let mut sites = self
            .snapshot
            .occurrences_for_target(target)
            .into_iter()
            .flatten()
            .filter(|site| {
                self.snapshot
                    .source_index
                    .module_for_site(site)
                    .and_then(|module| module.occurrences.occurrence_for_site(site))
                    .is_some_and(|occurrence| (occurrence.role == OccurrenceRole::Declaration) == definitions)
            })
            .cloned()
            .collect::<Vec<_>>();
        sites.sort();
        sites.dedup();
        sites
    }

    /// Returns lexical access context from the canonical source owner.
    pub fn access_context_at(&self, module: &ModuleId, offset: usize) -> AccessContext {
        let owner = self.snapshot.source_site_at(module, offset).map(|site| &site.id.owner);
        match owner {
            Some(SourceOwner::Callable(callable)) => AccessContext {
                enclosing_declaration: Some(callable.owner.clone()),
                enclosing_callable: Some(callable.clone()),
            },
            _ => AccessContext {
                enclosing_declaration: None,
                enclosing_callable: None,
            },
        }
    }

    /// Resolves a receiver from exact formal/advisory products at a source
    /// range. No request-time AST inference or name guessing is performed.
    pub fn resolve_receiver_at(&self, module: &ModuleId, range: SourceRange) -> Option<ResolvedReceiver> {
        let occurrence_site = self.snapshot.occurrence_at(module, range.start).map(|view| view.occurrence.site.clone());
        let expression_site = self
            .snapshot
            .source_index
            .expression_site_at(module, range.end.saturating_sub(1))
            .or_else(|| self.snapshot.source_index.expression_site_at(module, range.start))
            .map(|site| site.id.clone());
        let site = expression_site.clone().or(occurrence_site.clone());
        if let Some(receiver) = self.receiver_for_source_range(module, range) {
            return Some(receiver);
        }
        let range_target = self
            .snapshot
            .source_index
            .module(module)?
            .occurrences
            .all()
            .iter()
            .filter(|occurrence| range.start <= occurrence.range.start && occurrence.range.end <= range.end)
            .filter_map(|occurrence| {
                self.snapshot
                    .source_index
                    .target_for(&occurrence.site)
                    .cloned()
                    .or_else(|| self.snapshot.advisory.target(&occurrence.site).map(|target| target.target.clone()))
            })
            .find(|target| matches!(target, SemanticTargetId::Declaration(_)));
        let target = range_target.or_else(|| {
            site.as_ref()
                .and_then(|site| self.snapshot.source_index.target_for(site))
                .cloned()
                .or_else(|| occurrence_site.as_ref().and_then(|site| self.snapshot.source_index.target_for(site)).cloned())
        });
        let constructor_receiver = target.as_ref().and_then(|target| {
            let SemanticTargetId::Declaration(declaration) = target else { return None };
            let source = self.snapshot.source_index.module(module)?;
            let has_constructor_call = source.occurrences.all().iter().any(|occurrence| {
                range.start <= occurrence.range.start
                    && occurrence.range.end <= range.end
                    && occurrence.role == OccurrenceRole::Call
                    && matches!(&occurrence.hint, Some(OccurrenceHint::Name(name)) if name.as_ref() == "new")
            });
            has_constructor_call.then(|| ValueShape::Instance(declaration.clone()))
        });
        let target_site = target.as_ref().and_then(|target| match target {
            SemanticTargetId::Binding(binding) => Some(binding),
            _ => None,
        });
        let advisory_shape_for_site = |site: &SourceSiteId| {
            self.snapshot
                .advisory_fact(site)
                .filter(|fact| !matches!(fact.shape, ValueShape::Unknown))
                .map(|fact| fact.shape.clone())
        };
        let shape = constructor_receiver.or_else(|| {
            expression_site
                .as_ref()
                .and_then(|site| self.formal_shape_for_site(site))
                .or_else(|| target_site.and_then(|site| self.formal_shape_for_site(site)))
                .or_else(|| self.formal_shape_at(module, range.start))
                .or_else(|| expression_site.as_ref().and_then(advisory_shape_for_site))
                .or_else(|| target_site.and_then(advisory_shape_for_site))
                .or_else(|| occurrence_site.as_ref().and_then(advisory_shape_for_site))
                .or_else(|| match target.as_ref() {
                    Some(SemanticTargetId::Module(module)) => Some(ValueShape::Module(module.clone())),
                    Some(SemanticTargetId::Declaration(declaration)) => Some(ValueShape::ClassObject(declaration.clone())),
                    _ => None,
                })
        });
        let mut alternatives = Vec::new();
        if let Some(ref shape) = shape {
            collect_receiver_alternatives(&shape, &mut alternatives);
        }
        if alternatives.is_empty()
            && let Some(SemanticTargetId::Declaration(declaration)) = target.as_ref()
        {
            alternatives.push(ReceiverAlternative {
                declaration: declaration.clone(),
                mode: ReceiverMode::Class,
            });
        }
        alternatives.sort_by(|left, right| (&left.declaration, left.mode as u8).cmp(&(&right.declaration, right.mode as u8)));
        alternatives.dedup();
        (!alternatives.is_empty()).then(|| ResolvedReceiver {
            alternatives: Arc::from(alternatives.into_boxed_slice()),
        })
    }

    fn receiver_for_source_site(&self, module: &ModuleId, site: &Option<SourceSiteId>) -> Option<ResolvedReceiver> {
        let site = site.as_ref()?;
        let kind = self.snapshot.source_index.module(module)?.structure.receiver_kind(site)?;
        let SourceOwner::Callable(callable) = &site.owner else {
            return None;
        };
        let declaration = match kind {
            crate::source_index::SourceReceiverKind::SelfValue => callable.owner.clone(),
            crate::source_index::SourceReceiverKind::SuperValue => self.snapshot.hierarchy.superclass(&callable.owner).cloned()?,
        };
        Some(ResolvedReceiver {
            alternatives: Arc::from([ReceiverAlternative {
                declaration,
                mode: ReceiverMode::Instance,
            }]),
        })
    }

    fn receiver_for_source_range(&self, module: &ModuleId, range: SourceRange) -> Option<ResolvedReceiver> {
        let source = self.snapshot.source_index.module(module)?;
        for (site, kind) in &source.structure.receiver_kinds {
            let receiver_site = source.structure.site(site)?;
            if receiver_site.range.start < range.start || receiver_site.range.end > range.end {
                continue;
            }
            let callable = source
                .structure
                .callable_body_ranges
                .iter()
                .filter(|(_, range)| range.contains(receiver_site.range.start))
                .min_by_key(|(_, range)| range.len())
                .map(|(callable, _)| callable.clone());
            let Some(callable) = callable else { continue };
            let declaration = match kind {
                crate::source_index::SourceReceiverKind::SelfValue => callable.owner,
                crate::source_index::SourceReceiverKind::SuperValue => self.snapshot.hierarchy.superclass(&callable.owner).cloned()?,
            };
            return Some(ResolvedReceiver {
                alternatives: Arc::from([ReceiverAlternative {
                    declaration,
                    mode: ReceiverMode::Instance,
                }]),
            });
        }
        source
            .occurrences
            .all()
            .iter()
            .filter(|occurrence| range.start <= occurrence.range.start && occurrence.range.end <= range.end)
            .find_map(|occurrence| self.receiver_for_source_site(module, &Some(occurrence.site.clone())))
    }

    fn formal_shape_at(&self, module: &ModuleId, offset: usize) -> Option<ValueShape> {
        let fact = self.snapshot.formal_fact_at(module, offset)?;
        self.formal_shape_for_fact(&fact.fact)
    }

    fn formal_shape_for_site(&self, site: &SourceSiteId) -> Option<ValueShape> {
        let knowledge = self
            .snapshot
            .formal_binding_at(site)
            .map(|state| state.current.clone())
            .or_else(|| self.snapshot.formal_expression_at(site).map(|expression| expression.knowledge.clone()))?;
        (!matches!(&knowledge, TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_))).then(|| advisory_shape_from_formal(&self.snapshot.store, &knowledge))
    }

    fn formal_shape_for_fact(&self, fact: &crate::presentation::FormalFactRef) -> Option<ValueShape> {
        let knowledge = match fact {
            crate::presentation::FormalFactRef::Expression { callable, expression } => {
                self.snapshot.formal_expression(callable, *expression)?.knowledge.clone()
            }
            crate::presentation::FormalFactRef::Binding { callable, binding } => self.snapshot.formal_binding(callable, *binding)?.current.clone(),
            crate::presentation::FormalFactRef::Callable(_) => return None,
        };
        (!matches!(&knowledge, TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_))).then(|| advisory_shape_from_formal(&self.snapshot.store, &knowledge))
    }

    /// Returns all visible members across receiver alternatives, including
    /// inherited members, with duplicate canonical targets removed.
    pub fn members_for_receiver(&self, receiver: &ResolvedReceiver, access: &AccessContext) -> Vec<EditorMember> {
        let mut members = Vec::new();
        for alternative in receiver.alternatives.iter() {
            let side = match alternative.mode {
                ReceiverMode::Instance => crate::identity::DispatchSide::Instance,
                ReceiverMode::Class => crate::identity::DispatchSide::Class,
            };
            for dispatch_owner in self
                .snapshot
                .dispatch
                .dispatch_owners(self.snapshot.hierarchy.as_ref(), &alternative.declaration, side)
            {
                let current = dispatch_owner.declaration;
                if let Some(surface) = self.snapshot.surfaces.get(&current) {
                    let member_surface = surface.surface(dispatch_owner.side);
                    for (selector, callable) in &member_surface.callables_by_selector {
                        let visibility = member_surface.callable_visibility.get(selector).copied().unwrap_or_default();
                        if is_visible(self.snapshot.hierarchy.as_ref(), &current, visibility, access) {
                            members.push(EditorMember {
                                target: EditorMemberTarget::Callable(callable.clone()),
                                owner: current.clone(),
                                visibility,
                            });
                        }
                    }
                    for (name, field) in &member_surface.fields_by_name {
                        let visibility = member_surface.field_visibility.get(name).copied().unwrap_or_default();
                        if is_visible(self.snapshot.hierarchy.as_ref(), &current, visibility, access) {
                            members.push(EditorMember {
                                target: EditorMemberTarget::Field(field.clone()),
                                owner: current.clone(),
                                visibility,
                            });
                        }
                    }
                }
            }
        }
        members.sort_by(|left, right| member_sort_key(left).cmp(&member_sort_key(right)));
        members.dedup_by(|left, right| left.target == right.target);
        members
    }

    /// Returns canonical members matching one exact selector.
    pub fn resolve_member(&self, receiver: &ResolvedReceiver, selector: &Selector, access: &AccessContext) -> Vec<EditorMember> {
        self.members_for_receiver(receiver, access)
            .into_iter()
            .filter(|member| match &member.target {
                EditorMemberTarget::Callable(callable) => &callable.selector == selector,
                EditorMemberTarget::Field(field) => field.name.as_ref() == selector.encode().as_str(),
            })
            .collect()
    }

    /// Returns compiler-owned lexical bindings visible at a source position.
    pub fn visible_symbols_at(&self, module: &ModuleId, offset: usize) -> Vec<VisibleSymbol> {
        let Some(index) = self.snapshot.source_index.module(module) else {
            return Vec::new();
        };
        let mut symbols = index
            .structure
            .visible_bindings_at(offset)
            .into_iter()
            .map(|binding| visible_binding(index.structure.as_ref(), binding))
            .collect::<Vec<_>>();
        let binding_names = symbols.iter().map(|symbol| symbol.name.clone()).collect::<BTreeSet<_>>();
        symbols.extend(
            index
                .structure
                .declaration_sources
                .values()
                .filter(|declaration| !binding_names.contains(&declaration.name))
                .map(|declaration| VisibleSymbol {
                    name: declaration.name.clone(),
                    declaration_site: declaration.declaration_site.clone(),
                    target: SemanticTargetId::Declaration(declaration.id.clone()),
                }),
        );
        symbols.sort_by(|left, right| left.name.cmp(&right.name));
        symbols
    }
}

fn collect_receiver_alternatives(shape: &ValueShape, alternatives: &mut Vec<ReceiverAlternative>) {
    match shape {
        ValueShape::Instance(declaration) => alternatives.push(ReceiverAlternative {
            declaration: declaration.clone(),
            mode: ReceiverMode::Instance,
        }),
        ValueShape::ClassObject(declaration) => alternatives.push(ReceiverAlternative {
            declaration: declaration.clone(),
            mode: ReceiverMode::Class,
        }),
        ValueShape::Union(shapes) => shapes.iter().for_each(|shape| collect_receiver_alternatives(shape, alternatives)),
        _ => {}
    }
}

fn is_visible(hierarchy: &dyn TypeHierarchy, owner: &DeclarationId, visibility: MemberVisibility, access: &AccessContext) -> bool {
    match visibility {
        MemberVisibility::Public => true,
        MemberVisibility::Private => access.enclosing_declaration.as_ref() == Some(owner),
        MemberVisibility::Protected => access
            .enclosing_declaration
            .as_ref()
            .is_some_and(|context| hierarchy.is_subclass(context, owner)),
        MemberVisibility::Internal => access.enclosing_declaration.as_ref() == Some(owner),
    }
}

fn member_sort_key(member: &EditorMember) -> (DeclarationId, String, u8) {
    match &member.target {
        EditorMemberTarget::Callable(callable) => (member.owner.clone(), callable.selector.encode(), 0),
        EditorMemberTarget::Field(field) => (member.owner.clone(), field.name.to_string(), 1),
    }
}

fn visible_binding(index: &crate::source_index::SourceScopeIndex, binding: &SourceBindingInfo) -> VisibleSymbol {
    let target = index
        .target_for(&binding.declaration_site)
        .cloned()
        .unwrap_or_else(|| SemanticTargetId::Binding(binding.declaration_site.clone()));
    VisibleSymbol {
        name: binding.name.clone(),
        declaration_site: binding.declaration_site.clone(),
        target,
    }
}
