//! Compiler-owned, protocol-neutral semantic queries for editor features.

use crate::advisory::{AdvisoryFact, ValueShape, advisory_shape_from_formal};
use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId};
use crate::presentation::{CallablePresentation, FieldPresentation, FormalFactStatus, FormalPresentation, TypePresenter, present_declared_type};
use crate::snapshot::SemanticSnapshot;
use crate::source_index::{OccurrenceHint, OccurrenceRole, SourceBindingInfo, SourceBindingKind};
use crate::surface::MemberVisibility;
use crate::types::evidence::TypeKnowledge;
use crate::types::relation::TypeHierarchy;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorSlot};
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

/// Structural prefix of a call being written. This is intentionally
/// protocol-neutral: syntax recovery supplies only slots already present in
/// source, while semantic candidate selection remains compiler-owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialCallPattern {
    pub base: SelectorBase,
    pub kind: SelectorKind,
    pub written_slots: Arc<[SelectorSlot]>,
}

impl PartialCallPattern {
    pub fn from_selector_prefix(selector: &Selector) -> Self {
        Self {
            base: selector.base.clone(),
            kind: selector.kind,
            written_slots: Arc::from(selector.slots.to_vec()),
        }
    }
}

/// One visible lexical symbol and its canonical target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibleSymbol {
    pub name: Box<str>,
    pub declaration_site: SourceSiteId,
    pub target: SemanticTargetId,
}

/// Compiler-owned presentation metadata for a canonical native callable.
///
/// This deliberately projects only protocol-neutral documentation metadata;
/// clients do not need direct access to the native surface catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeCallablePresentation {
    pub documentation: Option<&'static str>,
    pub conceptual: Option<&'static str>,
}

/// Compiler-owned category for a source type hint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EditorTypeHintKind {
    Binding,
    Parameter,
    Field,
    Return,
}

