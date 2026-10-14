//! Reconstructs static values, constants, and types from a validated `SemanticMetadataBundle`.

use crate::identity::{DataComponentId, DeclarationId, VariantFieldId, VariantId};
use crate::metadata::export::MetadataExportError;
use crate::metadata::stable_identity::from_field_key_ref;
use crate::static_constant::{
    ImportedStaticConstantOrigin, StaticConstantId, StaticConstantProduct, StaticConstantTable,
};
use crate::static_value::{StaticFloat, StaticValueRef, StaticValueStore};
use crate::types::id::TypeId;
use crate::types::row::{RecordRowField, RecordRowTail};
use crate::types::store::{StaticArgument, TupleTypeElement, TypeStore};
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ProjectIdentity};
use phalcom_modules::ProjectUniverse;
use phalcom_type_meta::bundle::SemanticMetadataBundle;
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
use phalcom_type_meta::static_value::{StaticValueNode, StaticValueNodeId};
use phalcom_type_meta::type_node::{StaticArgumentRef, TypeNode, TypeNodeId};
use phalcom_type_meta::validate::{validate_metadata_bundle, ValidationLimits};
use std::collections::HashMap;
use std::sync::Arc;

pub fn resolve_stable_module(module: &StableModuleRef, projects: &ProjectUniverse) -> Option<ModuleId> {
    let project = match &module.project {
        StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe" => {
            ProjectIdentity::Universe
        }
        StableProjectRef::SourceArtifact { logical_uri, .. } => projects
            .projects()
            .iter()
            .find(|p| *p.source_identity.0.to_string_lossy() == **logical_uri)
            .map(|p| ProjectIdentity::Resolved(p.id))?,
        StableProjectRef::Session { .. } => {
            ProjectIdentity::Synthetic(phalcom_modules::identity::SyntheticProjectIdAllocator.allocate())
        }
        _ => return None,
    };
    let components = module
        .path
        .iter()
        .map(|seg| ModuleComponent::from_identifier(seg).ok())
        .collect::<Option<Vec<_>>>()?;
    Some(ModuleId {
        project,
        path: ModulePath::from_components(components),
    })
}

pub fn resolve_stable_declaration(decl: &StableDeclarationRef, projects: &ProjectUniverse) -> Option<DeclarationId> {
    let module = resolve_stable_module(&decl.module, projects)?;
    let name = decl.path.last()?.clone();
    Some(DeclarationId::new(module, name))
}

pub struct SemanticMetadataImporter<'a> {
    bundle: &'a SemanticMetadataBundle,
    store: &'a mut StaticValueStore,
    types: &'a mut TypeStore,
    projects: &'a ProjectUniverse,
    type_map: HashMap<TypeNodeId, TypeId>,
    value_map: HashMap<StaticValueNodeId, StaticValueRef>,
}

impl<'a> SemanticMetadataImporter<'a> {
    pub fn new(
        bundle: &'a SemanticMetadataBundle,
        store: &'a mut StaticValueStore,
        types: &'a mut TypeStore,
        projects: &'a ProjectUniverse,
    ) -> Self {
        Self {
            bundle,
            store,
            types,
            projects,
            type_map: HashMap::new(),
            value_map: HashMap::new(),
        }
    }

    pub fn resolve_type(&mut self, node_id: TypeNodeId) -> Result<TypeId, MetadataExportError> {
        if let Some(&existing) = self.type_map.get(&node_id) {
            return Ok(existing);
        }

        let entry = self
            .bundle
            .types
            .get(node_id.0 as usize)
            .ok_or_else(|| MetadataExportError::StaticValue("invalid type node index".into()))?;

        let ty = match &entry.form {
            TypeNode::Unit => self.types.unit(),
            TypeNode::Never => self.types.never(),
            TypeNode::Nominal { declaration } => {
                let decl = resolve_stable_declaration(declaration, self.projects)
                    .ok_or_else(|| MetadataExportError::StaticValue("unknown nominal declaration".into()))?;
                self.types.nominal_type(decl)
            }
            TypeNode::Tuple(elements) => {
                let mut elems = Vec::with_capacity(elements.len());
                for el in elements.iter() {
                    let ty = self.resolve_type(el.ty)?;
                    elems.push(TupleTypeElement {
                        label: el.label.clone(),
                        ty,
                    });
                }
                self.types.tuple(elems.into_boxed_slice())
            }
            TypeNode::Record(fields) => {
                let mut rec_fields = Vec::with_capacity(fields.len());
                for f in fields.iter() {
                    let ty = self.resolve_type(f.ty)?;
                    rec_fields.push(RecordRowField {
                        key: from_field_key_ref(&f.key),
                        ty,
                        presence: crate::types::store::StructuralMemberPresence::Required,
                    });
                }
                self.types
                    .record_row_type_checked(rec_fields, RecordRowTail::Closed)
                    .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
            }
            TypeNode::Pack(pack) => {
                let mut elems = Vec::with_capacity(pack.elements.len());
                for el in pack.elements.iter() {
                    let ty = self.resolve_type(el.ty)?;
                    elems.push(TupleTypeElement {
                        label: el.label.clone(),
                        ty,
                    });
                }
                self.types
                    .pack(elems.into_boxed_slice())
                    .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
            }
            TypeNode::ConstInt(i) => self.types.const_int(i.to_bigint()),
            TypeNode::Union(members) => {
                let mut member_tys = Vec::with_capacity(members.len());
                for m in members.iter() {
                    member_tys.push(self.resolve_type(*m)?);
                }
                self.types.union(&member_tys)
            }
            TypeNode::Applied { origin, arguments } => {
                let origin_ty = self.resolve_type(*origin)?;
                let mut static_args = Vec::with_capacity(arguments.len());
                for arg in arguments.iter() {
                    match arg {
                        StaticArgumentRef::Term(t) => {
                            let ty = self.resolve_type(*t)?;
                            static_args.push(StaticArgument::Term(ty));
                        }
                        StaticArgumentRef::DynamicType => {
                            static_args.push(StaticArgument::DynamicType);
                        }
                        _ => {
                            return Err(MetadataExportError::StaticValue("unsupported applied argument in import".into()));
                        }
                    }
                }
                self.types
                    .apply_static_form(origin_ty, &static_args)
                    .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
            }
            _ => self.types.unit(),
        };

        self.type_map.insert(node_id, ty);
        Ok(ty)
    }

