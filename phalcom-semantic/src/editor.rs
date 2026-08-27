//! Compiler-owned, protocol-neutral semantic queries for editor features.

use crate::advisory::ValueShape;
use crate::advisory::advisory_shape_from_formal;
use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId};
use crate::snapshot::SemanticSnapshot;
use crate::source_index::{OccurrenceRole, SourceBindingInfo};
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
        self.snapshot.occurrence_at(module, offset).and_then(|view| view.target.cloned())
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
        let expression_site = [range.start, range.end.saturating_sub(1)]
            .into_iter()
            .filter_map(|offset| self.snapshot.source_index.expression_site_at(module, offset))
            .max_by_key(|site| site.range.len())
            .map(|site| site.id.clone());
        let site = expression_site.clone().or(occurrence_site.clone());
        let access = site.as_ref().and_then(|site| match &site.owner {
            SourceOwner::Callable(callable) => Some(AccessContext {
                enclosing_declaration: Some(callable.owner.clone()),
                enclosing_callable: Some(callable.clone()),
            }),
            SourceOwner::Module(_) => None,
        });
        let source_text = self.snapshot.sources.get(module).and_then(|source| source.text.get(range.start..range.end));
        if let Some(owner) = access.as_ref().and_then(|access| access.enclosing_declaration.as_ref()) {
            if source_text.is_some_and(|text| text.trim() == "self") {
                return Some(ResolvedReceiver {
                    alternatives: Arc::from([ReceiverAlternative {
                        declaration: owner.clone(),
                        mode: ReceiverMode::Instance,
                    }]),
                });
            }
            if source_text.is_some_and(|text| text.trim() == "super") {
                return self.snapshot.hierarchy.superclass(owner).cloned().map(|declaration| ResolvedReceiver {
                    alternatives: Arc::from([ReceiverAlternative {
                        declaration,
                        mode: ReceiverMode::Instance,
                    }]),
                });
            }
        }
        let target = site
            .as_ref()
            .and_then(|site| self.snapshot.source_index.target_for(site))
            .or_else(|| occurrence_site.as_ref().and_then(|site| self.snapshot.source_index.target_for(site)));
        let target_site = target.and_then(|target| match target {
            SemanticTargetId::Binding(binding) => Some(binding),
            _ => None,
        });
        let fact_site = target_site.or(expression_site.as_ref()).or(occurrence_site.as_ref());
        let shape = target_site
            .as_ref()
            .and_then(|site| self.formal_shape_for_site(site))
            .or_else(|| self.formal_shape_at(module, range.start))
            .or_else(|| {
                fact_site
                    .and_then(|site| self.snapshot.advisory_fact(site))
                    .filter(|fact| !matches!(fact.shape, ValueShape::Unknown))
                    .map(|fact| fact.shape.clone())
            })
            .or_else(|| {
                occurrence_site
                    .as_ref()
                    .and_then(|site| self.snapshot.advisory_fact(site))
                    .filter(|fact| !matches!(fact.shape, ValueShape::Unknown))
                    .map(|fact| fact.shape.clone())
            })
            .or_else(|| match target {
                Some(SemanticTargetId::Module(module)) => Some(ValueShape::Module(module.clone())),
                Some(SemanticTargetId::Declaration(declaration)) => Some(ValueShape::ClassObject(declaration.clone())),
                _ => None,
            });
        let mut alternatives = Vec::new();
        if let Some(shape) = shape {
            collect_receiver_alternatives(&shape, &mut alternatives);
            if alternatives.is_empty()
                && let Some(source_text) = source_text
                && let Some(receiver) = self.resolve_chained_receiver(&shape, source_text)
            {
                return Some(receiver);
            }
        }
        if alternatives.is_empty()
            && let Some(SemanticTargetId::Declaration(declaration)) = target.cloned()
        {
            alternatives.push(ReceiverAlternative {
                declaration,
                mode: ReceiverMode::Class,
            });
        }
        alternatives.sort_by(|left, right| (&left.declaration, left.mode as u8).cmp(&(&right.declaration, right.mode as u8)));
        alternatives.dedup();
        (!alternatives.is_empty()).then(|| ResolvedReceiver {
            alternatives: Arc::from(alternatives.into_boxed_slice()),
        })
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

    fn resolve_chained_receiver(&self, shape: &ValueShape, source: &str) -> Option<ResolvedReceiver> {
        let parts = dotted_expression_parts(source);
        let mut current = shape.clone();
        for part in parts.into_iter().skip(1) {
            if let Some(name) = part.strip_suffix(')').and_then(|part| part.split_once('(').map(|(name, _)| name)) {
                let (ValueShape::ClassObject(declaration) | ValueShape::Instance(declaration)) = &current else {
                    return None;
                };
                let selector = Selector::try_decode_exact(&format!("{name}()")).ok()?;
                let side = match current {
                    ValueShape::ClassObject(_) => crate::identity::DispatchSide::Class,
                    ValueShape::Instance(_) => crate::identity::DispatchSide::Instance,
                    _ => unreachable!(),
                };
                let surface = self.snapshot.surfaces.get(declaration)?.surface(side);
                let Some(callable) = surface.get_callable_id(&selector).cloned() else {
                    if matches!(current, ValueShape::ClassObject(_)) && selector.encode() == "new()" {
                        current = ValueShape::Instance(declaration.clone());
                        continue;
                    }
                    return None;
                };
                let semantic_kind = surface.get_callable(&selector)?.kind;
                current = if matches!(semantic_kind, crate::dispatch::CallableSemanticKind::Constructor) {
                    ValueShape::Instance(declaration.clone())
                } else if let Some(summary) = self.snapshot.advisory_callable(&callable) {
                    summary.return_fact.shape.clone()
                } else {
                    return None;
                };
                continue;
            }
            let ValueShape::Module(module) = &current else { return None };
            let queries = self.snapshot.module_queries();
            current = if let Some(export) = queries.public_exports(module).and_then(|exports| exports.get(part)) {
                match &export.target {
                    phalcom_modules::interface::LinkedExportTarget::Module(target) => ValueShape::Module(target.clone()),
                    phalcom_modules::interface::LinkedExportTarget::Binding(symbol) => {
                        let declaration = crate::identity::DeclarationId::new(symbol.module.clone(), symbol.name.clone());
                        self.snapshot
                            .surfaces
                            .contains_key(&declaration)
                            .then_some(ValueShape::ClassObject(declaration))?
                    }
                }
            } else {
                let declaration = crate::identity::DeclarationId::new(module.clone(), (*part).into());
                self.snapshot
                    .surfaces
                    .contains_key(&declaration)
                    .then_some(ValueShape::ClassObject(declaration))?
            };
        }
        match current {
            ValueShape::Instance(declaration) => Some(ResolvedReceiver {
                alternatives: Arc::from([ReceiverAlternative {
                    declaration,
                    mode: ReceiverMode::Instance,
                }]),
            }),
            ValueShape::ClassObject(declaration) => Some(ResolvedReceiver {
                alternatives: Arc::from([ReceiverAlternative {
                    declaration,
                    mode: ReceiverMode::Class,
                }]),
            }),
            _ => None,
        }
    }

    /// Returns all visible members across receiver alternatives, including
    /// inherited members, with duplicate canonical targets removed.
    pub fn members_for_receiver(&self, receiver: &ResolvedReceiver, access: &AccessContext) -> Vec<EditorMember> {
        let mut members = Vec::new();
        for alternative in receiver.alternatives.iter() {
            let mut declaration = Some(alternative.declaration.clone());
            let mut visited = BTreeSet::new();
            while let Some(current) = declaration {
                if !visited.insert(current.clone()) {
                    break;
                }
                if let Some(surface) = self.snapshot.surfaces.get(&current) {
                    let member_surface = surface.surface(match alternative.mode {
                        ReceiverMode::Instance => crate::identity::DispatchSide::Instance,
                        ReceiverMode::Class => crate::identity::DispatchSide::Class,
                    });
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
                declaration = self.snapshot.hierarchy.superclass(&current).cloned();
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

fn dotted_expression_parts(source: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (offset, character) in source.char_indices() {
        match character {
            '(' | '[' | '{' => depth = depth.saturating_add(1),
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '.' if depth == 0 => {
                parts.push(source[start..offset].trim());
                start = offset + character.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(source[start..].trim());
    parts
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
