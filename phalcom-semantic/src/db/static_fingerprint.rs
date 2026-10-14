//! Stable semantic fingerprints for static values and session-local residuals.
//!
//! Type/kind identity comes from the canonical metadata type exporter. Residual
//! and value store offsets only locate nodes; they never enter the fingerprint.

use crate::checker::proof_predicate::ProofPredicateGoal;
use crate::generic_evidence::GenericConformanceKey;
use crate::identity::{DeclarationId, VariantId};
use crate::member_projection::{MemberDeclarationIdentity, MemberProjection, MemberProjectionArgument, MemberProjectionSubject};
use crate::metadata::MetadataExporter;
use crate::static_domain::StaticValueDomain;
use crate::static_eval::application::{CanonicalStaticProofFact, StaticSpecializationArgument, StaticSpecializationEnvironmentKey, StaticTrustedBranchContext};
use crate::static_eval::residual::{StaticEvalTerm, StaticResidualData, StaticResidualRef, StaticResidualStore};
use crate::static_value::{StaticValueData, StaticValueRef, StaticValueStore};
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::TypeTerm;
use crate::types::store::TypeStore;
use crate::types::type_member::{ConstTerm, TypeMemberArgument, TypeMemberSelector};
use phalcom_type_meta::fingerprint::{Fingerprint128, FingerprintBuilder};
use phalcom_type_meta::header::MetadataProfile;
use std::collections::BTreeMap;

/// Computes query-product fingerprints without publishing query-only type nodes
/// as an artifact metadata bundle.
pub(crate) struct StaticSemanticFingerprinter<'a> {
    types: MetadataExporter<'a>,
    values: &'a StaticValueStore,
    residuals: Option<&'a StaticResidualStore>,
    projects: &'a phalcom_modules::ProjectUniverse,
    value_memo: BTreeMap<StaticValueRef, Fingerprint128>,
    residual_memo: BTreeMap<StaticResidualRef, Fingerprint128>,
}

impl<'a> StaticSemanticFingerprinter<'a> {
    pub(crate) fn new(
        store: &'a TypeStore,
        values: &'a StaticValueStore,
        residuals: Option<&'a StaticResidualStore>,
        projects: &'a phalcom_modules::ProjectUniverse,
    ) -> Self {
        Self {
            types: MetadataExporter::new(store, None, None, None, MetadataProfile::RuntimePublic).with_project_universe_query(projects),
            values,
            residuals,
            projects,
            value_memo: BTreeMap::new(),
            residual_memo: BTreeMap::new(),
        }
    }

    pub(crate) fn ty(&mut self, ty: TypeId) -> Result<Fingerprint128, String> {
        self.types.type_fingerprint(ty).map_err(|error| error.to_string())
    }

    fn kind(&mut self, kind: KindId) -> Result<Fingerprint128, String> {
        self.types.kind_fingerprint(kind).map_err(|error| error.to_string())
    }

    fn declaration(&self, declaration: &DeclarationId) -> Fingerprint128 {
        self.types.declaration_fingerprint(declaration)
    }

    pub(crate) fn constant_owner(&self, id: &crate::static_constant::StaticConstantId) -> Result<Fingerprint128, String> {
        if let phalcom_modules::ProjectIdentity::Resolved(project) = id.module().project
            && self.projects.get_project(project).is_none()
        {
            return Err("symbolic static constant has no stable project identity".into());
        }
        Ok(self.declaration(&DeclarationId::new(id.module().clone(), id.name().into())))
    }

    fn domain(&mut self, domain: StaticValueDomain, hash: &mut FingerprintBuilder) -> Result<(), String> {
        match domain {
            StaticValueDomain::Kind(kind) => {
                hash.write_u8(0);
                hash.write_fingerprint(self.kind(kind)?);
            }
            StaticValueDomain::ValueType(ty) => {
                hash.write_u8(1);
                hash.write_fingerprint(self.ty(ty)?);
            }
        }
        Ok(())
    }

    fn parameter(&mut self, parameter: crate::types::id::TypeParameterId, hash: &mut FingerprintBuilder) -> Result<(), String> {
        hash.write_fingerprint(self.types.parameter_fingerprint(parameter).map_err(|error| error.to_string())?);
        Ok(())
    }