    pub fn import_all(
        mut self,
        constants_table: &mut StaticConstantTable,
    ) -> Result<Vec<StaticValueRef>, MetadataExportError> {
        validate_metadata_bundle(self.bundle, &ValidationLimits::default())
            .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?;

        // Reconstruct static values in topological order:
        for (i, entry) in self.bundle.static_values.iter().enumerate() {
            let node_id = StaticValueNodeId(i as u32);
            let val_ref = match &entry.node {
                StaticValueNode::Unit => self.store.intern_unit(),
                StaticValueNode::Bool(b) => self.store.intern_bool(*b),
                StaticValueNode::Char(c) => self.store.intern_char(*c),
                StaticValueNode::Int(i) => self.store.intern_int(i.to_bigint()),
                StaticValueNode::FloatBits(bits) => self.store.intern_float(StaticFloat::from_bits(*bits)),
                StaticValueNode::String(s) => self.store.intern_string(s),
                StaticValueNode::Symbol(s) => self.store.intern_symbol(s),
                StaticValueNode::Type(node) => {
                    let ty = self.resolve_type(*node)?;
                    self.store
                        .intern_type(ty, self.types)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
                StaticValueNode::StaticTerm(node) => {
                    let ty = self.resolve_type(*node)?;
                    self.store
                        .intern_static_term(ty, self.types)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
                StaticValueNode::Tuple(elements) => {
                    let children = elements.iter().map(|id| self.value_map[id]).collect();
                    self.store
                        .intern_tuple(children)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
                StaticValueNode::Sequence { family: _, result_type, elements } => {
                    let children = elements.iter().map(|id| self.value_map[id]).collect();
                    let res_ty = self.resolve_type(*result_type)?;
                    self.store
                        .intern_sequence(res_ty, children, self.types)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
                StaticValueNode::Record(fields) => {
                    let rec_fields = fields
                        .iter()
                        .map(|(k, v)| (from_field_key_ref(k), self.value_map[v]))
                        .collect();
                    self.store
                        .intern_record(rec_fields)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
                StaticValueNode::Pack { positionals, labeled } => {
                    let pos = positionals.iter().map(|id| self.value_map[id]).collect();
                    let lab = labeled
                        .iter()
                        .map(|(lbl, id)| (self.store.intern_symbol_id(lbl), self.value_map[id]))
                        .collect();
                    self.store
                        .intern_pack(pos, lab)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
                StaticValueNode::Data { owner, result_type, components } => {
                    let res_ty = self.resolve_type(*result_type)?;
                    let decl = resolve_stable_declaration(owner, self.projects)
                        .ok_or_else(|| MetadataExportError::StaticValue("unknown data owner in import".into()))?;
                    let constructor = crate::identity::DataConstructorId::new(decl.clone());
                    let comp_values = components
                        .iter()
                        .map(|(comp, val_id)| (DataComponentId::new(decl.clone(), comp.index), self.value_map[val_id]))
                        .collect();
                    self.store
                        .intern_data_reconstructed(decl, constructor, res_ty, comp_values)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
                StaticValueNode::Variant { variant, result_type, fields } => {
                    let res_ty = self.resolve_type(*result_type)?;
                    let decl = resolve_stable_declaration(&variant.owner, self.projects)
                        .ok_or_else(|| MetadataExportError::StaticValue("unknown variant owner in import".into()))?;
                    let selector = Selector::try_decode_exact(&variant.selector)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?;
                    let var_id = VariantId::new(decl, selector);
                    let field_values = fields
                        .iter()
                        .map(|(f, val_id)| (VariantFieldId::new(var_id.clone(), f.index), self.value_map[val_id]))
                        .collect();
                    self.store
                        .intern_variant_reconstructed(var_id, res_ty, field_values)
                        .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                }
            };
            self.value_map.insert(node_id, val_ref);
        }

        // Reconstruct static constants:
        for constant in self.bundle.static_constants.iter() {
            let module = resolve_stable_module(&constant.id.module, self.projects)
                .ok_or_else(|| MetadataExportError::StaticValue("unknown module in imported constant".into()))?;
            let id = StaticConstantId::new(module, &*constant.id.name);
            let val_ref = self.value_map.get(&constant.value).copied()
                .ok_or_else(|| MetadataExportError::StaticValue("unknown value index in imported constant".into()))?;
            let origin = ImportedStaticConstantOrigin {
                provider: constant.id.clone(),
                value_fingerprint: constant.value_fingerprint,
                dependency_fingerprint: constant.dependency_fingerprint,
            };
            let product = Arc::new(StaticConstantProduct::for_imported(id, val_ref, origin));
            constants_table.insert(product);
        }

        Ok(self.value_map.values().copied().collect())
    }
}
