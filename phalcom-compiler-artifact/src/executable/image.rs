//! Complete structurally decoded executable image. Decoding does not authorize
//! execution; the core validator owns that later boundary.

use super::compatibility::{ExecutableCompatibilityV1, PortableNativeAnchorV1, PrimordialRoleV1, native_executable_abi_digest};
use super::constant::ConstantRecipeV1;
use super::module::{PortableBootstrapPlanV1, PortableClassRecipeV1, PortableDetachedMethodV1, PortableModuleV1, PortableSourceMapV1};
use super::prototype::{ExecutablePrototypeLimitsV1, PortablePrototypeV1, PrototypeOwnerV1};
use super::semantic::*;
use super::{EMITTER_ABI_V1, EXECUTABLE_SCHEMA_V1, NATIVE_RUNTIME_ABI_V1, PORTABLE_BYTECODE_ABI_V1, VM_OPERAND_FRAME_ABI_V1};
use crate::module_world::{Decode, Wire};
use crate::{ArtifactError, BinaryReader, BinaryWriter, CodecError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutableImageLimitsV1 {
    pub max_modules: usize,
    pub max_prototypes: usize,
    pub max_total_instructions: usize,
    pub max_constants: usize,
    pub max_semantic_pools: usize,
    pub max_total_semantic_rows: usize,
    pub max_classes: usize,
    pub max_native_anchors: usize,
    pub max_primordial_roles: usize,
    pub max_source_maps: usize,
    pub max_detached_methods: usize,
    pub max_total_items: usize,
    pub prototypes: ExecutablePrototypeLimitsV1,
}

impl Default for ExecutableImageLimitsV1 {
    fn default() -> Self {
        Self {
            max_modules: 10_000,
            max_prototypes: 1_000_000,
            max_total_instructions: 16_000_000,
            max_constants: 8_000_000,
            max_semantic_pools: 1_000_000,
            max_total_semantic_rows: 16_000_000,
            max_classes: 1_000_000,
            max_native_anchors: 100_000,
            max_primordial_roles: 10_000,
            max_source_maps: 1_000_000,
            max_detached_methods: 1_000_000,
            max_total_items: 32_000_000,
            prototypes: ExecutablePrototypeLimitsV1::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableImageV1 {
    pub compatibility: ExecutableCompatibilityV1,
    pub modules: Vec<PortableModuleV1>,
    pub prototypes: Vec<PortablePrototypeV1>,
    pub constants: Vec<ConstantRecipeV1>,
    pub semantic_pools: Vec<PortableSemanticPoolV1>,
    pub classes: Vec<PortableClassRecipeV1>,
    pub native_anchors: Vec<PortableNativeAnchorV1>,
    pub primordial_roles: Vec<PrimordialRoleV1>,
    pub source_maps: Vec<PortableSourceMapV1>,
    pub detached_methods: Vec<PortableDetachedMethodV1>,
    pub bootstrap: PortableBootstrapPlanV1,
}

impl ExecutableImageV1 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let Self {
            compatibility,
            modules,
            prototypes,
            constants,
            semantic_pools,
            classes,
            native_anchors,
            primordial_roles,
            source_maps,
            detached_methods,
            bootstrap,
        } = self;
        self.validate_structure(&ExecutableImageLimitsV1::default())?;
        self.validate_native_abi_digest()?;
        let mut out = BinaryWriter::new();
        out.write_u32(EXECUTABLE_SCHEMA_V1);
        compatibility.write(&mut out)?;
        modules.write(&mut out)?;
        put_count(prototypes.len(), &mut out, "prototype count")?;
        for prototype in prototypes {
            let row = prototype.encode()?;
            put_count(row.len(), &mut out, "prototype byte length")?;
            out.write_bytes(&row);
        }
        constants.write(&mut out)?;
        semantic_pools.write(&mut out)?;
        classes.write(&mut out)?;
        native_anchors.write(&mut out)?;
        primordial_roles.write(&mut out)?;
        source_maps.write(&mut out)?;
        detached_methods.write(&mut out)?;
        bootstrap.write(&mut out)?;
        Ok(out.into_vec())
    }

    /// Decodes against a private allocation ledger and publishes the charge
    /// only after all rows and references are structurally valid.
    pub fn decode(bytes: &[u8], common: &ValidationLimits, caps: &ExecutableImageLimitsV1, budget: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let byte_limit = common.max_artifact_bytes.min(common.max_decoded_section_bytes);
        if bytes.len() as u64 > byte_limit {
            return Err(CodecError::LimitExceeded {
                item: "executable image bytes",
                count: bytes.len(),
                limit: usize::try_from(byte_limit).unwrap_or(usize::MAX),
            }
            .into());
        }
        let mut transaction = budget.clone();
        let result = decode_inner(bytes, common, caps, &mut transaction)?;
        *budget = transaction;
        Ok(result)
    }

    pub fn validate_structure(&self, caps: &ExecutableImageLimitsV1) -> Result<(), ArtifactError> {
        self.validate_structure_inner(caps, None)
    }

    /// Checks the compatibility digest separately from structural decoding.
    /// Later executable validation owns the decision to authorize this ABI.
    pub fn validate_native_abi_digest(&self) -> Result<(), ArtifactError> {
        let expected = native_executable_abi_digest(self.compatibility.native_runtime_abi, &self.native_anchors, &self.primordial_roles)?;
        if expected != self.compatibility.native_executable_abi_digest {
            return Err(invalid("native executable ABI digest mismatch"));
        }
        Ok(())
    }

    fn validate_structure_inner(&self, caps: &ExecutableImageLimitsV1, mut allocations: Option<&mut AllocationBudget>) -> Result<(), ArtifactError> {
        if self.compatibility.executable_schema != EXECUTABLE_SCHEMA_V1
            || self.compatibility.portable_bytecode_abi != PORTABLE_BYTECODE_ABI_V1
            || self.compatibility.emitter_abi != EMITTER_ABI_V1
            || self.compatibility.native_runtime_abi != NATIVE_RUNTIME_ABI_V1
            || self.compatibility.vm_operand_frame_abi != VM_OPERAND_FRAME_ABI_V1
        {
            return Err(invalid("unsupported executable compatibility"));
        }
        check_count(self.modules.len(), caps.max_modules, "executable modules")?;
        check_count(self.prototypes.len(), caps.max_prototypes, "executable prototypes")?;
        check_count(self.constants.len(), caps.max_constants, "executable constants")?;
        check_count(self.semantic_pools.len(), caps.max_semantic_pools, "executable semantic pools")?;
        check_count(self.classes.len(), caps.max_classes, "executable classes")?;
        check_count(self.native_anchors.len(), caps.max_native_anchors, "native anchors")?;
        check_count(self.primordial_roles.len(), caps.max_primordial_roles, "primordial roles")?;
        check_count(self.source_maps.len(), caps.max_source_maps, "source maps")?;
        check_count(self.detached_methods.len(), caps.max_detached_methods, "detached methods")?;

        let instructions = self
            .prototypes
            .iter()
            .try_fold(0usize, |total, row| total.checked_add(row.code.len()))
            .ok_or_else(|| invalid("total executable instruction count overflow"))?;
        check_count(instructions, caps.max_total_instructions, "total executable instructions")?;
        let semantic_rows = self
            .semantic_pools
            .iter()
            .try_fold(0usize, |total, pool| total.checked_add(pool.row_count()?))
            .ok_or_else(|| invalid("total executable semantic row count overflow"))?;
        check_count(semantic_rows, caps.max_total_semantic_rows, "total executable semantic rows")?;

        require_strict_order(self.modules.iter().map(|row| &row.identity), "module rows")?;
        require_strict_order(self.source_maps.iter().map(|row| (&row.module, &row.logical_uri)), "source-map rows")?;
        require_strict_order(self.classes.iter().map(|row| (&row.class.declaration, row.class.side)), "class rows")?;
        require_strict_order(self.native_anchors.iter().map(|row| &row.binding_key), "native anchor rows")?;
        require_strict_order(self.primordial_roles.iter().map(|row| &row.role), "primordial role rows")?;
        require_strict_order(self.bootstrap.deferred_modules.iter(), "deferred bootstrap modules")?;
        require_strict_order(self.bootstrap.required_declarations.iter(), "bootstrap declaration requirements")?;

        for (index, module) in self.modules.iter().enumerate() {
            let prototype = self.prototype(module.initializer.0, "module initializer")?;
            if !matches!(&prototype.owner, PrototypeOwnerV1::ModuleInitializer { module: owner } if owner == &module.identity) {
                return Err(invalid(format!("module row {index} initializer owner mismatch")));
            }
            let source = self.source_map(module.source.0, "module source map")?;
            if source.module != module.identity {
                return Err(invalid(format!("module row {index} source map owner mismatch")));
            }
            for linked in &module.linked_reads {
                let target = match linked {
                    super::module::PortableLinkedReadV1::Module { module }
                    | super::module::PortableLinkedReadV1::Binding { module, .. }
                    | super::module::PortableLinkedReadV1::StaticConstant { module, .. } => module,
                };
                if !self.has_module_or_upstream(target) {
                    return Err(invalid(format!("module row {index} has a linked read to an absent module")));
                }
            }
        }
        for (index, source) in self.source_maps.iter().enumerate() {
            if self.module(&source.module).is_none() {
                return Err(invalid(format!("source map row {index} references an absent module")));
            }
            if let Some(span) = &source.source_span
                && span.start > span.end
            {
                return Err(invalid(format!("source map row {index} has a reversed source span")));
            }
            if source.line_starts.first().is_some_and(|first| *first != 0) || source.line_starts.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(invalid(format!("source map row {index} has noncanonical line starts")));
            }
        }
        for (index, class) in self.classes.iter().enumerate() {
            if let super::module::PortableClassAllocationV1::Primordial { role } = &class.allocation {
                let matches_role = self
                    .primordial_roles
                    .binary_search_by(|candidate| candidate.role.cmp(role))
                    .ok()
                    .is_some_and(|role_index| self.primordial_roles[role_index].class == class.class);
                if !matches_role {
                    return Err(invalid(format!("class row {index} has an unresolved primordial role")));
                }
            }
            for target in class.superclass.iter().chain(class.metaclass.iter()) {
                if !self.has_class(target) {
                    return Err(invalid(format!("class row {index} references an absent class")));
                }
            }
        }
        for (index, role) in self.primordial_roles.iter().enumerate() {
            if !self.has_class(&role.class) {
                return Err(invalid(format!("primordial role row {index} references an absent class")));
            }
        }
        for (index, anchor) in self.native_anchors.iter().enumerate() {
            if !self.has_class(&anchor.owner) {
                return Err(invalid(format!("native anchor row {index} references an absent owner class")));
            }
        }
        for (index, prototype) in self.prototypes.iter().enumerate() {
            prototype.validate_structure(&caps.prototypes)?;
            self.source_map(prototype.source.0, "prototype source map")?;
            self.semantic_pool(prototype.semantics.0, "prototype semantic pool")?;
            for constant in &prototype.constants {
                self.constant(constant.0, "prototype constant")?;
            }
            if let PrototypeOwnerV1::Nested { parent, .. } = prototype.owner {
                if parent.0 as usize >= index {
                    return Err(invalid("nested prototype parent must precede its child"));
                }
                self.prototype(parent.0, "nested prototype parent")?;
            }
        }
        for (index, constant) in self.constants.iter().enumerate() {
            self.validate_constant(index as u32, constant)?;
        }
        reject_constant_cycles(&self.constants, allocations.as_deref_mut())?;
        for pool in &self.semantic_pools {
            pool.validate_references(self.constants.len())?;
        }
        for method in &self.detached_methods {
            let recipe = self.constant(method.method_constant.0, "detached method constant")?;
            if !matches!(recipe, ConstantRecipeV1::Method { .. }) {
                return Err(invalid("detached method does not reference a method recipe"));
            }
        }
        for entry in &self.bootstrap.eager_modules {
            let module = self
                .module(&entry.module)
                .ok_or_else(|| invalid("bootstrap references an absent eager module"))?;
            if module.initializer != entry.initializer || !module.eager {
                return Err(invalid("bootstrap eager module disagrees with module row"));
            }
        }
        for module in &self.bootstrap.deferred_modules {
            let row = self.module(module).ok_or_else(|| invalid("bootstrap references an absent deferred module"))?;
            if row.eager {
                return Err(invalid("deferred bootstrap module is marked eager"));
            }
        }

        for (index, eager) in self.bootstrap.eager_modules.iter().enumerate() {
            if self.bootstrap.eager_modules[..index].iter().any(|prior| prior.module == eager.module) {
                return Err(invalid("duplicate eager bootstrap module"));
            }
        }
        for module in &self.modules {
            let in_eager = self.bootstrap.eager_modules.iter().any(|entry| entry.module == module.identity);
            let in_deferred = self.bootstrap.deferred_modules.binary_search(&module.identity).is_ok();
            if in_eager == in_deferred {
                return Err(invalid("each executable module must appear exactly once in the bootstrap plan"));
            }
        }
        for (index, declaration) in self.bootstrap.required_declarations.iter().enumerate() {
            if self.module(&declaration.module).is_none() {
                return Err(invalid(format!("bootstrap declaration requirement {index} references an absent module")));
            }
        }
        for role in &self.bootstrap.native_runtime_roles {
            if self.primordial_roles.binary_search_by(|candidate| candidate.role.cmp(role)).is_err() {
                return Err(invalid("bootstrap references an absent primordial role"));
            }
        }
        Ok(())
    }

    fn validate_constant(&self, index: u32, constant: &ConstantRecipeV1) -> Result<(), ArtifactError> {
        let check = |id: &super::prototype::ConstantId| self.constant(id.0, "constant dependency").map(|_| ());
        match constant {
            ConstantRecipeV1::Character { scalar } if char::from_u32(*scalar).is_none() => {
                return Err(invalid(format!("constant {index} has an invalid Unicode scalar")));
            }
            ConstantRecipeV1::ClosureTemplate {
                prototype,
                module,
                lexical_class,
                super_anchor,
            } => {
                self.prototype(prototype.0, "closure template prototype")?;
                if !self.has_module_or_upstream(module) {
                    return Err(invalid(format!("constant {index} closure template references an absent module")));
                }
                if lexical_class.as_ref().is_some_and(|class| !self.has_class(class)) || super_anchor.as_ref().is_some_and(|class| !self.has_class(class)) {
                    return Err(invalid(format!("constant {index} closure template references an absent class")));
                }
            }
            ConstantRecipeV1::Class { reference } if !self.has_class(reference) => {
                return Err(invalid(format!("constant {index} references an absent class")));
            }
            ConstantRecipeV1::Module { module } if !self.has_module_or_upstream(module) => {
                return Err(invalid(format!("constant {index} references an absent module")));
            }
            ConstantRecipeV1::Method { recipe } => {
                if recipe.holder.as_ref().is_some_and(|class| !self.has_class(class))
                    || recipe.access_owner.as_ref().is_some_and(|class| !self.has_class(class))
                {
                    return Err(invalid(format!("constant {index} method recipe references an absent class")));
                }
                if let super::constant::MethodBodyV1::Closure { template } = &recipe.body {
                    check(template)?;
                }
                for contract in recipe.contracts.iter().flatten() {
                    check(&contract.value)?;
                }
                for attribute in &recipe.attributes {
                    check(attribute)?;
                }
                if let super::constant::MethodBodyV1::Native { anchor } = &recipe.body {
                    self.native_anchor(anchor.0, "method native anchor")?;
                }
            }
            ConstantRecipeV1::Vector { elements } | ConstantRecipeV1::FrozenArray { elements } => {
                for element in elements {
                    check(element)?;
                }
            }
            ConstantRecipeV1::Tuple {
                positionals,
                labels,
                labeled_values,
            }
            | ConstantRecipeV1::Pack {
                positionals,
                labels,
                labeled_values,
            } => {
                if labels.len() != labeled_values.len() {
                    return Err(invalid("labeled constant shape mismatch"));
                }
                for element in positionals.iter().chain(labeled_values) {
                    check(element)?;
                }
            }
            ConstantRecipeV1::Record { labels, values } => {
                if labels.len() != values.len() {
                    return Err(invalid("record constant shape mismatch"));
                }
                for value in values {
                    check(value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn prototype(&self, index: u32, kind: &str) -> Result<&PortablePrototypeV1, ArtifactError> {
        self.prototypes
            .get(index as usize)
            .ok_or_else(|| invalid(format!("{kind} reference {index} is out of bounds")))
    }
    fn constant(&self, index: u32, kind: &str) -> Result<&ConstantRecipeV1, ArtifactError> {
        self.constants
            .get(index as usize)
            .ok_or_else(|| invalid(format!("{kind} reference {index} is out of bounds")))
    }
    fn semantic_pool(&self, index: u32, kind: &str) -> Result<&PortableSemanticPoolV1, ArtifactError> {
        self.semantic_pools
            .get(index as usize)
            .ok_or_else(|| invalid(format!("{kind} reference {index} is out of bounds")))
    }
    fn source_map(&self, index: u32, kind: &str) -> Result<&PortableSourceMapV1, ArtifactError> {
        self.source_maps
            .get(index as usize)
            .ok_or_else(|| invalid(format!("{kind} reference {index} is out of bounds")))
    }
    fn native_anchor(&self, index: u32, kind: &str) -> Result<&PortableNativeAnchorV1, ArtifactError> {
        self.native_anchors
            .get(index as usize)
            .ok_or_else(|| invalid(format!("{kind} reference {index} is out of bounds")))
    }

    fn module(&self, identity: &phalcom_type_meta::identity::StableModuleRef) -> Option<&PortableModuleV1> {
        self.modules
            .binary_search_by(|candidate| candidate.identity.cmp(identity))
            .ok()
            .map(|index| &self.modules[index])
    }

    fn has_class(&self, identity: &super::constant::StableClassRefV1) -> bool {
        self.classes
            .binary_search_by(|candidate| (&candidate.class.declaration, candidate.class.side).cmp(&(&identity.declaration, identity.side)))
            .is_ok()
            || self.primordial_roles.iter().any(|role| &role.class == identity)
            || self.is_upstream(&identity.declaration.module)
    }

    /// Reports whether `module` belongs to a project with no module row in
    /// this image: a package image references its prefix (Universe and earlier
    /// packages) without carrying it, and the installer resolves those
    /// references against the already-installed VM.
    fn is_upstream(&self, module: &phalcom_type_meta::identity::StableModuleRef) -> bool {
        !self.modules.iter().any(|row| row.identity.project == module.project)
    }

    fn has_module_or_upstream(&self, identity: &phalcom_type_meta::identity::StableModuleRef) -> bool {
        self.module(identity).is_some() || self.is_upstream(identity)
    }

    fn has_local_class(&self, identity: &super::constant::StableClassRefV1) -> bool {
        self.classes
            .binary_search_by(|candidate| (&candidate.class.declaration, candidate.class.side).cmp(&(&identity.declaration, identity.side)))
            .is_ok()
            || self.primordial_roles.iter().any(|role| &role.class == identity)
    }

    /// Resolves every upstream reference of this package image against the
    /// `prefix` image it is composed over.
    ///
    /// Structural validation admits module and class references owned by
    /// projects this image does not carry. Composition closes that gap: each
    /// such module must be a module row of `prefix`, and each such class must
    /// be a class row or primordial role of `prefix`.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactError`] naming the first upstream reference that
    /// `prefix` does not provide.
    pub fn validate_upstream_over(&self, prefix: &Self) -> Result<(), ArtifactError> {
        use super::constant::StableClassRefV1;
        use phalcom_type_meta::identity::StableModuleRef;
        let module = |identity: &StableModuleRef| -> Result<(), ArtifactError> {
            if self.is_upstream(identity) && prefix.module(identity).is_none() {
                return Err(invalid(format!("upstream module {identity:?} is absent from the prefix image")));
            }
            Ok(())
        };
        let class = |identity: &StableClassRefV1| -> Result<(), ArtifactError> {
            if self.is_upstream(&identity.declaration.module) && !prefix.has_local_class(identity) {
                return Err(invalid(format!("upstream class {identity:?} is absent from the prefix image")));
            }
            Ok(())
        };
        for row in &self.modules {
            for linked in &row.linked_reads {
                match linked {
                    super::module::PortableLinkedReadV1::Module { module: target }
                    | super::module::PortableLinkedReadV1::Binding { module: target, .. }
                    | super::module::PortableLinkedReadV1::StaticConstant { module: target, .. } => module(target)?,
                }
            }
        }
        for row in &self.classes {
            for target in row.superclass.iter().chain(row.metaclass.iter()) {
                class(target)?;
            }
        }
        for anchor in &self.native_anchors {
            class(&anchor.owner)?;
        }
        for constant in &self.constants {
            match constant {
                ConstantRecipeV1::ClosureTemplate {
                    module: owner,
                    lexical_class,
                    super_anchor,
                    ..
                } => {
                    module(owner)?;
                    for target in lexical_class.iter().chain(super_anchor.iter()) {
                        class(target)?;
                    }
                }
                ConstantRecipeV1::Class { reference } => class(reference)?,
                ConstantRecipeV1::Module { module: owner } => module(owner)?,
                ConstantRecipeV1::Method { recipe } => {
                    for target in recipe.holder.iter().chain(recipe.access_owner.iter()) {
                        class(target)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn decode_inner(
    bytes: &[u8],
    common: &ValidationLimits,
    caps: &ExecutableImageLimitsV1,
    budget: &mut AllocationBudget,
) -> Result<ExecutableImageV1, ArtifactError> {
    let mut limits = *common;
    limits.max_modules = limits.max_modules.min(caps.max_modules);
    limits.max_semantic_entries_per_family = limits.max_semantic_entries_per_family.min(caps.max_total_semantic_rows);
    let mut input = Decode {
        reader: BinaryReader::new(bytes),
        limits,
        allocations: budget,
        items_left: caps.max_total_items,
        depth: 0,
    };
    let schema = input.reader.read_u32()?;
    if schema != EXECUTABLE_SCHEMA_V1 {
        return Err(CodecError::CorruptedData(format!("unsupported executable image schema {schema}")).into());
    }
    let compatibility = ExecutableCompatibilityV1::read(&mut input)?;
    let modules = read_vec_with_cap::<PortableModuleV1>(&mut input, caps.max_modules, "executable modules")?;

    let prototype_count = read_count(&mut input.reader, caps.max_prototypes, "executable prototypes")?;
    debit_items(&mut input, prototype_count, "executable prototypes")?;
    let mut prototypes = input.allocations.vector(prototype_count)?;
    let mut instruction_count = 0usize;
    for _ in 0..prototype_count {
        let row_len = read_count(
            &mut input.reader,
            common.max_decoded_section_bytes.min(usize::MAX as u64) as usize,
            "prototype bytes",
        )?;
        let row_bytes = input.reader.read_slice(row_len)?;
        let mut row_caps = caps.prototypes;
        row_caps.max_code = row_caps.max_code.min(caps.max_total_instructions.saturating_sub(instruction_count));
        let row = PortablePrototypeV1::decode(row_bytes, common, &row_caps, input.allocations)?;
        instruction_count = instruction_count
            .checked_add(row.code.len())
            .ok_or_else(|| invalid("total executable instruction count overflow"))?;
        check_count(instruction_count, caps.max_total_instructions, "total executable instructions")?;
        prototypes.push(row);
    }

    let constants = read_vec_with_cap::<ConstantRecipeV1>(&mut input, caps.max_constants, "executable constants")?;

    // Semantic pools have both a per-family limit and a cumulative row limit.
    // Temporarily debit their complete nested graph from that tighter counter,
    // then return the consumed items to the image-wide ledger.
    let image_items_left = input.items_left;
    let semantic_items_left = image_items_left.min(caps.max_total_semantic_rows);
    input.items_left = semantic_items_left;
    let semantic_pools = read_vec_with_cap::<PortableSemanticPoolV1>(&mut input, caps.max_semantic_pools, "executable semantic pools")?;
    let semantic_items_used = semantic_items_left - input.items_left;
    input.items_left = image_items_left
        .checked_sub(semantic_items_used)
        .ok_or_else(|| invalid("executable semantic item budget underflow"))?;

    let classes = read_vec_with_cap::<PortableClassRecipeV1>(&mut input, caps.max_classes, "executable classes")?;
    let native_anchors = read_vec_with_cap::<PortableNativeAnchorV1>(&mut input, caps.max_native_anchors, "native anchors")?;
    let primordial_roles = read_vec_with_cap::<PrimordialRoleV1>(&mut input, caps.max_primordial_roles, "primordial roles")?;
    let source_maps = read_vec_with_cap::<PortableSourceMapV1>(&mut input, caps.max_source_maps, "source maps")?;
    let detached_methods = read_vec_with_cap::<PortableDetachedMethodV1>(&mut input, caps.max_detached_methods, "detached methods")?;
    let bootstrap = read_bootstrap(&mut input, caps)?;
    if input.reader.has_remaining() {
        return Err(CodecError::CorruptedData("trailing executable image bytes".into()).into());
    }
    let image = ExecutableImageV1 {
        compatibility,
        modules,
        prototypes,
        constants,
        semantic_pools,
        classes,
        native_anchors,
        primordial_roles,
        source_maps,
        detached_methods,
        bootstrap,
    };
    image.validate_structure_inner(caps, Some(input.allocations))?;
    Ok(image)
}

fn read_vec_with_cap<T: Wire>(input: &mut Decode<'_>, limit: usize, item: &'static str) -> Result<Vec<T>, ArtifactError> {
    input.nested(|input| {
        let count = read_count(&mut input.reader, limit, item)?;
        debit_items(input, count, item)?;
        let mut rows = input.allocations.vector::<T>(count)?;
        for _ in 0..count {
            rows.push(T::read(input)?);
        }
        Ok(rows)
    })
}

fn read_bootstrap(input: &mut Decode<'_>, caps: &ExecutableImageLimitsV1) -> Result<PortableBootstrapPlanV1, ArtifactError> {
    input.nested(|input| {
        Ok(PortableBootstrapPlanV1 {
            native_runtime_roles: read_vec_with_cap(input, caps.max_primordial_roles, "bootstrap native roles")?,
            eager_modules: read_vec_with_cap(input, caps.max_modules, "eager bootstrap modules")?,
            deferred_modules: read_vec_with_cap(input, caps.max_modules, "deferred bootstrap modules")?,
            required_declarations: read_vec_with_cap(input, caps.max_total_semantic_rows, "bootstrap declarations")?,
        })
    })
}

fn debit_items(input: &mut Decode<'_>, count: usize, item: &'static str) -> Result<(), ArtifactError> {
    if count > input.items_left {
        return Err(CodecError::LimitExceeded {
            item,
            count,
            limit: input.items_left,
        }
        .into());
    }
    input.items_left -= count;
    Ok(())
}

fn put_count(count: usize, out: &mut BinaryWriter, item: &'static str) -> Result<(), ArtifactError> {
    out.write_u32(u32::try_from(count).map_err(|_| invalid(format!("{item} exceeds wire u32")))?);
    Ok(())
}

fn read_count(reader: &mut BinaryReader<'_>, limit: usize, item: &'static str) -> Result<usize, ArtifactError> {
    let count = reader.read_u32()? as usize;
    check_count(count, limit, item)?;
    Ok(count)
}

fn check_count(count: usize, limit: usize, item: &'static str) -> Result<(), ArtifactError> {
    if count > limit {
        return Err(CodecError::LimitExceeded { item, count, limit }.into());
    }
    Ok(())
}

fn require_strict_order<T: Ord>(values: impl Iterator<Item = T>, item: &'static str) -> Result<(), ArtifactError> {
    let mut previous: Option<T> = None;
    for value in values {
        if previous.as_ref().is_some_and(|candidate| candidate >= &value) {
            return Err(invalid(format!("{item} must be strictly ordered")));
        }
        previous = Some(value);
    }
    Ok(())
}

fn reject_constant_cycles(constants: &[ConstantRecipeV1], mut allocations: Option<&mut AllocationBudget>) -> Result<(), ArtifactError> {
    #[derive(Clone, Copy)]
    struct Frame {
        constant: usize,
        next_edge: usize,
    }

    let mut state = match allocations.as_deref_mut() {
        Some(budget) => budget.vector::<u8>(constants.len())?,
        None => try_scratch_vec::<u8>(constants.len())?,
    };
    state.resize(constants.len(), 0);
    let mut stack = match allocations.as_deref_mut() {
        Some(budget) => budget.vector::<Frame>(constants.len())?,
        None => try_scratch_vec::<Frame>(constants.len())?,
    };

    for root in 0..constants.len() {
        if state[root] != 0 {
            continue;
        }
        state[root] = 1;
        stack.push(Frame { constant: root, next_edge: 0 });
        while let Some(frame) = stack.last_mut() {
            let edge_count = constant_dependency_count(&constants[frame.constant])?;
            if frame.next_edge == edge_count {
                state[frame.constant] = 2;
                stack.pop();
                continue;
            }
            let edge = constant_dependency_at(&constants[frame.constant], frame.next_edge)
                .ok_or_else(|| invalid("executable constant dependency census is inconsistent"))?;
            frame.next_edge += 1;
            let child = edge.0 as usize;
            match state.get(child).copied() {
                Some(1) => return Err(invalid("executable constant graph contains a cycle")),
                Some(2) => {}
                Some(0) => {
                    state[child] = 1;
                    stack.push(Frame { constant: child, next_edge: 0 });
                }
                Some(_) => return Err(invalid("executable constant graph traversal state is invalid")),
                None => return Err(invalid("executable constant reference is out of bounds")),
            }
        }
    }
    Ok(())
}

fn try_scratch_vec<T>(count: usize) -> Result<Vec<T>, ArtifactError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| phalcom_type_meta::allocation::AllocationError::Failed)?;
    Ok(values)
}

fn constant_dependency_count(constant: &ConstantRecipeV1) -> Result<usize, ArtifactError> {
    let count = match constant {
        ConstantRecipeV1::Method { recipe } => {
            let closure = usize::from(matches!(&recipe.body, super::constant::MethodBodyV1::Closure { .. }));
            let contracts = recipe.contracts.as_ref().map_or(0, Vec::len);
            closure
                .checked_add(contracts)
                .and_then(|count| count.checked_add(recipe.attributes.len()))
                .ok_or_else(|| invalid("constant dependency count overflow"))?
        }
        ConstantRecipeV1::Vector { elements } | ConstantRecipeV1::FrozenArray { elements } => elements.len(),
        ConstantRecipeV1::Tuple {
            positionals, labeled_values, ..
        }
        | ConstantRecipeV1::Pack {
            positionals, labeled_values, ..
        } => positionals
            .len()
            .checked_add(labeled_values.len())
            .ok_or_else(|| invalid("constant dependency count overflow"))?,
        ConstantRecipeV1::Record { values, .. } => values.len(),
        _ => 0,
    };
    Ok(count)
}

fn constant_dependency_at(constant: &ConstantRecipeV1, index: usize) -> Option<&super::prototype::ConstantId> {
    match constant {
        ConstantRecipeV1::Method { recipe } => {
            if let super::constant::MethodBodyV1::Closure { template } = &recipe.body {
                if index == 0 {
                    return Some(template);
                }
                let index = index - 1;
                if let Some(contracts) = &recipe.contracts {
                    if let Some(contract) = contracts.get(index) {
                        return Some(&contract.value);
                    }
                    let index = index - contracts.len();
                    return recipe.attributes.get(index);
                }
                return recipe.attributes.get(index);
            }
            if let Some(contracts) = &recipe.contracts {
                if let Some(contract) = contracts.get(index) {
                    return Some(&contract.value);
                }
                return recipe.attributes.get(index - contracts.len());
            }
            recipe.attributes.get(index)
        }
        ConstantRecipeV1::Vector { elements } | ConstantRecipeV1::FrozenArray { elements } => elements.get(index),
        ConstantRecipeV1::Tuple {
            positionals, labeled_values, ..
        }
        | ConstantRecipeV1::Pack {
            positionals, labeled_values, ..
        } => {
            if index < positionals.len() {
                positionals.get(index)
            } else {
                labeled_values.get(index - positionals.len())
            }
        }
        ConstantRecipeV1::Record { values, .. } => values.get(index),
        _ => None,
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ArtifactError::LimitExceeded(message.into())
}

impl PortableSemanticPoolV1 {
    fn row_count(&self) -> Option<usize> {
        let mut rows = [
            self.type_recipes.len(),
            self.kind_recipes.len(),
            self.enum_specs.len(),
            self.variant_targets.len(),
            self.data_specs.len(),
            self.data_targets.len(),
            self.data_constructions.len(),
            self.anonymous_products.len(),
            self.call_environments.len(),
            self.associated_targets.len(),
            self.family_descriptors.len(),
            self.conditional_families.len(),
            self.family_operations.len(),
            self.family_candidate_sets.len(),
            self.trait_invocations.len(),
            self.trait_requirement_invocations.len(),
            self.type_parameter_class_receivers.len(),
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)?;
        let mut add = |count: usize| -> Option<()> {
            rows = rows.checked_add(count)?;
            Some(())
        };
        for recipe in &self.type_recipes {
            match recipe {
                PortableTypeRecipeV1::Applied { arguments, .. } => add(arguments.len())?,
                PortableTypeRecipeV1::Tuple { elements } | PortableTypeRecipeV1::Union { members: elements } => add(elements.len())?,
                PortableTypeRecipeV1::Record { labels, values } => {
                    add(labels.len())?;
                    add(values.len())?;
                }
                PortableTypeRecipeV1::Pack { positionals, labels, labeled } => {
                    add(positionals.len())?;
                    add(labels.len())?;
                    add(labeled.len())?;
                }
                PortableTypeRecipeV1::TupleLabeled { elements } => add(elements.len())?,
                PortableTypeRecipeV1::RecordKeys { fields } => add(fields.len())?,
                PortableTypeRecipeV1::Callable { parameters, .. } => add(parameters.len())?,
                PortableTypeRecipeV1::TypeLambda { parameters, .. } => add(parameters.len())?,
                PortableTypeRecipeV1::AppliedStatic { arguments, .. } => {
                    add(arguments.len())?;
                    for argument in arguments {
                        if let PortableStaticTypeArgumentV1::RecordRow { fields, .. } = argument {
                            add(fields.len())?;
                        }
                    }
                }
                _ => {}
            }
        }
        for kind in &self.kind_recipes {
            if let PortableKindRecipeV1::Arrow { parameters, .. } = kind {
                add(parameters.len())?;
            }
        }
        for spec in &self.enum_specs {
            add(spec.variants.len())?;
            for variant in &spec.variants {
                add(variant.fields.len())?;
                if let Some(layout) = &variant.layout {
                    add(layout.components.len())?;
                }
            }
        }
        for spec in &self.data_specs {
            add(spec.components.len())?;
            if let Some(layout) = &spec.layout {
                add(layout.components.len())?;
            }
        }
        for construction in &self.data_constructions {
            add(construction.argument_to_component.len())?;
            add(construction.component_to_logical.len())?;
            add(construction.layout.components.len())?;
        }
        for product in &self.anonymous_products {
            add(product.layout.components.len())?;
            match &product.kind {
                PortableAnonymousProductKindV1::Tuple { labels, .. } | PortableAnonymousProductKindV1::Pack { labels, .. } => add(labels.len())?,
                PortableAnonymousProductKindV1::Record {
                    logical_keys,
                    source_to_logical,
                } => {
                    add(logical_keys.len())?;
                    add(source_to_logical.len())?;
                }
            }
        }
        for environment in &self.call_environments {
            add(environment.bindings.len())?;
            add(environment.conformance_slots.len())?;
            for slot in &environment.conformance_slots {
                add(conformance_slot_rows(slot)?)?;
            }
        }
        for descriptor in &self.family_descriptors {
            add(descriptor.entries.len())?;
        }
        for family in &self.conditional_families {
            add(family.entries.len())?;
        }
        for operation in &self.family_operations {
            add(operation.slots.len())?;
        }
        for set in &self.family_candidate_sets {
            add(set.candidates.len())?;
        }
        for invocation in &self.trait_invocations {
            add(invocation.requirement_slots.len())?;
            add(invocation.requirement_targets.len())?;
            add(invocation.antecedent_slots.len())?;
            add(invocation.type_bindings.len())?;
        }
        Some(rows)
    }

    fn validate_references(&self, constant_len: usize) -> Result<(), ArtifactError> {
        let type_len = self.type_recipes.len();
        let kind_len = self.kind_recipes.len();
        let env_len = self.call_environments.len();
        let construction_len = self.data_constructions.len();
        for recipe in &self.type_recipes {
            match recipe {
                PortableTypeRecipeV1::Applied { origin, arguments } => {
                    check_type_ref(origin, type_len)?;
                    for argument in arguments {
                        check_type_ref(argument, type_len)?;
                    }
                }
                PortableTypeRecipeV1::Tuple { elements } | PortableTypeRecipeV1::Union { members: elements } => {
                    for element in elements {
                        check_type_ref(element, type_len)?;
                    }
                }
                PortableTypeRecipeV1::Record { labels, values } => {
                    if labels.len() != values.len() {
                        return Err(invalid("semantic record recipe shape mismatch"));
                    }
                    for value in values {
                        check_type_ref(value, type_len)?;
                    }
                }
                PortableTypeRecipeV1::Pack { positionals, labels, labeled } => {
                    if labels.len() != labeled.len() {
                        return Err(invalid("semantic pack recipe shape mismatch"));
                    }
                    for value in positionals.iter().chain(labeled) {
                        check_type_ref(value, type_len)?;
                    }
                }
                PortableTypeRecipeV1::TupleLabeled { elements } => {
                    for element in elements {
                        check_type_ref(&element.ty, type_len)?;
                    }
                }
                PortableTypeRecipeV1::RecordKeys { fields } => {
                    for field in fields {
                        check_type_ref(&field.ty, type_len)?;
                    }
                }
                PortableTypeRecipeV1::Callable { parameters, return_type } => {
                    check_type_ref(return_type, type_len)?;
                    for parameter in parameters {
                        check_type_ref(&parameter.ty, type_len)?;
                    }
                }
                PortableTypeRecipeV1::TypeLambda { parameters, body } => {
                    check_type_ref(body, type_len)?;
                    for parameter in parameters {
                        check_kind_ref(&parameter.kind, kind_len)?;
                    }
                }
                PortableTypeRecipeV1::AppliedStatic { origin, arguments } => {
                    check_type_ref(origin, type_len)?;
                    for argument in arguments {
                        match argument {
                            PortableStaticTypeArgumentV1::Term { ty } => check_type_ref(ty, type_len)?,
                            PortableStaticTypeArgumentV1::RecordRow { fields, .. } => {
                                for field in fields {
                                    check_type_ref(&field.ty, type_len)?;
                                }
                            }
                            PortableStaticTypeArgumentV1::DynamicType => {}
                        }
                    }
                }
                _ => {}
            }
        }
        for recipe in &self.kind_recipes {
            if let PortableKindRecipeV1::Arrow { parameters, result } = recipe {
                check_kind_ref(result, kind_len)?;
                for parameter in parameters {
                    check_kind_ref(parameter, kind_len)?;
                }
            }
        }
        for construction in &self.data_constructions {
            check_runtime_type_recipe(&construction.type_recipe, type_len)?;
        }
        for product in &self.anonymous_products {
            check_runtime_type_recipe(&product.type_recipe, type_len)?;
        }
        for target in &self.associated_targets {
            check_invocation_target(target, construction_len)?;
        }
        for descriptor in &self.family_descriptors {
            for entry in &descriptor.entries {
                check_family_target(&entry.target, construction_len)?;
            }
        }
        for candidates in &self.family_candidate_sets {
            for candidate in &candidates.candidates {
                if let Some(target) = &candidate.target {
                    check_invocation_target(target, construction_len)?;
                }
            }
        }
        for family in &self.conditional_families {
            for entry in &family.entries {
                if let Some(environment) = entry.call_environment {
                    check_env_ref(environment, env_len)?;
                }
                check_constant_ref(entry.fallback_method, constant_len)?;
            }
        }
        for environment in &self.call_environments {
            for binding in &environment.bindings {
                check_runtime_type_recipe(&binding.recipe, type_len)?;
            }
            for slot in &environment.conformance_slots {
                check_conformance_slot(slot, type_len)?;
            }
        }
        for invocation in &self.trait_invocations {
            if let Some(environment) = invocation.call_environment {
                check_env_ref(environment, env_len)?;
            }
            check_trait_selection_target(&invocation.selection, type_len)?;
            for binding in &invocation.type_bindings {
                check_runtime_type_recipe(&binding.recipe, type_len)?;
            }
            for target in &invocation.requirement_targets {
                check_trait_selection_target(&target.target, type_len)?;
            }
            for slot in &invocation.antecedent_slots {
                check_trait_selection_target(&slot.target, type_len)?;
            }
        }
        Ok(())
    }
}

fn check_type_ref(id: &PortableTypeRecipeId, limit: usize) -> Result<(), ArtifactError> {
    if (id.0 as usize) < limit {
        Ok(())
    } else {
        Err(invalid(format!("type recipe reference {} is out of bounds", id.0)))
    }
}

fn check_kind_ref(id: &PortableKindRecipeId, limit: usize) -> Result<(), ArtifactError> {
    if (id.0 as usize) < limit {
        Ok(())
    } else {
        Err(invalid(format!("kind recipe reference {} is out of bounds", id.0)))
    }
}

fn check_env_ref(id: PortableCallEnvironmentId, limit: usize) -> Result<(), ArtifactError> {
    if (id.0 as usize) < limit {
        Ok(())
    } else {
        Err(invalid(format!("call environment reference {} is out of bounds", id.0)))
    }
}

fn check_constant_ref(id: crate::executable::prototype::ConstantId, limit: usize) -> Result<(), ArtifactError> {
    if (id.0 as usize) < limit {
        Ok(())
    } else {
        Err(invalid(format!("constant reference {} is out of bounds", id.0)))
    }
}

fn check_runtime_type_recipe(recipe: &PortableRuntimeTypeRecipeV1, type_len: usize) -> Result<(), ArtifactError> {
    match recipe {
        PortableRuntimeTypeRecipeV1::Closed { ty } | PortableRuntimeTypeRecipeV1::Template { ty } => check_type_ref(ty, type_len),
    }
}

fn check_conformance_slot(slot: &PortableConformanceSlotV1, type_len: usize) -> Result<(), ArtifactError> {
    match slot {
        PortableConformanceSlotV1::Selection { nested, .. } => {
            for entry in nested {
                check_conformance_slot(&entry.recipe, type_len)?;
            }
        }
        PortableConformanceSlotV1::Forwarded { .. } => {}
        PortableConformanceSlotV1::DerivedConditional { bindings, nested, .. } => {
            for binding in bindings {
                check_runtime_type_recipe(&binding.recipe, type_len)?;
            }
            for entry in nested {
                check_conformance_slot(&entry.recipe, type_len)?;
            }
        }
    }
    Ok(())
}

fn check_trait_selection_target(target: &PortableTraitSelectionTargetV1, type_len: usize) -> Result<(), ArtifactError> {
    if let PortableTraitSelectionTargetV1::DataComponent { specialized_type, .. } = target {
        check_type_ref(specialized_type, type_len)?;
    }
    Ok(())
}

fn check_invocation_target(target: &PortableInvocationTargetV1, construction_len: usize) -> Result<(), ArtifactError> {
    if let PortableInvocationTargetV1::DataConstructor { construction, .. } = target
        && (*construction as usize) >= construction_len
    {
        return Err(invalid(format!("data construction reference {construction} is out of bounds")));
    }
    Ok(())
}

fn check_family_target(target: &PortableFamilyTargetV1, construction_len: usize) -> Result<(), ArtifactError> {
    match target {
        PortableFamilyTargetV1::Behavioral { target } => check_invocation_target(target, construction_len),
        PortableFamilyTargetV1::DataConstructor { construction, .. } if (*construction as usize) >= construction_len => {
            Err(invalid(format!("data construction reference {construction} is out of bounds")))
        }
        _ => Ok(()),
    }
}

fn conformance_slot_rows(slot: &PortableConformanceSlotV1) -> Option<usize> {
    match slot {
        PortableConformanceSlotV1::Selection { nested, .. } => nested
            .iter()
            .try_fold(nested.len(), |sum, entry| sum.checked_add(conformance_slot_rows(&entry.recipe)?)),
        PortableConformanceSlotV1::Forwarded { .. } => Some(0),
        PortableConformanceSlotV1::DerivedConditional { bindings, nested, .. } => {
            nested.iter().try_fold(bindings.len().checked_add(nested.len())?, |sum, entry| {
                sum.checked_add(conformance_slot_rows(&entry.recipe)?)
            })
        }
    }
}
