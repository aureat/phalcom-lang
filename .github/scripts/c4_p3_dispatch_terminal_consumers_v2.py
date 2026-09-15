from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label} anchor count: {count}")
    p.write_text(text.replace(old, new, 1))


# Resolver is pure: terminal status belongs to the consuming expression/call.
context = Path("phalcom-semantic/src/checker/context.rs")
text = context.read_text()
for line in [
    "                            self.record_call_status(AnalysisStatus::Blocked(reason.clone()));\n",
    "                            self.record_call_status(AnalysisStatus::DynamicBoundary(crate::types::evidence::DynamicReason::RuntimeReflection));\n",
    "                            self.record_call_status(AnalysisStatus::Cancelled);\n",
    "                            self.record_call_status(AnalysisStatus::BudgetExceeded(report.clone()));\n",
]:
    if text.count(line) != 1:
        raise SystemExit(f"context side-effect line count: {text.count(line)}")
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

# Canonical unresolved-call handling for trait terminals.
call = Path("phalcom-semantic/src/checker/call.rs")
text = call.read_text()
old_union = '''    Dynamic {
        receiver: TypeId,
        reason: DynamicReason,
    },
'''
new_union = old_union + '''    TraitTerminal {
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
text = text.replace(enum_anchor, enum_anchor + "    TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal),\n", 1)

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

status_anchor = "    let status = argument_status.unwrap_or_else(|| match &reason {\n"
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
call.write_text(text.replace(union_dynamic, union_terminal, 1))

# Expression consumers. Use a delimiter-aware arm copier rather than regex over
# nested Rust blocks/closures.
expr = Path("phalcom-semantic/src/checker/expression.rs")
text = expr.read_text()
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

marker = "ResolvedDispatchResult::Dynamic =>"
insertions = []
pos = 0
converted = 0
while True:
    start = text.find(marker, pos)
    if start < 0:
        break
    # Find line indentation.
    line_start = text.rfind("\n", 0, start) + 1
    indent = text[line_start:start]
    # Scan the arm body to its top-level comma, respecting strings/comments and
    # nested (), [], {} delimiters. Block arms without a comma end before the
    # next same-indent arm/closing brace.
    i = start + len(marker)
    stack = []
    in_string = False
    in_char = False
    escape = False
    line_comment = False
    block_comment = 0
    end = None
    while i < len(text):
        c = text[i]
        n = text[i + 1] if i + 1 < len(text) else ""
        if line_comment:
            if c == "\n":
                line_comment = False
            i += 1
            continue
        if block_comment:
            if c == "/" and n == "*":
                block_comment += 1; i += 2; continue
            if c == "*" and n == "/":
                block_comment -= 1; i += 2; continue
            i += 1; continue
        if in_string:
            if escape:
                escape = False
            elif c == "\\":
                escape = True
            elif c == '"':
                in_string = False
            i += 1; continue
        if in_char:
            if escape:
                escape = False
            elif c == "\\":
                escape = True
            elif c == "'":
                in_char = False
            i += 1; continue
        if c == "/" and n == "/":
            line_comment = True; i += 2; continue
        if c == "/" and n == "*":
            block_comment = 1; i += 2; continue
        if c == '"':
            in_string = True; i += 1; continue
        if c == "'" and n and (n.isalpha() or n == "_"):
            # Lifetime, not a char literal.
            i += 1; continue
        if c == "'":
            in_char = True; i += 1; continue
        if c in "([{":
            stack.append(c)
        elif c in ")]}":
            if stack:
                stack.pop()
            elif c == "}":
                end = i
                break
        elif c == "," and not stack:
            end = i + 1
            break
        if c == "\n" and not stack:
            next_line = i + 1
            if text.startswith(indent + "ResolvedDispatchResult::", next_line) or text.startswith(indent + "}", next_line):
                end = i
                break
        i += 1
    if end is None:
        raise SystemExit(f"could not delimit dynamic arm at byte {start}")
    arm = text[start:end]
    if "UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection)" in arm:
        terminal_arm = arm.replace(
            marker,
            "ResolvedDispatchResult::TraitTerminal(terminal) =>",
            1,
        ).replace(
            "UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection)",
            "UnresolvedApplicationReason::TraitTerminal(terminal)",
        )
        insertions.append((start, terminal_arm + "\n" + indent))
        converted += 1
    pos = end

if converted < 9:
    raise SystemExit(f"expected at least 9 unresolved dynamic expression arms, converted {converted}")
for start, insertion in reversed(insertions):
    text = text[:start] + insertion + text[start:]

old_reflected = '''        Some(ResolvedDispatchResult::Ambiguous(_) | ResolvedDispatchResult::Dynamic) => false,
'''
new_reflected = '''        Some(ResolvedDispatchResult::Ambiguous(_) | ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic) => false,
'''
if text.count(old_reflected) != 1:
    raise SystemExit(f"reflected terminal anchor count: {text.count(old_reflected)}")
expr.write_text(text.replace(old_reflected, new_reflected, 1))

# Iteration protocol probes consume the terminal through canonical unresolved
# application, preserving argument/status behavior.
statement = Path("phalcom-semantic/src/checker/statement.rs")
text = statement.read_text()
dynamic_block = '''            crate::dispatch::ResolvedDispatchResult::Dynamic => {
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection));
            }
'''
if text.count(dynamic_block) != 3:
    raise SystemExit(f"iteration dynamic block count: {text.count(dynamic_block)}")
terminal_block = '''            crate::dispatch::ResolvedDispatchResult::TraitTerminal(terminal) => {
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::TraitTerminal(terminal));
            }
'''
statement.write_text(text.replace(dynamic_block, terminal_block + dynamic_block))
