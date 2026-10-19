//! Stable artifact models for static values, constants, and value domains.

use crate::field_key::FieldKeyRef;
use crate::fingerprint::{Fingerprint128, FingerprintBuilder};
use crate::identity::{
    StableDataComponentRef, StableDeclarationRef, StableModuleRef, StableProjectRef, StableStaticConstantRef, StableVariantFieldRef, StableVariantRef,
};
use crate::kind::KindNodeId;
use crate::scalar::StaticInt;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

/// Stable artifact-pool-local index for one durable static value node.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct StaticValueNodeId(pub u32);

/// Stable static value domain reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum StaticValueDomainRef {
    Kind(KindNodeId),
    ValueType(TypeNodeId),
}

/// Structural family of a durable static sequence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum StaticSequenceFamilyRef {
    Array,
    Vector,
}

/// Canonical payload for one durable static value node.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum StaticValueNode {
    Unit,
    Bool(bool),
    Char(char),
    Int(StaticInt),
    FloatBits(u64),
    String(Box<str>),
    Symbol(Box<str>),

    Type(TypeNodeId),
    StaticTerm(TypeNodeId),

    Tuple(Box<[StaticValueNodeId]>),

    Sequence {
        family: StaticSequenceFamilyRef,
        result_type: TypeNodeId,
        elements: Box<[StaticValueNodeId]>,
    },

    Record(Box<[(FieldKeyRef, StaticValueNodeId)]>),

    Pack {
        positionals: Box<[StaticValueNodeId]>,
        labeled: Box<[(Box<str>, StaticValueNodeId)]>,
    },

    Data {
        owner: StableDeclarationRef,
        result_type: TypeNodeId,
        components: Box<[(StableDataComponentRef, StaticValueNodeId)]>,
    },

    Variant {
        variant: StableVariantRef,
        result_type: TypeNodeId,
        fields: Box<[(StableVariantFieldRef, StaticValueNodeId)]>,
    },
}

/// One entry in the metadata bundle's deduplicated static value table.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct StaticValueNodeEntry {
    pub node: StaticValueNode,
    pub structural_fingerprint: Fingerprint128,
}

/// Durable static constant record in the metadata bundle.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct StaticConstantRecord {
    pub id: StableStaticConstantRef,
    pub domain: StaticValueDomainRef,
    pub value: StaticValueNodeId,
    pub value_fingerprint: Fingerprint128,
    pub dependency_fingerprint: Fingerprint128,
}

fn write_stable_project(fp: &mut FingerprintBuilder, proj: &StableProjectRef) {
    match proj {
        StableProjectRef::Builtin { namespace, version } => {
            fp.write_u8(0);
            fp.write_str(namespace);
            fp.write_str(version);
        }
        StableProjectRef::Package {
            package,
            version,
            artifact_fingerprint,
        } => {
            fp.write_u8(1);
            fp.write_str(package);
            fp.write_str(version);
            fp.write_fingerprint(*artifact_fingerprint);
        }
        StableProjectRef::SourceArtifact {
            logical_uri,
            source_fingerprint,
        } => {
            fp.write_u8(2);
            fp.write_str(logical_uri);
            fp.write_fingerprint(*source_fingerprint);
        }
        StableProjectRef::Session { session_fingerprint } => {
            fp.write_u8(3);
            fp.write_fingerprint(*session_fingerprint);
        }
    }
}

fn write_stable_module(fp: &mut FingerprintBuilder, module: &StableModuleRef) {
    write_stable_project(fp, &module.project);
    fp.write_u32(module.path.len() as u32);
    for seg in module.path.iter() {
        fp.write_str(seg);
    }
}

fn write_stable_declaration(fp: &mut FingerprintBuilder, decl: &StableDeclarationRef) {
    write_stable_module(fp, &decl.module);
    fp.write_u32(decl.path.len() as u32);
    for seg in decl.path.iter() {
        fp.write_str(seg);
    }
}

fn write_stable_variant(fp: &mut FingerprintBuilder, variant: &StableVariantRef) {
    write_stable_declaration(fp, &variant.owner);
    fp.write_str(&variant.selector);
}

