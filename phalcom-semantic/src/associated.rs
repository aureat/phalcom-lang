use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{AssociatedFamilyId, AssociatedStaticDomainDeclarationId, DataComponentId, DataConstructorId, DeclarationId, VariantId};
use crate::types::type_member::TypeMemberDeclarationId;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::SelectorBase;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

/// The category of an associated family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssociatedFamilyKind {
    Variant,
    DataConstructor,
    StaticDomain,
    TypeMember,
    IntrinsicTypeMember,
}

/// A member belonging to an associated family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssociatedMemberId {
    Variant(VariantId),
    DataConstructor(DataConstructorId),
    DataComponent(DataComponentId),
    StaticDomain(AssociatedStaticDomainDeclarationId),
    TypeMember(TypeMemberDeclarationId),
}

/// Name/category identity collected from a declaration shell before its checked
/// signature or case product is available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedMemberShell {
    pub base: SelectorBase,
    pub kind: AssociatedFamilyKind,
    pub member: AssociatedMemberId,
    pub module: crate::identity::ModuleId,
    pub source_range: SourceRange,
}

/// Builds an owner's associated namespace with checked source type members and
/// associated static-domain declarations. The surface admits names; declaration
/// products remain the authority for their checked signatures and constructors.
pub fn build_associated_surface_with_declarations(
    owner: &DeclarationId,
    variants: Option<&[VariantId]>,
    type_members: &[crate::db::product::TypeMemberDeclarationProduct],
    static_domains: &[crate::db::product::AssociatedStaticDomainDeclarationProduct],
    behavior_bases: &HashSet<SelectorBase>,
    inherited_class_bases: &HashSet<SelectorBase>,
    module_id: &crate::identity::ModuleId,
    span: Option<SemanticSourceSpan>,
) -> (Arc<AssociatedSurface>, Arc<[SemanticDiagnostic]>) {
    let (base, base_diagnostics) = build_associated_surface(owner, variants, behavior_bases, inherited_class_bases, module_id, span);
    let mut shells = type_members
        .iter()
        .map(|member| AssociatedMemberShell {
            base: associated_base(&member.id.selector),
            kind: if member.signature.intrinsic.is_some() {
                AssociatedFamilyKind::IntrinsicTypeMember
            } else {
                AssociatedFamilyKind::TypeMember
            },
            member: AssociatedMemberId::TypeMember(member.id.clone()),
            module: member.id.owner.module.clone(),
            source_range: member.source_range,
        })
        .collect::<Vec<_>>();
    shells.extend(static_domains.iter().map(|domain| AssociatedMemberShell {
        base: SelectorBase::Named(domain.id.name.to_string()),
        kind: AssociatedFamilyKind::StaticDomain,
        member: AssociatedMemberId::StaticDomain(domain.id.clone()),
        module: domain.id.owner.module.clone(),
        source_range: domain.source_range,
    }));
    let (surface, declaration_diagnostics) = extend_associated_surface_with_shells(
        owner,
        base,
        &shells,
        behavior_bases,
        inherited_class_bases,
        module_id,
    );
    let mut diagnostics = base_diagnostics.to_vec();
    diagnostics.extend(declaration_diagnostics.iter().cloned());
    (surface, Arc::from(diagnostics.into_boxed_slice()))
}

pub fn extend_associated_surface_with_static_domains(
    owner: &DeclarationId,
    base_surface: Arc<AssociatedSurface>,
    static_domains: &[crate::db::product::AssociatedStaticDomainDeclarationProduct],
    behavior_bases: &HashSet<SelectorBase>,
    inherited_class_bases: &HashSet<SelectorBase>,
    module_id: &crate::identity::ModuleId,
) -> (Arc<AssociatedSurface>, Arc<[SemanticDiagnostic]>) {
    let shells = static_domains
        .iter()
        .map(|domain| AssociatedMemberShell {
            base: SelectorBase::Named(domain.id.name.to_string()),
            kind: AssociatedFamilyKind::StaticDomain,
            member: AssociatedMemberId::StaticDomain(domain.id.clone()),
            module: domain.id.owner.module.clone(),
            source_range: domain.source_range,
        })
        .collect::<Vec<_>>();
    extend_associated_surface_with_shells(owner, base_surface, &shells, behavior_bases, inherited_class_bases, module_id)
}

