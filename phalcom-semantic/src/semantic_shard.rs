//! Retained source-local semantic structure for incremental workspace updates.

use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, FieldId, ModuleId};
use crate::source::ParsedModuleUnit;
use phalcom_ast::ast::{ClassMember, EnumBehaviorMember, EnumMember, MemberBody, Statement, TypeAliasDef};
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind};
use std::collections::{BTreeMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Source-local declaration structure retained between semantic snapshots.
///
/// This shard contains syntax that can be extracted without solving cross-module
/// meaning. TypeStore forms, resolved aliases, hierarchy edges, and callable
/// products remain owned by the shared semantic workspace and its DB queries.
#[derive(Clone, Debug)]
pub struct ModuleSemanticStructureShard {
    /// Canonical module identity owned by this shard.
    pub module: ModuleId,
    /// Parsed source retained for query inputs and source provenance.
    pub source: Arc<ParsedModuleUnit>,
    /// Stable declaration blueprints needed for predeclaration.
    pub declarations: Arc<[DeclarationBlueprint]>,
    /// Alias identities declared by this module.
    pub aliases: Arc<[DeclarationId]>,
    /// Header input fingerprints keyed by declaration identity.
    pub declaration_header_fingerprints: Arc<BTreeMap<DeclarationId, u64>>,
    /// Direct superclass input fingerprints keyed by declaration identity.
    pub hierarchy_edge_fingerprints: Arc<BTreeMap<DeclarationId, u64>>,
    /// Callable declaration/signature input fingerprints keyed by callable identity.
    pub callable_signature_fingerprints: Arc<BTreeMap<CallableId, u64>>,
    /// Field declaration/signature input fingerprints keyed by field identity.
    pub field_signature_fingerprints: Arc<BTreeMap<FieldId, u64>>,
    /// Callable body input fingerprints keyed by callable identity.
    pub callable_body_fingerprints: Arc<BTreeMap<CallableId, u64>>,
    /// Retained alias source descriptors, avoiding a workspace-wide AST scan
    /// when only a different module changes.
    pub alias_sources: Arc<BTreeMap<DeclarationId, TypeAliasDef>>,
    /// Source identity used to decide whether this shard can be retained.
    pub source_fingerprint: u64,
}

impl ModuleSemanticStructureShard {
    /// Extracts source-local declaration structure once for one parsed module.
    pub fn from_source(source: Arc<ParsedModuleUnit>) -> Arc<Self> {
        let mut declarations = Vec::new();
        let mut aliases = Vec::new();
        let mut declaration_header_fingerprints = BTreeMap::new();
        let mut hierarchy_edge_fingerprints = BTreeMap::new();
        let mut callable_signature_fingerprints = BTreeMap::new();
        let mut field_signature_fingerprints = BTreeMap::new();
        let mut callable_body_fingerprints = BTreeMap::new();
        let mut alias_sources = BTreeMap::new();
        for statement in &source.program.statements {
            let (name, kind) = match statement {
                Statement::Class(class_def) => (&class_def.name, DeclarationKind::Class),
                Statement::Enum(enum_def) => (&enum_def.name, DeclarationKind::Class),
                Statement::TypeAlias(alias) => (&alias.name, DeclarationKind::Alias),
                _ => continue,
            };
            let declaration = DeclarationId::new(source.id.clone(), name.clone().into());
            declarations.push(DeclarationBlueprint { id: declaration.clone(), kind });
            if kind == DeclarationKind::Alias {
                aliases.push(declaration.clone());
            }
            match statement {
                Statement::Class(class_def) => {
                    declaration_header_fingerprints.insert(declaration.clone(), declaration_header_fingerprint(&source, class_def.range));
                    hierarchy_edge_fingerprints.insert(
                        declaration.clone(),
                        hierarchy_fingerprint(&source, class_def.superclass.as_ref().map(|superclass| superclass.range)),
                    );
                    collect_class_member_fingerprints(
                        &source,
                        &declaration,
                        &class_def.members,
                        &mut callable_signature_fingerprints,
                        &mut field_signature_fingerprints,
                        &mut callable_body_fingerprints,
                    );
                }
                Statement::Enum(enum_def) => {
                    declaration_header_fingerprints.insert(declaration.clone(), declaration_header_fingerprint(&source, enum_def.range));
                    hierarchy_edge_fingerprints.insert(declaration.clone(), hierarchy_fingerprint(&source, None));
                    collect_enum_member_fingerprints(
                        &source,
                        &declaration,
                        &enum_def.members,
                        &mut callable_signature_fingerprints,
                        &mut callable_body_fingerprints,
                    );
                }
                Statement::TypeAlias(alias) => {
                    alias_sources.insert(declaration, alias.clone());
                }
                _ => {}
            }
        }

        let source_fingerprint = Self::structural_fingerprint(&source);
        Arc::new(Self {
            module: source.id.clone(),
            source: source.clone(),
            declarations: Arc::from(declarations.into_boxed_slice()),
            aliases: Arc::from(aliases.into_boxed_slice()),
            declaration_header_fingerprints: Arc::new(declaration_header_fingerprints),
            hierarchy_edge_fingerprints: Arc::new(hierarchy_edge_fingerprints),
            callable_signature_fingerprints: Arc::new(callable_signature_fingerprints),
            field_signature_fingerprints: Arc::new(field_signature_fingerprints),
            callable_body_fingerprints: Arc::new(callable_body_fingerprints),
            alias_sources: Arc::new(alias_sources),
            source_fingerprint,
        })
    }

    /// Computes source identity after removing callable body text.
    ///
    /// Declaration names, signatures, fields, aliases, and hierarchy syntax
    /// remain in the fingerprint. Callable implementation edits do not.
    pub fn structural_fingerprint(source: &ParsedModuleUnit) -> u64 {
        let mut body_ranges = Vec::new();
        for statement in &source.program.statements {
            match statement {
                Statement::Class(class_def) => {
                    for member in &class_def.members {
                        if let Some(range) = class_member_body_range(member) {
                            body_ranges.push(range);
                        }
                    }
                }
                Statement::Enum(enum_def) => {
                    for member in &enum_def.members {
                        if let EnumMember::Behavior(behavior) = member {
                            if let Some(range) = enum_behavior_body_range(behavior) {
                                body_ranges.push(range);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        body_ranges.sort_by_key(|range| std::cmp::Reverse(range.start));
        let mut text = source.text.to_string();
        for range in body_ranges {
            let Some((open, close)) = body_braces(&text, range.start, range.end) else {
                continue;
            };
            text.replace_range(open + 1..close, "");
        }

        let mut hasher = DefaultHasher::new();
        source.id.hash(&mut hasher);
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// Retains source-local declaration structure while replacing the parsed
    /// source used by body and provenance queries. Module interfaces decide
    /// whether this contribution is structurally unchanged; body-only edits
    /// must not force declaration-shard reconstruction.
    pub fn with_source(previous: &Arc<Self>, source: Arc<ParsedModuleUnit>) -> Arc<Self> {
        let source_fingerprint = Self::structural_fingerprint(&source);
        Arc::new(Self {
            module: previous.module.clone(),
            source: source.clone(),
            declarations: previous.declarations.clone(),
            aliases: previous.aliases.clone(),
            declaration_header_fingerprints: previous.declaration_header_fingerprints.clone(),
            hierarchy_edge_fingerprints: previous.hierarchy_edge_fingerprints.clone(),
            callable_signature_fingerprints: previous.callable_signature_fingerprints.clone(),
            field_signature_fingerprints: previous.field_signature_fingerprints.clone(),
            callable_body_fingerprints: Arc::new(callable_body_fingerprints_for_source(&source)),
            alias_sources: previous.alias_sources.clone(),
            source_fingerprint,
        })
    }
}

fn collect_class_member_fingerprints(
    source: &ParsedModuleUnit,
    owner: &DeclarationId,
    members: &[ClassMember],
    callable_signatures: &mut BTreeMap<CallableId, u64>,
    fields: &mut BTreeMap<FieldId, u64>,
    callable_bodies: &mut BTreeMap<CallableId, u64>,
) {
    for member in members {
        if let Some(callable) = crate::checker::declaration_signature::callable_id_for_member(owner, member) {
            callable_signatures.insert(callable.clone(), member_signature_fingerprint(source, member.range()));
            if class_member_body_range(member).is_some() {
                callable_bodies.insert(callable, hash_range(source, member.range()));
            }
        }
        if let Some(field) = crate::checker::declaration_signature::field_id_for_member(owner, member) {
            fields.insert(field, hash_range(source, member.range()));
        }
    }
}

fn collect_enum_member_fingerprints(
    source: &ParsedModuleUnit,
    owner: &DeclarationId,
    members: &[EnumMember],
    callable_signatures: &mut BTreeMap<CallableId, u64>,
    callable_bodies: &mut BTreeMap<CallableId, u64>,
) {
    for member in members {
        let EnumMember::Behavior(behavior) = member else {
            continue;
        };
        let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(behavior);
        let side = enum_behavior_side(behavior);
        let Some(callable) = crate::checker::declaration_signature::callable_id_for_syntax(&CallableOwnerId::Declaration(owner.clone()), syntax, side) else {
            continue;
        };
        callable_signatures.insert(callable.clone(), enum_behavior_signature_fingerprint(source, behavior));
        if enum_behavior_body_range(behavior).is_some() {
            callable_bodies.insert(callable, hash_range(source, behavior.range()));
        }
    }
}

fn callable_body_fingerprints_for_source(source: &ParsedModuleUnit) -> BTreeMap<CallableId, u64> {
    let mut callable_signatures = BTreeMap::new();
    let mut fields = BTreeMap::new();
    let mut callable_bodies = BTreeMap::new();
    for statement in &source.program.statements {
        match statement {
            Statement::Class(class_def) => {
                let owner = DeclarationId::new(source.id.clone(), class_def.name.clone().into());
                collect_class_member_fingerprints(source, &owner, &class_def.members, &mut callable_signatures, &mut fields, &mut callable_bodies);
            }
            Statement::Enum(enum_def) => {
                let owner = DeclarationId::new(source.id.clone(), enum_def.name.clone().into());
                collect_enum_member_fingerprints(source, &owner, &enum_def.members, &mut callable_signatures, &mut callable_bodies);
            }
            _ => {}
        }
    }
    callable_bodies
}

fn declaration_header_fingerprint(source: &ParsedModuleUnit, range: phalcom_common::range::SourceRange) -> u64 {
    let mut body_ranges = Vec::new();
    for statement in &source.program.statements {
        let statement_range = match statement {
            Statement::Class(class_def) if class_def.range == range => {
                body_ranges.extend(class_def.members.iter().filter_map(class_member_body_range));
                true
            }
            Statement::Enum(enum_def) if enum_def.range == range => {
                body_ranges.extend(enum_def.members.iter().filter_map(|member| match member {
                    EnumMember::Behavior(behavior) => enum_behavior_body_range(behavior),
                    EnumMember::Variant(_) => None,
                }));
                true
            }
            _ => false,
        };
        if statement_range {
            break;
        }
    }
    hash_range_without_bodies(source, range, body_ranges)
}

fn hierarchy_fingerprint(source: &ParsedModuleUnit, range: Option<phalcom_common::range::SourceRange>) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.id.hash(&mut hasher);
    range.and_then(|range| source.text.get(range.start..range.end)).hash(&mut hasher);
    hasher.finish()
}

fn member_signature_fingerprint(source: &ParsedModuleUnit, range: phalcom_common::range::SourceRange) -> u64 {
    hash_range_without_bodies(source, range, vec![range])
}

fn enum_behavior_signature_fingerprint(source: &ParsedModuleUnit, behavior: &EnumBehaviorMember) -> u64 {
    let range = behavior.range();
    hash_range_without_bodies(source, range, vec![range])
}

fn hash_range(source: &ParsedModuleUnit, range: phalcom_common::range::SourceRange) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.id.hash(&mut hasher);
    source.text.get(range.start..range.end).hash(&mut hasher);
    hasher.finish()
}

fn hash_range_without_bodies(
    source: &ParsedModuleUnit,
    range: phalcom_common::range::SourceRange,
    mut body_ranges: Vec<phalcom_common::range::SourceRange>,
) -> u64 {
    let mut text = source.text.to_string();
    body_ranges.sort_by_key(|body| std::cmp::Reverse(body.start));
    for body in body_ranges {
        let Some((open, close)) = body_braces(&text, body.start, body.end) else {
            continue;
        };
        text.replace_range(open + 1..close, "");
    }
    let mut hasher = DefaultHasher::new();
    source.id.hash(&mut hasher);
    text.get(range.start..range.end).hash(&mut hasher);
    hasher.finish()
}

fn enum_behavior_side(member: &EnumBehaviorMember) -> DispatchSide {
    match member {
        EnumBehaviorMember::Method(method) if method.is_static || method.attributes.iter().any(|attribute| attribute.name == "class") => DispatchSide::Class,
        EnumBehaviorMember::Getter(getter) if getter.is_static || getter.attributes.iter().any(|attribute| attribute.name == "class") => DispatchSide::Class,
        EnumBehaviorMember::Setter(setter) if setter.is_static || setter.attributes.iter().any(|attribute| attribute.name == "class") => DispatchSide::Class,
        _ => DispatchSide::Instance,
    }
}

fn class_member_body_range(member: &ClassMember) -> Option<phalcom_common::range::SourceRange> {
    match member {
        ClassMember::Method(method) if matches!(method.body, MemberBody::Block(_)) => Some(method.range),
        ClassMember::Getter(getter) if matches!(getter.body, MemberBody::Block(_)) => Some(getter.range),
        ClassMember::Setter(setter) if matches!(setter.body, MemberBody::Block(_)) => Some(setter.range),
        ClassMember::Index(index) => Some(index.range),
        _ => None,
    }
}

fn enum_behavior_body_range(member: &EnumBehaviorMember) -> Option<phalcom_common::range::SourceRange> {
    match member {
        EnumBehaviorMember::Method(method) if matches!(method.body, MemberBody::Block(_)) => Some(method.range),
        EnumBehaviorMember::Getter(getter) if matches!(getter.body, MemberBody::Block(_)) => Some(getter.range),
        EnumBehaviorMember::Setter(setter) if matches!(setter.body, MemberBody::Block(_)) => Some(setter.range),
        EnumBehaviorMember::Index(index) => Some(index.range),
        _ => None,
    }
}

fn body_braces(text: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let end = end.min(text.len());
    let bytes = text.as_bytes();
    for open in start..end {
        if bytes.get(open) != Some(&b'{') {
            continue;
        }
        let Some(close) = matching_brace(text, open, end) else {
            continue;
        };
        if text[close + 1..end].trim().is_empty() {
            return Some((open, close));
        }
    }
    None
}

fn matching_brace(text: &str, open: usize, end: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for index in open..end {
        let byte = *bytes.get(index)?;
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            continue;
        }
        if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
            continue;
        }
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}
