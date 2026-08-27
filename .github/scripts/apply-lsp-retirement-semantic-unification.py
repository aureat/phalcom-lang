from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"anchor not found in {path}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


def regex_once(path: str, pattern: str, replacement: str) -> None:
    p = Path(path)
    text = p.read_text()
    new, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"pattern matched {count} times in {path}: {pattern[:120]!r}")
    p.write_text(new)

# 1. Canonical dispatch-owner traversal.
replace_once(
    "phalcom-semantic/src/dispatch.rs",
    "use crate::identity::{CallableId, DeclarationId};",
    "use crate::identity::{CallableId, DeclarationId, ModuleId};",
)
replace_once(
    "phalcom-semantic/src/dispatch.rs",
    "use std::collections::HashMap;",
    "use std::collections::{HashMap, HashSet};",
)
regex_once(
    "phalcom-semantic/src/dispatch.rs",
    r"    pub fn resolve_dispatch_with_trace\(.*?\n    pub fn resolve_dispatch_on_owner\(",
    '''    /// Returns the canonical owner/side traversal for one dispatch receiver.\n    ///\n    /// Class-object dispatch first walks the parallel class-side hierarchy.\n    /// When that hierarchy is exhausted it enters the canonical `Class`\n    /// instance behavior root, mirroring the runtime metaclass tower without\n    /// materializing semantic metaclass objects.\n    pub fn dispatch_owners(\n        &self,\n        hierarchy: &dyn TypeHierarchy,\n        start_decl: &DeclarationId,\n        side: DispatchSide,\n    ) -> Vec<DispatchOwner> {\n        let mut owners = Vec::new();\n        let mut visited = HashSet::new();\n        let mut current = Some(DispatchOwner {\n            declaration: start_decl.clone(),\n            side,\n        });\n        let mut entered_class_object_root = false;\n\n        while let Some(owner) = current {\n            if !visited.insert((owner.declaration.clone(), owner.side)) {\n                break;\n            }\n            owners.push(owner.clone());\n            current = if let Some(superclass) = hierarchy.superclass(&owner.declaration) {\n                Some(DispatchOwner {\n                    declaration: superclass.clone(),\n                    side: owner.side,\n                })\n            } else if owner.side == DispatchSide::Class && !entered_class_object_root {\n                entered_class_object_root = true;\n                Some(DispatchOwner {\n                    declaration: DeclarationId::new(ModuleId::core(), "Class".into()),\n                    side: DispatchSide::Instance,\n                })\n            } else {\n                None\n            };\n        }\n\n        owners\n    }\n\n    pub fn resolve_dispatch_with_trace(\n        &self,\n        hierarchy: &dyn TypeHierarchy,\n        start_decl: &DeclarationId,\n        side: DispatchSide,\n        selector: &Selector,\n    ) -> ResolvedDispatchResult {\n        let mut visited = Vec::new();\n        for owner in self.dispatch_owners(hierarchy, start_decl, side) {\n            visited.push(owner.declaration.clone());\n            if let Some(surface) = self.surfaces.get(&owner.declaration) {\n                if let Some(sig) = surface.get_callable(owner.side, selector) {\n                    let callable_id = surface\n                        .get_callable_id(owner.side, selector)\n                        .cloned()\n                        .unwrap_or_else(|| CallableId::new(owner.declaration.clone(), selector.clone(), owner.side));\n                    return ResolvedDispatchResult::Found(ResolvedDispatch {\n                        callable: callable_id,\n                        signature: sig.clone(),\n                        visited_owners: visited.into_boxed_slice(),\n                    });\n                }\n            }\n        }\n        ResolvedDispatchResult::Missing {\n            visited_owners: visited.into_boxed_slice(),\n        }\n    }\n\n    pub fn resolve_dispatch_on_owner(''',
)
regex_once(
    "phalcom-semantic/src/dispatch.rs",
    r"    pub fn resolve_callable_id\(.*?\n    }\n}",
    '''    pub fn resolve_callable_id(\n        &self,\n        hierarchy: &dyn TypeHierarchy,\n        start_decl: &DeclarationId,\n        side: DispatchSide,\n        selector: &Selector,\n    ) -> Option<CallableId> {\n        for owner in self.dispatch_owners(hierarchy, start_decl, side) {\n            if let Some(surface) = self.surfaces.get(&owner.declaration) {\n                if let Some(id) = surface.get_callable_id(owner.side, selector) {\n                    return Some(id.clone());\n                }\n            }\n        }\n        None\n    }\n}\n''',
)

