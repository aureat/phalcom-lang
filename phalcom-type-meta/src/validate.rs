use crate::bundle::SemanticMetadataBundle;
use crate::declaration::PublishedTypeSlot;
use crate::generic::{GenericParameterModeRef, VarianceRef};
use crate::header::{MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION, SEMANTIC_MODEL_VERSION, TYPE_METADATA_SCHEMA_VERSION, supports_type_metadata_schema};
use crate::identity::StableStaticDomainDeclarationRef;
use crate::kind::{KindNode, KindNodeId, StaticDomainRef};
use crate::scoped_type::{ScopedAnonymousRecordRemainderRef, ScopedAnonymousTupleRemainderRef, ScopedRecordRowTailRef, ScopedRecordTailRef, ScopedTypeNode};
use crate::type_node::{AnonymousRecordRemainderRef, AnonymousTupleRemainderRef, StaticArgumentRef, StructuralMemberPresenceRef, TypeNode};
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
    pub max_static_values: usize,
    pub max_static_constants: usize,
    pub max_static_definitions: usize,
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
            max_static_values: 262144,
            max_static_constants: 65536,
            max_static_definitions: 65536,
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
    #[error("static-domain declaration is missing from the metadata catalog: {0:?}")]
    MissingStaticDomainCatalogEntry(StableStaticDomainDeclarationRef),
    #[error("duplicate static-domain declaration catalog entry: {0:?}")]
    DuplicateStaticDomainCatalogEntry(StableStaticDomainDeclarationRef),
    #[error("static-domain case is missing from its declaration catalog: {declaration:?}::{case}")]
    MissingStaticDomainCase {
        declaration: StableStaticDomainDeclarationRef,
        case: Box<str>,
    },
    #[error("duplicate static-domain case in catalog: {declaration:?}::{case}")]
    DuplicateStaticDomainCase {
        declaration: StableStaticDomainDeclarationRef,
        case: Box<str>,
    },
    #[error("associated static-domain owner environment arity mismatch for {declaration:?}: expected {expected}, found {actual}")]
    StaticDomainOwnerArityMismatch {
        declaration: StableStaticDomainDeclarationRef,
        expected: usize,
        actual: usize,
    },
    #[error(
        "associated static-domain owner argument kind mismatch for {declaration:?} at index {index}: expected kind {expected_kind}, found kind {actual_kind}"
    )]
    StaticDomainOwnerKindMismatch {
        declaration: StableStaticDomainDeclarationRef,
        index: usize,
        expected_kind: u32,
        actual_kind: u32,
    },
    #[error("static-domain case payload arity mismatch for {declaration:?}::{case}: expected {expected}, found {actual}")]
    StaticDomainCaseArityMismatch {
        declaration: Box<StableStaticDomainDeclarationRef>,
        case: Box<str>,
        expected: usize,
        actual: usize,
    },
    #[error("static-domain case payload kind mismatch for {declaration:?}::{case} at index {index}: expected kind {expected_kind}, found kind {actual_kind}")]
    StaticDomainCaseKindMismatch {
        declaration: Box<StableStaticDomainDeclarationRef>,
        case: Box<str>,
        index: usize,
        expected_kind: u32,
        actual_kind: u32,
    },
    #[error("type application arity mismatch at node {node}: supplied {actual}, accepted at most {maximum}")]
    TypeApplicationArityMismatch { node: u32, maximum: usize, actual: usize },
    #[error("type application argument kind mismatch at node {node}, index {index}: expected kind {expected_kind}, found kind {actual_kind}")]
    TypeApplicationArgumentKindMismatch {
        node: u32,
        index: usize,
        expected_kind: u32,
        actual_kind: u32,
    },
    #[error("type application result kind mismatch at node {node}")]
    TypeApplicationResultKindMismatch { node: u32 },
    #[error("malformed metadata: {0}")]
    Malformed(String),
}

fn static_domain_declaration(domain: &StaticDomainRef) -> Option<StableStaticDomainDeclarationRef> {
    match domain {
        StaticDomainRef::ConstInt => None,
        StaticDomainRef::Nominal { declaration } => Some(StableStaticDomainDeclarationRef::Nominal(declaration.clone())),
        StaticDomainRef::Associated { declaration, .. } => Some(StableStaticDomainDeclarationRef::Associated(declaration.clone())),
    }
}

fn argument_kind_id(argument: &StaticArgumentRef, bundle: &SemanticMetadataBundle) -> Result<Option<KindNodeId>, MetadataValidationError> {
    match argument {
        StaticArgumentRef::Term(term) => {
            let Some(entry) = bundle.types.get(term.0 as usize) else {
                return Err(MetadataValidationError::InvalidTypeIndex {
                    index: term.0,
                    total: bundle.types.len(),
                });
            };
            Ok(Some(entry.kind))
        }
        StaticArgumentRef::RecordRow(_) => Ok(bundle
            .kinds
            .iter()
            .position(|entry| matches!(entry.node, KindNode::RecordRow))
            .map(|index| KindNodeId(index as u32))),
        StaticArgumentRef::GeneralRecordRow(_) => Err(MetadataValidationError::Malformed(
            "query-only RecordRow argument cannot be published in a versioned bundle".to_string(),
        )),
        StaticArgumentRef::DynamicType => Ok(bundle
            .kinds
            .iter()
            .position(|entry| matches!(entry.node, KindNode::Type))
            .map(|index| KindNodeId(index as u32))),
    }
}

fn argument_matches_kind(
    argument: &StaticArgumentRef,
    expected: KindNodeId,
    bundle: &SemanticMetadataBundle,
) -> Result<Option<KindNodeId>, MetadataValidationError> {
    let expected_entry = bundle.kinds.get(expected.0 as usize).ok_or(MetadataValidationError::InvalidKindIndex {
        index: expected.0,
        total: bundle.kinds.len(),
    })?;
    let actual = argument_kind_id(argument, bundle)?;
    match (argument, actual) {
        (StaticArgumentRef::RecordRow(_), _) if matches!(expected_entry.node, KindNode::RecordRow) => Ok(Some(expected)),
        (StaticArgumentRef::RecordRow(_), _) => Ok(None),
        (_, Some(actual)) if actual == expected => Ok(Some(actual)),
        (_, Some(actual)) => Ok(Some(actual)),
        (_, None) => Ok(None),
    }
}

fn validate_application_kinds(
    node: u32,
    callee_kind: KindNodeId,
    actual_kinds: &[KindNodeId],
    result_kind: KindNodeId,
    bundle: &SemanticMetadataBundle,
) -> Result<(), MetadataValidationError> {
    if actual_kinds.is_empty() {
        let callee = bundle.kinds.get(callee_kind.0 as usize).ok_or(MetadataValidationError::InvalidKindIndex {
            index: callee_kind.0,
            total: bundle.kinds.len(),
        })?;
        let actual_result = bundle.kinds.get(result_kind.0 as usize).ok_or(MetadataValidationError::InvalidKindIndex {
            index: result_kind.0,
            total: bundle.kinds.len(),
        })?;
        if callee.node != actual_result.node {
            return Err(MetadataValidationError::TypeApplicationResultKindMismatch { node });
        }
        return Ok(());
    }
    let callee = bundle.kinds.get(callee_kind.0 as usize).ok_or(MetadataValidationError::InvalidKindIndex {
        index: callee_kind.0,
        total: bundle.kinds.len(),
    })?;
    let KindNode::Arrow { parameters, result } = &callee.node else {
        return Err(MetadataValidationError::TypeApplicationArityMismatch {
            node,
            maximum: 0,
            actual: actual_kinds.len(),
        });
    };
    if actual_kinds.len() > parameters.len() {
        return Err(MetadataValidationError::TypeApplicationArityMismatch {
            node,
            maximum: parameters.len(),
            actual: actual_kinds.len(),
        });
    }
    for (index, (&actual, &expected)) in actual_kinds.iter().zip(parameters.iter()).enumerate() {
        if actual != expected {
            return Err(MetadataValidationError::TypeApplicationArgumentKindMismatch {
                node,
                index,
                expected_kind: expected.0,
                actual_kind: actual.0,
            });
        }
    }
    let expected_result = if actual_kinds.len() == parameters.len() {
        bundle.kinds.get(result.0 as usize).map(|entry| entry.node.clone())
    } else {
        Some(KindNode::Arrow {
            parameters: parameters[actual_kinds.len()..].into(),
            result: *result,
        })
    };
    let actual_result = bundle.kinds.get(result_kind.0 as usize).ok_or(MetadataValidationError::InvalidKindIndex {
        index: result_kind.0,
        total: bundle.kinds.len(),
    })?;
    if expected_result.as_ref() != Some(&actual_result.node) {
        return Err(MetadataValidationError::TypeApplicationResultKindMismatch { node });
    }
    Ok(())
}

