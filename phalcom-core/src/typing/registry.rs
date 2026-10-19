//! VM-owned RuntimeTypingRegistry.

use crate::heap::ClassId;
use crate::typing::handle::{MetadataPoolId, RuntimeCallableSignatureRef, RuntimeDeclarationMetadataRef, RuntimeFieldSignatureRef};
use crate::typing::loader::{LoadedSemanticMetadata, RuntimeNominalBindingTable};
use crate::typing::side_table::{MethodImplementationIndex, MethodSemanticIndex};
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableFieldRef};
use std::sync::Arc;

/// Central runtime typing registry owned by the VM.
#[derive(Clone, Debug, Default)]
pub struct RuntimeTypingRegistry {
    pools: Vec<Arc<LoadedSemanticMetadata>>,
    nominal_bindings: RuntimeNominalBindingTable,
    runtime_declarations: std::collections::HashMap<ClassId, StableDeclarationRef>,
    declaration_identities: std::collections::HashMap<(phalcom_modules::ModuleId, Box<str>), StableDeclarationRef>,
    declaration_metadata: std::collections::HashMap<StableDeclarationRef, RuntimeDeclarationMetadataRef>,
    callable_metadata: std::collections::HashMap<StableCallableRef, RuntimeCallableSignatureRef>,
    field_metadata: std::collections::HashMap<StableFieldRef, RuntimeFieldSignatureRef>,
    pub method_semantics: MethodSemanticIndex,
    pub method_implementations: MethodImplementationIndex,
}