/// Adds shell identities to a precomputed runtime namespace. Namespace
/// admission depends only on stable names/categories; checked member products
/// remain separate and cannot invalidate this surface when only a signature
/// changes.
pub fn extend_associated_surface_with_shells(
    owner: &DeclarationId,
    base_surface: Arc<AssociatedSurface>,
    shells: &[AssociatedMemberShell],
    behavior_bases: &HashSet<SelectorBase>,
    inherited_class_bases: &HashSet<SelectorBase>,
    _module_id: &crate::identity::ModuleId,
) -> (Arc<AssociatedSurface>, Arc<[SemanticDiagnostic]>) {
    let mut surface = (*base_surface).clone();
    let mut diagnostics = Vec::new();
    let mut counts = HashMap::<SelectorBase, usize>::new();
    for shell in shells {
        *counts.entry(shell.base.clone()).or_default() += 1;
    }
    for shell in shells {
        let base = shell.base.clone();
        // Intrinsic type members form the `::` namespace. A structural
        // intrinsic selector such as Tuple::At can share its canonical
        // selector with the runtime `[]` behavior family because the two
        // operations are selected in different semantic planes.
        let behavior_collision = behavior_bases.contains(&base) && shell.kind != AssociatedFamilyKind::IntrinsicTypeMember;
        if counts.get(&base).copied().unwrap_or_default() > 1
            || surface.families.contains_key(&base)
            || behavior_collision
            || inherited_class_bases.contains(&base)
        {
            diagnostics.push(SemanticDiagnostic::error_in(
                shell.module.clone(),
                DiagnosticCode::EnumFamilyCategoryConflict,
                format!(
                    "associated member `{}` on `{}` conflicts with another owner member",
                    format_selector_base(&base),
                    owner.name
                ),
                shell.source_range,
            ));
            continue;
        }

        let id = AssociatedFamilyId::new(owner.clone(), base.clone());
        surface.families.insert(
            base,
            AssociatedFamilyInfo {
                id,
                kind: shell.kind,
                members: Box::new([shell.member.clone()]),
            },
        );
    }
    (Arc::new(surface), Arc::from(diagnostics.into_boxed_slice()))
}

/// Builds an owner's associated namespace including checked source type members.
///
/// The returned surface is the sole name-admission/collision authority. Member signatures
/// remain separate semantic products, referenced here only by stable identity.
pub fn build_associated_surface_with_type_members(
    owner: &DeclarationId,
    variants: Option<&[VariantId]>,
    type_members: &[crate::db::product::TypeMemberDeclarationProduct],
    behavior_bases: &HashSet<SelectorBase>,
    inherited_class_bases: &HashSet<SelectorBase>,
    module_id: &crate::identity::ModuleId,
    span: Option<SemanticSourceSpan>,
) -> (Arc<AssociatedSurface>, Arc<[SemanticDiagnostic]>) {
    let (base, base_diagnostics) = build_associated_surface(owner, variants, behavior_bases, inherited_class_bases, module_id, span.clone());
    let shells = type_members
        .iter()
        .map(|member| AssociatedMemberShell {
            base: associated_base(&member.id.selector),
            kind: if member.signature.intrinsic.is_some() {
                AssociatedFamilyKind::IntrinsicTypeMember
            } else {
                AssociatedFamilyKind::TypeMember
            },
            member: AssociatedMemberId::TypeMember(member.id.clone()),
            module: member.id.owner.module.clone(),
            source_range: member.source_range,
        })
        .collect::<Vec<_>>();
    let (surface, member_diagnostics) =
        extend_associated_surface_with_shells(owner, base, &shells, behavior_bases, inherited_class_bases, module_id);
    let mut diagnostics = base_diagnostics.to_vec();
    diagnostics.extend(member_diagnostics.iter().cloned());
    (surface, Arc::from(diagnostics.into_boxed_slice()))
}

fn associated_base(selector: &crate::types::type_member::TypeMemberSelector) -> SelectorBase {
    match selector {
        crate::types::type_member::TypeMemberSelector::Named(name) => SelectorBase::Named(name.to_string()),
        crate::types::type_member::TypeMemberSelector::Subscript => SelectorBase::Subscript,
    }
}

/// Metadata describing one associated family on a declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedFamilyInfo {
    pub id: AssociatedFamilyId,
    pub kind: AssociatedFamilyKind,
    pub members: Box<[AssociatedMemberId]>,
}