    fn environment(&mut self, env: &StaticSpecializationEnvironmentKey, hash: &mut FingerprintBuilder) -> Result<(), String> {
        let mut arguments = Vec::with_capacity(env.argument_bindings().len());
        for (formal, argument) in env.argument_bindings() {
            let mut item = FingerprintBuilder::new();
            item.write_fingerprint(self.declaration(&formal.owner));
            item.write_u32(formal.index);
            self.domain(formal.domain, &mut item)?;
            match argument {
                StaticSpecializationArgument::Kinded(ty) => {
                    item.write_u8(0);
                    item.write_fingerprint(self.ty(*ty)?);
                }
                StaticSpecializationArgument::Value(value) => {
                    item.write_u8(1);
                    item.write_fingerprint(self.value(*value)?);
                }
            }
            arguments.push(item.finish());
        }
        arguments.sort_unstable();
        hash.write_u32(arguments.len() as u32);
        for argument in arguments {
            hash.write_fingerprint(argument);
        }

        let mut bindings = Vec::with_capacity(env.type_bindings().len());
        for (param, ty) in env.type_bindings() {
            let mut item = FingerprintBuilder::new();
            self.parameter(*param, &mut item)?;
            item.write_fingerprint(self.ty(*ty)?);
            bindings.push(item.finish());
        }
        bindings.sort_unstable();
        hash.write_u32(bindings.len() as u32);
        for binding in bindings {
            hash.write_fingerprint(binding);
        }
        self.facts(env.constraints(), hash)
    }

    fn facts(&mut self, facts: &[CanonicalStaticProofFact], hash: &mut FingerprintBuilder) -> Result<(), String> {
        let mut fingerprints = Vec::with_capacity(facts.len());
        for fact in facts {
            let mut item = FingerprintBuilder::new();
            match fact {
                CanonicalStaticProofFact::Equal(left, right) => {
                    item.write_u8(0);
                    let (a, b) = (self.ty(*left)?, self.ty(*right)?);
                    item.write_fingerprint(a.min(b));
                    item.write_fingerprint(a.max(b));
                }
                CanonicalStaticProofFact::Subtype { lower, upper } => {
                    item.write_u8(1);
                    item.write_fingerprint(self.ty(*lower)?);
                    item.write_fingerprint(self.ty(*upper)?);
                }
                CanonicalStaticProofFact::Conforms { key } => {
                    item.write_u8(2);
                    self.conformance(key, &mut item)?;
                }
            }
            fingerprints.push(item.finish());
        }
        fingerprints.sort_unstable();
        fingerprints.dedup();
        hash.write_u32(fingerprints.len() as u32);
        for fingerprint in fingerprints {
            hash.write_fingerprint(fingerprint);
        }
        Ok(())
    }

    fn trusted_context(&mut self, context: &StaticTrustedBranchContext, hash: &mut FingerprintBuilder) -> Result<(), String> {
        let mut bindings = Vec::with_capacity(context.type_bindings().len());
        for (param, ty) in context.type_bindings() {
            let mut item = FingerprintBuilder::new();
            self.parameter(*param, &mut item)?;
            item.write_fingerprint(self.ty(*ty)?);
            bindings.push(item.finish());
        }
        bindings.sort_unstable();
        hash.write_u32(bindings.len() as u32);
        for binding in bindings {
            hash.write_fingerprint(binding);
        }
        self.facts(context.constraints(), hash)
    }

    fn type_term(&mut self, term: &TypeTerm, hash: &mut FingerprintBuilder) -> Result<(), String> {
        if matches!(term, TypeTerm::Infer(_)) {
            return Err("inference-local type term cannot enter a published static fingerprint".into());
        }
        let id = self.types.export_type_term(term).map_err(|error| error.to_string())?;
        hash.write_fingerprint(self.types.type_node_fingerprint(id));
        Ok(())
    }

    fn conformance(&mut self, key: &GenericConformanceKey, hash: &mut FingerprintBuilder) -> Result<(), String> {
        self.type_term(&key.subject, hash)?;
        hash.write_fingerprint(self.declaration(&key.trait_ref.declaration));
        hash.write_u32(key.trait_ref.arguments.len() as u32);
        for ty in key.trait_ref.arguments.iter() {
            hash.write_fingerprint(self.ty(*ty)?);
        }
        Ok(())
    }

