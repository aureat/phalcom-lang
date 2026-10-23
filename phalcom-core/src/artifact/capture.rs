//! Capture of executable products emitted by the canonical Universe compiler.
//!
//! This module observes the existing emitter. It never constructs bytecode
//! from syntax and never retains build-VM handles in its output.

use crate::heap::{ObjRef, Object};
use crate::value::Value;
use crate::vm::{RuntimeOutput, VM};
use phalcom_compiler_artifact::executable::compatibility::{CompilationProfileV1, ExecutableCompatibilityV1, PortableNativeAnchorV1, PrimordialRoleV1};
use phalcom_compiler_artifact::executable::constant::ConstantRecipeV1;
use phalcom_compiler_artifact::executable::constant::NativeAnchorId;
use phalcom_compiler_artifact::executable::image::ExecutableImageV1;
use phalcom_compiler_artifact::executable::module::{
    PortableBootstrapModuleV1, PortableBootstrapPlanV1, PortableLinkedReadV1, PortableModuleV1, PortableSourceMapV1,
};
use phalcom_compiler_artifact::executable::prototype::{ConstantId, PortablePrototypeV1, PrototypeId, PrototypeOwnerV1, SemanticPoolId, SourceMapId};
use phalcom_compiler_artifact::executable::semantic::PortableSemanticPoolV1;
use phalcom_semantic::metadata::stable_identity::to_stable_module;
use phalcom_type_meta::identity::StableModuleRef;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use thiserror::Error;

struct MethodCaptureSnapshot {
    kind: crate::method::MethodKind,
    native_key: Option<phalcom_native_meta::PrimitiveKey>,
    signature: crate::method::Signature,
    holder: Option<crate::heap::ClassId>,
    visibility: crate::method::MemberVisibility,
    access_owner: Option<crate::heap::ClassId>,
    origin: crate::method::RuntimeAccessOrigin,
    contracts: Option<Vec<(crate::interner::Symbol, Value)>>,
    attributes: Vec<Value>,
    attributes_frozen: bool,
}

enum ConstantObjectSnapshot {
    String(String),
    LargeInt(num_bigint::BigInt),
    Closure(crate::heap::ClosureObject),
    Vector(Vec<Value>),
    Array { frozen: bool, elements: Vec<Value> },
    Class,
    Module(phalcom_modules::ModuleId),
    SelectorPattern(phalcom_common::selector::SelectorPattern),
    Product,
    Method(MethodCaptureSnapshot),
    Unsupported(&'static str),
}

#[derive(Debug, Error)]
pub enum ExecutableCaptureError {
    #[error("canonical executable capture bootstrap failed: {0}")]
    Bootstrap(String),
    #[error("module {module:?}: compiler did not emit a closure initializer")]
    MissingInitializer { module: StableModuleRef },
    #[error("module {module:?}, prototype {owner:?}: executable semantic pools are not yet portable ({families})")]
    UnsupportedSemanticPool {
        module: StableModuleRef,
        owner: PrototypeOwnerV1,
        families: String,
    },
    #[error("module {module:?}, prototype {owner:?}, semantic family {family}: {reason}")]
    UnsupportedSemanticProduct {
        module: StableModuleRef,
        owner: PrototypeOwnerV1,
        family: &'static str,
        reason: String,
    },
    #[error("module {module:?}, prototype {owner:?}, constant {index}: unsupported executable constant {object_kind}")]
    UnsupportedConstant {
        module: StableModuleRef,
        owner: PrototypeOwnerV1,
        index: u32,
        object_kind: &'static str,
    },
    #[error("module {module:?}, prototype {owner:?}, constant {index}: native method has no captured ABI anchor for {binding_key}")]
    MissingNativeAnchor {
        module: StableModuleRef,
        owner: PrototypeOwnerV1,
        index: u32,
        binding_key: String,
    },
    #[error("module {module:?}, prototype {owner:?}: template closure captures runtime upvalues")]
    CapturedTemplate { module: StableModuleRef, owner: PrototypeOwnerV1 },
    #[error("prototype {owner:?} refers to a non-closure heap object")]
    InvalidClosureHandle { owner: PrototypeOwnerV1 },
    #[error("module {module:?}, prototype {owner:?}: inconsistent emitted layout for {field}: {found} vs {expected}")]
    InconsistentLayout {
        module: StableModuleRef,
        owner: PrototypeOwnerV1,
        field: &'static str,
        found: usize,
        expected: usize,
    },
    #[error("executable image has unresolved or duplicate module {0:?}")]
    ModuleCoverage(StableModuleRef),
    #[error("executable image codec rejected captured products: {0}")]
    Codec(String),
}

/// Builds the actual canonical executable image using the ordinary source
/// compiler and bytecode emitter in a disposable VM.
pub fn capture_canonical_universe_executable(
    canonical: &crate::modules::CanonicalUniverseProgram,
) -> Result<phalcom_compiler_artifact::executable::staging::StagedExecutableSectionV1, ExecutableCaptureError> {
    VM::build_canonical_executable_payload(canonical)
}

/// Program whose emitted bytecode an [`ExecutableImageCapture`] records.
///
/// Universe capture records the canonical program; package capture records
/// exactly one package's owned modules from an ordinary compiled program
/// whose dependencies are earlier artifact generations.
pub(crate) trait CaptureSource {
    /// Compiled program supplying linked reads for captured modules.
    fn compiled(&self) -> &crate::modules::CompiledProgram;
    /// Modules this image must own, in any order.
    fn owned_modules(&self) -> Vec<phalcom_modules::ModuleId>;
    /// Whether the module initializer runs during image installation.
    fn is_eager(&self, id: &phalcom_modules::ModuleId) -> bool;
    /// Source text used for the portable line map.
    fn source_text(&self, id: &phalcom_modules::ModuleId) -> Option<std::sync::Arc<str>>;
    /// Path-free logical URI recorded in the source map.
    fn logical_uri(&self, id: &phalcom_modules::ModuleId) -> String;
    /// Semantic snapshot that owns the captured executable semantics.
    fn snapshot(&self) -> std::sync::Arc<phalcom_semantic::snapshot::SemanticSnapshot>;
}

impl CaptureSource for crate::modules::CanonicalUniverseProgram {
    fn compiled(&self) -> &crate::modules::CompiledProgram {
        self.program()
    }
    fn owned_modules(&self) -> Vec<phalcom_modules::ModuleId> {
        self.program().modules.keys().cloned().collect()
    }
    fn is_eager(&self, id: &phalcom_modules::ModuleId) -> bool {
        self.bootstrap_order().contains(id)
    }
    fn source_text(&self, id: &phalcom_modules::ModuleId) -> Option<std::sync::Arc<str>> {
        self.source_index().unit(id).map(|unit| unit.text.clone())
    }
    fn logical_uri(&self, id: &phalcom_modules::ModuleId) -> String {
        phalcom_modules::identity::universe_module_uri(id).unwrap_or_else(|| id.to_string())
    }
    fn snapshot(&self) -> std::sync::Arc<phalcom_semantic::snapshot::SemanticSnapshot> {
        self.formal_image().snapshot().clone()
    }
}

/// One package's owned modules inside an ordinary compiled program.
pub(crate) struct PackageCaptureSource<'a> {
    /// Program compiled over the package's prefix artifact authority.
    pub(crate) program: &'a crate::modules::CompiledProgram,
    /// Exact package-owned modules.
    pub(crate) scope: BTreeSet<phalcom_modules::ModuleId>,
    /// Package compilation snapshot.
    pub(crate) snapshot: std::sync::Arc<phalcom_semantic::snapshot::SemanticSnapshot>,
}

