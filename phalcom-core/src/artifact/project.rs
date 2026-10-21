//! Projection of emitted callable layouts into portable executable rows.
//!
//! The graph capture owns constant, semantic-pool and source-map relocation.
//! This adapter copies the existing emitter output and carries no VM handles.

use crate::callable::Callable;
use crate::chunk::Chunk;
use crate::interner::Interner;
use crate::parameters::{ParameterShape, RestKind};
use phalcom_compiler_artifact::executable::prototype::{
    ConstantId, ParameterShapeV1, PortablePrototypeV1, PrototypeOwnerV1, RestKindV1, SemanticPoolId, SourceMapId, UpvalueCaptureV1,
};
use phalcom_compiler_artifact::module_world::SpanV1;

/// Artifact-local references supplied by graph capture, rather than copied
/// from the emitting VM's runtime tables.
pub struct PrototypeRelocations {
    pub constants: Vec<ConstantId>,
    pub source: SourceMapId,
    pub semantics: SemanticPoolId,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutableProjectionError {
    #[error("{owner:?}: {field}={found} exceeds the portable operand width")]
    OperandOverflow {
        owner: PrototypeOwnerV1,
        field: &'static str,
        found: usize,
    },
    #[error("{owner:?}: emitted {field} count {found} disagrees with {expected}")]
    InconsistentLayout {
        owner: PrototypeOwnerV1,
        field: &'static str,
        found: usize,
        expected: usize,
    },
    #[error("runtime class `{name}` has no canonical declaration authority")]
    UnboundClass { name: String },
}

fn wire_u32(value: usize, field: &'static str, owner: &PrototypeOwnerV1) -> Result<u32, ExecutableProjectionError> {
    u32::try_from(value).map_err(|_| ExecutableProjectionError::OperandOverflow {
        owner: owner.clone(),
        field,
        found: value,
    })
}

/// Projects code and calling layout after graph capture has assigned every
/// referenced artifact row. Mutable caches and source text are excluded.
/// The returned row still requires full executable-image validation.
pub fn project_callable_prototype(
    callable: &Callable,
    interner: &Interner,
    owner: PrototypeOwnerV1,
    relocations: PrototypeRelocations,
) -> Result<PortablePrototypeV1, ExecutableProjectionError> {
    // Destructure exhaustively so a new emitter field requires an explicit
    // persistence decision here.
    let Callable {
        chunk,
        max_slots,
        num_upvalues,
        upvalues,
        arity,
        parameter_shape,
        name_sym,
        local_names,
    } = callable;
    let Chunk {
        code,
        constants,
        spans,
        source_id: _,
        caches: _,
        gcaches: _,
        executable_semantics: _,
    } = chunk;
    for (field, found, expected) in [
        ("constant relocations", relocations.constants.len(), constants.len()),
        ("instruction spans", spans.len(), code.len()),
        ("upvalue descriptors", upvalues.len(), *num_upvalues),
    ] {
        if found != expected {
            return Err(ExecutableProjectionError::InconsistentLayout { owner, field, found, expected });
        }
    }
    let ParameterShape {
        fixed_positionals,
        fixed_labels,
        rest,
    } = parameter_shape;
    let captures = upvalues
        .iter()
        .map(|capture| {
            let index = wire_u32(capture.index, "upvalue capture index", &owner)?;
            Ok(if capture.is_local {
                UpvalueCaptureV1::Local(index)
            } else {
                UpvalueCaptureV1::Upvalue(index)
            })
        })
        .collect::<Result<_, ExecutableProjectionError>>()?;
    let parameters = ParameterShapeV1 {
        fixed_positionals: wire_u32(*fixed_positionals, "fixed positional parameters", &owner)?,
        fixed_labels: fixed_labels.iter().map(|symbol| interner.lookup(*symbol).to_owned()).collect(),
        rest: rest.map(|kind| match kind {
            RestKind::Positional => RestKindV1::Positional,
            RestKind::Labeled => RestKindV1::Labeled,
            RestKind::Split => RestKindV1::Split,
            RestKind::Complete => RestKindV1::Complete,
        }),
    };
    let projected = PortablePrototypeV1 {
        max_slots: wire_u32(*max_slots, "maximum frame slots", &owner)?,
        arity: wire_u32(*arity, "arity", &owner)?,
        owner,
        code: code.iter().copied().map(Into::into).collect(),
        constants: relocations.constants,
        upvalues: captures,
        parameters,
        name: interner.lookup(*name_sym).to_owned(),
        local_names: local_names.iter().map(|symbol| interner.lookup(*symbol).to_owned()).collect(),
        source: relocations.source,
        semantics: relocations.semantics,
        spans: spans
            .iter()
            .map(|span| SpanV1 {
                start: span.start as u64,
                end: span.end as u64,
            })
            .collect(),
    };
    Ok(projected)
}

/// Resolves the registered semantic declaration of a runtime class. Display
/// names never establish its persistent identity.
pub fn project_class_reference(
    vm: &crate::vm::VM,
    class: crate::heap::ClassId,
) -> Result<phalcom_compiler_artifact::executable::constant::StableClassRefV1, ExecutableProjectionError> {
    use phalcom_compiler_artifact::executable::constant::StableClassRefV1;
    use phalcom_type_meta::identity::StableDispatchSide;

    let declaration_for_class = |class| {
        vm.typing_registry
            .declaration_for_nominal(class)
            .cloned()
            .or_else(|| {
                vm.classes.iter().find_map(|(key, candidate)| {
                    if *candidate != class {
                        return None;
                    }
                    let module = &vm.heap.module(key.module).id;
                    vm.typing_registry.declaration_identity(module, vm.interner.lookup(key.name)).cloned()
                })
            })
            .or_else(|| {
                phalcom_native_meta::UNIVERSE_BINDINGS
                    .iter()
                    .find(|binding| vm.universe.classes.resolve(binding.key) == class)
                    .map(|binding| {
                        phalcom_semantic::metadata::stable_identity::to_stable_declaration(&phalcom_semantic::core_surface::universe_declaration(binding.key))
                    })
            })
    };

    if let Some(declaration) = declaration_for_class(class) {
        return Ok(StableClassRefV1 {
            declaration,
            side: StableDispatchSide::Instance,
        });
    }
    if let Some(declaration) = vm
        .classes
        .values()
        .filter(|candidate| vm.heap.class(**candidate).class == class)
        .filter_map(|candidate| declaration_for_class(*candidate))
        .min()
    {
        return Ok(StableClassRefV1 {
            declaration,
            side: StableDispatchSide::Class,
        });
    }
    Err(ExecutableProjectionError::UnboundClass {
        name: vm.heap.class(class).name.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::{ExecutableProjectionError, PrototypeRelocations, project_callable_prototype};
    use crate::callable::Callable;
    use crate::chunk::Chunk;
    use crate::interner::Interner;
    use crate::parameters::ParameterShape;
    use phalcom_compiler_artifact::executable::prototype::{PrototypeId, PrototypeOwnerV1, SemanticPoolId, SourceMapId};

    #[test]
    fn callable_projection_rejects_inconsistent_upvalue_layout() {
        let mut interner = Interner::with_capacity(4);
        let callable = Callable {
            chunk: Chunk::new(),
            max_slots: 1,
            num_upvalues: 1,
            upvalues: Vec::new(),
            arity: 0,
            parameter_shape: ParameterShape::closure(0, false),
            name_sym: interner.intern("fixture"),
            local_names: Vec::new(),
        };
        let owner = PrototypeOwnerV1::Nested {
            parent: PrototypeId(0),
            ordinal: 0,
        };
        let error = project_callable_prototype(
            &callable,
            &interner,
            owner,
            PrototypeRelocations {
                constants: Vec::new(),
                source: SourceMapId(0),
                semantics: SemanticPoolId(0),
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            ExecutableProjectionError::InconsistentLayout {
                field: "upvalue descriptors",
                found: 0,
                expected: 1,
                ..
            }
        ));
    }
}

/// Copies the emitter's complete calling shape, including rest storage slots.
pub fn project_method_signature(
    signature: &crate::method::Signature,
    interner: &Interner,
) -> phalcom_compiler_artifact::executable::constant::MethodSignatureV1 {
    use crate::method::{RestMode, Signature, SignatureKind};
    use phalcom_compiler_artifact::executable::constant::{MethodSignatureKindV1, MethodSignatureV1, RestLayoutV1, RestModeV1};

    let Signature {
        selector,
        kind,
        positional_arity,
        rest,
    } = signature;
    MethodSignatureV1 {
        selector: interner.lookup(*selector).to_owned(),
        kind: match *kind {
            SignatureKind::Method(slots) => MethodSignatureKindV1::Method { slots },
            SignatureKind::Getter => MethodSignatureKindV1::Getter,
            SignatureKind::Setter => MethodSignatureKindV1::Setter,
            SignatureKind::SubscriptGet(slots) => MethodSignatureKindV1::SubscriptGet { slots },
            SignatureKind::SubscriptSet(slots) => MethodSignatureKindV1::SubscriptSet { slots },
        },
        positional_arity: *positional_arity,
        rest: rest.as_ref().map(|layout| RestLayoutV1 {
            fixed_positionals: layout.fixed_positionals(),
            fixed_labels: layout.fixed_labels().iter().map(|label| interner.lookup(*label).to_owned()).collect(),
            mode: match layout.mode() {
                RestMode::Positional { param_index } => RestModeV1::Positional { param_index },
                RestMode::Labeled { param_index } => RestModeV1::Labeled { param_index },
                RestMode::Split {
                    positional_param_index,
                    labeled_param_index,
                } => RestModeV1::Split {
                    positional_param_index,
                    labeled_param_index,
                },
                RestMode::Complete { param_index } => RestModeV1::Complete { param_index },
            },
        }),
    }
}

/// Preserves structural selector patterns as data rather than serializing
/// their interner-backed runtime matcher.
pub fn project_selector_pattern(pattern: &phalcom_common::selector::SelectorPattern) -> phalcom_compiler_artifact::executable::constant::SelectorPatternV1 {
    use phalcom_common::selector::{SelectorBase, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot};
    use phalcom_compiler_artifact::executable::constant::{SelectorBaseV1, SelectorKindPatternV1, SelectorKindV1, SelectorPatternV1, SelectorSlotV1};

    let SelectorPattern {
        base,
        kind,
        prefix,
        suffix,
        has_gap,
    } = pattern;
    let slots = |source: &[SelectorSlot]| {
        source
            .iter()
            .map(|slot| match slot {
                SelectorSlot::Positional => SelectorSlotV1::Positional,
                SelectorSlot::Label(label) => SelectorSlotV1::Label { label: label.clone() },
            })
            .collect()
    };
    SelectorPatternV1 {
        base: match base {
            SelectorBase::Named(name) => SelectorBaseV1::Named { name: name.clone() },
            SelectorBase::Subscript => SelectorBaseV1::Subscript,
        },
        kind: match kind {
            SelectorKindPattern::AnyNamed => SelectorKindPatternV1::AnyNamed,
            SelectorKindPattern::NamedAccessors => SelectorKindPatternV1::NamedAccessors,
            SelectorKindPattern::AnySubscript => SelectorKindPatternV1::AnySubscript,
            SelectorKindPattern::Exact(kind) => SelectorKindPatternV1::Exact {
                kind: match kind {
                    SelectorKind::Getter => SelectorKindV1::Getter,
                    SelectorKind::Setter => SelectorKindV1::Setter,
                    SelectorKind::Method => SelectorKindV1::Method,
                    SelectorKind::SubscriptGet => SelectorKindV1::SubscriptGet,
                    SelectorKind::SubscriptSet => SelectorKindV1::SubscriptSet,
                },
            },
        },
        prefix: slots(prefix),
        suffix: slots(suffix),
        has_gap: *has_gap,
    }
}