    fn variant(&self, variant: &VariantId, hash: &mut FingerprintBuilder) {
        hash.write_fingerprint(self.declaration(&variant.owner));
        hash.write_str(&variant.selector.encode());
    }

    fn member(&mut self, projection: &MemberProjection, hash: &mut FingerprintBuilder) -> Result<(), String> {
        match projection.subject {
            MemberProjectionSubject::Type(ty) => {
                hash.write_u8(0);
                hash.write_fingerprint(self.ty(ty)?);
            }
            MemberProjectionSubject::RecordRow(row) => {
                hash.write_u8(1);
                hash.write_fingerprint(self.types.record_row_fingerprint(row).map_err(|error| error.to_string())?);
            }
            MemberProjectionSubject::Value(value) => {
                hash.write_u8(2);
                hash.write_fingerprint(self.value(value)?);
            }
        }
        match &projection.member {
            MemberDeclarationIdentity::TypeMember(member) => {
                hash.write_u8(0);
                hash.write_fingerprint(self.declaration(&member.owner));
                match &member.selector {
                    TypeMemberSelector::Named(name) => {
                        hash.write_u8(0);
                        hash.write_str(name);
                    }
                    TypeMemberSelector::Subscript => hash.write_u8(1),
                }
            }
            MemberDeclarationIdentity::TraitAssociated { trait_ref, requirement } => {
                hash.write_u8(1);
                hash.write_fingerprint(self.declaration(&trait_ref.declaration));
                hash.write_u32(trait_ref.arguments.len() as u32);
                for ty in trait_ref.arguments.iter() {
                    hash.write_fingerprint(self.ty(*ty)?);
                }
                hash.write_fingerprint(self.declaration(&requirement.owner));
                hash.write_u32(requirement.index);
            }
            MemberDeclarationIdentity::Intrinsic(operation) => {
                hash.write_u8(2);
                hash.write_str(&format!("{operation:?}"));
            }
        }
        hash.write_u32(projection.arguments.len() as u32);
        for argument in projection.arguments.iter() {
            match argument {
                MemberProjectionArgument::Kinded(TypeMemberArgument::Type(ty)) => {
                    hash.write_u8(0);
                    hash.write_fingerprint(self.ty(*ty)?);
                }
                MemberProjectionArgument::Kinded(TypeMemberArgument::Const(ConstTerm::Int(value))) => {
                    hash.write_u8(1);
                    hash.write_str(&value.to_string());
                }
                MemberProjectionArgument::Kinded(TypeMemberArgument::RecordRow(row)) => {
                    hash.write_u8(2);
                    hash.write_fingerprint(self.types.record_row_fingerprint(*row).map_err(|error| error.to_string())?);
                }
                MemberProjectionArgument::Value(value) => {
                    hash.write_u8(3);
                    hash.write_fingerprint(self.value(*value)?);
                }
            }
        }
        match &projection.evidence {
            Some(key) => {
                hash.write_u8(1);
                self.conformance(key, hash)?;
            }
            None => hash.write_u8(0),
        }
        self.domain(projection.result_domain, hash)
    }

    fn term(&mut self, term: StaticEvalTerm, hash: &mut FingerprintBuilder) -> Result<(), String> {
        match term {
            StaticEvalTerm::Value(value) => {
                hash.write_u8(0);
                hash.write_fingerprint(self.value(value)?);
            }
            StaticEvalTerm::Kinded(ty) => {
                hash.write_u8(1);
                hash.write_fingerprint(self.ty(ty)?);
            }
            StaticEvalTerm::Residual(reference) => {
                hash.write_u8(2);
                hash.write_fingerprint(self.residual(reference)?);
            }
        }
        Ok(())
    }

