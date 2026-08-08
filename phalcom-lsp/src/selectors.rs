//! ADR-0012 comma-form selector reconstruction from AST nodes.
//!
//! The index (`crate::index`), go-to-definition, find-references, and
//! `workspace/symbol` all key on the **comma-form** selector — `move`,
//! `move()`, `move(_,to,duration)` — never a bare method name (ADR-0012, the
//! gate U-VSPHALCOM's `gen-core-table` already enforces on
//! `core-table.json`). This module is the `phalcom-lsp`-side port of that
//! same algorithm, applied to `phalcom_ast::ast` nodes instead of harvested
//! `syn` items.
//!
//! **Do not confuse with `phalcom-core/src/method/mod.rs::encode_selector`.**
//! That function builds the VM-internal runtime dispatch symbol and this
//! crate never links `phalcom-core` (ADR-0056 §2) — this module is the sole
//! source of selector spelling here, deliberately independent.

use phalcom_ast::ast::{ClassMember, FieldDef, GetterDef, IndexAccessor, IndexMethodDef, MethodDef, ParameterDef, SetterDef};

/// Builds the comma-form selector string from a method/constructor name and
/// its parameter list: `name(_,label,...)`, or `name()` for zero-arity.
///
/// A positional parameter (no label) renders as `_`; a labeled (keyword)
/// parameter renders as its label. Mirrors
/// `phalcom-core/bin/gen-core-table/main.rs`'s `comma_form`.
pub fn comma_form(name: &str, params: &[ParameterDef]) -> String {
    if params.is_empty() {
        return format!("{name}()");
    }
    let inner = params
        .iter()
        .map(|param| param.label.as_deref().map(encode_label_component).unwrap_or_else(|| "_".to_string()))
        .collect::<Vec<_>>()
        .join(",");
    format!("{name}({inner})")
}

/// The comma-form selector a call-site's method name and argument labels
/// would resolve to — the reference-side mirror of [`comma_form`], applied
/// to `phalcom_ast::ast::Argument` labels at a `MethodCall` send site rather
/// than a declaration's `ParameterDef`s.
pub fn comma_form_from_labels(name: &str, labels: &[Option<String>]) -> String {
    if labels.is_empty() {
        return format!("{name}()");
    }
    let inner = labels
        .iter()
        .map(|label| label.as_deref().map(encode_label_component).unwrap_or_else(|| "_".to_string()))
        .collect::<Vec<_>>()
        .join(",");
    format!("{name}({inner})")
}

/// Mirrors `phalcom-core::method::encode_label_component`. The LSP does not
/// link the runtime crate, but its definition/reference index must spell the
/// same reversible selector slots.
fn encode_label_component(text: &str) -> String {
    let safe = !text.is_empty()
        && !text.starts_with('~')
        && !matches!(text, "_" | "*" | "**" | "***")
        && text.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'_' | b'?' | b'!' | b'+' | b'-' | b'*' | b'/' | b'<' | b'>' | b'=' | b'&' | b'|' | b'^' | b'~' | b'%'
                )
        });
    if safe {
        text.to_string()
    } else {
        let mut encoded = String::with_capacity(1 + text.len() * 2);
        encoded.push('~');
        for byte in text.bytes() {
            use std::fmt::Write;
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
        }
        encoded
    }
}

/// The comma-form selector a getter's bare-name access resolves to.
///
/// A getter selector is the bare name with **no** parens — distinct from a
/// zero-arity method (`name` vs `name()`), so `foo` and `foo()` never alias
/// the same definition (ADR-0012).
pub fn getter_selector(g: &GetterDef) -> String {
    g.name.clone()
}

/// The comma-form selector a `name=(put)` write resolves to, given just the
/// bare property name.
///
/// Always the fixed setter role: literal `name=(put)` — never
/// comma-joined, since a setter takes exactly one argument and it is never
/// labeled. The single spelling both [`setter_selector`] (declaration side)
/// and `index.rs`'s `SetProperty` reference-site walk route through, so the
/// two can never drift apart.
pub fn setter_selector_from_name(name: &str) -> String {
    format!("{name}=(put)")
}

/// The comma-form selector a setter's `recv.name = value` write resolves to.
pub fn setter_selector(s: &SetterDef) -> String {
    setter_selector_from_name(&s.name)
}

/// The comma-form selector a method declaration defines.
pub fn method_selector(m: &MethodDef) -> String {
    comma_form(&m.name, &m.params)
}

/// The comma-form selector a declared class field's bare-name read resolves
/// to (U-ANNOT-LAYOUT §3.1).
///
/// Mirrors [`getter_selector`]'s bare-name-no-parens shape: a field read is
/// indistinguishable, selector-wise, from a getter access.
pub fn field_selector(f: &FieldDef) -> String {
    f.name.clone()
}

