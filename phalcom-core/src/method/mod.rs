//! Methods, signatures, and selector encoding.
//!
//! A [`MethodObject`] is a heap [`Object`](crate::heap::Object): either a
//! compiled bytecode closure or a native primitive, plus its [`Signature`] and a
//! handle to its holder class. All object links are `Copy` handles
//! ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).

mod object;

pub use object::{MethodKind, MethodObject, PrimitiveFn};

use crate::interner::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberVisibility {
    Public,
    Private,
    Protected,
    Internal,
}

/// The shape of a selector: what kind of message it names and its arity.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignatureKind {
    /// An ordinary method, `foo(_,_,_)`, of the given arity.
    Method(u8),
    /// A no-argument getter, `foo`.
    Getter,
    /// A one-argument setter, `foo=(put)`.
    Setter,
    /// A bracket subscript getter.
    SubscriptGet(u8),
    /// A bracket subscript setter.
    SubscriptSet(u8),
    /// A variadic method, `foo(*rest)`, whose payload is the *fixed/minimum*
    /// positional arity `F` preceding the rest parameter (0 for `sum(*numbers)`,
    /// 1 for `format(fmt, *args)`).
    ///
    /// The selector text this encodes to never includes `F` — it is always
    /// the bare `name(*)`, independent of how many fixed parameters precede
    /// the rest parameter (U9, `messages-and-selectors.md` §4; see
    /// [`encode_selector`]'s `Variadic` arm). `F` is carried only by
    /// [`Signature::positional_arity`] at runtime, read by the VM's
    /// call-prologue and derived-selector dispatch probe (`vm.rs::call_method`,
    /// `Bytecode::Invoke`'s miss arm).
    Variadic(u8),
}

/// A method's fully-resolved calling signature.
#[derive(Clone, Debug)]
pub struct Signature {
    /// The interned canonical selector symbol.
    pub selector: Symbol,
    /// The kind of selector this signature encodes.
    pub kind: SignatureKind,
    /// The number of positional parameters.
    pub positional_arity: u8,
    /// Whether the final parameter is variadic.
    pub variadic: bool,
}

impl Signature {
    /// Builds a signature for `selector` of the given `kind`, deriving arity.
    pub fn new(selector: Symbol, kind: SignatureKind) -> Self {
        let positional_arity = match kind {
            SignatureKind::Method(n) => n,
            SignatureKind::Getter => 0,
            SignatureKind::Setter => 1,
            SignatureKind::SubscriptGet(n) => n,
            SignatureKind::SubscriptSet(n) => n.checked_add(1).expect("SubscriptSet index arity must leave room for the assigned value"),
            SignatureKind::Variadic(f) => f,
        };
        Signature {
            selector,
            kind,
            positional_arity,
            variadic: matches!(kind, SignatureKind::Variadic(_)),
        }
    }

    /// Builds a signature with an explicit `positional_arity` and `variadic` flag.
    pub fn new_with_arity(selector: Symbol, kind: SignatureKind, positional_arity: u8, variadic: bool) -> Self {
        Signature {
            selector,
            kind,
            positional_arity,
            variadic,
        }
    }
}

/// Builds the canonical comma-form label-encoded selector string for
/// `name`/`labels`/`kind` (ADR-0012, amended 2026-07-11: `move(_,to,duration)`
/// spelling — a positional argument renders as `_`, a keyword argument as
/// its label, slots joined by `,`).
///
/// [`SignatureKind::Subscript`] ignores `name` entirely (U-INDEX,
/// ADR-0060) — the bracket delimiter itself carries the selector's whole
/// identity, so `[_]`/`[_,default]` and their `=(put)` setter counterparts use the same
/// `comma_form_slots` every keyword method uses, just bracket- rather than
/// paren-delimited and with no leading name.
pub fn encode_selector(name: &str, labels: &[Option<String>], kind: SignatureKind) -> String {
    match kind {
        SignatureKind::Method(0) => format!("{name}()"),
        SignatureKind::Method(_) => format!("{name}({})", comma_form_slots(labels)),
        SignatureKind::Getter => name.to_string(),
        SignatureKind::Setter => format!("{name}=(put)"),
        SignatureKind::SubscriptGet(_) => format!("[{}]", comma_form_slots(labels)),
        SignatureKind::SubscriptSet(_) => format!("[{}]=(put)", comma_form_slots(labels)),
        SignatureKind::Variadic(_) => format!("{name}(*)"),
    }
}