# 2. Bootstrap the public Class#new contract as compiler-owned kernel semantics.
replace_once(
    "phalcom-semantic/src/checker/context.rs",
    "use crate::dispatch::{CallableParameter, CallableSignature, DispatchResult, ResolvedDispatchResult, SurfaceDispatchResolver};",
    "use crate::dispatch::{CallableParameter, CallableSemanticKind, CallableSignature, DispatchResult, ResolvedDispatchResult, SurfaceDispatchResolver};",
)
replace_once(
    "phalcom-semantic/src/checker/context.rs",
    "use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};",
    "use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};\nuse crate::types::parameter::{SelfRole, SelfTypeTerm};",
)
regex_once(
    "phalcom-semantic/src/checker/context.rs",
    r"pub\(crate\) fn ensure_core_object_type_tests\(.*?\n}\n\n#\[cfg\(test\)\]",
    '''pub(crate) fn ensure_core_object_type_tests(store: &mut TypeStore, declarations: &DeclarationTypeTable, dispatch: &mut SurfaceDispatchResolver) {\n    let class = DeclarationId::new(ModuleId::core(), "Class".into());\n    let mut class_surface = dispatch\n        .get_surface(&class)\n        .cloned()\n        .unwrap_or_else(|| DeclarationSurface::new(Some(class.clone())));\n    if let Ok(selector) = Selector::method("new", Vec::new())\n        && class_surface.instance.get_callable(&selector).is_none()\n    {\n        let self_type = store.self_type(SelfTypeTerm {\n            owner: class.clone(),\n            side: DispatchSide::Instance,\n            role: SelfRole::InstanceType,\n        });\n        let signature = CallableSignature::new(\n            selector,\n            Vec::new(),\n            TypeKnowledge::established(self_type, EvidenceOrigin::ConstructorSemantics),\n        )\n        .with_kind(CallableSemanticKind::Constructor);\n        class_surface.add_callable(DispatchSide::Instance, signature);\n    }\n    dispatch.register_type(\n        declarations\n            .form(&class)\n            .unwrap_or_else(|| store.nominal_type(class.clone())),\n        class.clone(),\n    );\n    dispatch.register_surface(class, class_surface);\n\n    let object = DeclarationId::new(ModuleId::core(), "Object".into());\n    if let Some(bool_ty) = declarations.form(&DeclarationId::new(ModuleId::core(), "Bool".into())) {\n        let mut surface = dispatch\n            .get_surface(&object)\n            .cloned()\n            .unwrap_or_else(|| DeclarationSurface::new(Some(object.clone())));\n        for method in ["is", "is!"] {\n            let Ok(selector) = Selector::method(method, [phalcom_common::selector::SelectorSlot::Positional]) else {\n                continue;\n            };\n            if surface.instance.get_callable(&selector).is_some() {\n                continue;\n            }\n            let parameter = CallableParameter::new("class", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence));\n            let signature = CallableSignature::new(\n                selector,\n                vec![parameter],\n                TypeKnowledge::established(bool_ty, EvidenceOrigin::DeclarationSemantics),\n            );\n            surface.add_callable(DispatchSide::Instance, signature);\n        }\n        dispatch.register_type(\n            declarations\n                .form(&object)\n                .unwrap_or_else(|| store.nominal_type(object.clone())),\n            object.clone(),\n        );\n        dispatch.register_surface(object, surface);\n    }\n}\n\n#[cfg(test)]''',
)