impl CaptureSource for PackageCaptureSource<'_> {
    fn compiled(&self) -> &crate::modules::CompiledProgram {
        self.program
    }
    fn owned_modules(&self) -> Vec<phalcom_modules::ModuleId> {
        self.scope.iter().cloned().collect()
    }
    fn is_eager(&self, id: &phalcom_modules::ModuleId) -> bool {
        self.scope.contains(id)
    }
    fn source_text(&self, id: &phalcom_modules::ModuleId) -> Option<std::sync::Arc<str>> {
        self.program.modules.get(id).and_then(|module| module.source_text.clone())
    }
    fn logical_uri(&self, id: &phalcom_modules::ModuleId) -> String {
        let stable = to_stable_module(id);
        format!("telos-package:{}", stable.path.join("."))
    }
    fn snapshot(&self) -> std::sync::Arc<phalcom_semantic::snapshot::SemanticSnapshot> {
        self.snapshot.clone()
    }
}

pub(crate) struct ExecutableImageCapture {
    image: ExecutableImageV1,
    semantic_snapshot: std::sync::Arc<phalcom_semantic::snapshot::SemanticSnapshot>,
    source_ids: BTreeMap<phalcom_modules::ModuleId, SourceMapId>,
    prototype_by_closure: HashMap<ObjRef, PrototypeId>,
    constant_by_object: HashMap<ObjRef, ConstantId>,
    visiting_constants: HashSet<ObjRef>,
    nested_ordinals: HashMap<u32, u32>,
    captured_modules: BTreeSet<StableModuleRef>,
    captured_detached_callables: BTreeSet<phalcom_type_meta::identity::StableCallableRef>,
    rooted_objects: HashSet<ObjRef>,
    native_anchor_by_key: HashMap<phalcom_native_meta::PrimitiveKey, NativeAnchorId>,
    root_depth: Option<usize>,
    failure: Option<ExecutableCaptureError>,
}

impl ExecutableImageCapture {
    pub(crate) fn new(canonical: &dyn CaptureSource) -> Self {
        let mut source_ids = BTreeMap::new();
        let mut source_maps = Vec::new();
        let mut source_units = canonical
            .owned_modules()
            .into_iter()
            .filter_map(|id| canonical.source_text(&id).map(|text| (id, text)))
            .collect::<Vec<_>>();
        source_units.sort_by_key(|(id, _)| to_stable_module(id));

        for (id, unit) in source_units {
            let map_id = SourceMapId(u32::try_from(source_maps.len()).expect("canonical module count fits u32"));
            source_ids.insert(id.clone(), map_id);
            let mut line_starts = vec![0];
            line_starts.extend(unit.match_indices('\n').map(|(index, _)| (index + 1) as u64));
            source_maps.push(PortableSourceMapV1 {
                logical_uri: canonical.logical_uri(&id),
                module: to_stable_module(&id),
                line_starts,
                source_span: None,
            });
        }

        let image = ExecutableImageV1 {
            compatibility: ExecutableCompatibilityV1::current(CompilationProfileV1::Debug, &[], &[]).expect("empty native executable ABI has a valid digest"),
            modules: Vec::new(),
            prototypes: Vec::new(),
            constants: Vec::new(),
            semantic_pools: Vec::new(),
            classes: Vec::new(),
            native_anchors: Vec::<PortableNativeAnchorV1>::new(),
            primordial_roles: Vec::<PrimordialRoleV1>::new(),
            source_maps,
            detached_methods: Vec::new(),
            bootstrap: PortableBootstrapPlanV1 {
                native_runtime_roles: Vec::new(),
                eager_modules: Vec::new(),
                deferred_modules: Vec::new(),
                required_declarations: Vec::new(),
            },
        };

        Self {
            image,
            semantic_snapshot: canonical.snapshot(),
            source_ids,
            prototype_by_closure: HashMap::new(),
            constant_by_object: HashMap::new(),
            visiting_constants: HashSet::new(),
            nested_ordinals: HashMap::new(),
            captured_modules: BTreeSet::new(),
            captured_detached_callables: BTreeSet::new(),
            rooted_objects: HashSet::new(),
            native_anchor_by_key: HashMap::new(),
            root_depth: None,
            failure: None,
        }
    }