/// The bracket-form selector a bracket subscript method declaration defines
/// (U-INDEX, ADR-0060) — `[_]`, `[_,default]`, `[_]=(put)`, ... . Mirrors
/// [`comma_form`]'s label-joining exactly, just bracket- rather than
/// paren-delimited and with no leading name (a bracket method carries no
/// name token at all — see [`IndexMethodDef`]'s doc).
///
/// **Do not confuse with `phalcom-core`'s `SignatureKind::Subscript`
/// encoding** — same spelling, independently reimplemented here per this
/// module's top-level doc (`phalcom-lsp` never links `phalcom-core`).
pub fn index_selector(ix: &IndexMethodDef) -> String {
    let labels = ix
        .params
        .iter()
        .map(|param| param.label.as_deref().map(encode_label_component).unwrap_or_else(|| "_".to_string()))
        .collect::<Vec<_>>();
    let inner = labels.join(",");
    match &ix.accessor {
        IndexAccessor::Get => {
            format!("[{inner}]")
        }
        IndexAccessor::Set { .. } => {
            format!("[{inner}]=(put)")
        }
    }
}

/// The comma-form selector any [`ClassMember`] declaration defines.
///
/// The single dispatch point every definition-side index entry goes
/// through, so the member kinds can never drift into inconsistent
/// spellings.
pub fn class_member_selector(member: &ClassMember) -> String {
    match member {
        ClassMember::Method(m) => method_selector(m),
        ClassMember::Getter(g) => getter_selector(g),
        ClassMember::Setter(s) => setter_selector(s),
        ClassMember::Field(f) => field_selector(f),
        // A `@variant` arm is not itself a message selector — it names the
        // sibling class `phalcom-core`'s `expand_class_attributes` generates
        // at compile time, never a member dispatched on the enclosing class.
        // `phalcom-lsp` indexes the pre-expansion AST directly (ADR-0056 §2:
        // no `phalcom-core` link), so there is no generated selector to spell
        // here; the variant's own name is the closest stand-in.
        ClassMember::Variant(v) => v.name.clone(),
        ClassMember::Index(ix) => index_selector(ix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::ast::Statement;
    use phalcom_ast::parser::parse;

    fn parse_class(src: &str) -> phalcom_ast::ast::ClassDef {
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "unexpected parse errors: {:?}", parsed.errors);
        for statement in parsed.program.statements {
            if let Statement::Class(class_def) = statement {
                return class_def;
            }
        }
        panic!("no class found in {src:?}");
    }

    #[test]
    fn method_with_positional_and_labeled_params() {
        let class_def = parse_class("class Point {\n  move(_ x, to, duration) { }\n}\n");
        let ClassMember::Method(m) = &class_def.members[0] else {
            panic!("expected method")
        };
        assert_eq!(method_selector(m), "move(_,to,duration)");
    }

    #[test]
    fn zero_arity_method_is_not_bare_name() {
        let class_def = parse_class("class Point {\n  reset() { }\n}\n");
        let ClassMember::Method(m) = &class_def.members[0] else {
            panic!("expected method")
        };
        assert_eq!(method_selector(m), "reset()");
    }

    #[test]
    fn getter_has_no_parens_and_never_aliases_zero_arity_method() {
        let class_def = parse_class("class Point {\n  y { }\n  x() { }\n}\n");
        let ClassMember::Getter(g) = &class_def.members[0] else {
            panic!("expected getter")
        };
        let ClassMember::Method(m) = &class_def.members[1] else {
            panic!("expected method")
        };
        assert_eq!(getter_selector(g), "y");
        assert_eq!(method_selector(m), "x()");
        assert_ne!(getter_selector(g), method_selector(m));
    }

    #[test]
    fn setter_is_literal_single_slot() {
        let class_def = parse_class("class Point {\n  x=(put v) { }\n}\n");
        let ClassMember::Setter(s) = &class_def.members[0] else {
            panic!("expected setter")
        };
        assert_eq!(setter_selector(s), "x=(put)");
    }

    #[test]
    fn construct_is_comma_form() {
        let class_def = parse_class("class Point {\n  @constructor\n  new(_ x, y) { }\n}\n");
        let ClassMember::Method(m) = &class_def.members[0] else {
            panic!("expected method")
        };
        assert_eq!(method_selector(m), "new(_,y)");
    }

    #[test]
    fn call_site_labels_match_declaration_selector() {
        assert_eq!(
            comma_form_from_labels("move", &[None, Some("to".to_string()), Some("duration".to_string())]),
            "move(_,to,duration)"
        );
        assert_eq!(comma_form_from_labels("reset", &[]), "reset()");
    }

    #[test]
    fn subscript_get_and_set() {
        let class_def = parse_class("class Arr {\n  [_ idx] { }\n  [_ idx]=(put value) { }\n}\n");
        let ClassMember::Index(g) = &class_def.members[0] else {
            panic!("expected index get")
        };
        let ClassMember::Index(s) = &class_def.members[1] else {
            panic!("expected index set")
        };
        assert_eq!(index_selector(g), "[_]");
        assert_eq!(index_selector(s), "[_]=(put)");
    }
}
