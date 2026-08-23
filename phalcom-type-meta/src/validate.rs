use crate::bundle::SemanticMetadataBundle;
use crate::declaration::PublishedTypeSlot;
use crate::header::{MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION, SEMANTIC_MODEL_VERSION, TYPE_METADATA_SCHEMA_VERSION, supports_type_metadata_schema};
use crate::kind::{KindNode, KindNodeId};
use crate::scoped_type::{ScopedRecordTailRef, ScopedTypeNode};
use crate::type_node::TypeNode;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationLimits {
    pub max_total_bytes: usize,
    pub max_kind_nodes: usize,
    pub max_type_nodes: usize,
    pub max_scoped_nodes: usize,
    pub max_parameters: usize,
    pub max_signatures: usize,
    pub max_declarations: usize,
    pub max_callables: usize,
    pub max_fields: usize,
    pub max_occurrences: usize,
    pub max_lambda_depth: u32,
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            max_total_bytes: 32 * 1024 * 1024,
            max_kind_nodes: 65536,
            max_type_nodes: 262144,
            max_scoped_nodes: 131072,
            max_parameters: 65536,
            max_signatures: 65536,
            max_declarations: 65536,
            max_callables: 262144,
            max_fields: 262144,
            max_occurrences: 524288,
            max_lambda_depth: 32,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum MetadataValidationError {
    #[error("unsupported schema version: found {found}, supported range [{minimum}..={maximum}]")]
    UnsupportedSchemaVersion { found: u32, minimum: u32, maximum: u32 },
    #[error("unsupported semantic model version: {0} (expected {SEMANTIC_MODEL_VERSION})")]
    UnsupportedSemanticModelVersion(u32),
    #[error("record rows feature required for record row kinds and open records")]
    RecordRowFeatureRequired,
    #[error("record tail parameter missing: owner {owner:?}, index {index}")]
    RecordTailParameterMissing { owner: String, index: u32 },
    #[error("record tail kind mismatch: expected RecordRow, found kind index {actual_kind}")]
    RecordTailKindMismatch { actual_kind: u32 },
    #[error("record field order invalid at node {node}: fields must be sorted")]
    RecordFieldOrderInvalid { node: u32 },
    #[error("duplicate record field '{field}' at node {node}")]
    RecordDuplicateField { node: u32, field: Box<str> },
    #[error("scoped record tail out of scope at node {node}: depth {depth} index {index}")]
    ScopedRecordTailOutOfScope { node: u32, depth: u32, index: u32 },
    #[error("scoped record tail kind mismatch at node {node}: depth {depth} index {index} is not RecordRow")]
    ScopedRecordTailKindMismatch { node: u32, depth: u32, index: u32 },
    #[error("budget exceeded: {resource} count {count} exceeds limit {limit}")]
    BudgetExceeded { resource: &'static str, count: usize, limit: usize },
    #[error("invalid kind index: {index} (max {total})")]
    InvalidKindIndex { index: u32, total: usize },
    #[error("invalid type index: {index} (max {total})")]
    InvalidTypeIndex { index: u32, total: usize },
    #[error("invalid scoped type index: {index} (max {total})")]
    InvalidScopedTypeIndex { index: u32, total: usize },
    #[error("invalid generic signature index: {index} (max {total})")]
    InvalidSignatureIndex { index: u32, total: usize },
    #[error("topological order violation: node {index} references future node {target}")]
    TopologicalOrderViolation { index: u32, target: u32 },
    #[error("lambda scope violation: bound variable at depth {depth} index {index} exceeds max depth")]
    LambdaScopeViolation { depth: u32, index: u32 },
    #[error("duplicate type parameter owner: {0:?} index {1}")]
    DuplicateParameterOwner(String, u32),
    #[error("malformed metadata: {0}")]
    Malformed(String),
}

fn validate_schema_v1_feature_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    if bundle.header.features.record_rows {
        return Err(MetadataValidationError::Malformed("schema v1 cannot enable record_rows feature".to_string()));
    }
    for entry in bundle.kinds.iter() {
        if matches!(entry.node, KindNode::RecordRow) {
            return Err(MetadataValidationError::Malformed("schema v1 cannot contain KindNode::RecordRow".to_string()));
        }
    }
    for entry in bundle.types.iter() {
        if matches!(entry.form, TypeNode::OpenRecord(_)) {
            return Err(MetadataValidationError::Malformed("schema v1 cannot contain TypeNode::OpenRecord".to_string()));
        }
    }
    for entry in bundle.scoped_types.iter() {
        if matches!(entry.form, ScopedTypeNode::OpenRecord(_)) {
            return Err(MetadataValidationError::Malformed(
                "schema v1 cannot contain ScopedTypeNode::OpenRecord".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_schema_v2_feature_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    let mut has_row_construct = false;
    for entry in bundle.kinds.iter() {
        if matches!(entry.node, KindNode::RecordRow) {
            has_row_construct = true;
            break;
        }
    }
    if !has_row_construct {
        for entry in bundle.types.iter() {
            if matches!(entry.form, TypeNode::OpenRecord(_)) {
                has_row_construct = true;
                break;
            }
        }
    }
    if !has_row_construct {
        for entry in bundle.scoped_types.iter() {
            if matches!(entry.form, ScopedTypeNode::OpenRecord(_)) {
                has_row_construct = true;
                break;
            }
        }
    }

    if has_row_construct && !bundle.header.features.record_rows {
        return Err(MetadataValidationError::RecordRowFeatureRequired);
    }

    Ok(())
}

/// Iteratively validates a [`SemanticMetadataBundle`].
pub fn validate_metadata_bundle(bundle: &SemanticMetadataBundle, limits: &ValidationLimits) -> Result<(), MetadataValidationError> {
    if !supports_type_metadata_schema(bundle.header.schema_version) {
        return Err(MetadataValidationError::UnsupportedSchemaVersion {
            found: bundle.header.schema_version,
            minimum: MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION,
            maximum: TYPE_METADATA_SCHEMA_VERSION,
        });
    }
    if bundle.header.semantic_model_version != SEMANTIC_MODEL_VERSION {
        return Err(MetadataValidationError::UnsupportedSemanticModelVersion(bundle.header.semantic_model_version));
    }

    match bundle.header.schema_version {
        1 => validate_schema_v1_feature_floor(bundle)?,
        2 => validate_schema_v2_feature_floor(bundle)?,
        _ => unreachable!("version range checked above"),
    }

    if bundle.kinds.len() > limits.max_kind_nodes {
        return Err(MetadataValidationError::BudgetExceeded {
            resource: "kinds",
            count: bundle.kinds.len(),
            limit: limits.max_kind_nodes,
        });
    }
    if bundle.types.len() > limits.max_type_nodes {
        return Err(MetadataValidationError::BudgetExceeded {
            resource: "types",
            count: bundle.types.len(),
            limit: limits.max_type_nodes,
        });
    }
    if bundle.scoped_types.len() > limits.max_scoped_nodes {
        return Err(MetadataValidationError::BudgetExceeded {
            resource: "scoped_types",
            count: bundle.scoped_types.len(),
            limit: limits.max_scoped_nodes,
        });
    }

    // Validate kinds graph (strictly topologically sorted)
    for (i, entry) in bundle.kinds.iter().enumerate() {
        match &entry.node {
            KindNode::Type | KindNode::RecordRow => {}
            KindNode::Arrow { parameters, result } => {
                for &p in parameters.iter() {
                    if p.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: p.0 });
                    }
                }
                if result.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: result.0,
                    });
                }
            }
        }
    }

    // Index parameters by (owner_key, index) -> param_entry for fast tail lookup
    let mut param_map = std::collections::HashMap::new();
    let mut param_set = HashSet::new();
    for param in bundle.parameters.iter() {
        if param.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: param.kind.0,
                total: bundle.kinds.len(),
            });
        }
        let owner_key = format!("{:?}", param.id.owner);
        if !param_set.insert((owner_key.clone(), param.id.index)) {
            return Err(MetadataValidationError::DuplicateParameterOwner(owner_key.clone(), param.id.index));
        }
        param_map.insert((owner_key, param.id.index), param);
    }

    // Validate global types graph (strictly topologically sorted)
    for (i, entry) in bundle.types.iter().enumerate() {
        if entry.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: entry.kind.0,
                total: bundle.kinds.len(),
            });
        }
        match &entry.form {
            TypeNode::Never | TypeNode::Unit | TypeNode::Nominal { .. } | TypeNode::Parameter(_) | TypeNode::SelfType(_) => {}
            TypeNode::Applied { origin, arguments } => {
                if origin.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: origin.0,
                    });
                }
                for &arg in arguments.iter() {
                    if arg.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: arg.0,
                        });
                    }
                }
            }
            TypeNode::Union(members) => {
                for &m in members.iter() {
                    if m.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: m.0 });
                    }
                }
            }
            TypeNode::Tuple(elements) => {
                for el in elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                }
            }
            TypeNode::Record(fields) => {
                let mut prev_name: Option<&str> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
            }
            TypeNode::OpenRecord(open_rec) => {
                let mut prev_name: Option<&str> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
                let owner_key = format!("{:?}", open_rec.tail.owner);
                let tail_param = param_map
                    .get(&(owner_key.clone(), open_rec.tail.index))
                    .ok_or(MetadataValidationError::RecordTailParameterMissing {
                        owner: owner_key,
                        index: open_rec.tail.index,
                    })?;
                let tail_kind_entry = &bundle.kinds[tail_param.kind.0 as usize];
                if !matches!(tail_kind_entry.node, KindNode::RecordRow) {
                    return Err(MetadataValidationError::RecordTailKindMismatch {
                        actual_kind: tail_param.kind.0,
                    });
                }
            }
            TypeNode::Callable(call) => {
                for p in call.parameters.iter() {
                    if p.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: p.ty.0,
                        });
                    }
                }
                if call.return_type.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: call.return_type.0,
                    });
                }
            }
            TypeNode::TypeLambda(lambda) => {
                for &pk in lambda.parameter_kinds.iter() {
                    if pk.0 as usize >= bundle.kinds.len() {
                        return Err(MetadataValidationError::InvalidKindIndex {
                            index: pk.0,
                            total: bundle.kinds.len(),
                        });
                    }
                }
                if lambda.body.0 as usize >= bundle.scoped_types.len() {
                    return Err(MetadataValidationError::InvalidScopedTypeIndex {
                        index: lambda.body.0,
                        total: bundle.scoped_types.len(),
                    });
                }
            }
        }
    }

    // Validate scoped lambda graph (strictly topologically sorted)
    // We maintain a stack/mapping of lambda scope parameter kinds
    let mut lambda_param_kinds: Vec<Box<[KindNodeId]>> = Vec::new();
    for (i, entry) in bundle.scoped_types.iter().enumerate() {
        if entry.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: entry.kind.0,
                total: bundle.kinds.len(),
            });
        }
        match &entry.form {
            ScopedTypeNode::Bound { depth, index } => {
                if *depth > limits.max_lambda_depth {
                    return Err(MetadataValidationError::LambdaScopeViolation { depth: *depth, index: *index });
                }
            }
            ScopedTypeNode::Free(t) => {
                if t.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: t.0,
                        total: bundle.types.len(),
                    });
                }
            }
            ScopedTypeNode::Applied { origin, arguments } => {
                if origin.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: origin.0,
                    });
                }
                for &arg in arguments.iter() {
                    if arg.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: arg.0,
                        });
                    }
                }
            }
            ScopedTypeNode::Union(members) => {
                for &m in members.iter() {
                    if m.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: m.0 });
                    }
                }
            }
            ScopedTypeNode::Tuple(elements) => {
                for el in elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                }
            }
            ScopedTypeNode::Record(fields) => {
                let mut prev_name: Option<&str> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
            }
            ScopedTypeNode::OpenRecord(open_rec) => {
                let mut prev_name: Option<&str> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
                match &open_rec.tail {
                    ScopedRecordTailRef::Bound { depth, index } => {
                        if *depth as usize >= lambda_param_kinds.len() {
                            // Depth exceeds known lambda scopes at this node
                            // (Conservative fallback: still check depth limit)
                            if *depth > limits.max_lambda_depth {
                                return Err(MetadataValidationError::LambdaScopeViolation { depth: *depth, index: *index });
                            }
                        } else {
                            let kinds = &lambda_param_kinds[lambda_param_kinds.len() - 1 - *depth as usize];
                            if *index as usize >= kinds.len() {
                                return Err(MetadataValidationError::ScopedRecordTailOutOfScope {
                                    node: i as u32,
                                    depth: *depth,
                                    index: *index,
                                });
                            }
                            let k_id = kinds[*index as usize];
                            if k_id.0 as usize >= bundle.kinds.len() || !matches!(bundle.kinds[k_id.0 as usize].node, KindNode::RecordRow) {
                                return Err(MetadataValidationError::ScopedRecordTailKindMismatch {
                                    node: i as u32,
                                    depth: *depth,
                                    index: *index,
                                });
                            }
                        }
                    }
                    ScopedRecordTailRef::FreeParameter(param_ref) => {
                        let owner_key = format!("{:?}", param_ref.owner);
                        let tail_param = param_map
                            .get(&(owner_key.clone(), param_ref.index))
                            .ok_or(MetadataValidationError::RecordTailParameterMissing {
                                owner: owner_key,
                                index: param_ref.index,
                            })?;
                        let tail_kind_entry = &bundle.kinds[tail_param.kind.0 as usize];
                        if !matches!(tail_kind_entry.node, KindNode::RecordRow) {
                            return Err(MetadataValidationError::RecordTailKindMismatch {
                                actual_kind: tail_param.kind.0,
                            });
                        }
                    }
                }
            }
            ScopedTypeNode::Callable(call) => {
                for p in call.parameters.iter() {
                    if p.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: p.ty.0,
                        });
                    }
                }
                if call.return_type.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: call.return_type.0,
                    });
                }
            }
            ScopedTypeNode::Lambda { parameter_kinds, body } => {
                for &pk in parameter_kinds.iter() {
                    if pk.0 as usize >= bundle.kinds.len() {
                        return Err(MetadataValidationError::InvalidKindIndex {
                            index: pk.0,
                            total: bundle.kinds.len(),
                        });
                    }
                }
                if body.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: body.0,
                    });
                }
                lambda_param_kinds.push(parameter_kinds.clone());
            }
        }
    }

    // Validate parameters
    let mut param_set = HashSet::new();
    for param in bundle.parameters.iter() {
        if param.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: param.kind.0,
                total: bundle.kinds.len(),
            });
        }
        if !param_set.insert((format!("{:?}", param.id.owner), param.id.index)) {
            return Err(MetadataValidationError::DuplicateParameterOwner(
                format!("{:?}", param.id.owner),
                param.id.index,
            ));
        }
    }

    // Validate generic signatures & constraints
    for sig in bundle.generic_signatures.iter() {
        for c in sig.constraints.iter() {
            match c {
                crate::generic::GenericConstraintRef::Subtype { lower, upper } => {
                    if lower.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: lower.0,
                            total: bundle.types.len(),
                        });
                    }
                    if upper.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: upper.0,
                            total: bundle.types.len(),
                        });
                    }
                }
                crate::generic::GenericConstraintRef::Equivalent { left, right } => {
                    if left.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: left.0,
                            total: bundle.types.len(),
                        });
                    }
                    if right.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: right.0,
                            total: bundle.types.len(),
                        });
                    }
                }
            }
        }
    }

    // Validate declarations
    for decl in bundle.declarations.iter() {
        if decl.form.0 as usize >= bundle.types.len() {
            return Err(MetadataValidationError::InvalidTypeIndex {
                index: decl.form.0,
                total: bundle.types.len(),
            });
        }
        if decl.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: decl.kind.0,
                total: bundle.kinds.len(),
            });
        }
        if let Some(sig_id) = decl.generic_signature {
            if sig_id.0 as usize >= bundle.generic_signatures.len() {
                return Err(MetadataValidationError::InvalidSignatureIndex {
                    index: sig_id.0,
                    total: bundle.generic_signatures.len(),
                });
            }
        }
        if let Some(sup) = decl.superclass_template {
            if sup.0 as usize >= bundle.types.len() {
                return Err(MetadataValidationError::InvalidTypeIndex {
                    index: sup.0,
                    total: bundle.types.len(),
                });
            }
        }
    }

    // Validate callables
    for call in bundle.callables.iter() {
        for p in call.parameters.iter() {
            if let PublishedTypeSlot::Known { form, .. } = p.ty {
                if form.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: form.0,
                        total: bundle.types.len(),
                    });
                }
            }
        }
        if let PublishedTypeSlot::Known { form, .. } = call.return_type {
            if form.0 as usize >= bundle.types.len() {
                return Err(MetadataValidationError::InvalidTypeIndex {
                    index: form.0,
                    total: bundle.types.len(),
                });
            }
        }
    }

    Ok(())
}