    /// Captures the VM-free native ABI floor and all required primordial class
    /// roles before any Universe initializer runs. The anchor owner comes from
    /// the native metadata key; runtime class objects are used only to capture
    /// their allocation/layout recipes.
    pub(crate) fn capture_native_anchors(&mut self, vm: &mut VM) {
        if self.failure.is_some() {
            return;
        }

        let mut descriptors = crate::native::PRIMITIVES.iter().collect::<Vec<_>>();
        descriptors.sort_by_key(|descriptor| native_binding_key(descriptor.surface.key));
        let catalog_fingerprint = phalcom_native_surface::catalog_fingerprint();
        for descriptor in descriptors {
            let key = descriptor.surface.key;
            let binding_key = native_binding_key(key);
            let selector = vm.interner.intern(key.selector);
            let signature = descriptor.runtime_signature(vm, selector);
            let signature = crate::artifact::project::project_method_signature(&signature, &vm.interner);
            let owner = phalcom_compiler_artifact::executable::constant::StableClassRefV1 {
                declaration: phalcom_semantic::metadata::stable_identity::to_stable_declaration(&phalcom_semantic::core_surface::universe_declaration(
                    key.owner,
                )),
                side: match key.side {
                    phalcom_native_meta::NativeDispatch::Instance => phalcom_type_meta::identity::StableDispatchSide::Instance,
                    phalcom_native_meta::NativeDispatch::Class => phalcom_type_meta::identity::StableDispatchSide::Class,
                },
            };
            let abi = match descriptor.abi {
                phalcom_native_meta::PrimitiveAbi::Value => "value",
                phalcom_native_meta::PrimitiveAbi::Shape => "shape",
            };
            let contract_identity = format!("native-surface-catalog-v1:{}:primitive-abi:{abi}", hex_digest(&catalog_fingerprint.0),);
            let id = match u32::try_from(self.image.native_anchors.len()) {
                Ok(id) => NativeAnchorId(id),
                Err(_) => {
                    self.failure = Some(ExecutableCaptureError::Codec("native anchor count exceeds u32".into()));
                    return;
                }
            };
            if self.native_anchor_by_key.insert(key, id).is_some() {
                self.failure = Some(ExecutableCaptureError::Codec(format!("duplicate native anchor key {binding_key}")));
                return;
            }
            self.image.native_anchors.push(PortableNativeAnchorV1 {
                binding_key,
                owner,
                signature,
                contract_identity: Some(contract_identity),
            });
        }

        for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
            let class = vm.universe.classes.resolve(binding.key);
            let reference = phalcom_compiler_artifact::executable::constant::StableClassRefV1 {
                declaration: phalcom_semantic::metadata::stable_identity::to_stable_declaration(&phalcom_semantic::core_surface::universe_declaration(
                    binding.key,
                )),
                side: phalcom_type_meta::identity::StableDispatchSide::Instance,
            };
            if let Err(error) = self.capture_runtime_class(vm, class, &reference) {
                self.failure = Some(error);
                return;
            }
        }
    }

    pub(crate) fn capture_module(
        &mut self,
        vm: &mut VM,
        canonical: &dyn CaptureSource,
        id: &phalcom_modules::ModuleId,
        closure: ObjRef,
    ) {
        if self.failure.is_some() {
            return;
        }
        if let Err(error) = self.capture_module_inner(vm, canonical, id, closure) {
            self.failure = Some(error);
        }
    }

    pub(crate) fn capture_detached_methods(&mut self, vm: &mut VM) {
        if self.failure.is_some() {
            return;
        }
        let mut methods = vm
            .detached_method_objects
            .iter()
            .map(|(callable, method)| (callable.clone(), *method))
            .collect::<Vec<_>>();
        methods.sort_by(|(left, _), (right, _)| left.cmp(right));
        for (callable, method) in methods {
            let stable_callable = phalcom_semantic::metadata::stable_identity::to_stable_callable(&callable);
            if !self.captured_detached_callables.insert(stable_callable.clone()) {
                continue;
            }
            let owner = PrototypeOwnerV1::Callable {
                callable: stable_callable.clone(),
            };
            let closure = match vm.heap.get(method) {
                Object::Method(method) => match method.kind {
                    crate::method::MethodKind::Closure(closure) => Some(closure),
                    crate::method::MethodKind::Primitive(_) => None,
                },
                _ => None,
            };
            if let Some(closure) = closure
                && let Err(error) = self.capture_closure(vm, closure, owner.clone())
            {
                self.failure = Some(error);
                return;
            }
            let module = callable.owner.module();
            let index = self.image.constants.len();
            match self.capture_constant(vm, Value::obj(method), module, &owner, index) {
                Ok(method_constant) => self
                    .image
                    .detached_methods
                    .push(phalcom_compiler_artifact::executable::module::PortableDetachedMethodV1 {
                        callable: stable_callable,
                        method_constant,
                    }),
                Err(error) => {
                    self.failure = Some(error);
                    return;
                }
            }
        }
    }

    pub(crate) fn capture_class_layouts(&mut self, vm: &VM, module: &phalcom_modules::ModuleId) {
        if self.failure.is_some() {
            return;
        }
        let mut layouts = vm
            .field_layouts
            .iter()
            .filter(|(key, _)| vm.heap.module(key.module).id == *module)
            .map(|(key, layout)| (*key, layout.clone()))
            .collect::<Vec<_>>();
        layouts.sort_by_key(|(key, _)| vm.interner.lookup(key.name).to_owned());
        for (key, layout) in layouts {
            if let Err(error) = self.capture_class_layout(vm, key, &layout) {
                self.failure = Some(error);
                return;
            }
        }
        if let Err(error) = self.capture_generated_class_layouts(vm, module) {
            self.failure = Some(error);
        }
    }

    fn capture_generated_class_layouts(&mut self, vm: &VM, module: &phalcom_modules::ModuleId) -> Result<(), ExecutableCaptureError> {
        let object_class = self.class_reference(vm, vm.universe.classes.object_class)?;
        let object_class_side = self.class_reference(vm, vm.heap.class(vm.universe.classes.object_class).class)?;
        let metaclass_instance = self.class_reference(vm, vm.universe.classes.metaclass_class)?;

        let enum_owners = self
            .semantic_snapshot
            .enum_semantics
            .iter_module(module)
            .map(|(owner, _)| phalcom_semantic::metadata::stable_identity::to_stable_declaration(owner))
            .collect::<Vec<_>>();
        let data_owners = self
            .semantic_snapshot
            .data_semantics
            .iter_module(module)
            .map(|(owner, _)| phalcom_semantic::metadata::stable_identity::to_stable_declaration(owner))
            .collect::<Vec<_>>();
        for declaration in enum_owners {
            self.capture_generated_nominal_layout(
                declaration,
                false,
                &object_class,
                &object_class_side,
                &metaclass_instance,
            );
        }
        for declaration in data_owners {
            self.capture_generated_nominal_layout(
                declaration,
                true,
                &object_class,
                &object_class_side,
                &metaclass_instance,
            );
        }
        Ok(())
    }

    fn capture_generated_nominal_layout(
        &mut self,
        declaration: phalcom_type_meta::identity::StableDeclarationRef,
        native_repr: bool,
        object_class: &phalcom_compiler_artifact::executable::constant::StableClassRefV1,
        object_class_side: &phalcom_compiler_artifact::executable::constant::StableClassRefV1,
        metaclass_instance: &phalcom_compiler_artifact::executable::constant::StableClassRefV1,
    ) {
        use phalcom_compiler_artifact::executable::constant::StableClassRefV1;
        use phalcom_compiler_artifact::executable::module::{PortableClassAllocationV1, PortableClassRecipeV1};
        use phalcom_type_meta::identity::StableDispatchSide;

        let instance = StableClassRefV1 {
            declaration: declaration.clone(),
            side: StableDispatchSide::Instance,
        };
        let class_side = StableClassRefV1 {
            declaration,
            side: StableDispatchSide::Class,
        };
        if !self.image.classes.iter().any(|row| row.class == instance) {
            self.image.classes.push(PortableClassRecipeV1 {
                class: instance.clone(),
                allocation: PortableClassAllocationV1::SourceDeclared,
                superclass: Some(object_class.clone()),
                metaclass: Some(class_side.clone()),
                fields: vec![],
                field_count: 0,
                immutable_slots: vec![],
                native_repr,
                sealed: false,
                is_abstract: false,
            });
        }
        if !self.image.classes.iter().any(|row| row.class == class_side) {
            self.image.classes.push(PortableClassRecipeV1 {
                class: class_side,
                allocation: PortableClassAllocationV1::SourceDeclared,
                superclass: Some(object_class_side.clone()),
                metaclass: Some(metaclass_instance.clone()),
                fields: vec![],
                field_count: 0,
                immutable_slots: vec![],
                native_repr: false,
                sealed: false,
                is_abstract: false,
            });
        }
    }

    fn capture_class_layout(&mut self, vm: &VM, key: crate::vm::ClassKey, layout: &crate::vm::ClassLayout) -> Result<(), ExecutableCaptureError> {
        use phalcom_compiler_artifact::executable::constant::StableClassRefV1;
        use phalcom_compiler_artifact::executable::module::{PortableClassAllocationV1, PortableClassRecipeV1, PortableFieldLayoutV1};
        use phalcom_type_meta::identity::StableDispatchSide;

        let module = vm.heap.module(key.module).id.clone();
        let name = vm.interner.lookup(key.name);
        let declaration =
            vm.typing_registry
                .declaration_identity(&module, name)
                .cloned()
                .unwrap_or_else(|| phalcom_type_meta::identity::StableDeclarationRef {
                    module: to_stable_module(&module),
                    path: Box::new([name.into()]),
                });
        let instance = StableClassRefV1 {
            declaration: declaration.clone(),
            side: StableDispatchSide::Instance,
        };
        let class_side = StableClassRefV1 {
            declaration: declaration.clone(),
            side: StableDispatchSide::Class,
        };
        let parent = vm.class_parents.get(&key).copied();
        let (parent_instance, parent_class) = if let Some(parent) = parent {
            let parent_module = vm.heap.module(parent.module).id.clone();
            let parent_name = vm.interner.lookup(parent.name);
            let parent_declaration = vm
                .typing_registry
                .declaration_identity(&parent_module, parent_name)
                .cloned()
                .unwrap_or_else(|| phalcom_type_meta::identity::StableDeclarationRef {
                    module: to_stable_module(&parent_module),
                    path: Box::new([parent_name.into()]),
                });
            (
                StableClassRefV1 {
                    declaration: parent_declaration.clone(),
                    side: StableDispatchSide::Instance,
                },
                StableClassRefV1 {
                    declaration: parent_declaration,
                    side: StableDispatchSide::Class,
                },
            )
        } else {
            let object = self.class_reference(vm, vm.universe.classes.object_class)?;
            let object_meta = self.class_reference(vm, vm.heap.class(vm.universe.classes.object_class).class)?;
            (object, object_meta)
        };
        let metaclass = self.class_reference(vm, vm.universe.classes.metaclass_class)?;
        let mut instance_fields = layout
            .field_slots
            .iter()
            .map(|(field, slot)| PortableFieldLayoutV1 {
                name: vm.interner.lookup(*field).to_owned(),
                slot: u32::from(*slot),
            })
            .collect::<Vec<_>>();
        instance_fields.sort_by_key(|field| field.slot);
        let mut class_fields = layout
            .static_field_slots
            .iter()
            .map(|(field, slot)| PortableFieldLayoutV1 {
                name: vm.interner.lookup(*field).to_owned(),
                slot: u32::from(*slot),
            })
            .collect::<Vec<_>>();
        class_fields.sort_by_key(|field| field.slot);
        let mut instance_immutable = layout.immutable_instance_slots.iter().map(|slot| u32::from(*slot)).collect::<Vec<_>>();
        instance_immutable.sort_unstable();
        let mut class_immutable = layout.immutable_static_slots.iter().map(|slot| u32::from(*slot)).collect::<Vec<_>>();
        class_immutable.sort_unstable();

        if !self.image.classes.iter().any(|row| row.class == instance) {
            self.image.classes.push(PortableClassRecipeV1 {
                class: instance,
                allocation: PortableClassAllocationV1::SourceDeclared,
                superclass: Some(parent_instance),
                metaclass: Some(class_side.clone()),
                fields: instance_fields,
                field_count: u32::from(layout.field_count),
                immutable_slots: instance_immutable,
                native_repr: false,
                sealed: vm.sealed_classes.contains_key(&key),
                is_abstract: layout.is_abstract,
            });
        }
        if !self.image.classes.iter().any(|row| row.class == class_side) {
            self.image.classes.push(PortableClassRecipeV1 {
                class: class_side,
                allocation: PortableClassAllocationV1::SourceDeclared,
                superclass: Some(parent_class),
                metaclass: Some(metaclass),
                fields: class_fields,
                field_count: u32::from(layout.static_field_count),
                immutable_slots: class_immutable,
                native_repr: false,
                sealed: false,
                is_abstract: false,
            });
        }
        Ok(())
    }

    fn capture_module_inner(
        &mut self,
        vm: &mut VM,
        canonical: &dyn CaptureSource,
        id: &phalcom_modules::ModuleId,
        closure: ObjRef,
    ) -> Result<(), ExecutableCaptureError> {
        let stable_module = to_stable_module(id);
        if !self.captured_modules.insert(stable_module.clone()) {
            return Err(ExecutableCaptureError::ModuleCoverage(stable_module));
        }
        self.root(vm, closure);
        let source = *self
            .source_ids
            .get(id)
            .ok_or_else(|| ExecutableCaptureError::MissingInitializer { module: stable_module.clone() })?;
        let owner = PrototypeOwnerV1::ModuleInitializer { module: stable_module.clone() };
        let initializer = self.capture_closure(vm, closure, owner)?;
        let compiled = canonical
            .compiled()
            .modules
            .get(id)
            .ok_or_else(|| ExecutableCaptureError::MissingInitializer { module: stable_module.clone() })?;
        let eager = canonical.is_eager(id);
        let linked_reads = compiled
            .linked_reads
            .iter()
            .map(|read| match read {
                phalcom_modules::LinkedReadSpec::Module(module) => PortableLinkedReadV1::Module {
                    module: to_stable_module(module),
                },
                phalcom_modules::LinkedReadSpec::Binding(binding) => PortableLinkedReadV1::Binding {
                    module: to_stable_module(&binding.module),
                    name: binding.name.to_string(),
                },
            })
            .collect();
        self.image.modules.push(PortableModuleV1 {
            identity: stable_module.clone(),
            initializer,
            linked_reads,
            source,
            eager,
        });
        if eager {
            self.image.bootstrap.eager_modules.push(PortableBootstrapModuleV1 {
                module: stable_module,
                initializer,
            });
        } else {
            self.image.bootstrap.deferred_modules.push(stable_module);
        }
        Ok(())
    }

    fn capture_closure(&mut self, vm: &mut VM, closure_ref: ObjRef, owner: PrototypeOwnerV1) -> Result<PrototypeId, ExecutableCaptureError> {
        if let Some(prototype) = self.prototype_by_closure.get(&closure_ref) {
            return Ok(*prototype);
        }
        self.root(vm, closure_ref);
        let closure = match vm.heap.get(closure_ref) {
            Object::Closure(closure) => (**closure).clone(),
            _ => return Err(ExecutableCaptureError::InvalidClosureHandle { owner }),
        };
        let module_id = vm.heap.module(closure.module).id.clone();
        let stable_module = to_stable_module(&module_id);
        if !closure.upvalues.is_empty() {
            return Err(ExecutableCaptureError::CapturedTemplate { module: stable_module, owner });
        }
        let source = *self
            .source_ids
            .get(&module_id)
            .ok_or_else(|| ExecutableCaptureError::MissingInitializer { module: stable_module.clone() })?;
        let callable = closure.callable;
        let prototype_id =
            PrototypeId(u32::try_from(self.image.prototypes.len()).map_err(|_| ExecutableCaptureError::Codec("prototype index exceeds u32".into()))?);
        let semantics = SemanticPoolId(
            u32::try_from(self.image.semantic_pools.len()).map_err(|_| ExecutableCaptureError::Codec("semantic pool index exceeds u32".into()))?,
        );
        self.image.semantic_pools.push(empty_semantic_pool());
        self.image.prototypes.push(PortablePrototypeV1 {
            owner: owner.clone(),
            code: Vec::new(),
            constants: Vec::new(),
            max_slots: 0,
            arity: 0,
            upvalues: Vec::new(),
            parameters: phalcom_compiler_artifact::executable::prototype::ParameterShapeV1 {
                fixed_positionals: 0,
                fixed_labels: Vec::new(),
                rest: None,
            },
            name: String::new(),
            local_names: Vec::new(),
            source,
            semantics,
            spans: Vec::new(),
        });
        self.prototype_by_closure.insert(closure_ref, prototype_id);

        let mut constants = Vec::with_capacity(callable.chunk.constants.len());
        for (index, value) in callable.chunk.constants.iter().copied().enumerate() {
            constants.push(self.capture_constant(vm, value, &module_id, &owner, index)?);
        }
        let semantic_pool = crate::artifact::semantic::project_semantic_pool(vm, &self.semantic_snapshot, &callable.chunk.executable_semantics, &constants)
            .map_err(|error| ExecutableCaptureError::UnsupportedSemanticProduct {
                module: stable_module.clone(),
                owner: owner.clone(),
                family: error.family,
                reason: error.reason,
            })?;
        self.image.semantic_pools[semantics.0 as usize] = semantic_pool;
        let relocations = crate::artifact::project::PrototypeRelocations { constants, source, semantics };
        let projected = crate::artifact::project::project_callable_prototype(&callable, &vm.interner, owner, relocations)
            .map_err(|error| ExecutableCaptureError::Codec(error.to_string()))?;
        self.image.prototypes[prototype_id.0 as usize] = projected;
        Ok(prototype_id)
    }

    fn capture_constant(
        &mut self,
        vm: &mut VM,
        value: Value,
        module: &phalcom_modules::ModuleId,
        owner: &PrototypeOwnerV1,
        index: usize,
    ) -> Result<ConstantId, ExecutableCaptureError> {
        if value.is_obj() {
            let object = value.as_obj().expect("object-tagged Value has ObjRef");
            self.root(vm, object);
            if let Some(id) = self.constant_by_object.get(&object) {
                return Ok(*id);
            }
            if !self.visiting_constants.insert(object) {
                return Err(self.unsupported(module, owner, index, "constant graph cycle"));
            }
            let recipe = self.capture_object(vm, object, module, owner, index)?;
            self.visiting_constants.remove(&object);
            let id = ConstantId(u32::try_from(self.image.constants.len()).map_err(|_| ExecutableCaptureError::Codec("constant index exceeds u32".into()))?);
            self.image.constants.push(recipe);
            self.constant_by_object.insert(object, id);
            return Ok(id);
        }

        let recipe = project_immediate_constant(value, |symbol| vm.interner.lookup(symbol).to_owned())
            .ok_or_else(|| self.unsupported(module, owner, index, "non-portable immediate value"))?;
        let id = ConstantId(u32::try_from(self.image.constants.len()).map_err(|_| ExecutableCaptureError::Codec("constant index exceeds u32".into()))?);
        self.image.constants.push(recipe);
        Ok(id)
    }

    fn capture_object(
        &mut self,
        vm: &mut VM,
        object: ObjRef,
        module: &phalcom_modules::ModuleId,
        owner: &PrototypeOwnerV1,
        index: usize,
    ) -> Result<ConstantRecipeV1, ExecutableCaptureError> {
        let snapshot = match vm.heap.get(object) {
            Object::Str(value) => ConstantObjectSnapshot::String(value.as_str().to_owned()),
            Object::LargeInt(value) => ConstantObjectSnapshot::LargeInt(value.clone()),
            Object::Closure(value) => ConstantObjectSnapshot::Closure((**value).clone()),
            Object::Vector(value) => ConstantObjectSnapshot::Vector(value.elements().to_vec()),
            Object::Array(value) => ConstantObjectSnapshot::Array {
                frozen: value.is_frozen(),
                elements: value.elements().to_vec(),
            },
            Object::Class(_) => ConstantObjectSnapshot::Class,
            Object::Module(value) => ConstantObjectSnapshot::Module(value.id.clone()),
            Object::SelectorPattern(value) => ConstantObjectSnapshot::SelectorPattern(value.pattern.clone()),
            Object::Tuple(_) | Object::Record(_) | Object::Pack(_) => ConstantObjectSnapshot::Product,
            Object::Method(value) => ConstantObjectSnapshot::Method(MethodCaptureSnapshot {
                kind: value.kind,
                native_key: vm
                    .typing_registry
                    .method_implementations
                    .get(object)
                    .and_then(|implementation| implementation.key),
                signature: value.signature.clone(),
                holder: value.holder,
                visibility: value.visibility,
                access_owner: value.access_owner,
                origin: value.origin,
                contracts: value.contracts.clone(),
                attributes: value.attributes.clone(),
                attributes_frozen: value.attributes_frozen,
            }),
            other => ConstantObjectSnapshot::Unsupported(object_kind(other)),
        };

        match snapshot {
            ConstantObjectSnapshot::String(text) => Ok(ConstantRecipeV1::String { text }),
            ConstantObjectSnapshot::LargeInt(value) => {
                let (sign, magnitude) = value.to_bytes_le();
                Ok(ConstantRecipeV1::BigInteger {
                    negative: sign == num_bigint::Sign::Minus,
                    magnitude,
                })
            }
            ConstantObjectSnapshot::Closure(closure) => {
                let parent = self
                    .image
                    .prototypes
                    .iter()
                    .position(|prototype| &prototype.owner == owner)
                    .map(|index| PrototypeId(index as u32))
                    .ok_or_else(|| self.unsupported(module, owner, index, "closure without captured parent"))?;
                let next = self.nested_ordinals.entry(parent.0).or_default();
                let ordinal = *next;
                *next = next
                    .checked_add(1)
                    .ok_or_else(|| ExecutableCaptureError::Codec("nested callable ordinal overflow".into()))?;
                let nested_owner = PrototypeOwnerV1::Nested { parent, ordinal };
                let prototype = self.capture_closure(vm, object, nested_owner)?;
                let closure_module = vm.heap.module(closure.module).id.clone();
                let lexical_class = closure.lexical_class.map(|class| self.class_reference(vm, class)).transpose()?;
                let super_anchor = closure.super_anchor.map(|class| self.class_reference(vm, class)).transpose()?;
                Ok(ConstantRecipeV1::ClosureTemplate {
                    prototype,
                    module: to_stable_module(&closure_module),
                    lexical_class,
                    super_anchor,
                })
            }
            ConstantObjectSnapshot::Vector(values) => {
                let elements = values
                    .into_iter()
                    .enumerate()
                    .map(|(element, value)| self.capture_constant(vm, value, module, owner, element))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ConstantRecipeV1::Vector { elements })
            }
            ConstantObjectSnapshot::Array {
                frozen: true,
                elements: values,
            } => {
                let elements = values
                    .into_iter()
                    .enumerate()
                    .map(|(element, value)| self.capture_constant(vm, value, module, owner, element))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ConstantRecipeV1::FrozenArray { elements })
            }
            ConstantObjectSnapshot::Array { frozen: false, .. } => Err(self.unsupported(module, owner, index, "mutable array")),
            ConstantObjectSnapshot::Class => {
                let reference = self.class_reference(vm, object)?;
                self.capture_runtime_class(vm, object, &reference)?;
                Ok(ConstantRecipeV1::Class { reference })
            }
            ConstantObjectSnapshot::Module(module) => Ok(ConstantRecipeV1::Module {
                module: to_stable_module(&module),
            }),
            ConstantObjectSnapshot::SelectorPattern(pattern) => Ok(ConstantRecipeV1::SelectorPattern {
                pattern: crate::artifact::project::project_selector_pattern(&pattern),
            }),
            ConstantObjectSnapshot::Product => self.capture_product(vm, object, module, owner),
            ConstantObjectSnapshot::Method(method) => self.capture_method(vm, method, module, owner, index),
            ConstantObjectSnapshot::Unsupported(kind) => Err(self.unsupported(module, owner, index, kind)),
        }
    }

    fn capture_product(
        &mut self,
        vm: &mut VM,
        object: ObjRef,
        module: &phalcom_modules::ModuleId,
        owner: &PrototypeOwnerV1,
    ) -> Result<ConstantRecipeV1, ExecutableCaptureError> {
        if let Some(view) = vm.tuple_view(object) {
            let labels = view.labels().iter().map(|label| vm.interner.lookup(*label).to_owned()).collect::<Vec<_>>();
            let positionals = view.positionals();
            let labeled_values = view.labeled_values();
            drop(view);
            let positionals = positionals
                .iter()
                .copied()
                .enumerate()
                .map(|(index, value)| self.capture_constant(vm, value, module, owner, index))
                .collect::<Result<Vec<_>, _>>()?;
            let labeled_values = labeled_values
                .iter()
                .copied()
                .enumerate()
                .map(|(index, value)| self.capture_constant(vm, value, module, owner, index))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(ConstantRecipeV1::Tuple {
                positionals,
                labels,
                labeled_values,
            });
        }
        if let Some(view) = vm.record_view(object) {
            let entries = view.entries();
            drop(view);
            let mut labels = Vec::with_capacity(entries.len());
            let mut values = Vec::with_capacity(entries.len());
            for (index, (key, value)) in entries.into_iter().enumerate() {
                let label = match key {
                    phalcom_semantic::field_key::FieldKey::String(value) | phalcom_semantic::field_key::FieldKey::Symbol(value) => value.to_string(),
                    _ => {
                        return Err(self.unsupported(module, owner, index, "record key outside string/symbol schema"));
                    }
                };
                labels.push(label);
                values.push(self.capture_constant(vm, value, module, owner, index)?);
            }
            return Ok(ConstantRecipeV1::Record { labels, values });
        }
        if let Some(view) = vm.pack_view(object) {
            let labels = (0..view.labeled_len())
                .filter_map(|index| view.get_label(index))
                .map(|label| vm.interner.lookup(label).to_owned())
                .collect::<Vec<_>>();
            let positionals = view.positionals();
            let labeled_values = view.labeled_values();
            drop(view);
            let positionals = positionals
                .into_iter()
                .enumerate()
                .map(|(index, value)| self.capture_constant(vm, value, module, owner, index))
                .collect::<Result<Vec<_>, _>>()?;
            let labeled_values = labeled_values
                .into_iter()
                .enumerate()
                .map(|(index, value)| self.capture_constant(vm, value, module, owner, index))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(ConstantRecipeV1::Pack {
                positionals,
                labels,
                labeled_values,
            });
        }
        Err(self.unsupported(module, owner, 0, "invalid immutable product layout"))
    }

    fn capture_method(
        &mut self,
        vm: &mut VM,
        method: MethodCaptureSnapshot,
        module: &phalcom_modules::ModuleId,
        owner: &PrototypeOwnerV1,
        index: usize,
    ) -> Result<ConstantRecipeV1, ExecutableCaptureError> {
        use crate::method::{MemberVisibility, MethodKind};
        use phalcom_compiler_artifact::executable::constant::{ContractRecipeV1, MethodBodyV1, MethodOriginV1, MethodRecipeV1, VisibilityV1};

        let body = match method.kind {
            MethodKind::Closure(closure) => {
                let template = self.capture_constant(vm, Value::obj(closure), module, owner, index)?;
                MethodBodyV1::Closure { template }
            }
            MethodKind::Primitive(_) => {
                let Some(key) = method.native_key else {
                    return Err(self.unsupported(module, owner, index, "native method without stable primitive key"));
                };
                let Some(anchor) = self.native_anchor_by_key.get(&key).copied() else {
                    return Err(ExecutableCaptureError::MissingNativeAnchor {
                        module: to_stable_module(module),
                        owner: owner.clone(),
                        index: u32::try_from(index).unwrap_or(u32::MAX),
                        binding_key: native_binding_key(key),
                    });
                };
                MethodBodyV1::Native { anchor }
            }
        };
        let holder = method.holder.map(|class| self.class_reference(vm, class)).transpose()?;
        let access_owner = method.access_owner.map(|class| self.class_reference(vm, class)).transpose()?;
        let origin = MethodOriginV1 {
            module: method
                .origin
                .module
                .map(|origin_obj| match vm.heap.get(origin_obj) {
                    Object::Module(module) => Ok(to_stable_module(&module.id)),
                    _ => Err(self.unsupported(module, owner, index, "method origin is not a module")),
                })
                .transpose()?,
            package: method
                .origin
                .package
                .map(|origin_obj| match vm.heap.get(origin_obj) {
                    Object::Module(module) => Ok(to_stable_module(&module.id)),
                    _ => Err(self.unsupported(module, owner, index, "method package origin is not a module")),
                })
                .transpose()?,
            project: method
                .origin
                .project
                .map(|project| phalcom_semantic::metadata::stable_identity::to_stable_project(&project)),
        };
        let visibility = match method.visibility {
            MemberVisibility::Public => VisibilityV1::Public,
            MemberVisibility::Private => VisibilityV1::Private,
            MemberVisibility::Protected => VisibilityV1::Protected,
            MemberVisibility::Module => VisibilityV1::Module,
            MemberVisibility::Package => VisibilityV1::Package,
            MemberVisibility::Internal => VisibilityV1::Internal,
        };
        let contracts = method
            .contracts
            .as_ref()
            .map(|entries| {
                entries
                    .iter()
                    .map(|(predicate, value)| {
                        Ok(ContractRecipeV1 {
                            predicate: vm.interner.lookup(*predicate).to_owned(),
                            value: self.capture_constant(vm, *value, module, owner, index)?,
                        })
                    })
                    .collect::<Result<Vec<_>, ExecutableCaptureError>>()
            })
            .transpose()?;
        let attributes = method
            .attributes
            .iter()
            .copied()
            .enumerate()
            .map(|(attribute, value)| self.capture_constant(vm, value, module, owner, attribute))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ConstantRecipeV1::Method {
            recipe: MethodRecipeV1 {
                body,
                signature: crate::artifact::project::project_method_signature(&method.signature, &vm.interner),
                holder,
                access_owner,
                visibility,
                origin,
                contracts,
                attributes,
                attributes_frozen: method.attributes_frozen,
            },
        })
    }

    fn class_reference(
        &mut self,
        vm: &VM,
        class: crate::heap::ClassId,
    ) -> Result<phalcom_compiler_artifact::executable::constant::StableClassRefV1, ExecutableCaptureError> {
        crate::artifact::project::project_class_reference(vm, class).map_err(|error| ExecutableCaptureError::Codec(error.to_string()))
    }

    fn capture_runtime_class(
        &mut self,
        vm: &VM,
        class: ObjRef,
        reference: &phalcom_compiler_artifact::executable::constant::StableClassRefV1,
    ) -> Result<(), ExecutableCaptureError> {
        if self.image.classes.iter().any(|row| row.class == *reference) {
            return Ok(());
        }
        let (superclass_id, metaclass_id, fields, field_count, mut immutable_slots, native_repr, is_abstract) = {
            let object = vm.heap.class(class);
            let mut fields = object
                .field_slots
                .iter()
                .map(|(name, slot)| phalcom_compiler_artifact::executable::module::PortableFieldLayoutV1 {
                    name: vm.interner.lookup(*name).to_owned(),
                    slot: u32::from(*slot),
                })
                .collect::<Vec<_>>();
            fields.sort_by_key(|field| field.slot);
            let mut immutable_slots = object.immutable_instance_slots.iter().map(|slot| u32::from(*slot)).collect::<Vec<_>>();
            immutable_slots.sort_unstable();
            (
                object.superclass,
                object.class,
                fields,
                u32::from(object.field_count),
                immutable_slots,
                object.native_repr,
                object.is_abstract,
            )
        };
        let superclass = superclass_id.map(|class| self.class_reference(vm, class)).transpose()?;
        let metaclass = self.class_reference(vm, metaclass_id)?;

        let key = phalcom_native_meta::UNIVERSE_BINDINGS
            .iter()
            .find(|binding| vm.universe.classes.resolve(binding.key) == class);
        let allocation = if let Some(binding) = key {
            phalcom_compiler_artifact::executable::module::PortableClassAllocationV1::Primordial {
                role: format!("universe:{}", binding.key.name()),
            }
        } else {
            phalcom_compiler_artifact::executable::module::PortableClassAllocationV1::SourceDeclared
        };
        self.image.classes.push(phalcom_compiler_artifact::executable::module::PortableClassRecipeV1 {
            class: reference.clone(),
            allocation,
            superclass,
            metaclass: Some(metaclass.clone()),
            fields,
            field_count,
            immutable_slots: std::mem::take(&mut immutable_slots),
            native_repr,
            sealed: vm.classes.iter().any(|(key, owner)| *owner == class && vm.sealed_classes.contains_key(key)),
            is_abstract,
        });
        if let Some(binding) = key {
            self.image.primordial_roles.push(PrimordialRoleV1 {
                role: format!("universe:{}", binding.key.name()),
                class: reference.clone(),
            });
        }
        if let Some(superclass_id) = superclass_id {
            let reference = self.class_reference(vm, superclass_id)?;
            self.capture_runtime_class(vm, superclass_id, &reference)?;
        }
        let reference = self.class_reference(vm, metaclass_id)?;
        self.capture_runtime_class(vm, metaclass_id, &reference)?;
        Ok(())
    }

    fn unsupported(&self, module: &phalcom_modules::ModuleId, owner: &PrototypeOwnerV1, index: usize, object_kind: &'static str) -> ExecutableCaptureError {
        ExecutableCaptureError::UnsupportedConstant {
            module: to_stable_module(module),
            owner: owner.clone(),
            index: u32::try_from(index).unwrap_or(u32::MAX),
            object_kind,
        }
    }

    fn root(&mut self, vm: &mut VM, object: ObjRef) {
        if self.root_depth.is_none() {
            self.root_depth = Some(vm.temp_root_depth());
        }
        if self.rooted_objects.insert(object) {
            vm.push_temp_root(Value::obj(object));
        }
    }

    pub(crate) fn finish(&mut self, vm: &mut VM, canonical: &dyn CaptureSource) -> Result<ExecutableImageV1, ExecutableCaptureError> {
        if let Some(error) = self.failure.take() {
            self.release_roots(vm);
            return Err(error);
        }

        if let Err(error) = self.complete_native_owner_class_recipes(vm) {
            self.release_roots(vm);
            return Err(error);
        }

        let mut missing = canonical
            .owned_modules()
            .iter()
            .map(to_stable_module)
            .filter(|module| !self.captured_modules.contains(module))
            .collect::<Vec<_>>();
        missing.sort();
        if let Some(module) = missing.into_iter().next() {
            self.release_roots(vm);
            return Err(ExecutableCaptureError::ModuleCoverage(module));
        }
        if let Err(error) = super::canonicalize::canonicalize_graph_ids(&mut self.image) {
            self.release_roots(vm);
            return Err(ExecutableCaptureError::Codec(error));
        }
        self.image.modules.sort_by(|left, right| left.identity.cmp(&right.identity));
        self.image.bootstrap.deferred_modules.sort();
        self.image.bootstrap.required_declarations.sort();
        self.image
            .classes
            .sort_by(|left, right| (&left.class.declaration, left.class.side).cmp(&(&right.class.declaration, right.class.side)));
        self.image.primordial_roles.sort_by(|left, right| left.role.cmp(&right.role));
        self.image.bootstrap.native_runtime_roles = self.image.primordial_roles.iter().map(|role| role.role.clone()).collect();
        self.image.detached_methods.sort_by(|left, right| left.callable.cmp(&right.callable));
        self.image.compatibility = ExecutableCompatibilityV1::current(CompilationProfileV1::Debug, &self.image.native_anchors, &self.image.primordial_roles)
            .map_err(|error| ExecutableCaptureError::Codec(error.to_string()))?;
        self.release_roots(vm);
        Ok(self.image.clone())
    }

    fn complete_native_owner_class_recipes(&mut self, vm: &VM) -> Result<(), ExecutableCaptureError> {
        let native_owners = self
            .image
            .native_anchors
            .iter()
            .map(|anchor| (anchor.binding_key.clone(), anchor.owner.clone()))
            .collect::<Vec<_>>();
        for (binding_key, owner) in native_owners {
            let instance_owner = phalcom_compiler_artifact::executable::constant::StableClassRefV1 {
                declaration: owner.declaration.clone(),
                side: phalcom_type_meta::identity::StableDispatchSide::Instance,
            };
            if self.image.classes.iter().any(|recipe| recipe.class == owner) && self.image.classes.iter().any(|recipe| recipe.class == instance_owner) {
                continue;
            }
            let class = vm
                .typing_registry
                .resolve_nominal(&owner.declaration)
                .or_else(|| {
                    vm.classes
                        .values()
                        .copied()
                        .find(|class| self.class_reference(vm, *class).is_ok_and(|reference| reference == instance_owner))
                })
                .ok_or_else(|| ExecutableCaptureError::Codec(format!("native anchor {binding_key} owner class was not materialized by canonical modules")))?;
            let resolved = self.class_reference(vm, class)?;
            if resolved != instance_owner {
                return Err(ExecutableCaptureError::Codec(format!(
                    "native anchor {binding_key} owner resolved to a different stable class identity"
                )));
            }
            self.capture_runtime_class(vm, class, &instance_owner)?;
        }
        Ok(())
    }

    pub(crate) fn release_roots(&mut self, vm: &mut VM) {
        if let Some(depth) = self.root_depth.take() {
            vm.truncate_temp_roots(depth);
        }
        self.rooted_objects.clear();
    }
}

