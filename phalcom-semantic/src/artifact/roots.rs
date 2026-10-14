//! Stable primary-owner roots collected before either relocation graph closes.
use super::export::register_structural_primary_roots;
use crate::declaration_type::{DeclaredTypeFact, DeclaredTypeState};
use crate::metadata::stable_identity::{to_stable_callable, to_stable_declaration, to_stable_impl, to_stable_variant};
use crate::snapshot::SemanticSnapshot;
use crate::static_value::StaticExportContext;
use crate::types::parameter::{GenericConstraint, GenericSignature, TypeTerm};
use crate::types::store::artifact::TypeExportContext;
use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::semantic::type_store::{CanonicalRootFamilyTag as Family, CanonicalTypeRootKey, StableOwnerRef};

struct Roots<'a> {
    types: &'a mut TypeExportContext,
    owner: StableOwnerRef,
    family: Family,
    ordinal: u32,
}
impl<'a> Roots<'a> {
    fn new(types: &'a mut TypeExportContext, owner: StableOwnerRef, family: Family) -> Self {
        Self {
            types,
            owner,
            family,
            ordinal: 100,
        }
    }
    fn key(&mut self) -> Result<CanonicalTypeRootKey, ArtifactError> {
        let ordinal = self.ordinal;
        self.ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
        Ok(CanonicalTypeRootKey {
            family_tag: self.family,
            owner: self.owner.clone(),
            product_subkey: ordinal,
        })
    }
    fn ty(&mut self, ty: crate::types::id::TypeId) -> Result<(), ArtifactError> {
        let key = self.key()?;
        self.types.register_type_root(self.types.expected_lineage(), key, ty)?;
        Ok(())
    }
    fn term(&mut self, term: &TypeTerm) -> Result<(), ArtifactError> {
        match term {
            TypeTerm::Canonical(ty) => self.ty(*ty),
            TypeTerm::SelfType(_) => Ok(()),
            TypeTerm::Infer(_) => Err(ArtifactError::LimitExceeded("inference variable in published root".into())),
        }
    }
    fn fact(&mut self, fact: &DeclaredTypeFact) -> Result<(), ArtifactError> {
        if let DeclaredTypeState::Known(term) = &fact.state {
            self.term(term)?;
        }
        Ok(())
    }
    fn constraint(&mut self, c: &GenericConstraint) -> Result<(), ArtifactError> {
        match c {
            GenericConstraint::Subtype { lower, upper } => {
                self.term(lower)?;
                self.term(upper)?;
            }
            GenericConstraint::Equivalent { left, right } => {
                self.term(left)?;
                self.term(right)?;
            }
            GenericConstraint::Conforms { subject, trait_ref } => {
                self.term(subject)?;
                for &ty in &trait_ref.arguments {
                    self.ty(ty)?;
                }
            }
        }
        Ok(())
    }
    fn generic(&mut self, g: &Option<GenericSignature>) -> Result<(), ArtifactError> {
        if let Some(g) = g {
            for &p in &g.parameters {
                let key = self.key()?;
                self.types.register_parameter_root(self.types.expected_lineage(), key, p)?;
            }
            for &k in &g.parameter_kinds {
                let key = self.key()?;
                self.types.register_kind_root(self.types.expected_lineage(), key, k)?;
            }
            for c in &g.constraints {
                self.constraint(c)?;
            }
        }
        Ok(())
    }
    fn static_data(&mut self, info: &Option<crate::algebraic_semantics::StaticAlgebraicDeclarationInfo>) -> Result<(), ArtifactError> {
        if let Some(info) = info {
            let key = self.key()?;
            self.types.register_domain_root(self.types.expected_lineage(), key, info.domain)?;
            for &c in &info.constructors {
                let key = self.key()?;
                self.types.register_constructor_root(self.types.expected_lineage(), key, c)?;
            }
        }
        Ok(())
    }
}