# 3. Receiver-sensitive formal -> advisory projection.
replace_once(
    "phalcom-semantic/src/advisory/formal.rs",
    "use crate::types::id::TypeId;",
    "use crate::types::id::TypeId;\nuse crate::types::parameter::{SelfRole, SelfTypeTerm};",
)
replace_once(
    "phalcom-semantic/src/advisory/formal.rs",
    '''pub fn advisory_shape_from_formal(store: &TypeStore, knowledge: &TypeKnowledge) -> ValueShape {\n    match knowledge {\n        TypeKnowledge::Known(evidence) => shape_from_type(store, evidence.ty(), DEFAULT_FORMAL_PROJECTION_DEPTH),\n        TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => ValueShape::Unknown,\n    }\n}\n''',
    '''pub fn advisory_shape_from_formal(store: &TypeStore, knowledge: &TypeKnowledge) -> ValueShape {\n    match knowledge {\n        TypeKnowledge::Known(evidence) => shape_from_type(store, evidence.ty(), DEFAULT_FORMAL_PROJECTION_DEPTH),\n        TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => ValueShape::Unknown,\n    }\n}\n\n/// Projects formal knowledge relative to a concrete runtime receiver shape.\n/// This is the advisory counterpart of formal `Self` specialization: the\n/// callable contract remains canonical while only its dependent result is\n/// projected for editor/runtime-shape consumers.\npub fn advisory_shape_from_formal_for_receiver(\n    store: &TypeStore,\n    knowledge: &TypeKnowledge,\n    receiver: &ValueShape,\n) -> ValueShape {\n    match knowledge {\n        TypeKnowledge::Known(evidence) => shape_from_type_for_receiver(store, evidence.ty(), receiver, DEFAULT_FORMAL_PROJECTION_DEPTH),\n        TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => ValueShape::Unknown,\n    }\n}\n\nfn shape_from_type_for_receiver(store: &TypeStore, ty: TypeId, receiver: &ValueShape, depth: usize) -> ValueShape {\n    if depth == 0 {\n        return ValueShape::Unknown;\n    }\n    match store.get(ty) {\n        TypeData::SelfType(term) => shape_from_self(term, receiver, depth - 1),\n        TypeData::Never => ValueShape::Never,\n        TypeData::Unit => ValueShape::Unit,\n        TypeData::ClassObject { declaration } => ValueShape::ClassObject(declaration.clone()),\n        TypeData::Nominal { declaration } => ValueShape::Instance(declaration.clone()),\n        TypeData::Applied { origin, .. } => shape_from_type_for_receiver(store, *origin, receiver, depth - 1),\n        TypeData::Union(types) => ValueShape::bounded_union(\n            types\n                .iter()\n                .map(|ty| shape_from_type_for_receiver(store, *ty, receiver, depth - 1)),\n        ),\n        TypeData::Tuple(elements) => ValueShape::Tuple(\n            elements\n                .iter()\n                .map(|element| shape_from_type_for_receiver(store, element.ty, receiver, depth - 1))\n                .collect::<Vec<_>>()\n                .into(),\n        ),\n        TypeData::Record(_) | TypeData::Callable(_) | TypeData::Parameter(_) | TypeData::Lambda(_) => ValueShape::Unknown,\n    }\n}\n\nfn shape_from_self(term: &SelfTypeTerm, receiver: &ValueShape, depth: usize) -> ValueShape {\n    if depth == 0 {\n        return ValueShape::Unknown;\n    }\n    match term.role {\n        SelfRole::ReceiverValue => receiver.clone(),\n        SelfRole::InstanceType => match receiver {\n            ValueShape::ClassObject(declaration) | ValueShape::Instance(declaration) => ValueShape::Instance(declaration.clone()),\n            ValueShape::Union(shapes) => ValueShape::bounded_union(shapes.iter().map(|shape| shape_from_self(term, shape, depth - 1))),\n            _ => ValueShape::Unknown,\n        },\n    }\n}\n''',
)
replace_once(
    "phalcom-semantic/src/advisory/mod.rs",
    "pub use formal::{advisory_fact_from_formal, advisory_shape_from_formal};",
    "pub use formal::{advisory_fact_from_formal, advisory_shape_from_formal, advisory_shape_from_formal_for_receiver};",
)

