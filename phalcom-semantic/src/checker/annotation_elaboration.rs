//! Annotation elaboration supporting query-local inference holes `_` (COLL006.C1.P1).
//!
//! Law: InferVarId != TypeId. An inference hole creates a query-local inference variable
//! with the target binder's declared KindId/domain. Unsolved variables never escape into
//! durable canonical products.

use crate::checker::context::CheckingContext;
use crate::checker::inference::{InferenceContextId, InferenceFrameId, InferenceStaticArgument, InferenceTerm};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::InferVarId;
use crate::types::annotation::{TypeFormationSite, TypeResolver, resolve_static_generic_value};
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::kind::KindData;
use phalcom_ast::ast::{GenericArgumentSyntax, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;

/// Origin of a single inference hole in an elaborated annotation.
#[derive(Clone, Debug, PartialEq)]
pub struct InferenceHoleOrigin {
    pub range: SourceRange,
    pub parameter: Option<TypeParameterId>,
    pub kind: KindId,
    pub variable: InferVarId,
}

/// The checked outcome of elaborating a type annotation.
#[derive(Clone, Debug)]
pub enum AnnotationElaboration {
    /// Fully canonical type without inference holes.
    Canonical(TypeId),

    /// An inference-bearing annotation containing one or more query-local holes.
    Inference {
        context: InferenceContextId,
        frame: InferenceFrameId,
        term: InferenceTerm,
        holes: Box<[InferenceHoleOrigin]>,
    },
}

/// Checks if an annotation syntax contains any `GenericArgumentSyntax::InferenceHole`.
pub fn annotation_contains_inference_hole(annotation: &TypeAnnotation) -> bool {
    match &annotation.expr {
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            annotation_contains_inference_hole(origin)
                || arguments.iter().any(argument_contains_inference_hole)
        }
        TypeAnnotationExpr::Union { members, .. } => {
            members.iter().any(annotation_contains_inference_hole)
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            elements.iter().any(|el| match el {
                phalcom_ast::ast::TypeTupleItem::Element(e) => annotation_contains_inference_hole(&e.ty),
                phalcom_ast::ast::TypeTupleItem::Expansion(e) => annotation_contains_inference_hole(&e.operand),
                phalcom_ast::ast::TypeTupleItem::Remainder(r) => match &r.remainder {
                    phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } => {
                        annotation_contains_inference_hole(constraint)
                    }
                    _ => false,
                },
            })
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            parameters.iter().any(|item| match item {
                phalcom_ast::ast::TypeCallableItem::Parameter(p) => annotation_contains_inference_hole(&p.ty),
                _ => false,
            }) || annotation_contains_inference_hole(result)
        }
        _ => false,
    }
}

pub fn argument_contains_inference_hole(arg: &GenericArgumentSyntax) -> bool {
    match arg {
        GenericArgumentSyntax::InferenceHole { .. } => true,
        GenericArgumentSyntax::Single(ann) => annotation_contains_inference_hole(ann),
        GenericArgumentSyntax::Expansion(e) => annotation_contains_inference_hole(&e.operand),
        GenericArgumentSyntax::Remainder(r) => match &r.remainder {
            phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } => {
                annotation_contains_inference_hole(constraint)
            }
            _ => false,
        },
        GenericArgumentSyntax::Value(_) => false,
    }
}

/// Elaborates a type annotation, delegating to canonical resolution if no inference holes
/// are present, or allocating query-local inference variables with destination binder kinds
/// for each hole.
pub fn elaborate_annotation(
    ctx: &mut CheckingContext<'_>,
    resolver: &dyn TypeResolver,
    annotation: &TypeAnnotation,
) -> Result<AnnotationElaboration, Vec<SemanticDiagnostic>> {
    if !annotation_contains_inference_hole(annotation) {
        let (knowledge, _causal) = ctx.resolve_type_annotation(resolver, annotation);
        if let Some(ty) = knowledge.ty() {
            if let Some((declaration, arguments)) = ctx.store.applied_nominal_static_parts(ty)
                && declaration == ctx.core_ids.vector
                && let [crate::types::store::StaticArgument::Term(_element), crate::types::store::StaticArgument::Term(extent)] = arguments[..]
                && let crate::types::store::TypeData::ConstInt(value) = ctx.store.get(extent)
                && value.to_bigint() < 0.into()
            {
                return Err(vec![SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::GenericInferenceConflict,
                    "Vector extent must be nonnegative",
                    annotation.range,
                )]);
            }
            return Ok(AnnotationElaboration::Canonical(ty));
        } else {
            return Err(Vec::new());
        }
    }

    let (context, frame) = ctx.create_inference_context();
    ctx.push_inference_frame(context, frame);

    let mut holes = Vec::new();
    let term = match elaborate_annotation_inner(ctx, resolver, annotation, context, frame, &mut holes) {
        Ok(term) => term,
        Err(err) => {
            ctx.pop_inference_frame(context, frame);
            ctx.finish_inference_context(context);
            return Err(err);
        }
    };

    Ok(AnnotationElaboration::Inference {
        context,
        frame,
        term,
        holes: holes.into_boxed_slice(),
    })
}