fn validate_static_domain_catalog(
    bundle: &SemanticMetadataBundle,
) -> Result<std::collections::HashMap<StableStaticDomainDeclarationRef, &crate::declaration::StaticDomainDeclarationRecord>, MetadataValidationError> {
    let mut catalog = std::collections::HashMap::new();
    for record in bundle.static_domains.iter() {
        if catalog.insert(record.declaration.clone(), record).is_some() {
            return Err(MetadataValidationError::DuplicateStaticDomainCatalogEntry(record.declaration.clone()));
        }
        for parameter in record.owner_parameters.iter() {
            if parameter.kind.0 as usize >= bundle.kinds.len() {
                return Err(MetadataValidationError::InvalidKindIndex {
                    index: parameter.kind.0,
                    total: bundle.kinds.len(),
                });
            }
            let kind = &bundle.kinds[parameter.kind.0 as usize].node;
            let mode_valid = match parameter.mode {
                GenericParameterModeRef::Single => true,
                GenericParameterModeRef::PositionalPack => matches!(kind, KindNode::TypePack),
                GenericParameterModeRef::LabeledPack => matches!(kind, KindNode::LabeledTypePack | KindNode::RecordRow),
                GenericParameterModeRef::CompletePack => matches!(kind, KindNode::CompleteTypePack),
            };
            if !mode_valid {
                return Err(MetadataValidationError::Malformed(
                    "static-domain owner parameter mode does not match its kind".to_string(),
                ));
            }
        }
        let mut previous_case: Option<&str> = None;
        for case in record.cases.iter() {
            if let Some(previous) = previous_case {
                if case.name.as_ref() == previous {
                    return Err(MetadataValidationError::DuplicateStaticDomainCase {
                        declaration: record.declaration.clone(),
                        case: case.name.clone(),
                    });
                }
                if case.name.as_ref() < previous {
                    return Err(MetadataValidationError::Malformed(
                        "static-domain cases must be sorted by stable case name".to_string(),
                    ));
                }
            }
            previous_case = Some(&case.name);
            for &kind in case.parameter_kinds.iter() {
                if kind.0 as usize >= bundle.kinds.len() {
                    return Err(MetadataValidationError::InvalidKindIndex {
                        index: kind.0,
                        total: bundle.kinds.len(),
                    });
                }
                if !matches!(bundle.kinds[kind.0 as usize].node, KindNode::Type | KindNode::StaticDomain(_)) {
                    return Err(MetadataValidationError::Malformed(
                        "static-domain case payload kinds must be Type or StaticDomain".to_string(),
                    ));
                }
            }
        }
    }
    Ok(catalog)
}

fn validate_static_domain_instance(
    domain: &StaticDomainRef,
    catalog: &std::collections::HashMap<StableStaticDomainDeclarationRef, &crate::declaration::StaticDomainDeclarationRecord>,
    bundle: &SemanticMetadataBundle,
    param_map: &std::collections::HashMap<(String, u32), &crate::generic::TypeParameterRecord>,
) -> Result<(), MetadataValidationError> {
    let Some(declaration) = static_domain_declaration(domain) else {
        return Ok(());
    };
    let Some(record) = catalog.get(&declaration).copied() else {
        return Err(MetadataValidationError::MissingStaticDomainCatalogEntry(declaration));
    };
    if let StaticDomainRef::Associated { owner_environment, .. } = domain {
        if owner_environment.len() != record.owner_parameters.len() {
            return Err(MetadataValidationError::StaticDomainOwnerArityMismatch {
                declaration,
                expected: record.owner_parameters.len(),
                actual: owner_environment.len(),
            });
        }
        for (index, (argument, parameter)) in owner_environment.iter().zip(record.owner_parameters.iter()).enumerate() {
            let actual = argument_matches_kind(argument, parameter.kind, bundle)?;
            if actual.is_none() || actual != Some(parameter.kind) {
                return Err(MetadataValidationError::StaticDomainOwnerKindMismatch {
                    declaration: declaration.clone(),
                    index,
                    expected_kind: parameter.kind.0,
                    actual_kind: actual.map_or(u32::MAX, |kind| kind.0),
                });
            }
            validate_static_argument_row(argument, None, bundle, param_map)?;
        }
    }
    Ok(())
}

fn validate_static_argument_row(
    argument: &StaticArgumentRef,
    before_type_node: Option<u32>,
    bundle: &SemanticMetadataBundle,
    param_map: &std::collections::HashMap<(String, u32), &crate::generic::TypeParameterRecord>,
) -> Result<(), MetadataValidationError> {
    if matches!(argument, StaticArgumentRef::GeneralRecordRow(_)) {
        return Err(MetadataValidationError::Malformed(
            "query-only RecordRow argument cannot be published in a versioned bundle".to_string(),
        ));
    }
    let StaticArgumentRef::RecordRow(row) = argument else {
        return Ok(());
    };
    let mut previous_key: Option<&crate::field_key::FieldKeyRef> = None;
    for field in row.fields.iter() {
        let index = field.ty.0 as usize;
        if index >= bundle.types.len() {
            return Err(MetadataValidationError::InvalidTypeIndex {
                index: field.ty.0,
                total: bundle.types.len(),
            });
        }
        if before_type_node.is_some_and(|before| field.ty.0 >= before) {
            return Err(MetadataValidationError::TopologicalOrderViolation {
                index: before_type_node.unwrap_or_default(),
                target: field.ty.0,
            });
        }
        if !matches!(bundle.kinds[bundle.types[index].kind.0 as usize].node, KindNode::Type) {
            return Err(MetadataValidationError::Malformed("record-row field values must have kind Type".to_string()));
        }
        if let Some(previous) = previous_key {
            if &field.key < previous {
                return Err(MetadataValidationError::RecordFieldOrderInvalid {
                    node: before_type_node.unwrap_or(u32::MAX),
                });
            }
            if &field.key == previous {
                return Err(MetadataValidationError::RecordDuplicateField {
                    node: before_type_node.unwrap_or(u32::MAX),
                    field: field.key.to_string().into_boxed_str(),
                });
            }
        }
        previous_key = Some(&field.key);
    }
    if let Some(tail) = &row.tail {
        let owner_key = format!("{:?}", tail.owner);
        let parameter = param_map
            .get(&(owner_key.clone(), tail.index))
            .ok_or(MetadataValidationError::RecordTailParameterMissing {
                owner: owner_key,
                index: tail.index,
            })?;
        if !matches!(bundle.kinds[parameter.kind.0 as usize].node, KindNode::RecordRow) {
            return Err(MetadataValidationError::RecordTailKindMismatch { actual_kind: parameter.kind.0 });
        }
    }
    if let Some(AnonymousRecordRemainderRef::Typed(ty)) = &row.remainder {
        let index = ty.0 as usize;
        if index >= bundle.types.len() {
            return Err(MetadataValidationError::InvalidTypeIndex {
                index: ty.0,
                total: bundle.types.len(),
            });
        }
        if before_type_node.is_some_and(|before| ty.0 >= before) {
            return Err(MetadataValidationError::TopologicalOrderViolation {
                index: before_type_node.unwrap_or_default(),
                target: ty.0,
            });
        }
        if !matches!(bundle.kinds[bundle.types[index].kind.0 as usize].node, KindNode::Type) {
            return Err(MetadataValidationError::Malformed("record-row remainder type must have kind Type".to_string()));
        }
    }
    Ok(())
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
            if matches!(entry.form, ScopedTypeNode::OpenRecord(_) | ScopedTypeNode::RecordRow(_)) {
                has_row_construct = true;
                break;
            }
        }
    }

    if has_row_construct && !bundle.header.features.record_rows {
        return Err(MetadataValidationError::RecordRowFeatureRequired);
    }

    validate_pre_v3_pack_floor(bundle)
}

