from pathlib import Path
import re


def replace_once(path: str, old: str, new: str, label: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label} anchor count: {count}")
    p.write_text(text.replace(old, new, 1))


# The resolver must be pure with respect to expression/call status. A union
# receiver resolves its arms before the call-causal frame exists, so terminal
# status publication belongs in call consumption, not dispatch discovery.
context = Path("phalcom-semantic/src/checker/context.rs")
text = context.read_text()
for line in [
    "                            self.record_call_status(AnalysisStatus::Blocked(reason.clone()));\n",
    "                            self.record_call_status(AnalysisStatus::DynamicBoundary(crate::types::evidence::DynamicReason::RuntimeReflection));\n",
    "                            self.record_call_status(AnalysisStatus::Cancelled);\n",
    "                            self.record_call_status(AnalysisStatus::BudgetExceeded(report.clone()));\n",
]:
    if text.count(line) != 1:
        raise SystemExit(f"context side-effect line count for {line.strip()!r}: {text.count(line)}")
    text = text.replace(line, "", 1)
old_internal = '''                        crate::trait_dispatch::TraitDispatchResolution::InternalFailure(message) => {
                            let incident = self.record_internal_incident(
                                InternalSemanticIncidentKind::DatabaseInvariantViolation,
                                InternalSemanticIncidentDetails::Message { message: message.clone() },
                                None,
                            );
                            self.record_call_status(AnalysisStatus::InternalFailure(incident));
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message));
                        }
'''
new_internal = '''                        crate::trait_dispatch::TraitDispatchResolution::InternalFailure(message) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message));
                        }
'''
if text.count(old_internal) != 1:
    raise SystemExit(f"context internal terminal block count: {text.count(old_internal)}")
context.write_text(text.replace(old_internal, new_internal, 1))

# Canonical unresolved-call handling for trait proof terminals.
call = Path("phalcom-semantic/src/checker/call.rs")
text = call.read_text()
old_union = '''    Dynamic {
        receiver: TypeId,
        reason: DynamicReason,
    },
'''
new_union = '''    Dynamic {
        receiver: TypeId,
        reason: DynamicReason,
    },
    TraitTerminal {
        receiver: TypeId,
        terminal: crate::trait_dispatch::TraitDispatchTerminal,
    },
'''
if text.count(old_union) != 1:
    raise SystemExit(f"union terminal anchor count: {text.count(old_union)}")
text = text.replace(old_union, new_union, 1)

enum_anchor = "pub(crate) enum UnresolvedApplicationReason {\n"
if text.count(enum_anchor) != 1:
    raise SystemExit(f"unresolved reason enum count: {text.count(enum_anchor)}")
text = text.replace(
    enum_anchor,
    enum_anchor + "    TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal),\n",
    1,
)

knowledge_anchor = '''        UnresolvedApplicationReason::IterationArgumentUnavailable => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
    };
'''
knowledge_replacement = '''        UnresolvedApplicationReason::IterationArgumentUnavailable => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
        UnresolvedApplicationReason::TraitTerminal(terminal) => match terminal {
            crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
            crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection),
            crate::trait_dispatch::TraitDispatchTerminal::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
            crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
            crate::trait_dispatch::TraitDispatchTerminal::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
            crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
            | crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(_) => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
        },
    };
'''
if text.count(knowledge_anchor) != 1:
    raise SystemExit(f"terminal knowledge anchor count: {text.count(knowledge_anchor)}")
text = text.replace(knowledge_anchor, knowledge_replacement, 1)

status_anchor = '''    let status = argument_status.unwrap_or_else(|| match &reason {
'''
status_replacement = '''    let trait_terminal_status = match &reason {
        UnresolvedApplicationReason::TraitTerminal(terminal) => match terminal {
            crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason) => Some(AnalysisStatus::Blocked(reason.clone())),
            crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => Some(AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection)),
            crate::trait_dispatch::TraitDispatchTerminal::Cancelled => Some(AnalysisStatus::Cancelled),
            crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report) => Some(AnalysisStatus::BudgetExceeded(report.clone())),
            crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message) => {
                let incident = ctx.record_internal_incident(
                    InternalSemanticIncidentKind::DatabaseInvariantViolation,
                    InternalSemanticIncidentDetails::Message { message: message.clone() },
                    None,
                );
                Some(AnalysisStatus::InternalFailure(incident))
            }
            crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
            | crate::trait_dispatch::TraitDispatchTerminal::Unknown(_) => None,
        },
        _ => None,
    };
    let status = argument_status.or(trait_terminal_status).unwrap_or_else(|| match &reason {
'''
if text.count(status_anchor) != 1:
    raise SystemExit(f"terminal status anchor count: {text.count(status_anchor)}")
text = text.replace(status_anchor, status_replacement, 1)

