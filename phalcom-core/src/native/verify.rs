//! Verification of native descriptors against parsed universe AST sources.

use super::descriptor::PrimitiveDescriptor;
use super::source::{NativeMemberKey, NativeSourceIndex};
use phalcom_native_meta::{NativeAnchorPolicy, PrimitiveKey, UNIVERSE_CLASS_RELATIONS, UniverseKey};
use thiserror::Error;

/// Migration mode for source/native contract verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeContractMode {
    /// Require every source `@native` anchor to resolve, but tolerate native
    /// descriptors whose source anchors have not migrated yet.
    AnchorsMustResolve,
    /// Require a complete source/native bijection, except for hidden native
    /// descriptors.
    Strict,
}

#[derive(Error, Debug, Clone)]
pub enum NativeContractError {
    #[error("native descriptor missing source anchor for owner {:?}, side {:?}, selector '{selector}' (defined in {source_file}:{source_line})", key.owner, key.side)]
    MissingSourceAnchor {
        key: PrimitiveKey,
        selector: String,
        source_file: &'static str,
        source_line: u32,
    },

    #[error("source member marked @native has no registered native descriptor for owner {:?}, side {:?}, selector '{selector}' in class '{class}'", key.owner, key.side)]
    MissingNativeDescriptor { key: NativeMemberKey, class: String, selector: String },

    #[error("visibility mismatch for owner {:?}, side {:?}, selector '{selector}': descriptor is {desc_vis:?}, source anchor is {source_vis:?}", key.owner, key.side)]
    VisibilityMismatch {
        key: PrimitiveKey,
        selector: String,
        desc_vis: phalcom_native_meta::NativeVisibility,
        source_vis: phalcom_native_meta::NativeVisibility,
    },

    #[error("universe class {key:?} has no canonical source presentation")]
    MissingClassPresentation { key: UniverseKey },

    #[error("universe class {key:?} source presentation must be marked @native")]
    NativeClassAnchorRequired { key: UniverseKey },

    #[error("universe class {key:?} superclass mismatch: expected {expected:?}, source declares {actual:?}")]
    SuperclassMismatch {
        key: UniverseKey,
        expected: Option<UniverseKey>,
        actual: Option<String>,
    },

    #[error("native source member is missing a complete type annotation for owner {key:?}, selector '{selector}'")]
    MissingTypeAnnotation { key: PrimitiveKey, selector: String },
}

