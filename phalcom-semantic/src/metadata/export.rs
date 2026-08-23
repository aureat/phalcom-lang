//! Canonical exporter: projects compiler-owned semantic types, signatures, and declarations
//! into a stable, deduplicated, versioned `SemanticMetadataBundle`.

use super::stable_identity::*;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{CallableId, DeclarationId, FieldId};
use crate::signature::{CallableSignatureTable, FieldSignatureTable};
use crate::types::id::{KindId, ScopedTypeId, TypeId, TypeLambdaId, TypeParameterId};
use crate::types::kind::KindData;
use crate::types::parameter::{GenericConstraint, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm};
use crate::types::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore};
use crate::types::type_lambda::{ScopedCallableParameter, ScopedCallableType, ScopedRecordField, ScopedTupleElement, ScopedTypeData, TypeLambdaData};
use crate::types::variance::Variance;
use phalcom_type_meta::bundle::{ModuleMetadataRoot, RuntimeTypeFormKey, RuntimeTypeFormRoot, SemanticMetadataBundle};
use phalcom_type_meta::declaration::{
    AliasRecordId, CallableParameterRecord, CallableRecordId, CallableSemanticRecord, DeclarationRecordId, DeclarationTypeFlags, DeclarationTypeRecord,
    FieldMutabilityRef, FieldRecordId, FieldSemanticRecord, MetadataUnavailableReason, PublishedTypeAuthority, PublishedTypeSlot, RestModeRef, TypeAliasRecord,
};
use phalcom_type_meta::fingerprint::{Fingerprint128, FingerprintBuilder};
use phalcom_type_meta::generic::{
    GenericConstraintRef, GenericSignatureRecord, GenericSignatureRecordId, StableTypeParameterOwnerRef, StableTypeParameterRef, TypeParameterRecord,
    VarianceRef,
};
use phalcom_type_meta::header::{
    ArtifactIdentityScheme, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity, SEMANTIC_MODEL_VERSION, SemanticMetadataHeader,
    TYPE_METADATA_SCHEMA_VERSION,
};
use phalcom_type_meta::identity::SourceSpanRef;
use phalcom_type_meta::kind::{KindNode, KindNodeEntry, KindNodeId};
use phalcom_type_meta::scoped_type::{
    ScopedCallableParamRef, ScopedCallableTypeRef, ScopedRecordFieldRef, ScopedRecordTailRef, ScopedTupleElementRef, ScopedTypeNode, ScopedTypeNodeEntry,
    ScopedTypeNodeId, TypeLambdaRef,
};
use phalcom_type_meta::type_node::{
    CallableParamRef, CallableTypeRef, RecordFieldRef, SelfRoleRef, SelfTypeRef, TupleElementRef, TypeNode, TypeNodeEntry, TypeNodeId,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MetadataExportError {
    #[error("cannot export inference variable in durable metadata")]
    InferenceVariable,
    #[error("non-exportable internal form: {0:?}")]
    NonExportableForm(TypeId),
}

/// Exporter context driving hash-consing and topological node indexing.
pub struct MetadataExporter<'a> {
    store: &'a TypeStore,
    declarations: Option<&'a DeclarationTypeTable>,
    callables: Option<&'a CallableSignatureTable>,
    fields: Option<&'a FieldSignatureTable>,
    profile: MetadataProfile,

    // Deduplication index maps
    kind_map: HashMap<KindData, KindNodeId>,
    kinds: Vec<KindNodeEntry>,

    type_map: HashMap<TypeNode, TypeNodeId>,
    types: Vec<TypeNodeEntry>,

    scoped_map: HashMap<ScopedTypeNode, ScopedTypeNodeId>,
    scoped_types: Vec<ScopedTypeNodeEntry>,

    parameters: Vec<TypeParameterRecord>,
    param_map: HashMap<TypeParameterId, StableTypeParameterRef>,

    generic_signatures: Vec<GenericSignatureRecord>,
    sig_map: HashMap<(TypeParameterOwner, usize), GenericSignatureRecordId>,
}