fn validate_pre_v3_pack_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    if bundle.kinds.iter().any(|entry| matches!(entry.node, KindNode::CompleteTypePack)) {
        return Err(MetadataValidationError::Malformed(
            "schema before v3 cannot contain KindNode::CompleteTypePack".to_string(),
        ));
    }
    if bundle.types.iter().any(|entry| {
        matches!(
            entry.form,
            TypeNode::CompleteTypePack { .. } | TypeNode::TypePackConcat(_) | TypeNode::LabeledTypePackConcat(_)
        )
    }) {
        return Err(MetadataValidationError::Malformed(
            "schema before v3 cannot contain C2 pack composition nodes".to_string(),
        ));
    }
    if bundle.parameters.iter().any(|parameter| parameter.mode != GenericParameterModeRef::Single) {
        return Err(MetadataValidationError::Malformed(
            "schema before v3 cannot contain variadic generic parameter modes".to_string(),
        ));
    }
    Ok(())
}

fn validate_schema_v3_feature_floor(_bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    Ok(())
}

fn validate_pre_v4_scoped_pack_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    if bundle.scoped_types.iter().any(|entry| {
        matches!(
            entry.form,
            ScopedTypeNode::TypePack(_)
                | ScopedTypeNode::LabeledTypePack(_)
                | ScopedTypeNode::CompleteTypePack { .. }
                | ScopedTypeNode::TypePackConcat(_)
                | ScopedTypeNode::LabeledTypePackConcat(_)
                | ScopedTypeNode::TupleFromPacks { .. }
                | ScopedTypeNode::CallableFromPacks { .. }
                | ScopedTypeNode::PackProjection { .. }
                | ScopedTypeNode::Lambda { parameter_modes: Some(_), .. }
        )
    }) {
        return Err(MetadataValidationError::Malformed(
            "schema before v4 cannot contain C2 scoped pack nodes or explicit lambda parameter modes".to_string(),
        ));
    }
    if bundle.types.iter().any(|entry| {
        if let TypeNode::TypeLambda(lambda) = &entry.form {
            lambda.parameter_modes.is_some()
        } else {
            false
        }
    }) {
        return Err(MetadataValidationError::Malformed(
            "schema before v4 cannot contain explicit type lambda parameter modes".to_string(),
        ));
    }
    Ok(())
}

