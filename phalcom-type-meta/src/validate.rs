use crate::bundle::SemanticMetadataBundle;
use crate::declaration::PublishedTypeSlot;
use crate::generic::{GenericParameterModeRef, VarianceRef};
use crate::header::{MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION, SEMANTIC_MODEL_VERSION, TYPE_METADATA_SCHEMA_VERSION, supports_type_metadata_schema};
use crate::identity::StableStaticDomainDeclarationRef;
use crate::kind::{KindNode, KindNodeId, StaticDomainRef};
use crate::scoped_type::{ScopedAnonymousRecordRemainderRef, ScopedAnonymousTupleRemainderRef, ScopedRecordRowTailRef, ScopedRecordTailRef, ScopedTypeNode};
use crate::type_node::{AnonymousRecordRemainderRef, AnonymousTupleRemainderRef, RecordKeyRef, StaticArgumentRef, StructuralMemberPresenceRef, TypeNode};
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
    #[error(transparent)]
    Allocation(#[from] crate::allocation::AllocationError),
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

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BorrowedStaticDomainDecl<'a> {
    Nominal(&'a crate::identity::StableDeclarationRef),
    Associated(&'a crate::identity::StableAssociatedStaticDomainDeclarationRef),
}

impl<'a> From<&'a StableStaticDomainDeclarationRef> for BorrowedStaticDomainDecl<'a> {
    fn from(r: &'a StableStaticDomainDeclarationRef) -> Self {
        match r {
            StableStaticDomainDeclarationRef::Nominal(d) => BorrowedStaticDomainDecl::Nominal(d),
            StableStaticDomainDeclarationRef::Associated(d) => BorrowedStaticDomainDecl::Associated(d),
        }
    }
}

fn borrowed_static_domain_declaration<'a>(domain: &'a StaticDomainRef) -> Option<BorrowedStaticDomainDecl<'a>> {
    match domain {
        StaticDomainRef::ConstInt => None,
        StaticDomainRef::Nominal { declaration } => Some(BorrowedStaticDomainDecl::Nominal(declaration)),
        StaticDomainRef::Associated { declaration, .. } => Some(BorrowedStaticDomainDecl::Associated(declaration)),
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
    let actual_result = bundle.kinds.get(result_kind.0 as usize).ok_or(MetadataValidationError::InvalidKindIndex { index: result_kind.0, total: bundle.kinds.len() })?;
    let matches = if actual_kinds.len() == parameters.len() {
        bundle.kinds.get(result.0 as usize).is_some_and(|entry|entry.node==actual_result.node)
    } else {
        matches!(&actual_result.node,KindNode::Arrow { parameters: tail, result: r } if tail.as_ref()==&parameters[actual_kinds.len()..] && r==result)
    };
    if !matches {return Err(MetadataValidationError::TypeApplicationResultKindMismatch {node});}
    Ok(())
}

fn validate_static_domain_catalog<'a>(
    bundle: &'a SemanticMetadataBundle,
    budget: &mut crate::allocation::AllocationBudget,
) -> Result<Vec<(BorrowedStaticDomainDecl<'a>, &'a crate::declaration::StaticDomainDeclarationRecord)>, MetadataValidationError> {
    let mut catalog = budget.vector::<(BorrowedStaticDomainDecl<'a>, &'a crate::declaration::StaticDomainDeclarationRecord)>(bundle.static_domains.len())?;
    for record in bundle.static_domains.iter() {
        catalog.push((BorrowedStaticDomainDecl::from(&record.declaration), record));
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
    catalog.sort_by(|a, b| a.0.cmp(&b.0));
    for window in catalog.windows(2) {
        if window[0].0 == window[1].0 {
            return Err(MetadataValidationError::DuplicateStaticDomainCatalogEntry(window[0].1.declaration.clone()));
        }
    }
    Ok(catalog)
}

fn lookup_static_domain<'a, 'b>(
    catalog: &'b [(BorrowedStaticDomainDecl<'a>, &'a crate::declaration::StaticDomainDeclarationRecord)],
    decl: BorrowedStaticDomainDecl<'_>,
) -> Option<&'a crate::declaration::StaticDomainDeclarationRecord> {
    catalog.binary_search_by(|(k, _)| k.cmp(&decl)).ok().map(|pos| catalog[pos].1)
}

