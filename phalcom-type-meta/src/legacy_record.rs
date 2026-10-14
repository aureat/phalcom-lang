//! Relocate pre-v13 Record graphs after the loader supplies canonical Symbol identity.
//! Existing digests remain version-tagged legacy provenance; this module never hashes types.

use crate::bundle::{SemanticMetadataBundle, TypeUseStatusRef};
use crate::declaration::PublishedTypeSlot;
use crate::fingerprint::Fingerprint128;
use crate::generic::GenericConstraintRef;
use crate::header::TYPE_METADATA_SCHEMA_VERSION;
use crate::identity::{StableDeclarationRef, StableProjectRef};
use crate::kind::{KindNode, StaticDomainRef};
use crate::scoped_type::{
    ScopedAnonymousRecordRemainderRef, ScopedAnonymousTupleRemainderRef, ScopedRecordRowTailRef, ScopedRecordTailRef, ScopedTypeNode, ScopedTypeNodeEntry,
    ScopedTypeNodeId,
};
use crate::type_node::{
    AnonymousRecordRemainderRef, AnonymousTupleRemainderRef, StaticArgumentRef, TypeMemberArgumentRef, TypeNode, TypeNodeEntry, TypeNodeId,
};

/// Only a loader with a validated canonical Universe identity may supply this.
/// The version comes from the loader, never from a legacy producer header.
pub fn validate_symbol_identity(symbol: &StableDeclarationRef) -> bool {
    matches!(&symbol.module.project, StableProjectRef::Builtin { namespace, version } if namespace.as_ref() == "universe" && !version.is_empty())
        && symbol.module.path.iter().map(AsRef::as_ref).eq(["scalar", "symbol"])
        && symbol.path.iter().map(AsRef::as_ref).eq(["Symbol"])
}

const INJECTED_KEY: u32 = u32::MAX;

/// Converts only old Record remainder variants. Fixed `name` fields are handled
/// by the decoder independently and need no nominal Symbol type identity.
pub fn normalize_remainder_json(value: &mut serde_json::Value) -> bool {
    let mut changed = false;
    match value {
        serde_json::Value::Object(object) => {
            if let Some(remainder) = object.get_mut("remainder") {
                changed |= normalize_one_remainder(remainder);
            }
            if let Some(serde_json::Value::Object(tail)) = object.get_mut("tail") {
                if let Some(remainder) = tail.get_mut("Anonymous") {
                    changed |= normalize_one_remainder(remainder);
                }
            }
            for child in object.values_mut() {
                changed |= normalize_remainder_json(child);
            }
        }
        serde_json::Value::Array(children) => {
            for child in children {
                changed |= normalize_remainder_json(child);
            }
        }
        _ => {}
    }
    changed
}

fn normalize_one_remainder(value: &mut serde_json::Value) -> bool {
    if value == "Unconstrained" {
        *value = serde_json::json!({"Unconstrained": {"key_type": INJECTED_KEY}});
        return true;
    }
    if let Some(ty) = value.get("Typed").filter(|ty| ty.is_u64()).cloned() {
        *value = serde_json::json!({"Typed": {"key_type": INJECTED_KEY, "value_type": ty}});
        return true;
    }
    false
}