impl<'a> MetadataExporter<'a> {
    pub fn new(
        store: &'a TypeStore,
        declarations: Option<&'a DeclarationTypeTable>,
        callables: Option<&'a CallableSignatureTable>,
        fields: Option<&'a FieldSignatureTable>,
        profile: MetadataProfile,
    ) -> Self {
        Self {
            store,
            declarations,
            callables,
            fields,
            profile,
            kind_map: HashMap::new(),
            kinds: Vec::new(),
            type_map: HashMap::new(),
            types: Vec::new(),
            scoped_map: HashMap::new(),
            scoped_types: Vec::new(),
            parameters: Vec::new(),
            param_map: HashMap::new(),
            generic_signatures: Vec::new(),
            sig_map: HashMap::new(),
        }
    }

    pub fn export_kind(&mut self, kind: KindId) -> KindNodeId {
        let data = self.store.get_kind(kind).clone();
        if let Some(&id) = self.kind_map.get(&data) {
            return id;
        }

        let node = match data {
            KindData::Type => KindNode::Type,
            KindData::RecordRow => KindNode::RecordRow,
            KindData::Arrow { ref parameters, result } => {
                let p_nodes: Vec<KindNodeId> = parameters.iter().map(|&p| self.export_kind(p)).collect();
                let r_node = self.export_kind(result);
                KindNode::Arrow {
                    parameters: p_nodes.into_boxed_slice(),
                    result: r_node,
                }
            }
        };

        let mut fp_b = FingerprintBuilder::new();
        match &node {
            KindNode::Type => fp_b.write_u8(1),
            KindNode::RecordRow => fp_b.write_u8(3),
            KindNode::Arrow { parameters, result } => {
                fp_b.write_u8(2);
                fp_b.write_u32(parameters.len() as u32);
                for &p in parameters.iter() {
                    fp_b.write_fingerprint(self.kinds[p.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.kinds[result.0 as usize].structural_fingerprint);
            }
        }
        let structural_fingerprint = fp_b.finish();

        let id = KindNodeId(self.kinds.len() as u32);
        self.kinds.push(KindNodeEntry { node, structural_fingerprint });
        self.kind_map.insert(data, id);
        id
    }

    pub fn export_type_parameter(&mut self, param: TypeParameterId) -> StableTypeParameterRef {
        if let Some(r) = self.param_map.get(&param) {
            return r.clone();
        }
        let data = self.store.type_parameter(param);
        let owner_ref = match &data.owner {
            TypeParameterOwner::Declaration(d) => StableTypeParameterOwnerRef::Declaration(to_stable_declaration(d)),
            TypeParameterOwner::Callable(c) => StableTypeParameterOwnerRef::Callable(to_stable_callable(c)),
        };
        let param_ref = StableTypeParameterRef {
            owner: owner_ref,
            index: data.index,
        };
        let kind_id = self.export_kind(data.kind);
        let variance_ref = match data.variance {
            Variance::Covariant => VarianceRef::Covariant,
            Variance::Contravariant => VarianceRef::Contravariant,
            Variance::Invariant => VarianceRef::Invariant,
        };
        let source_span = data.source.as_ref().map(|s| SourceSpanRef {
            start: s.range.start as u32,
            end: s.range.end as u32,
        });
        self.parameters.push(TypeParameterRecord {
            id: param_ref.clone(),
            name: data.name.clone(),
            kind: kind_id,
            variance: variance_ref,
            source: source_span,
        });
        self.param_map.insert(param, param_ref.clone());
        param_ref
    }

    pub fn export_scoped_type(&mut self, scoped_id: ScopedTypeId) -> Result<ScopedTypeNodeId, MetadataExportError> {
        let scoped_data = self.store.arena().get_scoped(scoped_id).clone();
        let scoped_node = match scoped_data {
            ScopedTypeData::Bound { depth, index } => ScopedTypeNode::Bound { depth, index },
            ScopedTypeData::Free(ty) => {
                let global_id = self.export_type_form(ty)?;
                ScopedTypeNode::Free(global_id)
            }
            ScopedTypeData::Applied { origin, ref arguments } => {
                let o = self.export_scoped_type(origin)?;
                let mut args = Vec::new();
                for &arg in arguments.iter() {
                    args.push(self.export_scoped_type(arg)?);
                }
                ScopedTypeNode::Applied {
                    origin: o,
                    arguments: args.into_boxed_slice(),
                }
            }
            ScopedTypeData::Union(ref members) => {
                let mut m_nodes = Vec::new();
                for &m in members.iter() {
                    m_nodes.push(self.export_scoped_type(m)?);
                }
                ScopedTypeNode::Union(m_nodes.into_boxed_slice())
            }
            ScopedTypeData::Tuple(ref elements) => {
                let mut elems = Vec::new();
                for el in elements.iter() {
                    let ty = self.export_scoped_type(el.ty)?;
                    elems.push(ScopedTupleElementRef { label: el.label.clone(), ty });
                }
                ScopedTypeNode::Tuple(elems.into_boxed_slice())
            }
            ScopedTypeData::Record(ref fields) => {
                let mut f_nodes = Vec::new();
                for f in fields.iter() {
                    let ty = self.export_scoped_type(f.ty)?;
                    f_nodes.push(ScopedRecordFieldRef { name: f.name.clone(), ty });
                }
                ScopedTypeNode::Record(f_nodes.into_boxed_slice())
            }
            ScopedTypeData::Callable(ref call) => {
                let mut params = Vec::new();
                for p in call.parameters.iter() {
                    let ty = self.export_scoped_type(p.ty)?;
                    params.push(ScopedCallableParamRef {
                        label: p.label.clone(),
                        ty,
                        rest: p.rest,
                    });
                }
                let return_type = self.export_scoped_type(call.return_type)?;
                ScopedTypeNode::Callable(ScopedCallableTypeRef {
                    parameters: params.into_boxed_slice(),
                    return_type,
                })
            }
            ScopedTypeData::Lambda(lam_id) => {
                let lam_data = self.store.arena().get_lambda(lam_id).clone();
                let p_kinds: Vec<KindNodeId> = lam_data.parameter_kinds.iter().map(|&pk| self.export_kind(pk)).collect();
                let body = self.export_scoped_type(lam_data.body)?;
                ScopedTypeNode::Lambda {
                    parameter_kinds: p_kinds.into_boxed_slice(),
                    body,
                }
            }
        };

        if let Some(&id) = self.scoped_map.get(&scoped_node) {
            return Ok(id);
        }

        let mut fp_b = FingerprintBuilder::new();
        match &scoped_node {
            ScopedTypeNode::Bound { depth, index } => {
                fp_b.write_u8(1);
                fp_b.write_u32(*depth);
                fp_b.write_u32(*index);
            }
            ScopedTypeNode::Free(t) => {
                fp_b.write_u8(2);
                fp_b.write_fingerprint(self.types[t.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::Applied { origin, arguments } => {
                fp_b.write_u8(3);
                fp_b.write_fingerprint(self.scoped_types[origin.0 as usize].structural_fingerprint);
                fp_b.write_u32(arguments.len() as u32);
                for arg in arguments.iter() {
                    fp_b.write_fingerprint(self.scoped_types[arg.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Union(members) => {
                fp_b.write_u8(4);
                fp_b.write_u32(members.len() as u32);
                for m in members.iter() {
                    fp_b.write_fingerprint(self.scoped_types[m.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Tuple(elements) => {
                fp_b.write_u8(5);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    fp_b.write_fingerprint(self.scoped_types[el.ty.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Record(fields) => {
                fp_b.write_u8(6);
                fp_b.write_u32(fields.len() as u32);
                for f in fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_fingerprint(self.scoped_types[f.ty.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Callable(call) => {
                fp_b.write_u8(7);
                fp_b.write_u32(call.parameters.len() as u32);
                for p in call.parameters.iter() {
                    fp_b.write_u8(if p.rest { 1 } else { 0 });
                    fp_b.write_fingerprint(self.scoped_types[p.ty.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.scoped_types[call.return_type.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::Lambda { parameter_kinds, body } => {
                fp_b.write_u8(8);
                fp_b.write_u32(parameter_kinds.len() as u32);
                for pk in parameter_kinds.iter() {
                    fp_b.write_fingerprint(self.kinds[pk.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.scoped_types[body.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::OpenRecord(open_rec) => {
                fp_b.write_u8(9);
                fp_b.write_u32(open_rec.fields.len() as u32);
                for f in open_rec.fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_fingerprint(self.scoped_types[f.ty.0 as usize].structural_fingerprint);
                }
                match &open_rec.tail {
                    ScopedRecordTailRef::Bound { depth, index } => {
                        fp_b.write_u8(1);
                        fp_b.write_u32(*depth);
                        fp_b.write_u32(*index);
                    }
                    ScopedRecordTailRef::FreeParameter(p) => {
                        fp_b.write_u8(2);
                        fp_b.write_u32(p.index);
                    }
                }
            }
        }
        let structural_fingerprint = fp_b.finish();
        let kind_id = self.export_kind(self.store.kind_of(self.store.unit()));
        let id = ScopedTypeNodeId(self.scoped_types.len() as u32);
        self.scoped_types.push(ScopedTypeNodeEntry {
            kind: kind_id,
            form: scoped_node.clone(),
            structural_fingerprint,
        });
        self.scoped_map.insert(scoped_node, id);
        Ok(id)
    }

    pub fn export_type_form(&mut self, ty: TypeId) -> Result<TypeNodeId, MetadataExportError> {
        let kind_id = self.export_kind(self.store.kind_of(ty));
        let data = self.store.get(ty).clone();

        let form = match data {
            TypeData::Never => TypeNode::Never,
            TypeData::Unit => TypeNode::Unit,
            TypeData::Nominal { declaration } => TypeNode::Nominal {
                declaration: to_stable_declaration(&declaration),
            },
            TypeData::Applied { origin, ref arguments } => {
                let origin_id = self.export_type_form(origin)?;
                let mut arg_ids = Vec::new();
                for &arg in arguments.iter() {
                    arg_ids.push(self.export_type_form(arg)?);
                }
                TypeNode::Applied {
                    origin: origin_id,
                    arguments: arg_ids.into_boxed_slice(),
                }
            }
            TypeData::Union(ref members) => {
                let mut member_ids = Vec::new();
                for &m in members.iter() {
                    member_ids.push(self.export_type_form(m)?);
                }
                TypeNode::Union(member_ids.into_boxed_slice())
            }
            TypeData::Tuple(ref elements) => {
                let mut elem_refs = Vec::new();
                for el in elements.iter() {
                    let ty_id = self.export_type_form(el.ty)?;
                    elem_refs.push(TupleElementRef {
                        label: el.label.clone(),
                        ty: ty_id,
                    });
                }
                TypeNode::Tuple(elem_refs.into_boxed_slice())
            }
            TypeData::Record(ref fields) => {
                let mut field_refs = Vec::new();
                for f in fields.iter() {
                    let ty_id = self.export_type_form(f.ty)?;
                    field_refs.push(RecordFieldRef {
                        name: f.name.clone(),
                        ty: ty_id,
                    });
                }
                TypeNode::Record(field_refs.into_boxed_slice())
            }
            TypeData::Callable(ref call) => {
                let mut params = Vec::new();
                for p in call.parameters.iter() {
                    let ty_id = self.export_type_form(p.ty)?;
                    params.push(CallableParamRef {
                        label: p.label.clone(),
                        ty: ty_id,
                        rest: p.rest,
                    });
                }
                let return_type = self.export_type_form(call.return_type)?;
                TypeNode::Callable(CallableTypeRef {
                    parameters: params.into_boxed_slice(),
                    return_type,
                })
            }
            TypeData::Parameter(param) => {
                let stable_param = self.export_type_parameter(param);
                TypeNode::Parameter(stable_param)
            }
            TypeData::SelfType(s) => TypeNode::SelfType(SelfTypeRef {
                owner: to_stable_declaration(&s.owner),
                side: to_stable_dispatch_side(s.side),
                role: match s.role {
                    SelfRole::InstanceType => SelfRoleRef::InstanceType,
                    SelfRole::ReceiverValue => SelfRoleRef::ReceiverValue,
                },
            }),
            TypeData::Lambda(lam_id) => {
                let lam_data = self.store.arena().get_lambda(lam_id).clone();
                let p_kinds: Vec<KindNodeId> = lam_data.parameter_kinds.iter().map(|&pk| self.export_kind(pk)).collect();
                let body = self.export_scoped_type(lam_data.body)?;
                TypeNode::TypeLambda(TypeLambdaRef {
                    parameter_kinds: p_kinds.into_boxed_slice(),
                    body,
                })
            }
            TypeData::ClassObject { .. } => return Err(MetadataExportError::NonExportableForm(ty)),
        };

        if let Some(&id) = self.type_map.get(&form) {
            return Ok(id);
        }

        let mut fp_b = FingerprintBuilder::new();
        match &form {
            TypeNode::Never => fp_b.write_u8(1),
            TypeNode::Unit => fp_b.write_u8(2),
            TypeNode::Nominal { declaration } => {
                fp_b.write_u8(3);
                for c in declaration.path.iter() {
                    fp_b.write_str(c);
                }
            }
            TypeNode::Applied { origin, arguments } => {
                fp_b.write_u8(4);
                fp_b.write_fingerprint(self.types[origin.0 as usize].structural_fingerprint);
                fp_b.write_u32(arguments.len() as u32);
                for arg in arguments.iter() {
                    fp_b.write_fingerprint(self.types[arg.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::Union(members) => {
                fp_b.write_u8(5);
                fp_b.write_u32(members.len() as u32);
                for m in members.iter() {
                    fp_b.write_fingerprint(self.types[m.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::Tuple(elements) => {
                fp_b.write_u8(6);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    fp_b.write_fingerprint(self.types[el.ty.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::Record(fields) => {
                fp_b.write_u8(7);
                fp_b.write_u32(fields.len() as u32);
                for f in fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_fingerprint(self.types[f.ty.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::OpenRecord(open_rec) => {
                fp_b.write_u8(12);
                fp_b.write_u32(open_rec.fields.len() as u32);
                for f in open_rec.fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_fingerprint(self.types[f.ty.0 as usize].structural_fingerprint);
                }
                fp_b.write_u32(open_rec.tail.index);
            }
            TypeNode::Callable(call) => {
                fp_b.write_u8(8);
                fp_b.write_u32(call.parameters.len() as u32);
                for p in call.parameters.iter() {
                    fp_b.write_u8(if p.rest { 1 } else { 0 });
                    fp_b.write_fingerprint(self.types[p.ty.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.types[call.return_type.0 as usize].structural_fingerprint);
            }
            TypeNode::Parameter(param) => {
                fp_b.write_u8(9);
                fp_b.write_u32(param.index);
            }
            TypeNode::SelfType(s) => {
                fp_b.write_u8(10);
                for c in s.owner.path.iter() {
                    fp_b.write_str(c);
                }
            }
            TypeNode::TypeLambda(lambda) => {
                fp_b.write_u8(11);
                fp_b.write_u32(lambda.parameter_kinds.len() as u32);
                for pk in lambda.parameter_kinds.iter() {
                    fp_b.write_fingerprint(self.kinds[pk.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.scoped_types[lambda.body.0 as usize].structural_fingerprint);
            }
        }
        let structural_fingerprint = fp_b.finish();

        let id = TypeNodeId(self.types.len() as u32);
        self.types.push(TypeNodeEntry {
            kind: kind_id,
            form: form.clone(),
            structural_fingerprint,
        });
        self.type_map.insert(form, id);
        Ok(id)
    }

    pub fn export_type_term(&mut self, term: &TypeTerm) -> Result<TypeNodeId, MetadataExportError> {
        match term {
            TypeTerm::Canonical(ty) => self.export_type_form(*ty),
            TypeTerm::SelfType(s) => {
                let self_node = TypeNode::SelfType(SelfTypeRef {
                    owner: to_stable_declaration(&s.owner),
                    side: to_stable_dispatch_side(s.side),
                    role: match s.role {
                        SelfRole::InstanceType => SelfRoleRef::InstanceType,
                        SelfRole::ReceiverValue => SelfRoleRef::ReceiverValue,
                    },
                });
                if let Some(&id) = self.type_map.get(&self_node) {
                    return Ok(id);
                }
                let kind_id = self.export_kind(self.store.kind_of(self.store.unit()));
                let mut fp_b = FingerprintBuilder::new();
                fp_b.write_u8(10);
                for c in to_stable_declaration(&s.owner).path.iter() {
                    fp_b.write_str(c);
                }
                let structural_fingerprint = fp_b.finish();
                let id = TypeNodeId(self.types.len() as u32);
                self.types.push(TypeNodeEntry {
                    kind: kind_id,
                    form: self_node.clone(),
                    structural_fingerprint,
                });
                self.type_map.insert(self_node, id);
                Ok(id)
            }
            TypeTerm::Infer(_) => Err(MetadataExportError::InferenceVariable),
        }
    }

    pub fn export_generic_signature(&mut self, sig: &GenericSignature) -> Result<GenericSignatureRecordId, MetadataExportError> {
        let key = (sig.owner.clone(), sig.parameters.len());
        if let Some(&id) = self.sig_map.get(&key) {
            return Ok(id);
        }

        let owner_ref = match &sig.owner {
            TypeParameterOwner::Declaration(d) => StableTypeParameterOwnerRef::Declaration(to_stable_declaration(d)),
            TypeParameterOwner::Callable(c) => StableTypeParameterOwnerRef::Callable(to_stable_callable(c)),
        };

        let mut param_refs = Vec::new();
        for &param in sig.parameters.iter() {
            param_refs.push(self.export_type_parameter(param));
        }

        let mut constraint_refs = Vec::new();
        for c in sig.constraints.iter() {
            match c {
                GenericConstraint::Subtype { lower, upper } => {
                    let l = self.export_type_term(lower)?;
                    let u = self.export_type_term(upper)?;
                    constraint_refs.push(GenericConstraintRef::Subtype { lower: l, upper: u });
                }
                GenericConstraint::Equivalent { left, right } => {
                    let l = self.export_type_term(left)?;
                    let r = self.export_type_term(right)?;
                    constraint_refs.push(GenericConstraintRef::Equivalent { left: l, right: r });
                }
            }
        }

        let id = GenericSignatureRecordId(self.generic_signatures.len() as u32);
        self.generic_signatures.push(GenericSignatureRecord {
            owner: owner_ref,
            parameters: param_refs.into_boxed_slice(),
            constraints: constraint_refs.into_boxed_slice(),
        });
        self.sig_map.insert(key, id);
        Ok(id)
    }

    pub fn build_bundle(mut self, runtime_roots: &[(&phalcom_modules::ModuleId, &str, TypeId)]) -> Result<SemanticMetadataBundle, MetadataExportError> {
        let mut out_declarations = Vec::new();
        let mut out_callables = Vec::new();
        let mut out_fields = Vec::new();
        let mut out_roots = Vec::new();

        if let Some(decls) = self.declarations {
            for (decl_id, info) in decls.iter() {
                let form_id = self.export_type_form(info.form)?;
                let kind_id = self.export_kind(info.kind);
                let sig_id = if let Some(ref sig) = info.generic_signature {
                    Some(self.export_generic_signature(sig)?)
                } else {
                    None
                };
                let sup_template = if let Some(ref tmpl) = info.supertype_template {
                    Some(self.export_type_form(tmpl.supertype)?)
                } else {
                    None
                };

                out_declarations.push(DeclarationTypeRecord {
                    declaration: to_stable_declaration(decl_id),
                    form: form_id,
                    kind: kind_id,
                    generic_signature: sig_id,
                    superclass_template: sup_template,
                    instance_callables: Box::new([]),
                    class_callables: Box::new([]),
                    instance_fields: Box::new([]),
                    class_fields: Box::new([]),
                    flags: DeclarationTypeFlags::default(),
                    source: None,
                });
            }
        }

        if let Some(call_table) = self.callables {
            for (call_id, sig) in call_table.iter() {
                let sig_id = if let Some(ref g) = sig.generics {
                    Some(self.export_generic_signature(g)?)
                } else {
                    None
                };
                let mut params = Vec::new();
                for p in sig.parameters.iter() {
                    let ty_slot = match self.export_type_term(&p.ty) {
                        Ok(t) => PublishedTypeSlot::Known {
                            form: t,
                            authority: PublishedTypeAuthority::DeclaredAnnotation,
                        },
                        Err(_) => PublishedTypeSlot::Unavailable {
                            reason: MetadataUnavailableReason::IncompatibleModel,
                        },
                    };
                    params.push(CallableParameterRecord {
                        index: p.index,
                        local_name: p.local_name.clone(),
                        external_label: p.external_label.clone(),
                        rest: match p.rest {
                            phalcom_ast::ast::RestMode::None => RestModeRef::None,
                            phalcom_ast::ast::RestMode::Positional => RestModeRef::Anonymous,
                            phalcom_ast::ast::RestMode::Labeled | phalcom_ast::ast::RestMode::Complete => RestModeRef::Named,
                        },
                        ty: ty_slot,
                        source: None,
                    });
                }
                let return_slot = match self.export_type_term(&sig.return_type) {
                    Ok(t) => PublishedTypeSlot::Known {
                        form: t,
                        authority: PublishedTypeAuthority::DeclaredAnnotation,
                    },
                    Err(_) => PublishedTypeSlot::Unavailable {
                        reason: MetadataUnavailableReason::IncompatibleModel,
                    },
                };
                out_callables.push(CallableSemanticRecord {
                    callable: to_stable_callable(call_id),
                    generic_signature: sig_id,
                    parameters: params.into_boxed_slice(),
                    return_type: return_slot,
                    source: None,
                });
            }
        }

        if let Some(field_table) = self.fields {
            for (field_id, sig) in field_table.iter() {
                let ty_slot = match self.export_type_term(&sig.ty) {
                    Ok(t) => PublishedTypeSlot::Known {
                        form: t,
                        authority: PublishedTypeAuthority::DeclaredAnnotation,
                    },
                    Err(_) => PublishedTypeSlot::Unavailable {
                        reason: MetadataUnavailableReason::IncompatibleModel,
                    },
                };
                out_fields.push(FieldSemanticRecord {
                    field: to_stable_field(field_id),
                    mutability: match sig.mutable {
                        false => FieldMutabilityRef::Immutable,
                        true => FieldMutabilityRef::Mutable,
                    },
                    ty: ty_slot,
                    source: None,
                });
            }
        }

        for &(mod_id, key, ty) in runtime_roots {
            let form_id = self.export_type_form(ty)?;
            out_roots.push(RuntimeTypeFormRoot {
                module: to_stable_module(mod_id),
                local_key: RuntimeTypeFormKey(key.into()),
                form: form_id,
            });
        }

        let header = SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("phalcom-semantic".into()),
            producer_version: "0.1.0".into(),
            native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
            profile: self.profile,
            features: MetadataFeatures {
                type_lambdas: true,
                record_rows: false,
                runtime_type_constants: !out_roots.is_empty(),
                source_occurrences: false,
                advanced_sections: Box::new([]),
            },
            identity_scheme: ArtifactIdentityScheme::V1Standard,
            source_fingerprint: Fingerprint128::ZERO,
            interface_fingerprint: Fingerprint128::ZERO,
        };

        Ok(SemanticMetadataBundle {
            header,
            kinds: self.kinds.into_boxed_slice(),
            types: self.types.into_boxed_slice(),
            scoped_types: self.scoped_types.into_boxed_slice(),
            parameters: self.parameters.into_boxed_slice(),
            generic_signatures: self.generic_signatures.into_boxed_slice(),
            declarations: out_declarations.into_boxed_slice(),
            aliases: Box::new([]),
            callables: out_callables.into_boxed_slice(),
            fields: out_fields.into_boxed_slice(),
            module_roots: Box::new([]),
            runtime_roots: out_roots.into_boxed_slice(),
            occurrences: Box::new([]),
            extensions: Box::new([]),
        })
    }
}