fn native_binding_key(key: phalcom_native_meta::PrimitiveKey) -> String {
    let owner = key.owner.name();
    let side = match key.side {
        phalcom_native_meta::NativeDispatch::Instance => "instance",
        phalcom_native_meta::NativeDispatch::Class => "class",
    };
    format!("universe:{}:{}/{}/{}:{}", owner.len(), owner, side, key.selector.len(), key.selector)
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn empty_semantic_pool() -> PortableSemanticPoolV1 {
    PortableSemanticPoolV1 {
        type_recipes: Vec::new(),
        kind_recipes: Vec::new(),
        enum_specs: Vec::new(),
        variant_targets: Vec::new(),
        data_specs: Vec::new(),
        data_targets: Vec::new(),
        data_constructions: Vec::new(),
        anonymous_products: Vec::new(),
        call_environments: Vec::new(),
        associated_targets: Vec::new(),
        family_descriptors: Vec::new(),
        conditional_families: Vec::new(),
        family_operations: Vec::new(),
        family_candidate_sets: Vec::new(),
        trait_invocations: Vec::new(),
        trait_requirement_invocations: Vec::new(),
        type_parameter_class_receivers: Vec::new(),
        provenance: None,
    }
}

fn object_kind(object: &Object) -> &'static str {
    match object {
        Object::Instance(_) => "instance",
        Object::Class(_) => "class",
        Object::Method(_) => "method",
        Object::Module(_) => "module",
        Object::Closure(_) => "closure",
        Object::Str(_) => "string",
        Object::Block(_) => "block",
        Object::BoundMethod(_) => "bound method",
        Object::Upvalue(_) => "upvalue",
        Object::List(_) => "mutable list",
        Object::Array(_) => "array",
        Object::Vector(_) => "vector",
        Object::Buffer(_) => "buffer",
        Object::Slice(_) => "slice",
        Object::ArrayLiteralBuilder(_) => "array literal builder",
        Object::Fiber(_) => "fiber",
        Object::Map(_) => "mutable map",
        Object::Set(_) => "mutable set",
        Object::MutableBag(_) => "mutable bag",
        Object::IndexedDict(_) => "mutable indexed dictionary",
        Object::OrderedSet(_) => "mutable ordered set",
        Object::PersistentHashNode(_) => "persistent hash node",
        Object::Bag(_) => "bag",
        Object::Bijection(_) => "bijection",
        Object::Bytes(_) => "bytes",
        Object::Tuple(_) => "tuple",
        Object::Record(_) => "record",
        Object::Pack(_) => "pack",
        Object::Range(_) => "range",
        Object::Family(_) => "family",
        Object::Selector(_) => "selector",
        Object::SelectorPattern(_) => "selector pattern",
        Object::MethodFamily(_) => "method family",
        Object::BoundMethodFamily(_) => "bound method family",
        Object::LargeInt(_) => "large integer",
        Object::PackBuilder(_) => "pack builder",
        Object::RecordLiteralBuilder(_) => "record literal builder",
        Object::Project(_) => "project",
        Object::ProjectManifest(_) => "project manifest",
        Object::PackageInfo(_) => "package info",
        Object::PackageAuthor(_) => "package author",
        Object::PackageRequirement(_) => "package requirement",
        Object::ResolvedProjectDependency(_) => "resolved project dependency",
        Object::ModuleDependency(_) => "module dependency",
        Object::ExportTable(_) => "export table",
        Object::Export(_) => "export",
        Object::ChildModuleTable(_) => "child module table",
        Object::ModuleIdentity(_) => "module identity",
        Object::PackageIdentity(_) => "package identity",
        Object::ProjectIdentity(_) => "project identity",
        Object::Uri(_) => "URI",
        Object::Typing(_) => "typing object",
        Object::AdtCase(_) => "ADT case",
        Object::Data(_) => "data instance",
        Object::AssociatedFamily(_) => "associated family",
    }
}