/// Joins `labels` into a comma-form slot list: `_` for a positional
/// argument, the label text for a keyword argument.
fn comma_form_slots(labels: &[Option<String>]) -> String {
    labels
        .iter()
        .map(|label| match label {
            None => "_".to_string(),
            Some(text) => encode_label_component(text),
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Encodes a Symbol label for safe embedding in comma-form selector slots.
///
/// The escape transport is deliberately total and UTF-8 byte based: labels
/// that could be mistaken for slot markers or delimiters are `~` + lowercase
/// hexadecimal bytes, while legacy self-delimiting labels remain readable.
pub fn encode_label_component(symbol_text: &str) -> String {
    let reserved = matches!(symbol_text, "_" | "*" | "**" | "***");
    let safe = !symbol_text.is_empty()
        && !symbol_text.starts_with('~')
        && !reserved
        && symbol_text.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'_' | b'?' | b'!' | b'+' | b'-' | b'*' | b'/' | b'<' | b'>' | b'=' | b'&' | b'|' | b'^' | b'~' | b'%'
                )
        });
    if safe {
        symbol_text.to_string()
    } else {
        let mut encoded = String::with_capacity(1 + symbol_text.len() * 2);
        encoded.push('~');
        for byte in symbol_text.bytes() {
            use std::fmt::Write;
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
        }
        encoded
    }
}