# Union application consumes a terminal arm inside the call-causal frame.
union_dynamic = '''            UnionCallArm::Dynamic { receiver, reason } => {
                let arm_explanation = record_union_arm_explanation(
                    ctx,
                    *receiver,
                    None,
                    crate::explain::UnionArmOutcome::Dynamic { reason: reason.clone() },
                    Vec::new(),
                );
                explanation_parents.push(arm_explanation);
                ctx.record_call_dependency(CausalInvalidity::Clean, Some(arm_explanation));
                meet_union_status(&mut status, AnalysisStatus::DynamicBoundary(reason.clone()));
                publication_inputs.push(TypeKnowledge::Dynamic(reason.clone()));
                all_found = false;
            }
'''
union_terminal = union_dynamic + '''            UnionCallArm::TraitTerminal { receiver: _, terminal } => {
                let knowledge = match terminal {
                    crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
                    crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection),
                    crate::trait_dispatch::TraitDispatchTerminal::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
                    crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
                    crate::trait_dispatch::TraitDispatchTerminal::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
                    crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
                    | crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(_) => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
                };
                let arm_status = match terminal {
                    crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason) => Some(AnalysisStatus::Blocked(reason.clone())),
                    crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => Some(AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection)),
                    crate::trait_dispatch::TraitDispatchTerminal::Cancelled => Some(AnalysisStatus::Cancelled),
                    crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report) => Some(AnalysisStatus::BudgetExceeded(report.clone())),
                    crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message) => {
                        let incident = ctx.record_internal_incident(
                            InternalSemanticIncidentKind::DatabaseInvariantViolation,
                            InternalSemanticIncidentDetails::Message { message: message.clone() },
                            Some(call_range),
                        );
                        Some(AnalysisStatus::InternalFailure(incident))
                    }
                    crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
                    | crate::trait_dispatch::TraitDispatchTerminal::Unknown(_) => None,
                };
                if let Some(arm_status) = arm_status {
                    meet_union_status(&mut status, arm_status);
                }
                publication_inputs.push(knowledge);
                all_found = false;
            }
'''
if text.count(union_dynamic) != 1:
    raise SystemExit(f"union dynamic application anchor count: {text.count(union_dynamic)}")
text = text.replace(union_dynamic, union_terminal, 1)
call.write_text(text)

# Ordinary expression consumers use the same arguments/premise as their
# dynamic sibling, but retain the proof terminal as the unresolved reason.
expr = Path("phalcom-semantic/src/checker/expression.rs")
text = expr.read_text()

# Union member dispatch retains terminal identity in the arm product.
union_expr = '''                    ResolvedDispatchResult::Dynamic => UnionCallArm::Dynamic {
                        receiver,
                        reason: DynamicReason::RuntimeReflection,
                    },
'''
union_expr_new = '''                    ResolvedDispatchResult::TraitTerminal(terminal) => UnionCallArm::TraitTerminal { receiver, terminal },
''' + union_expr
if text.count(union_expr) != 1:
    raise SystemExit(f"union expression dynamic anchor count: {text.count(union_expr)}")
text = text.replace(union_expr, union_expr_new, 1)

# Duplicate every remaining dynamic arm whose body is an unresolved canonical
# application. The copy keeps the exact control/assignment shape and only
# substitutes the unresolved reason.
pattern = re.compile(
    r"(?P<indent>^[ \t]*)ResolvedDispatchResult::Dynamic =>(?P<body>.*?)(?=\n(?P=indent)(?:ResolvedDispatchResult::|}))",
    re.MULTILINE | re.DOTALL,
)
parts = []
pos = 0
converted = 0
for match in pattern.finditer(text):
    body = match.group("body")
    if "UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection)" not in body:
        continue
    indent = match.group("indent")
    terminal_body = body.replace(
        "UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection)",
        "UnresolvedApplicationReason::TraitTerminal(terminal)",
    )
    parts.append(text[pos:match.start()])
    parts.append(f"{indent}ResolvedDispatchResult::TraitTerminal(terminal) =>{terminal_body}\n")
    parts.append(match.group(0))
    pos = match.end()
    converted += 1
parts.append(text[pos:])
if converted < 9:
    raise SystemExit(f"expected at least 9 unresolved dynamic expression arms, converted {converted}")
text = "".join(parts)

# Reflected-binary target selection only asks whether a concrete direct target
# is usable. A terminal proof is not usable and must not trigger reflected
# precedence as though dispatch were missing.
old_reflected = '''        Some(ResolvedDispatchResult::Ambiguous(_) | ResolvedDispatchResult::Dynamic) => false,
'''
new_reflected = '''        Some(ResolvedDispatchResult::Ambiguous(_) | ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic) => false,
'''
if text.count(old_reflected) != 1:
    raise SystemExit(f"reflected terminal anchor count: {text.count(old_reflected)}")
text = text.replace(old_reflected, new_reflected, 1)
expr.write_text(text)

# Iteration protocol probes are canonical applications too. Preserve terminal
# evidence instead of falling through to a later protocol shape or runtime
# dynamic dispatch.
statement = Path("phalcom-semantic/src/checker/statement.rs")
text = statement.read_text()
for dynamic_block in [
    '''            crate::dispatch::ResolvedDispatchResult::Dynamic => {
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection));
            }
''',
]:
    count = text.count(dynamic_block)
    if count != 3:
        raise SystemExit(f"iteration dynamic block count: {count}")
    terminal_block = '''            crate::dispatch::ResolvedDispatchResult::TraitTerminal(terminal) => {
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::TraitTerminal(terminal));
            }
'''
    text = text.replace(dynamic_block, terminal_block + dynamic_block)
statement.write_text(text)