    pub(crate) fn residual(&mut self, reference: StaticResidualRef) -> Result<Fingerprint128, String> {
        if let Some(&cached) = self.residual_memo.get(&reference) {
            return Ok(cached);
        }
        let mut work = vec![(reference, false)];
        while let Some((current, postorder)) = work.pop() {
            if self.residual_memo.contains_key(&current) {
                continue;
            }
            if postorder {
                let fingerprint = self.hash_residual_node(current)?;
                self.residual_memo.insert(current, fingerprint);
                continue;
            }
            work.push((current, true));
            let store = self.residuals.ok_or("published symbolic product has no owning residual store")?;
            let node = store.get(current).map_err(|error| error.to_string())?;
            let mut push_term = |term: StaticEvalTerm| {
                if let StaticEvalTerm::Residual(child) = term {
                    work.push((child, false));
                }
            };
            match node {
                StaticResidualData::Apply { application, .. } => application.arguments.iter().copied().for_each(&mut push_term),
                StaticResidualData::If {
                    condition,
                    then_term,
                    else_term,
                    ..
                } => {
                    for term in [condition, then_term, else_term] {
                        push_term(*term);
                    }
                }
                StaticResidualData::Match { semantics, .. } => {
                    push_term(semantics.scrutinee);
                    semantics.arms.iter().for_each(|arm| push_term(arm.body));
                }
                StaticResidualData::Pack {
                    positionals, labeled_values, ..
                } => {
                    positionals.iter().chain(labeled_values.iter()).copied().for_each(&mut push_term);
                }
                StaticResidualData::Tuple { elements, .. } => elements.iter().copied().for_each(&mut push_term),
                StaticResidualData::Record { fields, .. } => fields.iter().for_each(|(_, term)| push_term(*term)),
                StaticResidualData::Data { components, .. } => components.iter().for_each(|(_, term)| push_term(*term)),
                StaticResidualData::Variant { fields, .. } => fields.iter().for_each(|(_, term)| push_term(*term)),
                StaticResidualData::Parameter { .. } | StaticResidualData::Proof { .. } | StaticResidualData::MemberProjection { .. } => {}
            }
        }
        self.residual_memo.get(&reference).copied().ok_or_else(|| "unfingerprinted residual".into())
    }

    fn hash_residual_node(&mut self, reference: StaticResidualRef) -> Result<Fingerprint128, String> {
        let store = self.residuals.ok_or("published symbolic product has no owning residual store")?;
        let data = store.get(reference).map_err(|error| error.to_string())?.clone();
        let trusted = store.trusted_context(reference).map_err(|error| error.to_string())?.clone();
        let mut hash = FingerprintBuilder::new();
        match data {
            StaticResidualData::Parameter { parameter } => {
                hash.write_u8(0);
                hash.write_fingerprint(self.declaration(&parameter.owner));
                hash.write_u32(parameter.index);
                self.domain(parameter.domain, &mut hash)?;
            }
            StaticResidualData::Apply { application, result_domain } => {
                hash.write_u8(1);
                hash.write_fingerprint(self.declaration(&application.definition));
                self.domain(result_domain, &mut hash)?;
                self.environment(&application.environment, &mut hash)?;
                hash.write_u32(application.arguments.len() as u32);
                for argument in application.arguments.iter() {
                    self.term(*argument, &mut hash)?;
                }
            }
            StaticResidualData::Proof {
                goal,
                result_type,
                environment,
            } => {
                hash.write_u8(2);
                hash.write_fingerprint(self.ty(result_type)?);
                self.environment(&environment, &mut hash)?;
                match goal {
                    ProofPredicateGoal::Equivalent { left, right } => {
                        hash.write_u8(0);
                        let (a, b) = (self.ty(left)?, self.ty(right)?);
                        hash.write_fingerprint(a.min(b));
                        hash.write_fingerprint(a.max(b));
                    }
                    ProofPredicateGoal::Subtype { lower, upper } => {
                        hash.write_u8(1);
                        hash.write_fingerprint(self.ty(lower)?);
                        hash.write_fingerprint(self.ty(upper)?);
                    }
                    ProofPredicateGoal::Conforms { key } => {
                        hash.write_u8(2);
                        self.conformance(&key, &mut hash)?;
                    }
                }
            }
            StaticResidualData::MemberProjection {
                projection,
                result_domain,
                environment,
            } => {
                hash.write_u8(3);
                self.domain(result_domain, &mut hash)?;
                self.environment(&environment, &mut hash)?;
                self.member(&projection, &mut hash)?;
            }
            StaticResidualData::If { .. } => return Err("If residual has no accepted standalone specialization contract".into()),
            StaticResidualData::Match { .. } => {
                return Err("Match residual lacks standalone checked pattern identity for stable publication".into());
            }
            StaticResidualData::Pack {
                positionals,
                labels,
                labeled_values,
                result_type,
            } => {
                if labels.len() != labeled_values.len() {
                    return Err("Pack residual has mismatched labeled lanes".into());
                }
                hash.write_u8(6);
                hash.write_fingerprint(self.ty(result_type)?);
                hash.write_u32(positionals.len() as u32);
                for term in positionals.iter() {
                    self.term(*term, &mut hash)?;
                }
                hash.write_u32(labels.len() as u32);
                for (label, term) in labels.iter().zip(labeled_values.iter()) {
                    hash.write_str(label);
                    self.term(*term, &mut hash)?;
                }
            }
            StaticResidualData::Tuple { elements, result_type } => {
                hash.write_u8(7);
                hash.write_fingerprint(self.ty(result_type)?);
                for term in elements.iter() {
                    self.term(*term, &mut hash)?;
                }
            }
            StaticResidualData::Record { fields, result_type } => {
                hash.write_u8(8);
                hash.write_fingerprint(self.ty(result_type)?);
                for (key, term) in fields.iter() {
                    hash.write_bytes(&key.stable_fingerprint_bytes());
                    self.term(*term, &mut hash)?;
                }
            }
            StaticResidualData::Data {
                owner,
                constructor,
                result_type,
                components,
            } => {
                hash.write_u8(9);
                hash.write_fingerprint(self.declaration(&owner));
                hash.write_fingerprint(self.declaration(&constructor.owner));
                hash.write_fingerprint(self.ty(result_type)?);
                for (component, term) in components.iter() {
                    hash.write_fingerprint(self.declaration(&component.owner));
                    hash.write_u32(component.index);
                    self.term(*term, &mut hash)?;
                }
            }
            StaticResidualData::Variant { variant, result_type, fields } => {
                hash.write_u8(10);
                self.variant(&variant, &mut hash);
                hash.write_fingerprint(self.ty(result_type)?);
                for (field, term) in fields.iter() {
                    self.variant(&field.variant, &mut hash);
                    hash.write_u32(field.index);
                    self.term(*term, &mut hash)?;
                }
            }
        }
        self.trusted_context(&trusted, &mut hash)?;
        Ok(hash.finish())
    }