fn lookup_parameter<'a>(
    param_map: &[((&'a crate::generic::StableTypeParameterOwnerRef, u32), &'a crate::generic::TypeParameterRecord)],
    owner: &crate::generic::StableTypeParameterOwnerRef,
    index: u32,
) -> Option<&'a crate::generic::TypeParameterRecord> {
    param_map
        .binary_search_by(|((o, idx), _)| (o, *idx).cmp(&(&owner, index)))
        .ok()
        .map(|pos| param_map[pos].1)
}

fn validate_static_domain_instance<'a>(
    domain: &StaticDomainRef,
    catalog: &[(BorrowedStaticDomainDecl<'a>, &'a crate::declaration::StaticDomainDeclarationRecord)],
    bundle: &'a SemanticMetadataBundle,
    param_map: &[((&'a crate::generic::StableTypeParameterOwnerRef, u32), &'a crate::generic::TypeParameterRecord)],
) -> Result<(), MetadataValidationError> {
    let Some(decl) = borrowed_static_domain_declaration(domain) else {
        return Ok(());
    };
    let Some(record) = lookup_static_domain(catalog, decl) else {
        let owned_decl = match decl {
            BorrowedStaticDomainDecl::Nominal(d) => StableStaticDomainDeclarationRef::Nominal(d.clone()),
            BorrowedStaticDomainDecl::Associated(d) => StableStaticDomainDeclarationRef::Associated(d.clone()),
        };
        return Err(MetadataValidationError::MissingStaticDomainCatalogEntry(owned_decl));
    };
    if let StaticDomainRef::Associated { owner_environment, .. } = domain {
        if owner_environment.len() != record.owner_parameters.len() {
            let owned_decl = match decl {
                BorrowedStaticDomainDecl::Nominal(d) => StableStaticDomainDeclarationRef::Nominal(d.clone()),
                BorrowedStaticDomainDecl::Associated(d) => StableStaticDomainDeclarationRef::Associated(d.clone()),
            };
            return Err(MetadataValidationError::StaticDomainOwnerArityMismatch {
                declaration: owned_decl,
                expected: record.owner_parameters.len(),
                actual: owner_environment.len(),
            });
        }
        for (index, (argument, parameter)) in owner_environment.iter().zip(record.owner_parameters.iter()).enumerate() {
            let actual = argument_matches_kind(argument, parameter.kind, bundle)?;
            if actual.is_none() || actual != Some(parameter.kind) {
                let owned_decl = match decl {
                    BorrowedStaticDomainDecl::Nominal(d) => StableStaticDomainDeclarationRef::Nominal(d.clone()),
                    BorrowedStaticDomainDecl::Associated(d) => StableStaticDomainDeclarationRef::Associated(d.clone()),
                };
                return Err(MetadataValidationError::StaticDomainOwnerKindMismatch {
                    declaration: owned_decl,
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

fn validate_static_argument_row<'a>(
    argument: &StaticArgumentRef,
    before_type_node: Option<u32>,
    bundle: &'a SemanticMetadataBundle,
    param_map: &[((&'a crate::generic::StableTypeParameterOwnerRef, u32), &'a crate::generic::TypeParameterRecord)],
) -> Result<(), MetadataValidationError> {
    let StaticArgumentRef::RecordRow(row) = argument else {
        return Ok(());
    };
    let mut previous_name: Option<&RecordKeyRef> = None;
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
        if let Some(previous) = previous_name {
            if &field.key < previous {
                return Err(MetadataValidationError::RecordFieldOrderInvalid {
                    node: before_type_node.unwrap_or(u32::MAX),
                });
            }
            if &field.key == previous {
                return Err(MetadataValidationError::RecordDuplicateField {
                    node: before_type_node.unwrap_or(u32::MAX),
                    field: format!("{:?}", field.key).into_boxed_str(),
                });
            }
        }
        previous_name = Some(&field.key);
    }
    if let Some(tail) = &row.tail {
        let parameter = lookup_parameter(param_map, &tail.owner, tail.index)
            .ok_or_else(|| MetadataValidationError::RecordTailParameterMissing {
                owner: format!("{:?}", tail.owner),
                index: tail.index,
            })?;
        if !matches!(bundle.kinds[parameter.kind.0 as usize].node, KindNode::RecordRow) {
            return Err(MetadataValidationError::RecordTailKindMismatch { actual_kind: parameter.kind.0 });
        }
    }
    if let Some(remainder) = &row.remainder {
        validate_record_remainder(remainder, before_type_node, bundle)?;
    }
    Ok(())
}

fn validate_record_remainder(
    remainder: &AnonymousRecordRemainderRef,
    before_type_node: Option<u32>,
    bundle: &SemanticMetadataBundle,
) -> Result<(), MetadataValidationError> {
    let (key_type, value_type) = match remainder {
        AnonymousRecordRemainderRef::Unconstrained { key_type } => (key_type, None),
        AnonymousRecordRemainderRef::Typed { key_type, value_type } => (key_type, Some(value_type)),
    };
    for ty in std::iter::once(key_type).chain(value_type) {
        let Some(entry) = bundle.types.get(ty.0 as usize) else {
            return Err(MetadataValidationError::InvalidTypeIndex {
                index: ty.0,
                total: bundle.types.len(),
            });
        };
        if before_type_node.is_some_and(|before| ty.0 >= before) {
            return Err(MetadataValidationError::TopologicalOrderViolation {
                index: before_type_node.unwrap_or_default(),
                target: ty.0,
            });
        }
        if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
            return Err(MetadataValidationError::Malformed("record remainder domain must have kind Type".to_string()));
        }
    }
    Ok(())
}

fn validate_scoped_record_remainder(
    remainder: &ScopedAnonymousRecordRemainderRef,
    before_node: u32,
    bundle: &SemanticMetadataBundle,
) -> Result<(), MetadataValidationError> {
    let (key_type, value_type) = match remainder {
        ScopedAnonymousRecordRemainderRef::Unconstrained { key_type } => (key_type, None),
        ScopedAnonymousRecordRemainderRef::Typed { key_type, value_type } => (key_type, Some(value_type)),
    };
    for ty in std::iter::once(key_type).chain(value_type) {
        if ty.0 >= before_node {
            return Err(MetadataValidationError::TopologicalOrderViolation {
                index: before_node,
                target: ty.0,
            });
        }
        let Some(entry) = bundle.scoped_types.get(ty.0 as usize) else {
            return Err(MetadataValidationError::InvalidTypeIndex {
                index: ty.0,
                total: bundle.scoped_types.len(),
            });
        };
        if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::Type) {
            return Err(MetadataValidationError::Malformed(
                "scoped record remainder domain must have kind Type".to_string(),
            ));
        }
    }
    Ok(())
}

fn pre_v13_has_heterogeneous_record_key(bundle: &SemanticMetadataBundle) -> bool {
    let global_fields = |fields: &[crate::type_node::RecordFieldRef]| fields.iter().any(|field| !matches!(field.key, RecordKeyRef::Symbol(_)));
    let scoped_fields = |fields: &[crate::scoped_type::ScopedRecordFieldRef]| fields.iter().any(|field| !matches!(field.key, RecordKeyRef::Symbol(_)));
    let arguments = |args: &[StaticArgumentRef]| {
        args.iter()
            .any(|arg| matches!(arg, StaticArgumentRef::RecordRow(row) if global_fields(&row.fields)))
    };
    bundle.types.iter().any(|entry| match &entry.form {
        TypeNode::Record(fields) => global_fields(fields),
        TypeNode::OpenRecord(record) => global_fields(&record.fields),
        TypeNode::AnonymousRecord(record) => global_fields(&record.fields),
        TypeNode::Applied { arguments: args, .. } => arguments(args),
        TypeNode::StaticData(data) => {
            arguments(&data.arguments) || matches!(&data.domain, StaticDomainRef::Associated { owner_environment, .. } if arguments(owner_environment))
        }
        _ => false,
    }) || bundle.scoped_types.iter().any(|entry| match &entry.form {
        ScopedTypeNode::Record(fields) => scoped_fields(fields),
        ScopedTypeNode::OpenRecord(record) => scoped_fields(&record.fields),
        ScopedTypeNode::AnonymousRecord(record) => scoped_fields(&record.fields),
        ScopedTypeNode::RecordRow(row) => scoped_fields(&row.fields),
        _ => false,
    }) || bundle.kinds.iter().any(|entry| {
        matches!(&entry.node,
        KindNode::StaticDomain(StaticDomainRef::Associated { owner_environment, .. }) if arguments(owner_environment))
    })
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
    validate_metadata_bundle_with_budget(bundle, limits, &mut crate::allocation::AllocationBudget::new(usize::MAX))
}

pub fn validate_metadata_bundle_with_budget(bundle: &SemanticMetadataBundle, limits: &ValidationLimits, budget: &mut crate::allocation::AllocationBudget) -> Result<(), MetadataValidationError> {
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
    if bundle
        .header
        .legacy_fingerprint_schema_version
        .is_some_and(|version| version < MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION || version >= 13 || bundle.header.schema_version != 13)
    {
        return Err(MetadataValidationError::Malformed("invalid legacy fingerprint provenance".into()));
    }

    if bundle.header.schema_version < 13 && pre_v13_has_heterogeneous_record_key(bundle) {
        return Err(MetadataValidationError::Malformed(
            "schema before v13 cannot contain heterogeneous Record field keys".into(),
        ));
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

    let max_application_arguments = bundle.types.iter().filter_map(|e|if let TypeNode::Applied { arguments, .. }=&e.form{Some(arguments.len())}else{None})
        .chain(bundle.scoped_types.iter().filter_map(|e|if let ScopedTypeNode::Applied { arguments, .. }=&e.form{Some(arguments.len())}else{None})).max().unwrap_or(0);
    let mut actual_kinds_scratch=budget.vector::<KindNodeId>(max_application_arguments)?;

    // Index parameters by (owner, index) -> param_entry for fast tail lookup
    let mut param_map = budget.vector::<((&crate::generic::StableTypeParameterOwnerRef, u32), &crate::generic::TypeParameterRecord)>(bundle.parameters.len())?;
    for param in bundle.parameters.iter() {
        if param.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: param.kind.0,
                total: bundle.kinds.len(),
            });
        }
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
        param_map.push(((&param.id.owner, param.id.index), param));
    }
    param_map.sort_by(|a, b| a.0.cmp(&b.0));
    for window in param_map.windows(2) {
        if window[0].0 == window[1].0 {
            return Err(MetadataValidationError::DuplicateParameterOwner(format!("{:?}", window[0].0.0), window[0].0.1));
        }
    }

    let static_domain_catalog = if bundle.header.schema_version >= 8 {
        validate_static_domain_catalog(bundle, budget)?
    } else {
        Vec::new()
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
                    if let Some(decl) = borrowed_static_domain_declaration(&static_data.domain) {
                        let record = lookup_static_domain(&static_domain_catalog, decl)
                            .ok_or_else(|| {
                                let owned_decl = match decl {
                                    BorrowedStaticDomainDecl::Nominal(d) => StableStaticDomainDeclarationRef::Nominal(d.clone()),
                                    BorrowedStaticDomainDecl::Associated(d) => StableStaticDomainDeclarationRef::Associated(d.clone()),
                                };
                                MetadataValidationError::MissingStaticDomainCatalogEntry(owned_decl)
                            })?;
                        let case = record.cases.iter().find(|case| case.name == static_data.constructor.case_name).ok_or_else(|| {
                            let owned_decl = match decl {
                                BorrowedStaticDomainDecl::Nominal(d) => StableStaticDomainDeclarationRef::Nominal(d.clone()),
                                BorrowedStaticDomainDecl::Associated(d) => StableStaticDomainDeclarationRef::Associated(d.clone()),
                            };
                            MetadataValidationError::MissingStaticDomainCase {
                                declaration: owned_decl,
                                case: static_data.constructor.case_name.clone(),
                            }
                        })?;
                        if case.parameter_kinds.len() != static_data.arguments.len() {
                            let owned_decl = match decl {
                                BorrowedStaticDomainDecl::Nominal(d) => StableStaticDomainDeclarationRef::Nominal(d.clone()),
                                BorrowedStaticDomainDecl::Associated(d) => StableStaticDomainDeclarationRef::Associated(d.clone()),
                            };
                            return Err(MetadataValidationError::StaticDomainCaseArityMismatch {
                                declaration: Box::new(owned_decl),
                                case: case.name.clone(),
                                expected: case.parameter_kinds.len(),
                                actual: static_data.arguments.len(),
                            });
                        }
                        for (index, (argument, &expected_kind)) in static_data.arguments.iter().zip(case.parameter_kinds.iter()).enumerate() {
                            let actual_kind = argument_matches_kind(argument, expected_kind, bundle)?;
                            if actual_kind != Some(expected_kind) {
                                let owned_decl = match decl {
                                    BorrowedStaticDomainDecl::Nominal(d) => StableStaticDomainDeclarationRef::Nominal(d.clone()),
                                    BorrowedStaticDomainDecl::Associated(d) => StableStaticDomainDeclarationRef::Associated(d.clone()),
                                };
                                return Err(MetadataValidationError::StaticDomainCaseKindMismatch {
                                    declaration: Box::new(owned_decl),
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
                    actual_kinds_scratch.clear();
                    let actual_kinds = &mut actual_kinds_scratch;
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
                let mut prev_name: Option<&RecordKeyRef> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: format!("{:?}", f.key).into_boxed_str(),
                            });
                        }
                    }
                    prev_name = Some(&f.key);
                }
            }
            TypeNode::OpenRecord(open_rec) => {
                let mut prev_name: Option<&RecordKeyRef> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: format!("{:?}", f.key).into_boxed_str(),
                            });
                        }
                    }
                    prev_name = Some(&f.key);
                }
                let tail_param = lookup_parameter(&param_map, &open_rec.tail.owner, open_rec.tail.index)
                    .ok_or_else(|| MetadataValidationError::RecordTailParameterMissing {
                        owner: format!("{:?}", open_rec.tail.owner),
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
                let mut prev_name: Option<&RecordKeyRef> = None;
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
                    if let Some(prev) = prev_name {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: format!("{:?}", f.key).into_boxed_str(),
                            });
                        }
                    }
                    prev_name = Some(&f.key);
                }
                validate_record_remainder(&anon_rec.remainder, Some(i as u32), bundle)?;
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
    let lambda_count=bundle.scoped_types.iter().filter(|e|matches!(e.form, ScopedTypeNode::Lambda { .. })).count();
    let mut lambda_param_kinds: Vec<&[KindNodeId]> = budget.vector(lambda_count)?;
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
                    actual_kinds_scratch.clear();
                    actual_kinds_scratch.extend(arguments.iter().map(|argument|bundle.scoped_types[argument.0 as usize].kind));
                    let actual_kinds=&actual_kinds_scratch;
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
                let mut prev_name: Option<&RecordKeyRef> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: format!("{:?}", f.key).into_boxed_str(),
                            });
                        }
                    }
                    prev_name = Some(&f.key);
                }
            }
            ScopedTypeNode::OpenRecord(open_rec) => {
                let mut prev_name: Option<&RecordKeyRef> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: format!("{:?}", f.key).into_boxed_str(),
                            });
                        }
                    }
                    prev_name = Some(&f.key);
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
                        let tail_param = lookup_parameter(&param_map, &param_ref.owner, param_ref.index)
                            .ok_or_else(|| MetadataValidationError::RecordTailParameterMissing {
                                owner: format!("{:?}", param_ref.owner),
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
                        validate_scoped_record_remainder(remainder, i as u32, bundle)?;
                    }
                }
            }
            ScopedTypeNode::RecordRow(row) => {
                if !matches!(bundle.kinds[entry.kind.0 as usize].node, KindNode::RecordRow) {
                    return Err(MetadataValidationError::Malformed(format!(
                        "scoped RecordRow node {i} must have kind RecordRow"
                    )));
                }
                let mut previous_name: Option<&RecordKeyRef> = None;
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
                            format!("{:?}", field.key)
                        )));
                    }
                    if let Some(previous) = previous_name {
                        if &field.key < previous {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        }
                        if &field.key == previous {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: format!("{:?}", field.key).into_boxed_str(),
                            });
                        }
                    }
                    previous_name = Some(&field.key);
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
                        let record_parameter =
                            lookup_parameter(&param_map, &parameter.owner, parameter.index)
                                .ok_or_else(|| MetadataValidationError::RecordTailParameterMissing {
                                    owner: format!("{:?}", parameter.owner),
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
                        validate_scoped_record_remainder(remainder, i as u32, bundle)?;
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
                lambda_param_kinds.push(parameter_kinds);
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
                let mut prev_name: Option<&RecordKeyRef> = None;
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
                    if let Some(prev) = prev_name {
                        if &f.key < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &f.key == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: format!("{:?}", f.key).into_boxed_str(),
                            });
                        }
                    }
                    prev_name = Some(&f.key);
                }
                validate_scoped_record_remainder(&anon_rec.remainder, i as u32, bundle)?;
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

    // Parameter kind bounds and owner uniqueness were established by param_map.

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

    Ok(())
}

/// Computes the exact scratch memory required by validation indexes:
/// `param_map` and `static_domain_catalog`.
pub fn compute_validator_scratch_bytes(param_count: usize, static_domain_count: usize) -> usize {
    std::mem::size_of::<((&crate::generic::StableTypeParameterOwnerRef, u32), &crate::generic::TypeParameterRecord)>()
        .saturating_mul(param_count)
        .saturating_add(
            std::mem::size_of::<(BorrowedStaticDomainDecl<'_>, &crate::declaration::StaticDomainDeclarationRecord)>()
                .saturating_mul(static_domain_count),
        )
}
