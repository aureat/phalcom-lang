use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{AssociatedFamilyId, DeclarationId, VariantId};
use phalcom_common::selector::SelectorBase;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

/// The category of an associated family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssociatedFamilyKind {
    Variant,
}

/// A member belonging to an associated family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssociatedMemberId {
    Variant(VariantId),
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