# 4. Keep public call identity separate from advisory transfer identity.
replace_once(
    "phalcom-semantic/src/advisory/analyzer.rs",
    "    pub resolve_callable_for_shape: Option<&'a dyn Fn(&ValueShape, &str, &[PackItem]) -> Option<CallableId>>,",
    "    pub resolve_callable_for_shape: Option<&'a dyn Fn(&ValueShape, &str, &[PackItem]) -> Option<CallableId>>,\n    /// Projects a canonical callable's formal result against the concrete receiver.\n    pub resolve_formal_call_result: Option<&'a dyn Fn(&CallableId, Option<&ValueShape>) -> Option<AdvisoryFact>>,\n    /// Maps a public callable identity to the compiler-owned advisory transfer/summary identity.\n    pub advisory_transfer_target: Option<&'a dyn Fn(&CallableId) -> CallableId>,",
)
replace_once(
    "phalcom-semantic/src/advisory/analyzer.rs",
    "    /// Exact call expression range.\n    pub range: SourceRange,",
    "    /// Compiler-owned advisory parameter/summary transfer target.\n    pub transfer_target: CallableId,\n    /// Exact call expression range.\n    pub range: SourceRange,",
)
regex_once(
    "phalcom-semantic/src/advisory/analyzer.rs",
    r"fn resolved_call_or_unknown_with_arguments\(.*?\nfn literal\(",
    '''fn resolved_call_or_unknown_with_arguments(range: SourceRange, arguments: &[AdvisoryCallArgument], context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {\n    let Some(callable) = (context.resolved_callable_for_range)(range) else {\n        return unknown_at(context, range);\n    };\n    resolved_callable_fact(callable, None, range, arguments, context)\n}\n\nfn resolved_call_or_unknown_with_shape(\n    range: SourceRange,\n    receiver: &ValueShape,\n    name: &str,\n    args: &[PackItem],\n    arguments: &[AdvisoryCallArgument],\n    context: &AdvisoryExpressionContext<'_>,\n) -> AdvisoryFact {\n    let callable = if let Some(callable) = (context.resolved_callable_for_range)(range) {\n        callable\n    } else {\n        let Some(resolve) = context.resolve_callable_for_shape else {\n            return unknown_at(context, range);\n        };\n        let Some(callable) = resolve(receiver, name, args) else {\n            return unknown_at(context, range);\n        };\n        callable\n    };\n    resolved_callable_fact(callable, Some(receiver), range, arguments, context)\n}\n\nfn resolved_callable_fact(\n    callable: CallableId,\n    receiver: Option<&ValueShape>,\n    range: SourceRange,\n    arguments: &[AdvisoryCallArgument],\n    context: &AdvisoryExpressionContext<'_>,\n) -> AdvisoryFact {\n    let transfer_target = context\n        .advisory_transfer_target\n        .map(|resolve| resolve(&callable))\n        .unwrap_or_else(|| callable.clone());\n    observe_call(callable.clone(), transfer_target.clone(), range, arguments, context);\n\n    if let Some(resolve) = context.resolve_formal_call_result\n        && let Some(fact) = resolve(&callable, receiver)\n    {\n        return fact.derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(callable));\n    }\n\n    context\n        .callable_returns\n        .get(&callable)\n        .or_else(|| context.callable_returns.get(&transfer_target))\n        .cloned()\n        .map(|fact| fact.derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(callable)))\n        .unwrap_or_else(AdvisoryFact::unknown)\n}\n\nfn observe_call(\n    target: CallableId,\n    transfer_target: CallableId,\n    range: SourceRange,\n    arguments: &[AdvisoryCallArgument],\n    context: &AdvisoryExpressionContext<'_>,\n) {\n    if let Some(observer) = context.call_observer {\n        observer(AdvisoryCallObservation {\n            target,\n            transfer_target,\n            range,\n            arguments: arguments.to_vec(),\n        });\n    }\n}\n\nfn literal(''',
)