/// Verifies that all registered descriptors match universe source anchors and vice versa.
pub fn verify_native_contracts(ast_index: &NativeSourceIndex, descriptors: &[&'static PrimitiveDescriptor]) -> Result<(), NativeContractError> {
    verify_native_contracts_with_mode(ast_index, descriptors, NativeContractMode::Strict)
}

/// Verifies source/native contracts under an explicit migration mode.
pub fn verify_native_contracts_with_mode(
    ast_index: &NativeSourceIndex,
    descriptors: &[&'static PrimitiveDescriptor],
    mode: NativeContractMode,
) -> Result<(), NativeContractError> {
    // 1. Every source anchor must resolve exactly once. The source index
    // rejects duplicate keys while building, so one map hit is sufficient.
    for (source_key, anchor) in &ast_index.members {
        let desc_found = descriptors
            .iter()
            .any(|d| d.surface.key.owner == source_key.owner && d.surface.key.side == source_key.side && d.surface.key.selector == source_key.selector);
        if !desc_found {
            let key = NativeMemberKey {
                owner: source_key.owner,
                side: source_key.side,
                selector: source_key.selector.clone(),
            };
            return Err(NativeContractError::MissingNativeDescriptor {
                key,
                class: anchor.class_name.clone(),
                selector: anchor.selector.clone(),
            });
        }
    }

    // 2. Every descriptor with AnchorPolicy::Required must have an AST anchor
    // in strict mode. During migration this direction is intentionally
    // deferred so bootstrap can enforce newly authored anchors immediately.
    if mode == NativeContractMode::Strict {
        for desc in descriptors {
            if desc.surface.anchor == NativeAnchorPolicy::Hidden {
                continue;
            }

            let key = desc.surface.key;
            let source_key = source_key(key);
            let Some(anchor) = ast_index.members.get(&source_key) else {
                return Err(NativeContractError::MissingSourceAnchor {
                    key,
                    selector: key.selector.to_string(),
                    source_file: desc.source.file,
                    source_line: desc.source.line,
                });
            };

            if anchor.visibility != desc.surface.visibility {
                return Err(NativeContractError::VisibilityMismatch {
                    key,
                    selector: key.selector.to_string(),
                    desc_vis: desc.surface.visibility,
                    source_vis: anchor.visibility,
                });
            }

            let typed = ast_index
                .census
                .members
                .iter()
                .find(|member| member.owner == Some(key.owner) && member.side == key.side && member.selector == key.selector)
                .is_some_and(|member| member.typed);
            if !typed {
                return Err(NativeContractError::MissingTypeAnnotation {
                    key,
                    selector: key.selector.to_owned(),
                });
            }
        }

        verify_class_presentations(ast_index)?;
    } else {
        for desc in descriptors {
            let source_key = source_key(desc.surface.key);
            if let Some(anchor) = ast_index.members.get(&source_key) {
                if anchor.visibility != desc.surface.visibility {
                    return Err(NativeContractError::VisibilityMismatch {
                        key: desc.surface.key,
                        selector: desc.surface.key.selector.to_string(),
                        desc_vis: desc.surface.visibility,
                        source_vis: anchor.visibility,
                    });
                }
            }
        }
    }

    Ok(())
}

fn verify_class_presentations(ast_index: &NativeSourceIndex) -> Result<(), NativeContractError> {
    for relation in UNIVERSE_CLASS_RELATIONS {
        let is_runtime_support = phalcom_native_meta::UNIVERSE_BINDINGS
            .iter()
            .any(|b| b.key == relation.class && b.kind == phalcom_native_meta::UniverseBindingKind::RuntimeSupportClass);
        if is_runtime_support {
            continue;
        }

        let Some(row) = ast_index.presentations.get(&relation.class) else {
            return Err(NativeContractError::MissingClassPresentation { key: relation.class });
        };
        if !row.native {
            return Err(NativeContractError::NativeClassAnchorRequired { key: relation.class });
        }

        let actual = row.superclass.as_deref().and_then(UniverseKey::from_name);
        if actual != relation.superclass {
            return Err(NativeContractError::SuperclassMismatch {
                key: relation.class,
                expected: relation.superclass,
                actual: row.superclass.clone(),
            });
        }
    }
    Ok(())
}

fn source_key(key: PrimitiveKey) -> NativeMemberKey {
    NativeMemberKey {
        owner: key.owner,
        side: key.side,
        selector: key.selector.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PhResult;
    use crate::native::{PrimitiveDescriptor, PrimitiveEntry};
    use crate::value::Value;
    use phalcom_ast::parser;
    use phalcom_native_meta::{
        CallableTypeSpec, EffectSpec, NativeLifecycleSpec, NativeSourceSpec, NativeStability, NativeTrust, ParameterTupleSpec, PrimitiveAbi,
        PrimitiveSurfaceSpec, ReturnFlowSpec, TerminationSpec, TypeExprSpec,
    };

    fn dummy_primitive(_vm: &mut crate::vm::VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
        Ok(Value::none())
    }

    static EMPTY_PARAMS: ParameterTupleSpec = ParameterTupleSpec {
        positional: &[],
        labeled: &[],
        rest: None,
    };
    static RETURN_TYPE: TypeExprSpec = TypeExprSpec::Universe(phalcom_native_meta::UniverseKey::String);
    static CALLABLE: CallableTypeSpec = CallableTypeSpec {
        type_params: &[],
        params: &EMPTY_PARAMS,
        return_type: &RETURN_TYPE,
    };
    static SURFACE: PrimitiveSurfaceSpec = PrimitiveSurfaceSpec {
        key: PrimitiveKey {
            owner: phalcom_native_meta::UniverseKey::String,
            side: phalcom_native_meta::NativeDispatch::Instance,
            selector: "_$byteCount",
        },
        visibility: phalcom_native_meta::NativeVisibility::Internal,
        stability: NativeStability::Stable,
        anchor: NativeAnchorPolicy::Required,
        params: &EMPTY_PARAMS,
        returns: &RETURN_TYPE,
        callable: &CALLABLE,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        effects: EffectSpec::Unknown,
        flow: ReturnFlowSpec::Unknown,
        termination: TerminationSpec::Unknown,
        since: None,
        deprecated_since: None,
        replacement: None,
        lifecycle: NativeLifecycleSpec::UNKNOWN,
        intrinsic: None,
        trust: NativeTrust::Ordinary,
        docs: None,
        conceptual: None,
    };
    static DESCRIPTOR: PrimitiveDescriptor = PrimitiveDescriptor {
        surface: &SURFACE,
        abi: PrimitiveAbi::Value,
        entry: PrimitiveEntry::Value(dummy_primitive),
        source: NativeSourceSpec {
            module_path: "test",
            rust_name: "dummy_primitive",
            file: "verify.rs",
            line: 1,
        },
    };

    #[test]
    fn migration_mode_accepts_unanchored_required_descriptors() {
        let program = parser::parse("class String { }", 0).program;
        let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(
            ["scalar", "string"].into_iter().map(|part| phalcom_modules::ModuleComponent::from_identifier(part).unwrap()).collect::<Vec<_>>(),
        ));
        let index = NativeSourceIndex::from_program_at(&module, &program).expect("index builds");
        verify_native_contracts_with_mode(&index, &[], NativeContractMode::AnchorsMustResolve).expect("migration mode passes");
    }

    #[test]
    fn migration_mode_rejects_source_anchor_without_descriptor() {
        let parsed = parser::parse("class String {\n  @native\n  @internal\n  _$byteCount\n}\n", 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
        let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(
            ["scalar", "string"].into_iter().map(|part| phalcom_modules::ModuleComponent::from_identifier(part).unwrap()).collect::<Vec<_>>(),
        ));
        let index = NativeSourceIndex::from_program_at(&module, &parsed.program).expect("index builds");
        let error = verify_native_contracts_with_mode(&index, &[], NativeContractMode::AnchorsMustResolve).expect_err("orphan source anchor must fail");
        assert!(matches!(error, NativeContractError::MissingNativeDescriptor { .. }));
    }

    #[test]
    fn strict_mode_rejects_required_descriptor_without_source_anchor() {
        let program = parser::parse("class String { }", 0).program;
        let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(
            ["scalar", "string"].into_iter().map(|part| phalcom_modules::ModuleComponent::from_identifier(part).unwrap()).collect::<Vec<_>>(),
        ));
        let index = NativeSourceIndex::from_program_at(&module, &program).expect("index builds");
        let error = verify_native_contracts(&index, &[&DESCRIPTOR]).expect_err("strict mode must require the source anchor");
        assert!(matches!(error, NativeContractError::MissingSourceAnchor { .. }));
    }
}