fn validate_schema_v4_feature_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    // In schema v4, all emitted top-level lambdas and scoped lambdas must provide explicit parameter modes aligned with kinds
    for entry in bundle.types.iter() {
        if let TypeNode::TypeLambda(lambda) = &entry.form {
            let Some(modes) = &lambda.parameter_modes else {
                return Err(MetadataValidationError::Malformed(
                    "schema v4 requires explicit parameter modes for type lambdas".to_string(),
                ));
            };
            if modes.len() != lambda.parameter_kinds.len() {
                return Err(MetadataValidationError::Malformed(
                    "schema v4 lambda parameter modes length does not match parameter kinds length".to_string(),
                ));
            }
            for (&kind_id, &mode) in lambda.parameter_kinds.iter().zip(modes.iter()) {
                let kind_node = &bundle.kinds[kind_id.0 as usize].node;
                let valid = match mode {
                    GenericParameterModeRef::Single => true,
                    GenericParameterModeRef::PositionalPack => matches!(kind_node, KindNode::TypePack),
                    GenericParameterModeRef::LabeledPack => matches!(kind_node, KindNode::LabeledTypePack | KindNode::RecordRow),
                    GenericParameterModeRef::CompletePack => matches!(kind_node, KindNode::CompleteTypePack),
                };
                if !valid {
                    return Err(MetadataValidationError::Malformed(
                        "schema v4 lambda parameter mode does not match parameter kind".to_string(),
                    ));
                }
            }
        }
    }
    for entry in bundle.scoped_types.iter() {
        if let ScopedTypeNode::Lambda {
            parameter_kinds,
            parameter_modes,
            ..
        } = &entry.form
        {
            let Some(modes) = parameter_modes else {
                return Err(MetadataValidationError::Malformed(
                    "schema v4 requires explicit parameter modes for scoped lambdas".to_string(),
                ));
            };
            if modes.len() != parameter_kinds.len() {
                return Err(MetadataValidationError::Malformed(
                    "schema v4 scoped lambda parameter modes length does not match parameter kinds length".to_string(),
                ));
            }
            for (&kind_id, &mode) in parameter_kinds.iter().zip(modes.iter()) {
                let kind_node = &bundle.kinds[kind_id.0 as usize].node;
                let valid = match mode {
                    GenericParameterModeRef::Single => true,
                    GenericParameterModeRef::PositionalPack => matches!(kind_node, KindNode::TypePack),
                    GenericParameterModeRef::LabeledPack => matches!(kind_node, KindNode::LabeledTypePack | KindNode::RecordRow),
                    GenericParameterModeRef::CompletePack => matches!(kind_node, KindNode::CompleteTypePack),
                };
                if !valid {
                    return Err(MetadataValidationError::Malformed(
                        "schema v4 scoped lambda parameter mode does not match parameter kind".to_string(),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_pre_v5_static_domain_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    if bundle.kinds.iter().any(|entry| matches!(entry.node, KindNode::StaticDomain(_)))
        || bundle
            .types
            .iter()
            .any(|entry| matches!(entry.form, TypeNode::ConstInt(_) | TypeNode::StaticData(_)))
    {
        return Err(MetadataValidationError::Malformed(
            "schema before v5 cannot contain static-domain or type-data nodes".to_string(),
        ));
    }
    Ok(())
}

fn validate_schema_v5_feature_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    let has_static_nodes = bundle.kinds.iter().any(|entry| matches!(entry.node, KindNode::StaticDomain(_)))
        || bundle
            .types
            .iter()
            .any(|entry| matches!(entry.form, TypeNode::ConstInt(_) | TypeNode::StaticData(_)));
    if has_static_nodes && !bundle.header.features.static_domains {
        return Err(MetadataValidationError::Malformed(
            "schema v5 static-domain nodes require the static_domains feature".to_string(),
        ));
    }
    Ok(())
}

fn validate_pre_v8_catalog_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    if !bundle.static_domains.is_empty() {
        return Err(MetadataValidationError::Malformed(
            "schema before v8 cannot contain a static-domain declaration catalog".to_string(),
        ));
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
        1 => {
            validate_schema_v1_feature_floor(bundle)?;
            validate_pre_v3_pack_floor(bundle)?;
            validate_pre_v4_scoped_pack_floor(bundle)?;
            validate_pre_v5_static_domain_floor(bundle)?;
        }
        2 => {
            validate_schema_v2_feature_floor(bundle)?;
            validate_pre_v4_scoped_pack_floor(bundle)?;
            validate_pre_v5_static_domain_floor(bundle)?;
        }
        3 => {
            validate_schema_v3_feature_floor(bundle)?;
            validate_pre_v4_scoped_pack_floor(bundle)?;
            validate_pre_v5_static_domain_floor(bundle)?;
        }
        4 => {
            validate_schema_v4_feature_floor(bundle)?;
            validate_pre_v5_static_domain_floor(bundle)?;
        }
        5..=7 => {
            validate_schema_v5_feature_floor(bundle)?;
            validate_pre_v8_catalog_floor(bundle)?;
        }
        8..=13 => validate_schema_v5_feature_floor(bundle)?,
        _ => unreachable!("version range checked above"),
    }

    if bundle.header.schema_version < 13 && (!bundle.static_values.is_empty() || !bundle.static_constants.is_empty() || !bundle.static_definitions.is_empty()) {
        return Err(MetadataValidationError::Malformed(
            "schema before v13 cannot contain static values, constants, or definitions".to_string(),
        ));
    }
    if bundle.static_values.len() > limits.max_static_values {
        return Err(MetadataValidationError::Malformed(format!(
            "static_values count {} exceeds limit {}",
            bundle.static_values.len(),
            limits.max_static_values
        )));
    }
    if bundle.static_constants.len() > limits.max_static_constants {
        return Err(MetadataValidationError::Malformed(format!(
            "static_constants count {} exceeds limit {}",
            bundle.static_constants.len(),
            limits.max_static_constants
        )));
    }
    if bundle.static_definitions.len() > limits.max_static_definitions {
        return Err(MetadataValidationError::Malformed(format!(
            "static_definitions count {} exceeds limit {}",
            bundle.static_definitions.len(),
            limits.max_static_definitions
        )));
    }

    if bundle.header.schema_version < 9 && bundle.scoped_types.iter().any(|entry| matches!(entry.form, ScopedTypeNode::RecordRow(_))) {
        return Err(MetadataValidationError::Malformed(
            "schema before v9 cannot contain ScopedTypeNode::RecordRow".to_string(),
        ));
    }
    if bundle.scoped_types.iter().any(|entry| matches!(entry.form, ScopedTypeNode::RecordRow(_))) && !bundle.header.features.record_rows {
        return Err(MetadataValidationError::RecordRowFeatureRequired);
    }

    if bundle.header.schema_version < 10
        && (bundle.types.iter().any(|entry| {
            matches!(entry.form, TypeNode::OpenTuple(_) | TypeNode::AnonymousRecord(_))
                || match &entry.form {
                    TypeNode::Tuple(elements) => elements.iter().any(|el| el.presence == StructuralMemberPresenceRef::Optional),
                    TypeNode::Record(fields) => fields.iter().any(|f| f.presence == StructuralMemberPresenceRef::Optional),
                    TypeNode::OpenRecord(open_rec) => open_rec.fields.iter().any(|f| f.presence == StructuralMemberPresenceRef::Optional),
                    _ => false,
                }
        }) || bundle.scoped_types.iter().any(|entry| {
            matches!(entry.form, ScopedTypeNode::OpenTuple(_) | ScopedTypeNode::AnonymousRecord(_))
                || match &entry.form {
                    ScopedTypeNode::Tuple(elements) => elements.iter().any(|el| el.presence == StructuralMemberPresenceRef::Optional),
                    ScopedTypeNode::Record(fields) => fields.iter().any(|f| f.presence == StructuralMemberPresenceRef::Optional),
                    ScopedTypeNode::OpenRecord(open_rec) => {
                        open_rec.fields.iter().any(|f| f.presence == StructuralMemberPresenceRef::Optional)
                            || matches!(open_rec.tail, ScopedRecordTailRef::Anonymous(_))
                    }
                    ScopedTypeNode::RecordRow(row) => {
                        row.fields.iter().any(|f| f.presence == StructuralMemberPresenceRef::Optional)
                            || matches!(row.tail, ScopedRecordRowTailRef::Anonymous(_))
                    }
                    _ => false,
                }
        }))
    {
        return Err(MetadataValidationError::Malformed(
            "schema before v10 cannot contain structural member presence, open tuples, or anonymous remainders".to_string(),
        ));
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
            KindNode::Type | KindNode::RecordRow | KindNode::TypePack | KindNode::LabeledTypePack | KindNode::CompleteTypePack | KindNode::StaticDomain(_) => {}
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
        let kind = &bundle.kinds[param.kind.0 as usize].node;
        let mode_is_valid = match param.mode {
            GenericParameterModeRef::Single => true,
            GenericParameterModeRef::PositionalPack => matches!(kind, KindNode::TypePack),
            GenericParameterModeRef::LabeledPack => matches!(kind, KindNode::LabeledTypePack | KindNode::RecordRow),
            GenericParameterModeRef::CompletePack => matches!(kind, KindNode::CompleteTypePack),
        };
        if param.mode != GenericParameterModeRef::Single {
            if !mode_is_valid {
                return Err(MetadataValidationError::Malformed(
                    "generic parameter mode does not match parameter kind".to_string(),
                ));
            }
            if param.variance != VarianceRef::Invariant {
                return Err(MetadataValidationError::Malformed("pack generic parameters must be invariant".to_string()));
            }
        }
    }

    let static_domain_catalog = if bundle.header.schema_version >= 8 {
        validate_static_domain_catalog(bundle)?
    } else {
        std::collections::HashMap::new()
    };
    if bundle.header.schema_version >= 8 {
        for entry in bundle.kinds.iter() {
            if let KindNode::StaticDomain(domain) = &entry.node {
                validate_static_domain_instance(domain, &static_domain_catalog, bundle, &param_map)?;
            }
        }
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
            TypeNode::ConstInt(_) => {
                if !matches!(
                    &bundle.kinds[entry.kind.0 as usize].node,
                    KindNode::StaticDomain(crate::kind::StaticDomainRef::ConstInt)
                ) {
                    return Err(MetadataValidationError::Malformed(
                        "const-int node must have the built-in const-int static domain kind".to_string(),
                    ));
                }
            }
            TypeNode::StaticData(static_data) => {
                match &static_data.domain {
                    crate::kind::StaticDomainRef::ConstInt => {
                        return Err(MetadataValidationError::Malformed(
                            "static-data node cannot use the built-in const-int domain".to_string(),
                        ));
                    }
                    crate::kind::StaticDomainRef::Nominal { declaration } => {
                        if &static_data.constructor.declaration != declaration {
                            return Err(MetadataValidationError::Malformed(
                                "static-data constructor owner does not match its nominal domain".to_string(),
                            ));
                        }
                    }
                    crate::kind::StaticDomainRef::Associated { declaration, .. } => {
                        if static_data.constructor.declaration != declaration.owner {
                            return Err(MetadataValidationError::Malformed(
                                "static-data constructor owner does not match its associated domain owner".to_string(),
                            ));
                        }
                    }
                }
                if bundle.header.schema_version >= 8 {
                    validate_static_domain_instance(&static_data.domain, &static_domain_catalog, bundle, &param_map)?;
                    if let Some(declaration) = static_domain_declaration(&static_data.domain) {
                        let record = static_domain_catalog
                            .get(&declaration)
                            .copied()
                            .ok_or_else(|| MetadataValidationError::MissingStaticDomainCatalogEntry(declaration.clone()))?;
                        let case = record.cases.iter().find(|case| case.name == static_data.constructor.case_name).ok_or_else(|| {
                            MetadataValidationError::MissingStaticDomainCase {
                                declaration: declaration.clone(),
                                case: static_data.constructor.case_name.clone(),
                            }
                        })?;
                        if case.parameter_kinds.len() != static_data.arguments.len() {
                            return Err(MetadataValidationError::StaticDomainCaseArityMismatch {
                                declaration: Box::new(declaration.clone()),
                                case: case.name.clone(),
                                expected: case.parameter_kinds.len(),
                                actual: static_data.arguments.len(),
                            });
                        }
                        for (index, (argument, &expected_kind)) in static_data.arguments.iter().zip(case.parameter_kinds.iter()).enumerate() {
                            let actual_kind = argument_matches_kind(argument, expected_kind, bundle)?;
                            if actual_kind != Some(expected_kind) {
                                return Err(MetadataValidationError::StaticDomainCaseKindMismatch {
                                    declaration: Box::new(declaration.clone()),
                                    case: case.name.clone(),
                                    index,
                                    expected_kind: expected_kind.0,
                                    actual_kind: actual_kind.map_or(u32::MAX, |kind| kind.0),
                                });
                            }
                        }
                    }
                }
                if !matches!(
                    &bundle.kinds[entry.kind.0 as usize].node,
                    KindNode::StaticDomain(domain) if domain == &static_data.domain
                ) {
                    return Err(MetadataValidationError::Malformed(
                        "static-data node kind does not match its domain".to_string(),
                    ));
                }
                for argument in static_data.arguments.iter() {
                    if bundle.header.schema_version >= 8 {
                        validate_static_argument_row(argument, Some(i as u32), bundle, &param_map)?;
                    }
                    match argument {
                        StaticArgumentRef::Term(term) => {
                            if term.0 as usize >= i {
                                return Err(MetadataValidationError::TopologicalOrderViolation {
                                    index: i as u32,
                                    target: term.0,
                                });
                            }
                        }
                        StaticArgumentRef::RecordRow(row) => {
                            for field in row.fields.iter() {
                                if field.ty.0 as usize >= i {
                                    return Err(MetadataValidationError::TopologicalOrderViolation {
                                        index: i as u32,
                                        target: field.ty.0,
                                    });
                                }
                            }
                        }
                        StaticArgumentRef::GeneralRecordRow(_) => {
                            return Err(MetadataValidationError::Malformed(
                                "query-only RecordRow argument cannot be published in a versioned bundle".to_string(),
                            ));
                        }
                        StaticArgumentRef::DynamicType => {}
                    }
                }
            }
            TypeNode::Applied { origin, arguments } => {
                if origin.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: origin.0,
                    });
                }
                for arg in arguments.iter() {
                    if bundle.header.schema_version >= 8 {
                        validate_static_argument_row(arg, Some(i as u32), bundle, &param_map)?;
                    }
                    match arg {
                        StaticArgumentRef::Term(arg) if arg.0 as usize >= i => {
                            return Err(MetadataValidationError::TopologicalOrderViolation {
                                index: i as u32,
                                target: arg.0,
                            });
                        }
                        StaticArgumentRef::RecordRow(row) => {
                            for field in row.fields.iter() {
                                if field.ty.0 as usize >= i {
                                    return Err(MetadataValidationError::TopologicalOrderViolation {
                                        index: i as u32,
                                        target: field.ty.0,
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if bundle.header.schema_version >= 8 {
                    let callee_kind = bundle.types[origin.0 as usize].kind;
                    let mut actual_kinds = Vec::with_capacity(arguments.len());
                    for (index, argument) in arguments.iter().enumerate() {
                        let Some(actual) = argument_kind_id(argument, bundle)? else {
                            return Err(MetadataValidationError::TypeApplicationArgumentKindMismatch {
                                node: i as u32,
                                index,
                                expected_kind: u32::MAX,
                                actual_kind: u32::MAX,
                            });
                        };
                        actual_kinds.push(actual);
                    }
                    validate_application_kinds(i as u32, callee_kind, &actual_kinds, entry.kind, bundle)?;
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
                    if el.presence == StructuralMemberPresenceRef::Optional {
                        return Err(MetadataValidationError::Malformed("tuple elements cannot be optional".to_string()));
                    }
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                }
            }
            TypeNode::Record(fields) => {
                let mut prev_key: Option<&crate::field_key::FieldKeyRef> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_key {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.key.to_string().into_boxed_str(),
                            });
                        }
                    }
                    prev_key = Some(&f.key);
                }
            }
            TypeNode::GeneralRecord(_) => {
                return Err(MetadataValidationError::Malformed(
                    "query-only generalized Record node cannot be published in a versioned bundle".to_string(),
                ));
            }
            TypeNode::OpenRecord(open_rec) => {
                let mut prev_key: Option<&crate::field_key::FieldKeyRef> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_key {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.key.to_string().into_boxed_str(),
                            });
                        }
                    }
                    prev_key = Some(&f.key);
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
            TypeNode::AssociatedProjection {
                subject,
                trait_application,
                requirement,
            } => {
                if subject.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: subject.0,
                    });
                }
                for &arg in trait_application.arguments.iter() {
                    if arg.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: arg.0,
                        });
                    }
                }
                if requirement.owner != trait_application.declaration {
                    return Err(MetadataValidationError::Malformed(format!(
                        "associated type requirement owner {:?} does not match trait application declaration {:?}",
                        requirement.owner, trait_application.declaration
                    )));
                }
            }
            TypeNode::TypePack(elements) => {
                for &el in elements.iter() {
                    if el.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: el.0 });
                    }
                }
            }
            TypeNode::LabeledTypePack(elements) => {
                for el in elements.iter() {
                    if el.element.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.element.0,
                        });
                    }
                }
            }
            TypeNode::CompleteTypePack { positionals, labeled } => {
                for child in [positionals, labeled] {
                    if child.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: child.0,
                        });
                    }
                }
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::CompleteTypePack) {
                    return Err(MetadataValidationError::Malformed("complete type pack node has wrong entry kind".to_string()));
                }
                if !matches!(bundle.kinds[bundle.types[positionals.0 as usize].kind.0 as usize].node, KindNode::TypePack)
                    || !matches!(bundle.kinds[bundle.types[labeled.0 as usize].kind.0 as usize].node, KindNode::LabeledTypePack)
                {
                    return Err(MetadataValidationError::Malformed("complete type pack children have wrong kinds".to_string()));
                }
            }
            TypeNode::TypePackConcat(children) | TypeNode::LabeledTypePackConcat(children) => {
                let positional = matches!(&entry.form, TypeNode::TypePackConcat(_));
                let valid_entry_kind = if positional {
                    matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::TypePack)
                } else {
                    matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::LabeledTypePack)
                };
                if !valid_entry_kind {
                    return Err(MetadataValidationError::Malformed("pack concat node has wrong entry kind".to_string()));
                }
                for child in children.iter() {
                    if child.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: child.0,
                        });
                    }
                    let kind = &bundle.kinds[bundle.types[child.0 as usize].kind.0 as usize].node;
                    let valid_child_kind = if positional {
                        matches!(kind, KindNode::TypePack)
                    } else {
                        matches!(kind, KindNode::LabeledTypePack)
                    };
                    if !valid_child_kind {
                        return Err(MetadataValidationError::Malformed("pack concat child has wrong kind".to_string()));
                    }
                }
            }
            TypeNode::TupleFromPacks { positionals, labeled } => {
                if positionals.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: positionals.0,
                    });
                }
                if labeled.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: labeled.0,
                    });
                }
            }
            TypeNode::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                if positionals.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: positionals.0,
                    });
                }
                if labeled.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: labeled.0,
                    });
                }
                if return_type.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: return_type.0,
                    });
                }
            }
            TypeNode::TypeMemberProjection(proj) => {
                if proj.subject.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: proj.subject.0,
                    });
                }
                for arg in proj.arguments.iter() {
                    if let crate::type_node::TypeMemberArgumentRef::Type(ty) = arg {
                        if ty.0 as usize >= i {
                            return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                        }
                    }
                }
            }
            TypeNode::OpenTuple(open_tuple) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed("OpenTuple node must have kind Type".to_string()));
                }
                for el in open_tuple.elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                    if el.presence == StructuralMemberPresenceRef::Optional {
                        return Err(MetadataValidationError::Malformed("tuple elements cannot be optional".to_string()));
                    }
                    if !matches!(bundle.kinds[bundle.types[el.ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("tuple elements must have kind Type".to_string()));
                    }
                }
                if let AnonymousTupleRemainderRef::Typed(ty) = open_tuple.positional_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("tuple positional remainder must have kind Type".to_string()));
                    }
                }
                if let AnonymousTupleRemainderRef::Typed(ty) = open_tuple.labeled_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("tuple labeled remainder must have kind Type".to_string()));
                    }
                }
            }
            TypeNode::AnonymousRecord(anon_rec) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed("AnonymousRecord node must have kind Type".to_string()));
                }
                let mut prev_key: Option<&crate::field_key::FieldKeyRef> = None;
                for f in anon_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if !matches!(bundle.kinds[bundle.types[f.ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("record field values must have kind Type".to_string()));
                    }
                    if let Some(prev) = prev_key {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.key.to_string().into_boxed_str(),
                            });
                        }
                    }
                    prev_key = Some(&f.key);
                }
                if let AnonymousRecordRemainderRef::Typed(ty) = anon_rec.remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("record remainder must have kind Type".to_string()));
                    }
                }
            }
            TypeNode::Pack(pack) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed("Pack node must have kind Type".to_string()));
                }
                for el in pack.elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                    if el.presence == StructuralMemberPresenceRef::Optional {
                        return Err(MetadataValidationError::Malformed("pack elements cannot be optional".to_string()));
                    }
                    if !matches!(bundle.kinds[bundle.types[el.ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("pack elements must have kind Type".to_string()));
                    }
                }
                if let AnonymousTupleRemainderRef::Typed(ty) = pack.positional_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("pack positional remainder must have kind Type".to_string()));
                    }
                }
                if let AnonymousTupleRemainderRef::Typed(ty) = pack.labeled_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("pack labeled remainder must have kind Type".to_string()));
                    }
                }
            }
            TypeNode::PackFromPacks { positionals, labeled } => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed("PackFromPacks node must have kind Type".to_string()));
                }
                if positionals.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: positionals.0,
                    });
                }
                if labeled.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: labeled.0,
                    });
                }
                let pos_kind = &bundle.kinds[bundle.types[positionals.0 as usize].kind.0 as usize].node;
                if !matches!(pos_kind, KindNode::TypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "PackFromPacks positionals must have kind TypePack".to_string(),
                    ));
                }
                let lab_kind = &bundle.kinds[bundle.types[labeled.0 as usize].kind.0 as usize].node;
                if !matches!(lab_kind, KindNode::LabeledTypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "PackFromPacks labeled must have kind LabeledTypePack".to_string(),
                    ));
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
                if bundle.header.schema_version >= 8 {
                    let callee_kind = bundle.scoped_types[origin.0 as usize].kind;
                    let actual_kinds = arguments
                        .iter()
                        .map(|argument| bundle.scoped_types[argument.0 as usize].kind)
                        .collect::<Vec<_>>();
                    validate_application_kinds(i as u32, callee_kind, &actual_kinds, entry.kind, bundle)?;
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
                    if el.presence == StructuralMemberPresenceRef::Optional {
                        return Err(MetadataValidationError::Malformed("scoped tuple elements cannot be optional".to_string()));
                    }
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                }
            }
            ScopedTypeNode::Record(fields) => {
                let mut prev_key: Option<&crate::field_key::FieldKeyRef> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_key {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.key.to_string().into_boxed_str(),
                            });
                        }
                    }
                    prev_key = Some(&f.key);
                }
            }
            ScopedTypeNode::OpenRecord(open_rec) => {
                let mut prev_key: Option<&crate::field_key::FieldKeyRef> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_key {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.key.to_string().into_boxed_str(),
                            });
                        }
                    }
                    prev_key = Some(&f.key);
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
                    ScopedRecordTailRef::Anonymous(remainder) => {
                        if let ScopedAnonymousRecordRemainderRef::Typed(ty) = remainder {
                            if ty.0 as usize >= i {
                                return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                            }
                            if !matches!(bundle.kinds[bundle.scoped_types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                                return Err(MetadataValidationError::Malformed("scoped record remainder must have kind Type".to_string()));
                            }
                        }
                    }
                }
            }
            ScopedTypeNode::RecordRow(row) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::RecordRow) {
                    return Err(MetadataValidationError::Malformed(format!(
                        "scoped RecordRow node {i} must have kind RecordRow"
                    )));
                }
                let mut previous_key: Option<&crate::field_key::FieldKeyRef> = None;
                for field in row.fields.iter() {
                    if field.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: field.ty.0,
                        });
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[field.ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed(format!(
                            "scoped RecordRow field '{}' at node {i} must have kind Type",
                            field.key
                        )));
                    }
                    if let Some(previous) = previous_key {
                        if &field.key < previous {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        }
                        if &field.key == previous {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: field.key.to_string().into_boxed_str(),
                            });
                        }
                    }
                    previous_key = Some(&field.key);
                }
                match &row.tail {
                    ScopedRecordRowTailRef::Closed => {}
                    ScopedRecordRowTailRef::Bound { depth, index } => {
                        if *depth as usize >= lambda_param_kinds.len() {
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
                            let kind = kinds[*index as usize];
                            if kind.0 as usize >= bundle.kinds.len() || !matches!(bundle.kinds[kind.0 as usize].node, KindNode::RecordRow) {
                                return Err(MetadataValidationError::ScopedRecordTailKindMismatch {
                                    node: i as u32,
                                    depth: *depth,
                                    index: *index,
                                });
                            }
                        }
                    }
                    ScopedRecordRowTailRef::FreeParameter(parameter) => {
                        let owner_key = format!("{:?}", parameter.owner);
                        let record_parameter =
                            param_map
                                .get(&(owner_key.clone(), parameter.index))
                                .ok_or(MetadataValidationError::RecordTailParameterMissing {
                                    owner: owner_key,
                                    index: parameter.index,
                                })?;
                        if !matches!(bundle.kinds[record_parameter.kind.0 as usize].node, KindNode::RecordRow) {
                            return Err(MetadataValidationError::RecordTailKindMismatch {
                                actual_kind: record_parameter.kind.0,
                            });
                        }
                    }
                    ScopedRecordRowTailRef::Projection(projection) => {
                        let Some(projected_type) = bundle.types.get(projection.0 as usize) else {
                            return Err(MetadataValidationError::InvalidTypeIndex {
                                index: projection.0,
                                total: bundle.types.len(),
                            });
                        };
                        if !matches!(bundle.kinds[projected_type.kind.0 as usize].node, KindNode::RecordRow) {
                            return Err(MetadataValidationError::Malformed(format!(
                                "scoped RecordRow projection tail at node {i} must have kind RecordRow"
                            )));
                        }
                    }
                    ScopedRecordRowTailRef::Anonymous(remainder) => {
                        if let ScopedAnonymousRecordRemainderRef::Typed(ty) = remainder {
                            if ty.0 as usize >= i {
                                return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                            }
                            if !matches!(bundle.kinds[bundle.scoped_types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                                return Err(MetadataValidationError::Malformed(
                                    "scoped record row remainder must have kind Type".to_string(),
                                ));
                            }
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
            ScopedTypeNode::Lambda {
                parameter_kinds,
                parameter_modes: _,
                body,
            } => {
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
            ScopedTypeNode::TypePack(elements) => {
                for &el in elements.iter() {
                    if el.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: el.0 });
                    }
                    let el_kind = &bundle.kinds[bundle.scoped_types[el.0 as usize].kind.0 as usize].node;
                    if !matches!(el_kind, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("Scoped TypePack element must have kind Type".to_string()));
                    }
                }
            }
            ScopedTypeNode::LabeledTypePack(elements) => {
                for el in elements.iter() {
                    if el.element.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.element.0,
                        });
                    }
                    let el_kind = &bundle.kinds[bundle.scoped_types[el.element.0 as usize].kind.0 as usize].node;
                    if !matches!(el_kind, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed(
                            "Scoped LabeledTypePack element must have kind Type".to_string(),
                        ));
                    }
                }
            }
            ScopedTypeNode::CompleteTypePack { positionals, labeled } => {
                if positionals.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: positionals.0,
                    });
                }
                if labeled.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: labeled.0,
                    });
                }
                let pos_kind = &bundle.kinds[bundle.scoped_types[positionals.0 as usize].kind.0 as usize].node;
                if !matches!(pos_kind, KindNode::TypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped CompleteTypePack positionals must have kind TypePack".to_string(),
                    ));
                }
                let lab_kind = &bundle.kinds[bundle.scoped_types[labeled.0 as usize].kind.0 as usize].node;
                if !matches!(lab_kind, KindNode::LabeledTypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped CompleteTypePack labeled must have kind LabeledTypePack".to_string(),
                    ));
                }
            }
            ScopedTypeNode::TypePackConcat(segments) => {
                for &seg in segments.iter() {
                    if seg.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: seg.0,
                        });
                    }
                    let seg_kind = &bundle.kinds[bundle.scoped_types[seg.0 as usize].kind.0 as usize].node;
                    if !matches!(seg_kind, KindNode::TypePack) {
                        return Err(MetadataValidationError::Malformed(
                            "Scoped TypePackConcat segment must have kind TypePack".to_string(),
                        ));
                    }
                }
            }
            ScopedTypeNode::LabeledTypePackConcat(segments) => {
                for &seg in segments.iter() {
                    if seg.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: seg.0,
                        });
                    }
                    let seg_kind = &bundle.kinds[bundle.scoped_types[seg.0 as usize].kind.0 as usize].node;
                    if !matches!(seg_kind, KindNode::LabeledTypePack) {
                        return Err(MetadataValidationError::Malformed(
                            "Scoped LabeledTypePackConcat segment must have kind LabeledTypePack".to_string(),
                        ));
                    }
                }
            }
            ScopedTypeNode::TupleFromPacks { positionals, labeled } => {
                if positionals.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: positionals.0,
                    });
                }
                if labeled.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: labeled.0,
                    });
                }
                let pos_kind = &bundle.kinds[bundle.scoped_types[positionals.0 as usize].kind.0 as usize].node;
                if !matches!(pos_kind, KindNode::TypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped TupleFromPacks positionals must have kind TypePack".to_string(),
                    ));
                }
                let lab_kind = &bundle.kinds[bundle.scoped_types[labeled.0 as usize].kind.0 as usize].node;
                if !matches!(lab_kind, KindNode::LabeledTypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped TupleFromPacks labeled must have kind LabeledTypePack".to_string(),
                    ));
                }
            }
            ScopedTypeNode::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                if positionals.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: positionals.0,
                    });
                }
                if labeled.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: labeled.0,
                    });
                }
                if return_type.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: return_type.0,
                    });
                }
                let pos_kind = &bundle.kinds[bundle.scoped_types[positionals.0 as usize].kind.0 as usize].node;
                if !matches!(pos_kind, KindNode::TypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped CallableFromPacks positionals must have kind TypePack".to_string(),
                    ));
                }
                let lab_kind = &bundle.kinds[bundle.scoped_types[labeled.0 as usize].kind.0 as usize].node;
                if !matches!(lab_kind, KindNode::LabeledTypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped CallableFromPacks labeled must have kind LabeledTypePack".to_string(),
                    ));
                }
                let ret_kind = &bundle.kinds[bundle.scoped_types[return_type.0 as usize].kind.0 as usize].node;
                if !matches!(ret_kind, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped CallableFromPacks return_type must have kind Type".to_string(),
                    ));
                }
            }
            ScopedTypeNode::PackProjection { subject, lane } => {
                if subject.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: subject.0,
                    });
                }
                let subj_kind = &bundle.kinds[bundle.scoped_types[subject.0 as usize].kind.0 as usize].node;
                let res_kind = &bundle.kinds[entry.kind.0 as usize].node;
                match lane {
                    crate::scoped_type::ScopedPackProjectionLaneRef::Positionals => {
                        if !matches!(subj_kind, KindNode::CompleteTypePack) || !matches!(res_kind, KindNode::TypePack) {
                            return Err(MetadataValidationError::Malformed(
                                "Invalid scoped PackProjection Positionals lane kinds".to_string(),
                            ));
                        }
                    }
                    crate::scoped_type::ScopedPackProjectionLaneRef::Labeled => {
                        if !matches!(subj_kind, KindNode::CompleteTypePack) || !matches!(res_kind, KindNode::LabeledTypePack) {
                            return Err(MetadataValidationError::Malformed(
                                "Invalid scoped PackProjection Labeled lane kinds".to_string(),
                            ));
                        }
                    }
                    crate::scoped_type::ScopedPackProjectionLaneRef::Complete => {
                        if !matches!(subj_kind, KindNode::CompleteTypePack) || !matches!(res_kind, KindNode::CompleteTypePack) {
                            return Err(MetadataValidationError::Malformed(
                                "Invalid scoped PackProjection Complete lane kinds".to_string(),
                            ));
                        }
                    }
                }
            }
            ScopedTypeNode::OpenTuple(open_tuple) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed("scoped OpenTuple node must have kind Type".to_string()));
                }
                for el in open_tuple.elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                    if el.presence == StructuralMemberPresenceRef::Optional {
                        return Err(MetadataValidationError::Malformed("scoped tuple elements cannot be optional".to_string()));
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[el.ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("scoped tuple elements must have kind Type".to_string()));
                    }
                }
                if let ScopedAnonymousTupleRemainderRef::Typed(ty) = open_tuple.positional_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed(
                            "scoped tuple positional remainder must have kind Type".to_string(),
                        ));
                    }
                }
                if let ScopedAnonymousTupleRemainderRef::Typed(ty) = open_tuple.labeled_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed(
                            "scoped tuple labeled remainder must have kind Type".to_string(),
                        ));
                    }
                }
            }
            ScopedTypeNode::AnonymousRecord(anon_rec) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed(
                        "scoped AnonymousRecord node must have kind Type".to_string(),
                    ));
                }
                let mut prev_key: Option<&crate::field_key::FieldKeyRef> = None;
                for f in anon_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[f.ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("scoped record field values must have kind Type".to_string()));
                    }
                    if let Some(prev) = prev_key {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.key.to_string().into_boxed_str(),
                            });
                        }
                    }
                    prev_key = Some(&f.key);
                }
                if let ScopedAnonymousRecordRemainderRef::Typed(ty) = anon_rec.remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("scoped record remainder must have kind Type".to_string()));
                    }
                }
            }
            ScopedTypeNode::Pack(pack) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed("scoped Pack node must have kind Type".to_string()));
                }
                for el in pack.elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                    if el.presence == StructuralMemberPresenceRef::Optional {
                        return Err(MetadataValidationError::Malformed("scoped pack elements cannot be optional".to_string()));
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[el.ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed("scoped pack elements must have kind Type".to_string()));
                    }
                }
                if let ScopedAnonymousTupleRemainderRef::Typed(ty) = pack.positional_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed(
                            "scoped pack positional remainder must have kind Type".to_string(),
                        ));
                    }
                }
                if let ScopedAnonymousTupleRemainderRef::Typed(ty) = pack.labeled_remainder {
                    if ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: ty.0 });
                    }
                    if !matches!(bundle.kinds[bundle.scoped_types[ty.0 as usize].kind.0 as usize].node, KindNode::Type) {
                        return Err(MetadataValidationError::Malformed(
                            "scoped pack labeled remainder must have kind Type".to_string(),
                        ));
                    }
                }
            }
            ScopedTypeNode::PackFromPacks { positionals, labeled } => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
                    return Err(MetadataValidationError::Malformed("scoped PackFromPacks node must have kind Type".to_string()));
                }
                if positionals.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: positionals.0,
                    });
                }
                if labeled.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: labeled.0,
                    });
                }
                let pos_kind = &bundle.kinds[bundle.scoped_types[positionals.0 as usize].kind.0 as usize].node;
                if !matches!(pos_kind, KindNode::TypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped PackFromPacks positionals must have kind TypePack".to_string(),
                    ));
                }
                let lab_kind = &bundle.kinds[bundle.scoped_types[labeled.0 as usize].kind.0 as usize].node;
                if !matches!(lab_kind, KindNode::LabeledTypePack) {
                    return Err(MetadataValidationError::Malformed(
                        "Scoped PackFromPacks labeled must have kind LabeledTypePack".to_string(),
                    ));
                }
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
                crate::generic::GenericConstraintRef::Conforms { subject, trait_application } => {
                    if subject.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: subject.0,
                            total: bundle.types.len(),
                        });
                    }
                    for argument in trait_application.arguments.iter() {
                        if argument.0 as usize >= bundle.types.len() {
                            return Err(MetadataValidationError::InvalidTypeIndex {
                                index: argument.0,
                                total: bundle.types.len(),
                            });
                        }
                    }
                    if trait_application.declaration.path.is_empty() {
                        return Err(MetadataValidationError::Malformed("conformance trait declaration path is empty".to_string()));
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

    validate_static_values(bundle)?;
    validate_static_constants(bundle)?;
    validate_static_definitions(bundle)?;

    Ok(())
}