/// Complete associated family namespace surface for one declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedSurface {
    pub owner: DeclarationId,
    pub families: BTreeMap<SelectorBase, AssociatedFamilyInfo>,
}

impl AssociatedSurface {
    pub fn new(owner: DeclarationId) -> Self {
        Self {
            owner,
            families: BTreeMap::new(),
        }
    }
}

/// Table of associated family surfaces indexed by declaration owner.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AssociatedFamilyTable {
    pub surfaces: HashMap<DeclarationId, Arc<AssociatedSurface>>,
}

impl AssociatedFamilyTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, owner: DeclarationId, surface: Arc<AssociatedSurface>) {
        self.surfaces.insert(owner, surface);
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        self.surfaces.retain(|owner, _| &owner.module != module);
    }

    pub fn remove(&mut self, owner: &DeclarationId) {
        self.surfaces.remove(owner);
    }
}

fn format_selector_base(base: &SelectorBase) -> &str {
    match base {
        SelectorBase::Named(name) => name.as_str(),
        SelectorBase::Subscript => "[]",
    }
}

/// Computes the [`AssociatedSurface`] for a declaration and validates family conflicts.
pub fn build_associated_surface(
    owner: &DeclarationId,
    variants: Option<&[VariantId]>,
    behavior_bases: &HashSet<SelectorBase>,
    inherited_class_bases: &HashSet<SelectorBase>,
    module_id: &crate::identity::ModuleId,
    span: Option<SemanticSourceSpan>,
) -> (Arc<AssociatedSurface>, Arc<[SemanticDiagnostic]>) {
    let mut diagnostics = Vec::new();
    let mut surface = AssociatedSurface::new(owner.clone());

    let mut variant_groups: BTreeMap<SelectorBase, Vec<VariantId>> = BTreeMap::new();
    if let Some(vars) = variants {
        for v in vars {
            let base = v.selector.base.clone();
            variant_groups.entry(base).or_default().push(v.clone());
        }
    }

    for (base, vars) in variant_groups {
        let has_inherited_conflict = inherited_class_bases.contains(&base);
        let has_declared_conflict = behavior_bases.contains(&base);

        if has_inherited_conflict {
            diagnostics.push(SemanticDiagnostic::error_in(
                module_id.clone(),
                DiagnosticCode::EnumFamilyInheritedBehaviorConflict,
                format!(
                    "variant family `{}` on `{}` conflicts with inherited class behavior",
                    format_selector_base(&base),
                    owner.name
                ),
                span.as_ref().map(|s| s.range).unwrap_or_default(),
            ));
            continue;
        }

        if has_declared_conflict {
            diagnostics.push(SemanticDiagnostic::error_in(
                module_id.clone(),
                DiagnosticCode::EnumFamilyCategoryConflict,
                format!(
                    "associated family `{}` on `{}` cannot contain both variants and class methods",
                    format_selector_base(&base),
                    owner.name
                ),
                span.as_ref().map(|s| s.range).unwrap_or_default(),
            ));
            continue;
        }

        let family_id = AssociatedFamilyId::new(owner.clone(), base.clone());
        let members: Vec<AssociatedMemberId> = vars.into_iter().map(AssociatedMemberId::Variant).collect();
        surface.families.insert(
            base,
            AssociatedFamilyInfo {
                id: family_id,
                kind: AssociatedFamilyKind::Variant,
                members: members.into_boxed_slice(),
            },
        );
    }

    (Arc::new(surface), Arc::from(diagnostics.into_boxed_slice()))
}

/// Builds an [`AssociatedSurface`] for a data declaration with its canonical constructor family.
pub fn build_data_associated_surface(owner: &DeclarationId) -> Arc<AssociatedSurface> {
    let mut surface = AssociatedSurface::new(owner.clone());
    let base = SelectorBase::Named(owner.name.to_string());
    let family_id = AssociatedFamilyId::new(owner.clone(), base.clone());
    surface.families.insert(
        base,
        AssociatedFamilyInfo {
            id: family_id,
            kind: AssociatedFamilyKind::DataConstructor,
            members: Box::new([AssociatedMemberId::DataConstructor(DataConstructorId::new(owner.clone()))]),
        },
    );
    Arc::new(surface)
}