# 5. Thread the receiver-sensitive projection through flow.
replace_once(
    "phalcom-semantic/src/advisory/flow.rs",
    "    pub resolve_callable_for_shape: Option<&'a dyn Fn(&ValueShape, &str, &[phalcom_ast::ast::PackItem]) -> Option<CallableId>>,",
    "    pub resolve_callable_for_shape: Option<&'a dyn Fn(&ValueShape, &str, &[phalcom_ast::ast::PackItem]) -> Option<CallableId>>,\n    pub resolve_formal_call_result: Option<&'a dyn Fn(&CallableId, Option<&ValueShape>) -> Option<AdvisoryFact>>,\n    pub advisory_transfer_target: Option<&'a dyn Fn(&CallableId) -> CallableId>,",
)
replace_once(
    "phalcom-semantic/src/advisory/flow.rs",
    "        resolve_callable_for_shape: context.resolve_callable_for_shape,\n        resolve_module_member: context.resolve_module_member,",
    "        resolve_callable_for_shape: context.resolve_callable_for_shape,\n        resolve_formal_call_result: context.resolve_formal_call_result,\n        advisory_transfer_target: context.advisory_transfer_target,\n        resolve_module_member: context.resolve_module_member,",
)
replace_once(
    "phalcom-semantic/src/advisory/flow.rs",
    "            call.target\n                .selector",
    "            call.transfer_target\n                .selector",
)
replace_once(
    "phalcom-semantic/src/advisory/flow.rs",
    "                .target\n                .selector",
    "                .transfer_target\n                .selector",
)
replace_once(
    "phalcom-semantic/src/advisory/flow.rs",
    "        let slot = AdvisoryParameterSlot::new(call.target.clone(), index as u32);",
    "        let slot = AdvisoryParameterSlot::new(call.transfer_target.clone(), index as u32);",
)