fn elaborate_annotation_inner(
    ctx: &mut CheckingContext<'_>,
    resolver: &dyn TypeResolver,
    annotation: &TypeAnnotation,
    context: InferenceContextId,
    frame: InferenceFrameId,
    holes: &mut Vec<InferenceHoleOrigin>,
) -> Result<InferenceTerm, Vec<SemanticDiagnostic>> {
    match &annotation.expr {
        TypeAnnotationExpr::Union { members, .. } => {
            let mut terms = Vec::with_capacity(members.len());
            for member in members {
                let term = if annotation_contains_inference_hole(member) {
                    elaborate_annotation_inner(ctx, resolver, member, context, frame, holes)?
                } else {
                    let (knowledge, _) = ctx.resolve_type_annotation(resolver, member);
                    InferenceTerm::Canonical(knowledge.ty().ok_or_else(Vec::new)?)
                };
                terms.push(term);
            }
            Ok(InferenceTerm::Union(terms.into_boxed_slice()))
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            let origin_term = if annotation_contains_inference_hole(origin) {
                elaborate_annotation_inner(ctx, resolver, origin, context, frame, holes)?
            } else {
                let ty = resolve_origin_type(ctx, resolver, origin).ok_or_else(Vec::new)?;
                InferenceTerm::Canonical(ty)
            };

            let origin_ty = match &origin_term {
                InferenceTerm::Canonical(ty) => Some(*ty),
                _ => None,
            };

            let origin_decl = origin_ty.and_then(|ty| ctx.store.nominal_origin_declaration(ty).cloned());

            let generic_sig = origin_decl
                .as_ref()
                .and_then(|decl| ctx.declarations.generic_signature(decl).cloned());

            let arrow_param_kinds = origin_ty.and_then(|ty| match ctx.store.get_kind(ctx.store.kind_of(ty)) {
                KindData::Arrow { parameters, .. } => Some(parameters.clone()),
                _ => None,
            });

            let mut static_args = Vec::with_capacity(arguments.len());
            for (idx, arg) in arguments.iter().enumerate() {
                let param_id = generic_sig.as_ref().and_then(|sig| sig.parameters.get(idx).copied());
                let param_kind = generic_sig
                    .as_ref()
                    .and_then(|sig| sig.parameter_kinds.get(idx).copied())
                    .or_else(|| arrow_param_kinds.as_ref().and_then(|p| p.get(idx).copied()))
                    .ok_or_else(|| vec![SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ApplicationTooManyArguments,
                        "generic argument has no corresponding parameter",
                        arg.range(),
                    )])?;

                match arg {
                    GenericArgumentSyntax::InferenceHole { range } => {
                        let session_rc = ctx.inference_session(context).unwrap();
                        let var = session_rc.borrow_mut().fresh_variable_in_frame(frame, param_kind);
                        holes.push(InferenceHoleOrigin {
                            range: *range,
                            parameter: param_id,
                            kind: param_kind,
                            variable: var,
                        });
                        static_args.push(InferenceStaticArgument::Term(InferenceTerm::Var(var)));
                    }
                    GenericArgumentSyntax::Single(arg_ann) => {
                        if annotation_contains_inference_hole(arg_ann) {
                            let sub_term = elaborate_annotation_inner(ctx, resolver, arg_ann, context, frame, holes)?;
                            static_args.push(InferenceStaticArgument::Term(sub_term));
                        } else {
                            let site = TypeFormationSite::module(ctx.current_module.clone());
                            let (res, _) = ctx.resolve_type_form(resolver, &site, arg_ann);
                            let ty = res.into_ready().ok_or_else(Vec::new)?;
                            static_args.push(InferenceStaticArgument::Term(InferenceTerm::Canonical(ty)));
                        }
                    }
                    GenericArgumentSyntax::Value(expr) => {
                        let mut diagnostics = Vec::new();
                        let value = resolve_static_generic_value(ctx.store, param_kind, expr, arg.range(), &mut diagnostics, &ctx.current_module);
                        let ty = value.into_ready().ok_or(diagnostics)?;
                        static_args.push(InferenceStaticArgument::Term(InferenceTerm::Canonical(ty)));
                    }
                    GenericArgumentSyntax::Expansion(_) | GenericArgumentSyntax::Remainder(_) => {
                        return Err(Vec::new());
                    }
                }
            }

            if origin_ty.and_then(|ty| ctx.store.nominal_origin_declaration(ty)) == Some(&ctx.core_ids.vector)
                && let Some(InferenceStaticArgument::Term(InferenceTerm::Canonical(extent))) = static_args.get(1)
                && let crate::types::store::TypeData::ConstInt(value) = ctx.store.get(*extent)
                && value.to_bigint() < 0.into()
            {
                return Err(vec![SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::GenericInferenceConflict,
                    "Vector extent must be nonnegative",
                    annotation.range,
                )]);
            }

            Ok(InferenceTerm::Applied {
                origin: Box::new(origin_term),
                arguments: static_args.into_boxed_slice(),
            })
        }
        _ => Err(Vec::new()),
    }
}

fn resolve_origin_type(
    ctx: &mut CheckingContext<'_>,
    resolver: &dyn TypeResolver,
    origin: &TypeAnnotation,
) -> Option<TypeId> {
    let site = TypeFormationSite::module(ctx.current_module.clone());
    if let TypeAnnotationExpr::Reference(reference) = &origin.expr {
        let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
        if let Some(decl) = resolver.resolve_type_name(&ctx.current_module, &reference.root, &members) {
            if let Some(form) = ctx.declarations.form(&decl) {
                return Some(form);
            }
        }
    }
    let (res, _) = ctx.resolve_type_form(resolver, &site, origin);
    res.into_ready()
}