fn validate_static_values(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    for (i, entry) in bundle.static_values.iter().enumerate() {
        let node = &entry.node;
        match node {
            crate::static_value::StaticValueNode::Tuple(elements) => {
                for elem in elements.iter() {
                    if elem.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: elem.0,
                        });
                    }
                }
            }
            crate::static_value::StaticValueNode::Sequence { result_type, elements, .. } => {
                if result_type.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: result_type.0,
                        total: bundle.types.len(),
                    });
                }
                for elem in elements.iter() {
                    if elem.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: elem.0,
                        });
                    }
                }
            }
            crate::static_value::StaticValueNode::Record(fields) => {
                for (_, val) in fields.iter() {
                    if val.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: val.0,
                        });
                    }
                }
            }
            crate::static_value::StaticValueNode::Pack { positionals, labeled } => {
                for pos in positionals.iter() {
                    if pos.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: pos.0,
                        });
                    }
                }
                for (_, val) in labeled.iter() {
                    if val.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: val.0,
                        });
                    }
                }
            }
            crate::static_value::StaticValueNode::Data { result_type, components, .. } => {
                if result_type.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: result_type.0,
                        total: bundle.types.len(),
                    });
                }
                for (_, val) in components.iter() {
                    if val.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: val.0,
                        });
                    }
                }
            }
            crate::static_value::StaticValueNode::Variant { result_type, fields, .. } => {
                if result_type.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: result_type.0,
                        total: bundle.types.len(),
                    });
                }
                for (_, val) in fields.iter() {
                    if val.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: val.0,
                        });
                    }
                }
            }
            crate::static_value::StaticValueNode::Type(ty) | crate::static_value::StaticValueNode::StaticTerm(ty) if ty.0 as usize >= bundle.types.len() => {
                return Err(MetadataValidationError::InvalidTypeIndex {
                    index: ty.0,
                    total: bundle.types.len(),
                });
            }
            _ => {}
        }

        let computed = crate::static_value::compute_static_value_fingerprint(
            node,
            |child| bundle.static_values[child.0 as usize].structural_fingerprint,
            |t| bundle.types[t.0 as usize].structural_fingerprint,
        );
        if computed != entry.structural_fingerprint {
            return Err(MetadataValidationError::Malformed(format!(
                "static value node {i} structural fingerprint mismatch"
            )));
        }
    }
    Ok(())
}