fn global(id: &mut TypeNodeId) {
    id.0 = id.0.saturating_add(1);
}
fn scoped(id: &mut ScopedTypeNodeId) {
    id.0 = id.0.saturating_add(1);
}
fn remainder_global(rem: &mut AnonymousRecordRemainderRef) {
    match rem {
        AnonymousRecordRemainderRef::Unconstrained { key_type } => key(key_type),
        AnonymousRecordRemainderRef::Typed { key_type, value_type } => {
            key(key_type);
            global(value_type);
        }
    }
}
fn remainder_scoped(rem: &mut ScopedAnonymousRecordRemainderRef, offset: bool) {
    match rem {
        ScopedAnonymousRecordRemainderRef::Unconstrained { key_type } => scoped_key(key_type, offset),
        ScopedAnonymousRecordRemainderRef::Typed { key_type, value_type } => {
            scoped_key(key_type, offset);
            if offset {
                scoped(value_type);
            }
        }
    }
}
fn key(id: &mut TypeNodeId) {
    if id.0 == INJECTED_KEY {
        id.0 = 0;
    } else {
        global(id);
    }
}
fn scoped_key(id: &mut ScopedTypeNodeId, offset: bool) {
    if id.0 == INJECTED_KEY {
        id.0 = 0;
    } else if offset {
        scoped(id);
    }
}
fn tuple_remainder(rem: &mut AnonymousTupleRemainderRef) {
    if let AnonymousTupleRemainderRef::Typed(id) = rem {
        global(id);
    }
}
fn scoped_tuple_remainder(rem: &mut ScopedAnonymousTupleRemainderRef, offset: bool) {
    if let ScopedAnonymousTupleRemainderRef::Typed(id) = rem {
        if offset {
            scoped(id);
        }
    }
}
fn row(row: &mut crate::type_node::RecordRowRef) {
    for field in &mut row.fields {
        global(&mut field.ty);
    }
    if let Some(rem) = &mut row.remainder {
        remainder_global(rem);
    }
}
fn argument(arg: &mut StaticArgumentRef) {
    match arg {
        StaticArgumentRef::Term(id) => global(id),
        StaticArgumentRef::RecordRow(row) => self::row(row),
        StaticArgumentRef::DynamicType => {}
    }
}
fn domain(domain: &mut StaticDomainRef) {
    if let StaticDomainRef::Associated { owner_environment, .. } = domain {
        for arg in owner_environment {
            argument(arg);
        }
    }
}
fn type_node(node: &mut TypeNode, scoped_offset: bool) {
    match node {
        TypeNode::Never | TypeNode::Unit | TypeNode::ConstInt(_) | TypeNode::Nominal { .. } | TypeNode::Parameter(_) | TypeNode::SelfType(_) => {}
        TypeNode::StaticData(data) => {
            domain(&mut data.domain);
            for arg in &mut data.arguments {
                argument(arg);
            }
        }
        TypeNode::Applied { origin, arguments } => {
            global(origin);
            for arg in arguments {
                argument(arg);
            }
        }
        TypeNode::Union(ids) | TypeNode::TypePack(ids) | TypeNode::TypePackConcat(ids) | TypeNode::LabeledTypePackConcat(ids) => {
            for id in ids {
                global(id);
            }
        }
        TypeNode::Tuple(elements) => {
            for el in elements {
                global(&mut el.ty);
            }
        }
        TypeNode::Record(fields) => {
            for field in fields {
                global(&mut field.ty);
            }
        }
        TypeNode::Callable(call) => {
            for param in &mut call.parameters {
                global(&mut param.ty);
            }
            global(&mut call.return_type);
        }
        TypeNode::TypeLambda(lambda) => {
            if scoped_offset {
                scoped(&mut lambda.body);
            }
        }
        TypeNode::OpenRecord(record) => {
            for field in &mut record.fields {
                global(&mut field.ty);
            }
        }
        TypeNode::AssociatedProjection {
            subject, trait_application, ..
        } => {
            global(subject);
            for id in &mut trait_application.arguments {
                global(id);
            }
        }
        TypeNode::LabeledTypePack(elements) => {
            for el in elements {
                global(&mut el.element);
            }
        }
        TypeNode::CompleteTypePack { positionals, labeled }
        | TypeNode::TupleFromPacks { positionals, labeled }
        | TypeNode::PackFromPacks { positionals, labeled } => {
            global(positionals);
            global(labeled);
        }
        TypeNode::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            global(positionals);
            global(labeled);
            global(return_type);
        }
        TypeNode::TypeMemberProjection(projection) => {
            global(&mut projection.subject);
            for arg in &mut projection.arguments {
                if let TypeMemberArgumentRef::Type(id) = arg {
                    global(id);
                }
            }
        }
        TypeNode::OpenTuple(tuple) => {
            for el in &mut tuple.elements {
                global(&mut el.ty);
            }
            tuple_remainder(&mut tuple.positional_remainder);
            tuple_remainder(&mut tuple.labeled_remainder);
        }
        TypeNode::AnonymousRecord(record) => {
            for field in &mut record.fields {
                global(&mut field.ty);
            }
            remainder_global(&mut record.remainder);
        }
        TypeNode::Pack(pack) => {
            for el in &mut pack.elements {
                global(&mut el.ty);
            }
            tuple_remainder(&mut pack.positional_remainder);
            tuple_remainder(&mut pack.labeled_remainder);
        }
    }
}
fn scoped_node(node: &mut ScopedTypeNode, offset: bool) {
    let shift = |id: &mut ScopedTypeNodeId| {
        if offset {
            scoped(id);
        }
    };
    match node {
        ScopedTypeNode::Bound { .. } => {}
        ScopedTypeNode::Free(id) => global(id),
        ScopedTypeNode::Applied { origin, arguments } => {
            shift(origin);
            for id in arguments {
                shift(id);
            }
        }
        ScopedTypeNode::Union(ids) | ScopedTypeNode::TypePack(ids) | ScopedTypeNode::TypePackConcat(ids) | ScopedTypeNode::LabeledTypePackConcat(ids) => {
            for id in ids {
                shift(id);
            }
        }
        ScopedTypeNode::Tuple(elements)
        | ScopedTypeNode::OpenTuple(crate::scoped_type::ScopedOpenTupleTypeRef { elements, .. })
        | ScopedTypeNode::Pack(crate::scoped_type::ScopedPackTypeRef { elements, .. }) => {
            for el in elements {
                shift(&mut el.ty);
            }
            match node {
                ScopedTypeNode::OpenTuple(tuple) => {
                    scoped_tuple_remainder(&mut tuple.positional_remainder, offset);
                    scoped_tuple_remainder(&mut tuple.labeled_remainder, offset);
                }
                ScopedTypeNode::Pack(pack) => {
                    scoped_tuple_remainder(&mut pack.positional_remainder, offset);
                    scoped_tuple_remainder(&mut pack.labeled_remainder, offset);
                }
                _ => {}
            }
        }
        ScopedTypeNode::Record(fields) => {
            for field in fields {
                shift(&mut field.ty);
            }
        }
        ScopedTypeNode::Callable(call) => {
            for param in &mut call.parameters {
                shift(&mut param.ty);
            }
            shift(&mut call.return_type);
        }
        ScopedTypeNode::Lambda { body, .. } => shift(body),
        ScopedTypeNode::OpenRecord(record) => {
            for field in &mut record.fields {
                shift(&mut field.ty);
            }
            if let ScopedRecordTailRef::Anonymous(rem) = &mut record.tail {
                remainder_scoped(rem, offset);
            }
        }
        ScopedTypeNode::RecordRow(row) => {
            for field in &mut row.fields {
                shift(&mut field.ty);
            }
            match &mut row.tail {
                ScopedRecordRowTailRef::Projection(id) => global(id),
                ScopedRecordRowTailRef::Anonymous(rem) => remainder_scoped(rem, offset),
                _ => {}
            }
        }
        ScopedTypeNode::LabeledTypePack(elements) => {
            for el in elements {
                shift(&mut el.element);
            }
        }
        ScopedTypeNode::CompleteTypePack { positionals, labeled }
        | ScopedTypeNode::TupleFromPacks { positionals, labeled }
        | ScopedTypeNode::PackFromPacks { positionals, labeled } => {
            shift(positionals);
            shift(labeled);
        }
        ScopedTypeNode::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            shift(positionals);
            shift(labeled);
            shift(return_type);
        }
        ScopedTypeNode::PackProjection { subject, .. } => shift(subject),
        ScopedTypeNode::AnonymousRecord(record) => {
            for field in &mut record.fields {
                shift(&mut field.ty);
            }
            remainder_scoped(&mut record.remainder, offset);
        }
    }
}

