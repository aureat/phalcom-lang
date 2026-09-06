//! Retained source-local semantic structure for incremental workspace updates.

use crate::identity::{DeclarationId, ModuleId};
use crate::source::ParsedModuleUnit;
use phalcom_ast::ast::{ClassMember, EnumBehaviorMember, EnumMember, MemberBody, Statement};
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind};
use std::collections::hash_map::DefaultHasher;
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
    /// Source identity used to decide whether this shard can be retained.
    pub source_fingerprint: u64,
}

impl ModuleSemanticStructureShard {
    /// Extracts source-local declaration structure once for one parsed module.
    pub fn from_source(source: Arc<ParsedModuleUnit>) -> Arc<Self> {
        let mut declarations = Vec::new();
        let mut aliases = Vec::new();
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
                aliases.push(declaration);
            }
        }

        let source_fingerprint = Self::structural_fingerprint(&source);
        Arc::new(Self {
            module: source.id.clone(),
            source,
            declarations: Arc::from(declarations.into_boxed_slice()),
            aliases: Arc::from(aliases.into_boxed_slice()),
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
            source,
            declarations: previous.declarations.clone(),
            aliases: previous.aliases.clone(),
            source_fingerprint,
        })
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