fn write_field_key(fp: &mut FingerprintBuilder, key: &FieldKeyRef) {
    match key {
        FieldKeyRef::Symbol(s) => {
            fp.write_u8(0);
            fp.write_str(s);
        }
        FieldKeyRef::String(s) => {
            fp.write_u8(1);
            fp.write_str(s);
        }
        FieldKeyRef::Int(i) => {
            fp.write_u8(2);
            fp.write_bytes(&i.to_bigint().to_signed_bytes_be());
        }
        FieldKeyRef::Bool(b) => {
            fp.write_u8(3);
            fp.write_u8(*b as u8);
        }
    }
}

/// Computes the content-based structural fingerprint for a static value node.
pub fn compute_static_value_fingerprint<FNode, FType>(node: &StaticValueNode, child_fingerprint: FNode, type_fingerprint: FType) -> Fingerprint128
where
    FNode: Fn(StaticValueNodeId) -> Fingerprint128,
    FType: Fn(TypeNodeId) -> Fingerprint128,
{
    let mut fp = FingerprintBuilder::new();
    match node {
        StaticValueNode::Unit => {
            fp.write_u8(0);
        }
        StaticValueNode::Bool(b) => {
            fp.write_u8(1);
            fp.write_u8(*b as u8);
        }
        StaticValueNode::Char(c) => {
            fp.write_u8(2);
            fp.write_u32(*c as u32);
        }
        StaticValueNode::Int(i) => {
            fp.write_u8(3);
            fp.write_bytes(&i.to_bigint().to_signed_bytes_be());
        }
        StaticValueNode::FloatBits(bits) => {
            fp.write_u8(4);
            fp.write_u64(*bits);
        }
        StaticValueNode::String(s) => {
            fp.write_u8(5);
            fp.write_str(s);
        }
        StaticValueNode::Symbol(s) => {
            fp.write_u8(6);
            fp.write_str(s);
        }
        StaticValueNode::Type(ty) => {
            fp.write_u8(7);
            fp.write_fingerprint(type_fingerprint(*ty));
        }
        StaticValueNode::StaticTerm(ty) => {
            fp.write_u8(8);
            fp.write_fingerprint(type_fingerprint(*ty));
        }
        StaticValueNode::Tuple(elements) => {
            fp.write_u8(9);
            fp.write_u32(elements.len() as u32);
            for elem in elements.iter() {
                fp.write_fingerprint(child_fingerprint(*elem));
            }
        }
        StaticValueNode::Sequence { family, result_type, elements } => {
            fp.write_u8(10);
            fp.write_u8(match family {
                StaticSequenceFamilyRef::Array => 0,
                StaticSequenceFamilyRef::Vector => 1,
            });
            fp.write_fingerprint(type_fingerprint(*result_type));
            fp.write_u32(elements.len() as u32);
            for elem in elements.iter() {
                fp.write_fingerprint(child_fingerprint(*elem));
            }
        }
        StaticValueNode::Record(fields) => {
            fp.write_u8(11);
            fp.write_u32(fields.len() as u32);
            for (key, val) in fields.iter() {
                write_field_key(&mut fp, key);
                fp.write_fingerprint(child_fingerprint(*val));
            }
        }
        StaticValueNode::Pack { positionals, labeled } => {
            fp.write_u8(12);
            fp.write_u32(positionals.len() as u32);
            for pos in positionals.iter() {
                fp.write_fingerprint(child_fingerprint(*pos));
            }
            fp.write_u32(labeled.len() as u32);
            for (label, val) in labeled.iter() {
                fp.write_str(label);
                fp.write_fingerprint(child_fingerprint(*val));
            }
        }
        StaticValueNode::Data {
            owner,
            result_type,
            components,
        } => {
            fp.write_u8(13);
            write_stable_declaration(&mut fp, owner);
            fp.write_fingerprint(type_fingerprint(*result_type));
            fp.write_u32(components.len() as u32);
            for (comp, val) in components.iter() {
                fp.write_u32(comp.index);
                fp.write_fingerprint(child_fingerprint(*val));
            }
        }
        StaticValueNode::Variant { variant, result_type, fields } => {
            fp.write_u8(14);
            write_stable_variant(&mut fp, variant);
            fp.write_fingerprint(type_fingerprint(*result_type));
            fp.write_u32(fields.len() as u32);
            for (field, val) in fields.iter() {
                fp.write_u32(field.index);
                fp.write_fingerprint(child_fingerprint(*val));
            }
        }
    }
    fp.finish()
}