# 6. Session adapters consume canonical dispatch and formal result contracts.
replace_once(
    "phalcom-semantic/src/session.rs",
    "    advisory_fact_from_formal, analyze_expr, analyze_statements,",
    "    advisory_fact_from_formal, advisory_shape_from_formal, advisory_shape_from_formal_for_receiver, analyze_expr, analyze_statements,",
)
regex_once(
    "phalcom-semantic/src/session.rs",
    r"    let resolve_callable_for_shape = \|receiver: &crate::advisory::ValueShape, name: &str, args: &\[PackItem\]\| \{.*?\n    };\n    let resolve_method_family",
    '''    let resolve_callable_for_shape = |receiver: &crate::advisory::ValueShape, name: &str, args: &[PackItem]| {\n        let slots = args\n            .iter()\n            .map(|arg| match arg {\n                PackItem::Positional { .. } | PackItem::Expand { .. } => Some(phalcom_common::selector::SelectorSlot::Positional),\n                PackItem::Labeled {\n                    label: PackLabel::Static { text, .. },\n                    ..\n                } => Some(phalcom_common::selector::SelectorSlot::Label(text.clone())),\n                PackItem::Labeled {\n                    label: PackLabel::Computed { .. },\n                    ..\n                } => None,\n            })\n            .collect::<Option<Vec<_>>>()?;\n        let selector = Selector::method(name, slots).ok()?;\n        let (owner, side) = match receiver {\n            crate::advisory::ValueShape::ClassObject(owner) => (owner, DispatchSide::Class),\n            crate::advisory::ValueShape::Instance(owner) => (owner, DispatchSide::Instance),\n            _ => return None,\n        };\n        dispatch.resolve_callable_id(hierarchy, owner, side, &selector)\n    };\n    let resolve_formal_call_result = |callable: &CallableId, receiver: Option<&crate::advisory::ValueShape>| {\n        let signature = dispatch\n            .get_surface(&callable.owner)?\n            .get_callable(callable.side, &callable.selector)?;\n        let shape = receiver.map_or_else(\n            || advisory_shape_from_formal(store, &signature.return_type),\n            |receiver| advisory_shape_from_formal_for_receiver(store, &signature.return_type, receiver),\n        );\n        (!matches!(shape, crate::advisory::ValueShape::Unknown)).then(|| {\n            AdvisoryFact::new(shape, AdvisoryConfidence::Interprocedural)\n                .derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(callable.clone()))\n        })\n    };\n    let advisory_transfer_target = |callable: &CallableId| {\n        let is_constructor = dispatch\n            .get_surface(&callable.owner)\n            .and_then(|surface| surface.get_callable(callable.side, &callable.selector))\n            .is_some_and(|signature| signature.kind == crate::dispatch::CallableSemanticKind::Constructor);\n        if is_constructor && callable.side == DispatchSide::Class {\n            CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Instance)\n        } else {\n            callable.clone()\n        }\n    };\n    let resolve_method_family''',
)
# Method-family lookup uses the same owner traversal.
regex_once(
    "phalcom-semantic/src/session.rs",
    r"        let mut current = Some\(owner\.clone\(\)\);\n        let mut exact = Vec::new\(\);\n        let mut rest_candidates = Vec::new\(\);\n        while let Some\(declaration\) = current \{(.*?)\n            current = hierarchy\.superclass\(&declaration\)\.cloned\(\);\n        \}",
    '''        let mut exact = Vec::new();\n        let mut rest_candidates = Vec::new();\n        for dispatch_owner in dispatch.dispatch_owners(hierarchy, owner, side) {\n            let declaration = dispatch_owner.declaration;\n            let lookup_side = dispatch_owner.side;\1\n        }''',
)
replace_once(
    "phalcom-semantic/src/session.rs",
    "                let members = surface.surface(side);",
    "                let members = surface.surface(lookup_side);",
)
# Pass new callbacks to field analysis and flow contexts.
replace_once(
    "phalcom-semantic/src/session.rs",
    "                Some(&resolve_callable_for_shape),\n                Some(&resolve_module_member),",
    "                Some(&resolve_callable_for_shape),\n                Some(&resolve_formal_call_result),\n                Some(&advisory_transfer_target),\n                Some(&resolve_module_member),",
)
# Two AdvisoryFlowContext constructions.
text = Path("phalcom-semantic/src/session.rs").read_text()
text = text.replace(
    "                    resolve_callable_for_shape: Some(&resolve_callable_for_shape),\n                    resolve_module_member: Some(&resolve_module_member),",
    "                    resolve_callable_for_shape: Some(&resolve_callable_for_shape),\n                    resolve_formal_call_result: Some(&resolve_formal_call_result),\n                    advisory_transfer_target: Some(&advisory_transfer_target),\n                    resolve_module_member: Some(&resolve_module_member),",
)
text = text.replace(
    "                resolve_callable_for_shape: Some(&resolve_callable_for_shape),\n                resolve_module_member: Some(&resolve_module_member),",
    "                resolve_callable_for_shape: Some(&resolve_callable_for_shape),\n                resolve_formal_call_result: Some(&resolve_formal_call_result),\n                advisory_transfer_target: Some(&advisory_transfer_target),\n                resolve_module_member: Some(&resolve_module_member),",
)
Path("phalcom-semantic/src/session.rs").write_text(text)
# Extend field helper signature and expression context.
replace_once(
    "phalcom-semantic/src/session.rs",
    "    resolve_callable_for_shape: Option<&dyn Fn(&crate::advisory::ValueShape, &str, &[PackItem]) -> Option<CallableId>>,\n    resolve_module_member:",
    "    resolve_callable_for_shape: Option<&dyn Fn(&crate::advisory::ValueShape, &str, &[PackItem]) -> Option<CallableId>>,\n    resolve_formal_call_result: Option<&dyn Fn(&CallableId, Option<&crate::advisory::ValueShape>) -> Option<AdvisoryFact>>,\n    advisory_transfer_target: Option<&dyn Fn(&CallableId) -> CallableId>,\n    resolve_module_member:",
)
replace_once(
    "phalcom-semantic/src/session.rs",
    "                    resolve_callable_for_shape,\n                    resolve_module_member,",
    "                    resolve_callable_for_shape,\n                    resolve_formal_call_result,\n                    advisory_transfer_target,\n                    resolve_module_member,",
)
# Remove the selector-spelling advisory return override.
regex_once(
    "phalcom-semantic/src/session.rs",
    r"fn advisory_return_fact\(store: &TypeStore, analysis: &crate::checker::CallableAnalysis\) -> AdvisoryFact \{\n    if analysis\.callable\.selector\.kind.*?\n    \}\n    if analysis\.exits\.normal_return_values\.is_empty\(\) \{",
    "fn advisory_return_fact(store: &TypeStore, analysis: &crate::checker::CallableAnalysis) -> AdvisoryFact {\n    if analysis.exits.normal_return_values.is_empty() {",
)