/// Decodes an escaped selector-label component. Malformed components remain
/// raw, keeping selector reflection total for arbitrary user Symbols.
pub fn decode_label_component(component: &str) -> String {
    let Some(hex) = component.strip_prefix('~') else {
        return component.to_string();
    };
    if hex.is_empty() || hex.len() % 2 != 0 {
        return component.to_string();
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks_exact(2) {
        let Some(high) = (pair[0] as char).to_digit(16) else {
            return component.to_string();
        };
        let Some(low) = (pair[1] as char).to_digit(16) else {
            return component.to_string();
        };
        bytes.push((high * 16 + low) as u8);
    }
    String::from_utf8(bytes).unwrap_or_else(|_| component.to_string())
}

/// Decomposes an encoded selector string back into `(name, labels, kind)` —
/// the exact inverse of [`encode_selector`].
///
/// Used by the `doesNotUnderstand(_:)` miss path to reify a missed send as a
/// [`Message`](crate::primitive::object) instance (method-lookup.md §2,
/// ADR-0012): the receiver of a proxy sees the *decomposed* selector
/// (`name`/`labels`) rather than re-parsing the raw symbol. `labels[i]` is
/// `Some(label)` for a keyword argument and `None` for a positional one, so
/// `labels.len()` equals the selector's positional arity for the argumentful
/// kinds.
///
/// This is **total**: it never panics. A string that matches no argumentful
/// shape (no `(`, no `[`) decodes to a [`SignatureKind::Getter`], because an
/// arbitrary user selector (e.g. from `Symbol.new("garbage")` sent via
/// `perform`, then missed) can reach here through the dNU path and must not
/// crash the VM.
///
/// # Round-trip note
///
/// [`SignatureKind::SubscriptGet`] carries no name in its encoding (U-INDEX,
/// ADR-0060), so this returns the conventional placeholder name `"[]"` for
/// it, with real index labels recovered; feeding that back through [`encode_selector`]
/// reproduces the original string (the encoder ignores the name for
/// subscripts).
///
/// [`SignatureKind::Variadic`] round-trips its *name* but not its fixed
/// arity: the selector text never carries `F` (U9 corrections §0 point 3), so
/// this always decodes to `Variadic(0)` regardless of the original method's
/// real fixed-prefix count.
pub fn decode_selector(selector: &str) -> (String, Vec<Option<String>>, SignatureKind) {
    // Subscript forms start with `[` and end with `]`
    if let Some(rest) = selector.strip_prefix('[') {
        if let Some(inner) = rest.strip_suffix("]=(put)") {
            let labels = parse_labels(inner);
            let n = labels.len() as u8;
            return ("[]=".to_string(), labels, SignatureKind::SubscriptSet(n));
        }
        if let Some(inner) = rest.strip_suffix(']') {
            let labels = parse_labels(inner);
            let n = labels.len() as u8;
            return ("[]".to_string(), labels, SignatureKind::SubscriptGet(n));
        }
        // Malformed subscript-like string: fall through to the getter default.
    }

    // Argumentful forms contain a `(`; everything else is a bare getter.
    let Some(open) = selector.find('(') else {
        return (selector.to_string(), Vec::new(), SignatureKind::Getter);
    };

    if !selector.ends_with(')') || open + 1 > selector.len() - 1 {
        return (selector.to_string(), Vec::new(), SignatureKind::Getter);
    }
    let head = &selector[..open];
    let inner = &selector[open + 1..selector.len() - 1];

    // Setter: canonical `name=(put)`.
    if inner == "put" {
        if let Some(name) = head.strip_suffix('=') {
            if is_identifier(name) {
                return (name.to_string(), vec![Some("put".to_string())], SignatureKind::Setter);
            }
        }
    }

    // Variadic (`name(*)`): the literal `*` marker, never a labeled arg list.
    // The fixed/minimum arity `F` cannot be recovered from the selector text
    // alone (U9 corrections §0 point 3, by design) — this returns `0` as a
    // documented limitation; today's only caller is the dNU `Message`
    // reification path (`vm.rs::new_message`), which never needs the real `F`.
    if inner == "*" {
        return (head.to_string(), Vec::new(), SignatureKind::Variadic(0));
    }

    // Historical `init ` prefixes are accepted on input for generated hidden
    // initializer selectors, but carry no distinct runtime signature kind.
    let name = head.strip_prefix("init ").unwrap_or(head).to_string();

    let labels = parse_labels(inner);
    let arity = labels.len() as u8;
    (name, labels, SignatureKind::Method(arity))
}

/// Parses a method/initializer paren body (`"to,duration"`, `"_,_"`, `""`)
/// into its per-argument labels: `Some(label)` for a keyword, `None` for the
/// positional placeholder `_`.
fn parse_labels(inner: &str) -> Vec<Option<String>> {
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(|token| match token {
            "_" | "*" | "**" | "***" => None,
            _ => Some(decode_label_component(token)),
        })
        .collect()
}

/// Returns whether `s` is a non-empty Phalcom identifier (leading letter or
/// `_`, then letters/digits/`_`), used to tell a setter from an operator
/// selector during [`decode_selector`].
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Turns a base `name` plus a [`SignatureKind`] into its textual signature.
pub fn make_signature(base: &str, kind: SignatureKind) -> String {
    let arity = match kind {
        SignatureKind::Method(n) => n,
        SignatureKind::Getter => 0,
        SignatureKind::Setter => 0, // Setter has 1 arg but the label list is empty in the AST.
        SignatureKind::SubscriptGet(n) => n,
        SignatureKind::SubscriptSet(n) => n,
        SignatureKind::Variadic(f) => f,
    };
    let labels = vec![None; arity as usize];
    encode_selector(base, &labels, kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`decode_selector`] inverts [`encode_selector`] across every
    /// [`SignatureKind`], including labeled/positional mixes and operators.
    #[test]
    fn decode_inverts_encode() {
        let cases: &[(&str, &[Option<String>], SignatureKind)] = &[
            ("wibble", &[], SignatureKind::Getter),
            ("size", &[], SignatureKind::Getter),
            ("foo", &[], SignatureKind::Method(0)),
            ("+", &[None], SignatureKind::Method(1)),
            ("==", &[None], SignatureKind::Method(1)),
            (">=", &[None], SignatureKind::Method(1)),
            ("move", &[None, None], SignatureKind::Method(2)),
        ];
        for (name, labels, kind) in cases {
            let encoded = encode_selector(name, labels, *kind);
            let (dname, dlabels, dkind) = decode_selector(&encoded);
            assert_eq!(&dname, name, "name mismatch for {encoded}");
            assert_eq!(&dlabels, labels, "labels mismatch for {encoded}");
            assert_eq!(dkind, *kind, "kind mismatch for {encoded}");
        }
    }

    /// A fully-labeled keyword selector recovers each keyword label in order.
    #[test]
    fn decode_labeled_selector() {
        let encoded = encode_selector("move", &[Some("to".to_string()), Some("duration".to_string())], SignatureKind::Method(2));
        let (name, labels, kind) = decode_selector(&encoded);
        assert_eq!(name, "move");
        assert_eq!(labels, vec![Some("to".to_string()), Some("duration".to_string())]);
        assert_eq!(kind, SignatureKind::Method(2));
    }

    #[test]
    fn label_component_escape_is_reversible_and_rest_safe() {
        for label in ["timeout", "+", "*", "**", "***", "_", "a,b", "x)", "~raw", "λ"] {
            let encoded = encode_label_component(label);
            assert_eq!(decode_label_component(&encoded), label, "{label}");
        }
        assert_eq!(encode_label_component("*"), "~2a");
        assert_eq!(encode_selector("foo", &[Some("*".to_string())], SignatureKind::Method(1)), "foo(~2a)");
        assert_eq!(decode_selector("foo(*)").2, SignatureKind::Variadic(0));
    }

    #[test]
    fn malformed_label_escape_is_total() {
        for raw in ["~", "~f", "~zz", "~ff"] {
            assert_eq!(decode_label_component(raw), raw);
        }
    }

    /// A setter is distinguished from an operator that merely ends in `=`.
    #[test]
    fn decode_setter_vs_operator() {
        let setter = encode_selector("class", &[], SignatureKind::Setter);
        let (name, labels, kind) = decode_selector(&setter);
        assert_eq!(name, "class");
        assert_eq!(labels, vec![Some("put".to_string())]);
        assert_eq!(kind, SignatureKind::Setter);

        // `==(_)` must decode to a one-arg method, not a setter named `=`.
        let (op_name, _, op_kind) = decode_selector("==(_)");
        assert_eq!(op_name, "==");
        assert_eq!(op_kind, SignatureKind::Method(1));
    }

    /// An arbitrary garbage selector (reachable via `perform` + miss) decodes
    /// to a getter rather than panicking.
    #[test]
    fn decode_garbage_is_total() {
        let (name, labels, kind) = decode_selector("garbage");
        assert_eq!(name, "garbage");
        assert!(labels.is_empty());
        assert_eq!(kind, SignatureKind::Getter);

        let (name, labels, kind) = decode_selector("(");
        assert_eq!(name, "(");
        assert!(labels.is_empty());
        assert_eq!(kind, SignatureKind::Getter);
    }

    #[test]
    fn method_selector_round_trip_including_init_prefix() {
        let (name, labels, kind) = decode_selector("init new(_,foo)");
        assert_eq!(name, "new");
        assert_eq!(labels, vec![None, Some("foo".to_string())]);
        assert_eq!(kind, SignatureKind::Method(2));
        assert_eq!(encode_selector(&name, &labels, kind), "new(_,foo)");
    }

    #[test]
    fn no_initializer_signature_kind() {
        let signature = Signature::new(Symbol(0), SignatureKind::Method(2));
        assert_eq!(signature.kind, SignatureKind::Method(2));
        assert_eq!(signature.positional_arity, 2);
    }

    /// Subscript Get and Set decodes and roundtrips correctly.
    #[test]
    fn decode_subscripts() {
        // `[idx] { ... }` — read, one positional arg.
        let get = encode_selector("", &[None], SignatureKind::SubscriptGet(1));
        assert_eq!(get, "[_]");
        let (name, labels, kind) = decode_selector(&get);
        assert_eq!(name, "[]");
        assert_eq!(labels, vec![None]);
        assert_eq!(kind, SignatureKind::SubscriptGet(1));
        assert_eq!(encode_selector(&name, &labels, kind), get);

        // `[_ idx]=(put value) { ... }` — write, positional index + fixed value role.
        let set = encode_selector("", &[None], SignatureKind::SubscriptSet(1));
        assert_eq!(set, "[_]=(put)");
        let (sname, slabels, skind) = decode_selector(&set);
        assert_eq!(sname, "[]=");
        assert_eq!(slabels, vec![None]);
        assert_eq!(skind, SignatureKind::SubscriptSet(1));
        assert_eq!(encode_selector(&sname, &slabels, skind), set);

        // `[] { ... }` — zero-arity read.
        let empty = encode_selector("", &[], SignatureKind::SubscriptGet(0));
        assert_eq!(empty, "[]");
        let (ename, elabels, ekind) = decode_selector(&empty);
        assert_eq!(ename, "[]");
        assert!(elabels.is_empty());
        assert_eq!(ekind, SignatureKind::SubscriptGet(0));

        // `[]=(put value) { ... }` — zero-arity write.
        let put_only = encode_selector("", &[], SignatureKind::SubscriptSet(0));
        assert_eq!(put_only, "[]=(put)");
        let (pname, plabels, pkind) = decode_selector(&put_only);
        assert_eq!(pname, "[]=");
        assert!(plabels.is_empty());
        assert_eq!(pkind, SignatureKind::SubscriptSet(0));
    }
}