    pub(crate) fn value(&mut self, reference: StaticValueRef) -> Result<Fingerprint128, String> {
        if let Some(&cached) = self.value_memo.get(&reference) {
            return Ok(cached);
        }
        let mut work = vec![(reference, false)];
        while let Some((current, postorder)) = work.pop() {
            if self.value_memo.contains_key(&current) {
                continue;
            }
            if postorder {
                let fingerprint = self.hash_value_node(current)?;
                self.value_memo.insert(current, fingerprint);
                continue;
            }
            work.push((current, true));
            let data = self.values.get(current).map_err(|error| error.to_string())?;
            match data {
                StaticValueData::Tuple(id) => {
                    for child in self.values.get_tuple(*id).map_err(|error| error.to_string())? {
                        work.push((*child, false));
                    }
                }
                StaticValueData::Sequence(id) => {
                    for child in self.values.get_sequence(*id).map_err(|error| error.to_string())?.elements() {
                        work.push((*child, false));
                    }
                }
                StaticValueData::Record(id) => {
                    for field in self.values.get_record(*id).map_err(|error| error.to_string())? {
                        work.push((field.value, false));
                    }
                }
                StaticValueData::Data(id) => {
                    for (_, child) in self.values.get_data(*id).map_err(|error| error.to_string())?.components.iter() {
                        work.push((*child, false));
                    }
                }
                StaticValueData::Variant(id) => {
                    for (_, child) in self.values.get_variant(*id).map_err(|error| error.to_string())?.fields.iter() {
                        work.push((*child, false));
                    }
                }
                StaticValueData::Pack(id) => {
                    for child in self.values.pack_positionals(*id).map_err(|error| error.to_string())? {
                        work.push((*child, false));
                    }
                    for (_, child) in self.values.pack_labeled(*id).map_err(|error| error.to_string())? {
                        work.push((child, false));
                    }
                }
                _ => {}
            }
        }
        self.value_memo.get(&reference).copied().ok_or_else(|| "unfingerprinted static value".into())
    }