/// Protocol-neutral type hint projection.
///
/// Formal and advisory channels remain separate. An advisory shape may explain
/// an otherwise unknown formal position, but it never replaces formal truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorTypeHint {
    pub kind: EditorTypeHintKind,
    pub source_range: SourceRange,
    pub insertion_offset: usize,
    pub target: Option<SemanticTargetId>,
    pub formal: Option<FormalPresentation>,
    pub advisory: Option<AdvisoryFact>,
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

    /// Returns compiler-owned protocol-neutral presentation for one callable.
    pub fn callable_presentation(&self, callable: &CallableId) -> Option<CallablePresentation> {
        let signature = self.snapshot.callable_signatures.get(callable)?;
        let source = self.snapshot.source_index.callable_source(callable);
        let presenter = TypePresenter::new(&self.snapshot.store);
        Some(CallablePresentation::from_signature(signature, source, &presenter))
    }

    /// Returns compiler-owned protocol-neutral presentation for one field.
    pub fn field_presentation(&self, field: &FieldId) -> Option<FieldPresentation> {
        let signature = self.snapshot.field_signatures.get(field)?;
        let presenter = TypePresenter::new(&self.snapshot.store);
        Some(FieldPresentation::from_signature(signature, &presenter))
    }

    /// Returns compiler-owned type hints within one source range.
    ///
    /// Annotation truth, canonical declaration/binding identity, and the formal
    /// versus advisory distinction are decided here. Protocol adapters may
    /// suppress hints for presentation preferences but must not reconstruct
    /// semantic eligibility from the AST.
    pub fn type_hints(&self, module: &ModuleId, visible: SourceRange) -> Vec<EditorTypeHint> {
        let Some(source) = self.snapshot.source_index.module(module) else {
            return Vec::new();
        };
        let presenter = TypePresenter::new(&self.snapshot.store);
        let mut hints = Vec::new();

        for binding in source.structure.bindings.values() {
            if matches!(
                binding.kind,
                SourceBindingKind::Import | SourceBindingKind::MethodParameter | SourceBindingKind::SetterParameter | SourceBindingKind::IndexParameter
            ) || binding.has_explicit_annotation
                || !ranges_overlap(binding.declaration_range, visible)
            {
                continue;
            }
            let formal = self.formal_binding_presentation(&binding.declaration_site, &presenter);
            let advisory = self.snapshot.advisory_fact(&binding.declaration_site).cloned();
            if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                continue;
            }
            hints.push(EditorTypeHint {
                kind: EditorTypeHintKind::Binding,
                source_range: binding.declaration_range,
                insertion_offset: binding.declaration_range.end,
                target: source.structure.target_for(&binding.declaration_site).cloned(),
                formal,
                advisory,
            });
        }

        for field in source.structure.field_sources.values() {
            if field.has_explicit_annotation || !ranges_overlap(field.name_range, visible) {
                continue;
            }
            let formal = self
                .snapshot
                .field_signatures
                .get(&field.id)
                .map(|signature| present_declared_type(&signature.declared_type, &presenter));
            let advisory = self.snapshot.advisory.field(&field.id).cloned();
            if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                continue;
            }
            hints.push(EditorTypeHint {
                kind: EditorTypeHintKind::Field,
                source_range: field.name_range,
                insertion_offset: field.name_range.end,
                target: Some(SemanticTargetId::Field(field.id.clone())),
                formal,
                advisory,
            });
        }

        for callable in source.structure.callable_sources.values() {
            let Some(signature) = self.snapshot.callable_signatures.get(&callable.id) else {
                continue;
            };
            let advisory = self.snapshot.advisory_callable(&callable.id);

            for parameter in signature.parameters.iter() {
                let Some(site) = callable.parameter_sites.get(&parameter.id) else {
                    continue;
                };
                let Some(binding) = source.structure.bindings.get(site) else {
                    continue;
                };
                if binding.has_explicit_annotation || !ranges_overlap(binding.declaration_range, visible) {
                    continue;
                }
                let formal = Some(present_declared_type(&parameter.declared_type, &presenter));
                let advisory = advisory
                    .and_then(|summary| summary.parameters.iter().find(|(slot, _)| slot == &parameter.id).map(|(_, fact)| fact))
                    .cloned();
                if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                    continue;
                }
                hints.push(EditorTypeHint {
                    kind: EditorTypeHintKind::Parameter,
                    source_range: binding.declaration_range,
                    insertion_offset: binding.declaration_range.end,
                    target: source.structure.target_for(site).cloned(),
                    formal,
                    advisory,
                });
            }

            if callable.has_explicit_return_annotation {
                continue;
            }
            let insertion_offset = source
                .structure
                .callable_body_ranges
                .get(&callable.id)
                .map_or(callable.declaration_range.end, |range| range.end);
            if insertion_offset < visible.start || insertion_offset > visible.end {
                continue;
            }
            let formal = Some(presenter.present_knowledge(&signature.published_return_knowledge()));
            let advisory = advisory.map(|summary| summary.return_fact.clone());
            if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                continue;
            }
            hints.push(EditorTypeHint {
                kind: EditorTypeHintKind::Return,
                source_range: callable.declaration_range,
                insertion_offset,
                target: Some(SemanticTargetId::Callable(callable.id.clone())),
                formal,
                advisory,
            });
        }

        hints.sort_by_key(|hint| (hint.insertion_offset, hint.kind));
        hints
    }

    fn formal_binding_presentation(&self, site: &SourceSiteId, presenter: &TypePresenter<'_>) -> Option<FormalPresentation> {
        let fact_ref = self.snapshot.formal_fact_for_site(site)?;
        let fact_site = self.snapshot.formal_fact(&fact_ref)?;
        let knowledge = match &fact_ref {
            crate::presentation::FormalFactRef::Binding { callable, binding } => self.snapshot.formal_binding(callable, *binding)?.current.clone(),
            _ => return None,
        };
        Some(match fact_site.status {
            FormalFactStatus::Ready => presenter.present_knowledge(&knowledge),
            FormalFactStatus::Unknown => FormalPresentation::Unknown,
            FormalFactStatus::Dynamic => FormalPresentation::Dynamic,
            FormalFactStatus::Invalid | FormalFactStatus::InvalidMultiple => FormalPresentation::Invalid,
            FormalFactStatus::Blocked => FormalPresentation::Blocked,
            FormalFactStatus::Cancelled => FormalPresentation::Cancelled,
            FormalFactStatus::BudgetExceeded => FormalPresentation::BudgetExceeded,
            FormalFactStatus::InternalFailure => FormalPresentation::InternalFailure,
            FormalFactStatus::Partial => FormalPresentation::Partial,
        })
    }

    /// Returns compiler-owned native presentation metadata for one callable.
    pub fn native_callable_presentation(&self, callable: &CallableId) -> Option<NativeCallablePresentation> {
        let signature = self.snapshot.callable_signatures.get(callable)?;
        let native_id = signature.native_id?;
        let record = phalcom_native_surface::NATIVE_SURFACE_CATALOG.find(native_id.0)?;
        Some(NativeCallablePresentation {
            documentation: record.docs(),
            conceptual: record.conceptual(),
        })
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
            .filter(|site| self.is_definition_site(target, site) == definitions)
            .cloned()
            .collect::<Vec<_>>();
        sites.sort();
        sites.dedup();
        sites
    }

    fn is_definition_site(&self, target: &SemanticTargetId, site: &SourceSiteId) -> bool {
        let Some(module) = self.snapshot.source_index.module_for_site(site) else {
            return false;
        };
        let Some(source_site) = module.structure.sites.get(site) else {
            return false;
        };
        match (target, &source_site.kind) {
            (SemanticTargetId::Binding(expected), crate::source_index::SourceSiteKind::BindingDeclaration) => expected == site,
            (SemanticTargetId::Declaration(expected), crate::source_index::SourceSiteKind::Declaration(actual)) => expected == actual,
            (SemanticTargetId::Callable(expected), crate::source_index::SourceSiteKind::Callable(actual)) => expected == actual,
            (SemanticTargetId::Field(expected), crate::source_index::SourceSiteKind::Field(actual)) => expected == actual,
            (SemanticTargetId::Module(expected), crate::source_index::SourceSiteKind::Module) => {
                matches!(&site.owner, SourceOwner::Module(actual) if actual == expected)
            }
            _ => false,
        }
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
            collect_receiver_alternatives(shape, &mut alternatives);
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
        members.sort_by_key(member_sort_key);
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

    /// Returns canonical callable candidates compatible with the structural
    /// prefix already written at an incomplete call site. Exact dispatch for
    /// each candidate selector is rechecked from every receiver alternative,
    /// so overridden superclass members cannot become accidental candidates.
    pub fn callable_candidates(&self, receiver: &ResolvedReceiver, pattern: &PartialCallPattern, access: &AccessContext) -> Vec<CallableId> {
        let mut candidates = self
            .members_for_receiver(receiver, access)
            .into_iter()
            .filter_map(|member| match member.target {
                EditorMemberTarget::Callable(callable)
                    if callable.selector.base == pattern.base
                        && callable.selector.kind == pattern.kind
                        && callable.selector.slots.len() >= pattern.written_slots.len()
                        && callable
                            .selector
                            .slots
                            .iter()
                            .zip(pattern.written_slots.iter())
                            .all(|(candidate, written)| candidate == written) =>
                {
                    Some(callable)
                }
                _ => None,
            })
            .filter(|callable| {
                receiver.alternatives.iter().any(|alternative| {
                    let side = match alternative.mode {
                        ReceiverMode::Instance => crate::identity::DispatchSide::Instance,
                        ReceiverMode::Class => crate::identity::DispatchSide::Class,
                    };
                    self.snapshot
                        .dispatch
                        .resolve_callable_id(self.snapshot.hierarchy.as_ref(), &alternative.declaration, side, &callable.selector)
                        .as_ref()
                        == Some(callable)
                })
            })
            .collect::<Vec<_>>();
        candidates.sort();
        candidates.dedup();
        candidates
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

fn ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    left.start <= right.end && right.start <= left.end
}

fn type_hint_has_usable_evidence(formal: Option<&FormalPresentation>, advisory: Option<&AdvisoryFact>) -> bool {
    if matches!(formal, Some(FormalPresentation::Known(_) | FormalPresentation::Dynamic)) {
        return true;
    }
    if formal.is_some() && !matches!(formal, Some(FormalPresentation::Unknown)) {
        return false;
    }
    advisory.is_some_and(|fact| !matches!(fact.shape, ValueShape::Unknown))
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