fn slot(slot: &mut PublishedTypeSlot) {
    if let PublishedTypeSlot::Known { form, .. } = slot {
        global(form);
    }
}

/// Insertion before existing nodes preserves topological child-before-parent
/// order for global and scoped graphs, including deeply nested static rows.
pub fn relocate(mut bundle: SemanticMetadataBundle, symbol: StableDeclarationRef) -> Result<SemanticMetadataBundle, &'static str> {
    if !validate_symbol_identity(&symbol) {
        return Err("invalid canonical Symbol identity");
    }
    if bundle.types.iter().any(|entry| {
        matches!(&entry.form, TypeNode::Nominal { declaration }
        if declaration.path.iter().map(AsRef::as_ref).eq(["Symbol"])
        && declaration.module.path.iter().map(AsRef::as_ref).eq(["scalar", "symbol"])
        && matches!(&declaration.module.project, StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe")
        && declaration != &symbol)
    }) {
        return Err("legacy Symbol identity conflicts with loader context");
    }
    let kind = bundle
        .kinds
        .iter()
        .position(|entry| matches!(entry.node, KindNode::Type))
        .ok_or("legacy graph has no Type kind")? as u32;
    let scoped_offset = bundle.scoped_types.iter().any(|entry| match &entry.form {
        ScopedTypeNode::AnonymousRecord(_) => true,
        ScopedTypeNode::OpenRecord(rec) => matches!(rec.tail, ScopedRecordTailRef::Anonymous(_)),
        ScopedTypeNode::RecordRow(row) => matches!(row.tail, ScopedRecordRowTailRef::Anonymous(_)),
        _ => false,
    });
    for kind in &mut bundle.kinds {
        if let KindNode::StaticDomain(domain) = &mut kind.node {
            self::domain(domain);
        }
    }
    for entry in &mut bundle.types {
        type_node(&mut entry.form, scoped_offset);
    }
    for entry in &mut bundle.scoped_types {
        scoped_node(&mut entry.form, scoped_offset);
    }
    for signature in &mut bundle.generic_signatures {
        for constraint in &mut signature.constraints {
            match constraint {
                GenericConstraintRef::Subtype { lower, upper } => {
                    global(lower);
                    global(upper);
                }
                GenericConstraintRef::Equivalent { left, right } => {
                    global(left);
                    global(right);
                }
                GenericConstraintRef::Conforms { subject, trait_application } => {
                    global(subject);
                    for arg in &mut trait_application.arguments {
                        global(arg);
                    }
                }
            }
        }
    }
    for declaration in &mut bundle.declarations {
        global(&mut declaration.form);
        if let Some(ty) = &mut declaration.superclass_template {
            global(ty);
        }
    }
    for alias in &mut bundle.aliases {
        global(&mut alias.target);
    }
    for callable in &mut bundle.callables {
        for param in &mut callable.parameters {
            slot(&mut param.ty);
        }
        slot(&mut callable.return_type);
    }
    for field in &mut bundle.fields {
        slot(&mut field.ty);
    }
    for root in &mut bundle.runtime_roots {
        global(&mut root.form);
    }
    for occurrence in &mut bundle.occurrences {
        if let TypeUseStatusRef::Known(id) = &mut occurrence.status {
            global(id);
        }
    }
    let mut types = Vec::with_capacity(bundle.types.len() + 1);
    types.push(TypeNodeEntry {
        kind: crate::kind::KindNodeId(kind),
        form: TypeNode::Nominal { declaration: symbol },
        structural_fingerprint: Fingerprint128::ZERO,
    });
    types.extend(bundle.types);
    bundle.types = types.into_boxed_slice();
    if scoped_offset {
        let mut scoped_types = Vec::with_capacity(bundle.scoped_types.len() + 1);
        scoped_types.push(ScopedTypeNodeEntry {
            kind: crate::kind::KindNodeId(kind),
            form: ScopedTypeNode::Free(TypeNodeId(0)),
            structural_fingerprint: Fingerprint128::ZERO,
        });
        scoped_types.extend(bundle.scoped_types);
        bundle.scoped_types = scoped_types.into_boxed_slice();
    }
    bundle.header.legacy_fingerprint_schema_version = Some(bundle.header.schema_version);
    bundle.header.schema_version = TYPE_METADATA_SCHEMA_VERSION;
    Ok(bundle)
}