/// Register every reference used by the current primary projection in stable owner order.
/// Required owner adapters extend this pass together with their wire projection.
pub fn register_semantic_world_roots(
    snapshot: &SemanticSnapshot,
    types: &mut TypeExportContext,
    statics: &mut StaticExportContext,
) -> Result<(), ArtifactError> {
    register_structural_primary_roots(
        types,
        &snapshot.data_semantics,
        &snapshot.enum_semantics,
        &snapshot.declarations,
        &snapshot.hierarchy,
        &snapshot.type_aliases,
    )?;
    for (id, info) in snapshot.declarations.iter() {
        Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::GenericSignature).generic(&info.generic_signature)?;
    }
    for (id, info) in snapshot.data_semantics.data_decls.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::DataContract);
        r.generic(&info.generic_signature)?;
        r.static_data(&info.static_data)?;
        for c in &info.components {
            r.fact(&c.declared_type)?;
        }
        for p in &info.constructor.parameters {
            r.fact(&p.declared_type)?;
        }
        r.ty(info.constructor.result_type_template)?;
        for c in &info.shape_template.contributions {
            match c {
                crate::data_semantics::DataProductContribution::Component(c) => r.fact(&c.declared_type)?,
                crate::data_semantics::DataProductContribution::Expansion(e) => r.fact(&e.operand)?,
            }
        }
    }
    for (id, info) in snapshot.enum_semantics.enums.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::EnumContract);
        r.generic(&info.generic_signature)?;
        r.static_data(&info.static_data)?;
        r.ty(info.default_result_type)?;
    }
    for (id, info) in snapshot.enum_semantics.variants.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Variant(to_stable_variant(id)), Family::VariantContract);
        for f in &info.fields {
            r.fact(&f.declared_type)?;
        }
        r.ty(info.result_type_template)?;
        r.ty(info.exact_case_template)?;
        // Case bindings are keyed by stable binder identity, not native parameter IDs.
        let mut bindings = info.case_environment.bindings.iter().collect::<Vec<_>>();
        bindings.sort_by_key(|(p, _)| {
            let p = snapshot.store.type_parameter(**p);
            (p.owner.clone(), p.index)
        });
        for (&p, &ty) in bindings {
            let key = r.key()?;
            r.types.register_parameter_root(r.types.expected_lineage(), key, p)?;
            r.ty(ty)?;
        }
        for c in &info.case_environment.equalities {
            r.constraint(c)?;
        }
        if let Some(c) = &info.constructor {
            r.generic(&c.generic_signature)?;
            for p in &c.parameters {
                r.fact(&p.declared_type)?;
            }
            r.ty(c.result_type_template)?;
            r.ty(c.exact_case_template)?;
        }
    }
    for (id, sig) in snapshot.callable_signatures.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Callable(to_stable_callable(id)), Family::CallableSignature);
        r.generic(&sig.generics)?;
        for p in &sig.parameters {
            r.fact(&p.contract.declared_type())?;
        }
        r.fact(&sig.declared_return)?;
    }
    for (id, head) in snapshot.conformance_index.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(id)), Family::TraitAndConformance);
        r.ty(head.target_head)?;
        r.generic(&head.generic_signature)?;
        for &ty in &head.trait_ref.arguments {
            r.ty(ty)?;
        }
    }
    for (decl_id, _surface) in snapshot.surfaces.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(decl_id)), Family::SurfaceAndDispatch);
        for (_, field_sig) in snapshot.field_signatures.iter_owner(decl_id) {
            r.fact(&field_sig.declared_type)?;
        }
    }
    for (id, surface) in snapshot.trait_surfaces.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Declaration(to_stable_declaration(id)), Family::TraitAndConformance);
        r.generic(&surface.generic_signature)?;
        if let Some(generics) = &surface.generic_signature {
            for constraint in &generics.constraints {
                if let GenericConstraint::Conforms { subject, trait_ref } = constraint {
                    if matches!(subject, TypeTerm::SelfType(_)) {
                        for &arg in &trait_ref.arguments {
                            r.ty(arg)?;
                        }
                    }
                }
            }
        }
    }
    for (id, product) in snapshot.type_member_declarations.iter() {
        let owner = phalcom_type_meta::identity::StableTypeMemberRef {
            owner: to_stable_declaration(&id.owner),
            selector: id.selector.name().into(),
        };
        let mut r = Roots::new(types, StableOwnerRef::TypeMember(owner), Family::TypeMemberAndDomain);
        for parameter in &product.signature.parameters {
            match parameter.domain {
                crate::types::type_member::TypeMemberDomain::Kind(kind) => {
                    let key = r.key()?;
                    r.types.register_kind_root(r.types.expected_lineage(), key, kind)?;
                }
                crate::types::type_member::TypeMemberDomain::ValueType(ty) => r.ty(ty)?,
            }
        }
        match product.signature.result_domain {
            crate::types::type_member::TypeMemberDomain::Kind(kind) => {
                let key = r.key()?;
                r.types.register_kind_root(r.types.expected_lineage(), key, kind)?;
            }
            crate::types::type_member::TypeMemberDomain::ValueType(ty) => r.ty(ty)?,
        }
    }
    let mut associated_domain_ordinal = 100u32;
    for (id, product) in snapshot.associated_static_domains.iter() {
        let owner = StableOwnerRef::Declaration(to_stable_declaration(&id.owner));
        let key = CanonicalTypeRootKey {
            family_tag: Family::TypeMemberAndDomain,
            owner: owner.clone(),
            product_subkey: associated_domain_ordinal,
        };
        associated_domain_ordinal = associated_domain_ordinal
            .checked_add(1)
            .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
        types.register_domain_root(types.expected_lineage(), key, product.domain)?;
        for &constructor in &product.constructors {
            let key = CanonicalTypeRootKey {
                family_tag: Family::TypeMemberAndDomain,
                owner: owner.clone(),
                product_subkey: associated_domain_ordinal,
            };
            associated_domain_ordinal = associated_domain_ordinal
                .checked_add(1)
                .ok_or_else(|| ArtifactError::LimitExceeded("root ordinal overflow".into()))?;
            types.register_constructor_root(types.expected_lineage(), key, constructor)?;
        }
    }
    for (id, plan) in snapshot.conformance_witness_plans.iter() {
        let mut r = Roots::new(types, StableOwnerRef::Impl(to_stable_impl(id)), Family::ConstructorContract);
        r.ty(plan.target_template)?;
        r.generic(&plan.generic_signature)?;
        for &ty in &plan.trait_ref_template.arguments {
            r.ty(ty)?;
        }
        for selection in plan.requirements.values() {
            if let crate::impls::RequirementSelectionTemplate::DataComponent { specialized_type, .. } = selection {
                r.ty(*specialized_type)?;
            }
        }
        for template in plan.associated_type_plan.bindings.values() {
            if let crate::impls::AssociatedTypeBindingValue::Type(ty) = template.value_template {
                r.ty(ty)?;
            }
        }
    }
    for (_, product) in snapshot.static_definitions.iter() {
        if let Some(body) = &product.checked_body {
            if let crate::static_definition::StaticDefinitionBodyRepr::Operational(template) = &body.root {
                super::operational::register_operational_template_roots(template, statics, types)?;
            }
        }
    }
    Ok(())
}