fn validate_static_constants(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    for (i, constant) in bundle.static_constants.iter().enumerate() {
        if constant.value.0 as usize >= bundle.static_values.len() {
            return Err(MetadataValidationError::Malformed(format!(
                "static constant {i} value index {} out of bounds",
                constant.value.0
            )));
        }
        let value_fp = bundle.static_values[constant.value.0 as usize].structural_fingerprint;
        if constant.value_fingerprint != value_fp {
            return Err(MetadataValidationError::Malformed(format!("static constant {i} value fingerprint mismatch")));
        }
        match constant.domain {
            crate::static_value::StaticValueDomainRef::Kind(k) => {
                if k.0 as usize >= bundle.kinds.len() {
                    return Err(MetadataValidationError::Malformed(format!(
                        "static constant {i} kind domain index {} out of bounds",
                        k.0
                    )));
                }
            }
            crate::static_value::StaticValueDomainRef::ValueType(t) => {
                if t.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: t.0,
                        total: bundle.types.len(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_static_definitions(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    for (i, def) in bundle.static_definitions.iter().enumerate() {
        for (p_idx, param) in def.parameters.iter().enumerate() {
            if param.index != p_idx as u32 {
                return Err(MetadataValidationError::Malformed(format!(
                    "static definition {i} parameter {p_idx} index mismatch"
                )));
            }
            match param.domain {
                crate::static_value::StaticValueDomainRef::Kind(k) => {
                    if k.0 as usize >= bundle.kinds.len() {
                        return Err(MetadataValidationError::Malformed(format!(
                            "static definition {i} parameter {p_idx} kind domain index {} out of bounds",
                            k.0
                        )));
                    }
                }
                crate::static_value::StaticValueDomainRef::ValueType(t) => {
                    if t.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: t.0,
                            total: bundle.types.len(),
                        });
                    }
                }
            }
        }
        match def.result_domain {
            crate::static_value::StaticValueDomainRef::Kind(k) => {
                if k.0 as usize >= bundle.kinds.len() {
                    return Err(MetadataValidationError::Malformed(format!(
                        "static definition {i} result kind domain index {} out of bounds",
                        k.0
                    )));
                }
            }
            crate::static_value::StaticValueDomainRef::ValueType(t) => {
                if t.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: t.0,
                        total: bundle.types.len(),
                    });
                }
            }
        }
    }
    Ok(())
}

pub fn compute_validator_scratch_bytes(param_count: usize, static_domain_count: usize) -> usize {
    let param_map = param_count * std::mem::size_of::<((&crate::generic::StableTypeParameterOwnerRef, u32), &crate::generic::TypeParameterRecord)>();
    let domain_map = static_domain_count
        * std::mem::size_of::<(
            (&crate::identity::StableDeclarationRef, &str),
            &crate::declaration::StaticDomainDeclarationRecord,
        )>();
    param_map.saturating_add(domain_map)
}