# 7. Editor enumeration consumes the same dispatch owner chain.
regex_once(
    "phalcom-semantic/src/editor.rs",
    r"        for alternative in receiver\.alternatives\.iter\(\) \{\n            let mut declaration = Some\(alternative\.declaration\.clone\(\)\);\n            let mut visited = BTreeSet::new\(\);\n            while let Some\(current\) = declaration \{.*?\n                declaration = self\.snapshot\.hierarchy\.superclass\(&current\)\.cloned\(\);\n            \}\n        \}",
    '''        for alternative in receiver.alternatives.iter() {\n            let side = match alternative.mode {\n                ReceiverMode::Instance => crate::identity::DispatchSide::Instance,\n                ReceiverMode::Class => crate::identity::DispatchSide::Class,\n            };\n            for dispatch_owner in self\n                .snapshot\n                .dispatch\n                .dispatch_owners(self.snapshot.hierarchy.as_ref(), &alternative.declaration, side)\n            {\n                let current = dispatch_owner.declaration;\n                if let Some(surface) = self.snapshot.surfaces.get(&current) {\n                    let member_surface = surface.surface(dispatch_owner.side);\n                    for (selector, callable) in &member_surface.callables_by_selector {\n                        let visibility = member_surface.callable_visibility.get(selector).copied().unwrap_or_default();\n                        if is_visible(self.snapshot.hierarchy.as_ref(), &current, visibility, access) {\n                            members.push(EditorMember {\n                                target: EditorMemberTarget::Callable(callable.clone()),\n                                owner: current.clone(),\n                                visibility,\n                            });\n                        }\n                    }\n                    for (name, field) in &member_surface.fields_by_name {\n                        let visibility = member_surface.field_visibility.get(name).copied().unwrap_or_default();\n                        if is_visible(self.snapshot.hierarchy.as_ref(), &current, visibility, access) {\n                            members.push(EditorMember {\n                                target: EditorMemberTarget::Field(field.clone()),\n                                owner: current.clone(),\n                                visibility,\n                            });\n                        }\n                    }\n                }\n            }\n        }''',
)
# BTreeSet remains used elsewhere in editor.rs for visible symbols.

print("semantic unification candidate applied")