    fn hash_value_node(&mut self, reference: StaticValueRef) -> Result<Fingerprint128, String> {
        let data = self.values.get(reference).map_err(|error| error.to_string())?.clone();
        let mut hash = FingerprintBuilder::new();
        match data {
            StaticValueData::Unit => hash.write_u8(0),
            StaticValueData::Bool(value) => {
                hash.write_u8(1);
                hash.write_u8(u8::from(value));
            }
            StaticValueData::Char(value) => {
                hash.write_u8(2);
                hash.write_u32(value as u32);
            }
            StaticValueData::Int(value) => {
                hash.write_u8(3);
                hash.write_bytes(&value.to_bigint().to_signed_bytes_le());
            }
            StaticValueData::Float(value) => {
                hash.write_u8(4);
                hash.write_u64(value.bits());
            }
            StaticValueData::String(id) => {
                hash.write_u8(5);
                hash.write_str(self.values.get_string(id).ok_or("missing static String")?);
            }
            StaticValueData::Symbol(id) => {
                hash.write_u8(6);
                hash.write_str(self.values.get_symbol(id).ok_or("missing static Symbol")?);
            }
            StaticValueData::Type(ty) => {
                hash.write_u8(7);
                hash.write_fingerprint(self.ty(ty)?);
            }
            StaticValueData::StaticTerm(ty) => {
                hash.write_u8(8);
                hash.write_fingerprint(self.ty(ty)?);
            }
            StaticValueData::Tuple(id) => {
                hash.write_u8(9);
                let children = self.values.get_tuple(id).map_err(|error| error.to_string())?.to_vec();
                hash.write_u32(children.len() as u32);
                for child in children {
                    hash.write_fingerprint(self.value(child)?);
                }
            }
            StaticValueData::Sequence(id) => {
                hash.write_u8(10);
                let sequence = self.values.get_sequence(id).map_err(|error| error.to_string())?;
                let (family, result_type, children) = (sequence.family(), sequence.result_type(), sequence.elements().to_vec());
                hash.write_u8(match family {
                    crate::static_value::StaticSequenceFamily::Array => 0,
                    crate::static_value::StaticSequenceFamily::Vector => 1,
                });
                hash.write_fingerprint(self.ty(result_type)?);
                for child in children {
                    hash.write_fingerprint(self.value(child)?);
                }
            }
            StaticValueData::Record(id) => {
                hash.write_u8(11);
                let fields = self.values.get_record(id).map_err(|error| error.to_string())?.to_vec();
                hash.write_u32(fields.len() as u32);
                for field in fields {
                    hash.write_bytes(&field.key.stable_fingerprint_bytes());
                    hash.write_fingerprint(self.value(field.value)?);
                }
            }
            StaticValueData::Data(id) => {
                hash.write_u8(12);
                let data = self.values.get_data(id).map_err(|error| error.to_string())?.clone();
                hash.write_fingerprint(self.declaration(&data.owner));
                hash.write_fingerprint(self.declaration(&data.constructor.owner));
                hash.write_fingerprint(self.ty(data.result_type)?);
                for (component, value) in data.components.iter() {
                    hash.write_fingerprint(self.declaration(&component.owner));
                    hash.write_u32(component.index);
                    hash.write_fingerprint(self.value(*value)?);
                }
            }
            StaticValueData::Variant(id) => {
                hash.write_u8(13);
                let data = self.values.get_variant(id).map_err(|error| error.to_string())?.clone();
                self.variant(&data.variant, &mut hash);
                hash.write_fingerprint(self.ty(data.result_type)?);
                for (field, value) in data.fields.iter() {
                    self.variant(&field.variant, &mut hash);
                    hash.write_u32(field.index);
                    hash.write_fingerprint(self.value(*value)?);
                }
            }
            StaticValueData::Pack(id) => {
                hash.write_u8(14);
                let positionals = self.values.pack_positionals(id).map_err(|error| error.to_string())?.to_vec();
                let labeled = self.values.pack_labeled(id).map_err(|error| error.to_string())?.to_vec();
                hash.write_u32(positionals.len() as u32);
                for value in positionals {
                    hash.write_fingerprint(self.value(value)?);
                }
                hash.write_u32(labeled.len() as u32);
                for (symbol, value) in labeled {
                    hash.write_str(self.values.get_symbol(symbol).ok_or("missing Pack label")?);
                    hash.write_fingerprint(self.value(value)?);
                }
            }
        }
        Ok(hash.finish())
    }
}