impl RuntimeTypingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_pool(&mut self, pool: LoadedSemanticMetadata) -> MetadataPoolId {
        let id = MetadataPoolId(self.pools.len() as u32);
        for (idx, decl) in pool.bundle.declarations.iter().enumerate() {
            self.declaration_metadata.insert(
                decl.declaration.clone(),
                RuntimeDeclarationMetadataRef {
                    pool: id,
                    record: phalcom_type_meta::declaration::DeclarationRecordId(idx as u32),
                },
            );
        }
        for (idx, callable) in pool.bundle.callables.iter().enumerate() {
            self.callable_metadata.insert(
                callable.callable.clone(),
                RuntimeCallableSignatureRef {
                    pool: id,
                    record: phalcom_type_meta::declaration::CallableRecordId(idx as u32),
                    specialization_receiver: None,
                },
            );
        }
        for (idx, field) in pool.bundle.fields.iter().enumerate() {
            self.field_metadata.insert(
                field.field.clone(),
                RuntimeFieldSignatureRef {
                    pool: id,
                    record: phalcom_type_meta::declaration::FieldRecordId(idx as u32),
                    specialization_receiver: None,
                },
            );
        }
        self.pools.push(Arc::new(pool));
        id
    }

    pub fn get_pool(&self, id: MetadataPoolId) -> Option<&Arc<LoadedSemanticMetadata>> {
        self.pools.get(id.0 as usize)
    }

    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }

    pub fn pools(&self) -> &[Arc<LoadedSemanticMetadata>] {
        &self.pools
    }

    pub fn register_nominal_binding(&mut self, decl: StableDeclarationRef, class: ClassId) {
        self.nominal_bindings.insert(decl.clone(), class);
        self.runtime_declarations.insert(class, decl);
    }

    /// Records the stable declaration corresponding to one runtime module
    /// symbol before its class is created by module initialization.
    pub fn register_declaration_identity(&mut self, module: phalcom_modules::ModuleId, name: Box<str>, declaration: StableDeclarationRef) {
        self.declaration_identities.insert((module, name), declaration);
    }

    pub fn declaration_identity(&self, module: &phalcom_modules::ModuleId, name: &str) -> Option<&StableDeclarationRef> {
        self.declaration_identities.get(&(module.clone(), name.into()))
    }

    pub fn resolve_nominal(&self, decl: &StableDeclarationRef) -> Option<ClassId> {
        self.nominal_bindings.get(decl)
    }

    /// Returns the exact metadata declaration previously associated with a
    /// runtime class. Class display names are presentation only and cannot
    /// identify this declaration.
    pub fn declaration_for_nominal(&self, class: ClassId) -> Option<&StableDeclarationRef> {
        self.runtime_declarations.get(&class)
    }

    pub fn lookup_declaration_metadata(&self, decl: &StableDeclarationRef) -> Option<RuntimeDeclarationMetadataRef> {
        self.declaration_metadata.get(decl).copied()
    }

    pub fn lookup_callable_metadata(&self, callable: &StableCallableRef) -> Option<RuntimeCallableSignatureRef> {
        self.callable_metadata.get(callable).copied()
    }

    pub fn lookup_field_metadata(&self, field: &StableFieldRef) -> Option<RuntimeFieldSignatureRef> {
        self.field_metadata.get(field).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_type_meta::bundle::SemanticMetadataBundle;
    use phalcom_type_meta::declaration::{
        CallableSemanticRecord, DeclarationTypeFlags, DeclarationTypeRecord, FieldMutabilityRef, FieldSemanticRecord, PublishedTypeAuthority, PublishedTypeSlot,
    };
    use phalcom_type_meta::fingerprint::Fingerprint128;
    use phalcom_type_meta::header::*;
    use phalcom_type_meta::identity::*;
    use phalcom_type_meta::type_node::TypeNodeId;

    fn test_header() -> SemanticMetadataHeader {
        SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            legacy_fingerprint_schema_version: None,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("test".into()),
            producer_version: "1".into(),
            native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
            profile: MetadataProfile::RuntimePublic,
            features: MetadataFeatures::default(),
            identity_scheme: ArtifactIdentityScheme::V1Standard,
            source_fingerprint: Fingerprint128::ZERO,
            interface_fingerprint: Fingerprint128::ZERO,
        }
    }

    #[test]
    fn register_pool_indexes_declarations_callables_and_fields() {
        let mut registry = RuntimeTypingRegistry::new();

        let decl_ref = StableDeclarationRef {
            module: StableModuleRef {
                project: StableProjectRef::Builtin {
                    namespace: "universe".into(),
                    version: "0.1.0".into(),
                },
                path: Box::new(["test".into()]),
            },
            path: Box::new(["MyClass".into()]),
        };

        let callable_ref = StableCallableRef {
            owner: StableCallableOwnerRef::Declaration(decl_ref.clone()),
            side: StableDispatchSide::Instance,
            selector: "myMethod".into(),
        };

        let field_ref = StableFieldRef {
            owner: decl_ref.clone(),
            side: StableDispatchSide::Instance,
            name: "myField".into(),
        };

        let bundle = SemanticMetadataBundle {
            header: test_header(),
            kinds: Box::new([]),
            types: Box::new([]),
            scoped_types: Box::new([]),
            parameters: Box::new([]),
            generic_signatures: Box::new([]),
            declarations: Box::new([DeclarationTypeRecord {
                declaration: decl_ref.clone(),
                form: TypeNodeId(0),
                kind: phalcom_type_meta::kind::KindNodeId(0),
                generic_signature: None,
                superclass_template: None,
                instance_callables: Box::new([]),
                class_callables: Box::new([]),
                instance_fields: Box::new([]),
                class_fields: Box::new([]),
                flags: DeclarationTypeFlags::default(),
                algebraic: None,
                source: None,
            }]),
            static_domains: Box::new([]),
            aliases: Box::new([]),
            callables: Box::new([CallableSemanticRecord {
                callable: callable_ref.clone(),
                generic_signature: None,
                parameters: Box::new([]),
                return_type: PublishedTypeSlot::Known {
                    form: TypeNodeId(0),
                    authority: PublishedTypeAuthority::DeclaredAnnotation,
                },
                source: None,
            }]),
            fields: Box::new([FieldSemanticRecord {
                field: field_ref.clone(),
                mutability: FieldMutabilityRef::Immutable,
                ty: PublishedTypeSlot::Known {
                    form: TypeNodeId(0),
                    authority: PublishedTypeAuthority::DeclaredAnnotation,
                },
                source: None,
            }]),
            module_roots: Box::new([]),
            runtime_roots: Box::new([]),
            occurrences: Box::new([]),
            extensions: Box::new([]),
        };

        let pool_id = registry.register_pool(LoadedSemanticMetadata::new(MetadataPoolId(0), Arc::new(bundle)));
        assert_eq!(pool_id, MetadataPoolId(0));

        let decl_meta = registry.lookup_declaration_metadata(&decl_ref).expect("declaration metadata");
        assert_eq!(decl_meta.pool, pool_id);
        assert_eq!(decl_meta.record.0, 0);

        let callable_meta = registry.lookup_callable_metadata(&callable_ref).expect("callable metadata");
        assert_eq!(callable_meta.pool, pool_id);
        assert_eq!(callable_meta.record.0, 0);

        let field_meta = registry.lookup_field_metadata(&field_ref).expect("field metadata");
        assert_eq!(field_meta.pool, pool_id);
        assert_eq!(field_meta.record.0, 0);
    }
}