fn project_immediate_constant(value: Value, mut symbol_text: impl FnMut(crate::interner::Symbol) -> String) -> Option<ConstantRecipeV1> {
    if value.is_nil() {
        Some(ConstantRecipeV1::Nil)
    } else if value.is_none() {
        Some(ConstantRecipeV1::None)
    } else if value.is_unit() {
        Some(ConstantRecipeV1::Unit)
    } else if value.is_empty_record() {
        Some(ConstantRecipeV1::EmptyRecord)
    } else if value.is_empty_pack() {
        Some(ConstantRecipeV1::EmptyPack)
    } else if let Some(value) = value.as_bool() {
        Some(ConstantRecipeV1::Boolean { value })
    } else if let Some(value) = value.as_int() {
        Some(ConstantRecipeV1::Integer { bits: value as u64 })
    } else if let Some(value) = value.as_float() {
        Some(ConstantRecipeV1::Float { bits: value.to_bits() })
    } else if let Some(value) = value.as_char() {
        Some(ConstantRecipeV1::Character { scalar: value as u32 })
    } else {
        value.symbol_value().map(|symbol| ConstantRecipeV1::Symbol { text: symbol_text(symbol) })
    }
}

#[cfg(test)]
mod tests {
    use super::project_immediate_constant;
    use crate::value::Value;
    use phalcom_compiler_artifact::executable::constant::ConstantRecipeV1;

    #[test]
    fn private_sentinels_and_singletons_are_not_projected_as_surface_constants() {
        assert!(matches!(
            project_immediate_constant(Value::nil(), |_| unreachable!("nil has no symbol")),
            Some(ConstantRecipeV1::Nil)
        ));
        assert!(project_immediate_constant(Value::uninitialized_binding(), |_| unreachable!("sentinel has no symbol")).is_none());
        assert!(
            project_immediate_constant(Value::adt_singleton(crate::adt::RuntimeVariantId(0)), |_| unreachable!(
                "singleton has no symbol"
            ))
            .is_none()
        );
        assert!(
            project_immediate_constant(Value::data_singleton(crate::data::RuntimeDataDescriptorId(0)), |_| unreachable!(
                "singleton has no symbol"
            ))
            .is_none()
        );
    }
}

/// Null output used for the isolated artifact build VM.
pub(crate) struct ArtifactBuildOutput;

impl RuntimeOutput for ArtifactBuildOutput {
    fn write(&mut self, _bytes: &[u8]) -> std::io::Result<()> {
        Ok(())
    }
}
