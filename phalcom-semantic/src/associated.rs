use crate::db::key::ProductFingerprint;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{AssociatedFamilyId, DataComponentId, DataConstructorId, DeclarationId, VariantId};
use crate::types::type_member::TypeMemberDeclarationId;
use phalcom_common::selector::SelectorBase;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

/// The category of an associated family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssociatedFamilyKind {
    Variant,
    DataConstructor,
    TypeMember,
    IntrinsicTypeMember,
}

/// A member belonging to an associated family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssociatedMemberId {
    Variant(VariantId),
    DataConstructor(DataConstructorId),
    DataComponent(DataComponentId),
    TypeMember(TypeMemberDeclarationId),
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
    let mut surface = (*base).clone();
    let mut diagnostics = base_diagnostics.to_vec();
    let mut source_selector_counts = HashMap::<SelectorBase, usize>::new();
    for member in type_members {
        *source_selector_counts.entry(associated_base(&member.id.selector)).or_default() += 1;
    }

    for member in type_members {
        let base = associated_base(&member.id.selector);
        let conflict = source_selector_counts.get(&base).copied().unwrap_or_default() > 1
            || surface.families.contains_key(&base)
            || behavior_bases.contains(&base)
            || inherited_class_bases.contains(&base);
        if conflict {
            diagnostics.push(SemanticDiagnostic::error_in(
                module_id.clone(),
                DiagnosticCode::EnumFamilyCategoryConflict,
                format!(
                    "associated member `{}` on `{}` conflicts with another owner member",
                    format_selector_base(&base),
                    owner.name
                ),
                member.source_range,
            ));
            continue;
        }

        let kind = if member.signature.intrinsic.is_some() {
            AssociatedFamilyKind::IntrinsicTypeMember
        } else {
            AssociatedFamilyKind::TypeMember
        };
        let id = AssociatedFamilyId::new(owner.clone(), base.clone());
        surface.families.insert(
            base,
            AssociatedFamilyInfo {
                id,
                kind,
                members: Box::new([AssociatedMemberId::TypeMember(member.id.clone())]),
                type_member_signature_fingerprint: Some(crate::db::fingerprint::type_member_declaration_product_fingerprint(member)),
            },
        );
    }

    (Arc::new(surface), Arc::from(diagnostics.into_boxed_slice()))
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
    /// Version stamp of the checked signature referenced by a type-member entry.
    pub type_member_signature_fingerprint: Option<ProductFingerprint>,
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
                type_member_signature_fingerprint: None,
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
            type_member_signature_fingerprint: None,
        },
    );
    Arc::new(surface)
}
