//! Methods, signatures, and selector encoding.
//!
//! A [`MethodObject`] is a heap [`Object`](crate::heap::Object): either a
//! compiled bytecode closure or a native primitive, plus its [`Signature`] and a
//! handle to its holder class. All object links are `Copy` handles
//! ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).

mod object;

pub use object::{ArgumentView, CallOutcome, LegacyPrimitiveFn, MethodKind, MethodObject, PrimitiveFn};

use crate::interner::Symbol;
use phalcom_common::selector as common_selector;

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
}

/// Normalized rest capture layout.  This is deliberately signature metadata,
/// never selector text parsed back during dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestMode {
    Positional { param_index: u16 },
    Labeled { param_index: u16 },
    Split { positional_param_index: u16, labeled_param_index: u16 },
    Complete { param_index: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestLayout {
    fixed_positionals: u8,
    fixed_labels: Box<[Symbol]>,
    mode: RestMode,
}

impl RestLayout {
    pub fn new(fixed_positionals: u8, fixed_labels: Box<[Symbol]>, mode: RestMode) -> Self {
        Self {
            fixed_positionals,
            fixed_labels,
            mode,
        }
    }
    pub fn mode(&self) -> RestMode {
        self.mode
    }
    pub fn fixed_positionals(&self) -> u8 {
        self.fixed_positionals
    }
    pub fn fixed_labels(&self) -> &[Symbol] {
        &self.fixed_labels
    }
    pub fn accepts(&self, positional_count: usize, labels: &[Symbol]) -> bool {
        let fixed = self.fixed_positionals as usize;
        match self.mode {
            RestMode::Positional { .. } => positional_count >= fixed && labels == self.fixed_labels.as_ref(),
            RestMode::Labeled { .. } => positional_count == fixed && labels.starts_with(self.fixed_labels.as_ref()),
            RestMode::Split { .. } | RestMode::Complete { .. } => positional_count >= fixed && labels.starts_with(self.fixed_labels.as_ref()),
        }
    }
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
    /// Rest-capable method layout, or `None` for exact-only methods.
    pub rest: Option<RestLayout>,
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
        };
        Signature {
            selector,
            kind,
            positional_arity,
            rest: None,
        }
    }

    /// Builds a signature with explicit arity and normalized rest metadata.
    pub fn new_with_arity(selector: Symbol, kind: SignatureKind, positional_arity: u8, rest: Option<RestLayout>) -> Self {
        Signature {
            selector,
            kind,
            positional_arity,
            rest,
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
    let common_kind = common_kind(kind);
    let slots = labels
        .iter()
        .map(|label| match label {
            None => common_selector::SelectorSlot::Positional,
            Some(text) => common_selector::SelectorSlot::Label(text.clone()),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    common_selector::Selector::new(common_base(kind, name), common_kind, slots.clone())
        .unwrap_or(common_selector::Selector {
            base: common_base(kind, name),
            kind: common_kind,
            slots,
        })
        .encode()
}

/// Encodes a Symbol label for safe embedding in comma-form selector slots.
///
/// The escape transport is deliberately total and UTF-8 byte based: labels
/// that could be mistaken for slot markers or delimiters are `~` + lowercase
/// hexadecimal bytes, while legacy self-delimiting labels remain readable.
pub fn encode_label_component(symbol_text: &str) -> String {
    common_selector::encode_label_component(symbol_text)
}

/// Decodes an escaped selector-label component. Malformed components remain
/// raw, keeping selector reflection total for arbitrary user Symbols.
pub fn decode_label_component(component: &str) -> String {
    common_selector::decode_label_component(component)
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
pub fn decode_selector(selector: &str) -> (String, Vec<Option<String>>, SignatureKind) {
    let decoded = common_selector::Selector::decode(selector);
    let name = match &decoded.base {
        common_selector::SelectorBase::Named(name) => name.clone(),
        common_selector::SelectorBase::Subscript => match decoded.kind {
            common_selector::SelectorKind::SubscriptSet => "[]=".to_string(),
            _ => "[]".to_string(),
        },
    };
    let labels = decoded
        .slots
        .iter()
        .map(|slot| match slot {
            common_selector::SelectorSlot::Positional => None,
            common_selector::SelectorSlot::Label(label) => Some(label.clone()),
        })
        .collect::<Vec<_>>();
    let kind = match decoded.kind {
        common_selector::SelectorKind::Getter => SignatureKind::Getter,
        common_selector::SelectorKind::Setter => SignatureKind::Setter,
        common_selector::SelectorKind::Method => SignatureKind::Method(u8::try_from(decoded.slots.len()).unwrap_or(u8::MAX)),
        common_selector::SelectorKind::SubscriptGet => SignatureKind::SubscriptGet(u8::try_from(decoded.slots.len()).unwrap_or(u8::MAX)),
        common_selector::SelectorKind::SubscriptSet => SignatureKind::SubscriptSet(u8::try_from(decoded.slots.len()).unwrap_or(u8::MAX)),
    };
    let labels = if matches!(kind, SignatureKind::Setter) {
        vec![Some("put".to_string())]
    } else {
        labels
    };
    (name, labels, kind)
}

fn common_base(kind: SignatureKind, name: &str) -> common_selector::SelectorBase {
    match kind {
        SignatureKind::SubscriptGet(_) | SignatureKind::SubscriptSet(_) => common_selector::SelectorBase::Subscript,
        SignatureKind::Method(_) | SignatureKind::Getter | SignatureKind::Setter => common_selector::SelectorBase::Named(name.to_string()),
    }
}

fn common_kind(kind: SignatureKind) -> common_selector::SelectorKind {
    match kind {
        SignatureKind::Method(_) => common_selector::SelectorKind::Method,
        SignatureKind::Getter => common_selector::SelectorKind::Getter,
        SignatureKind::Setter => common_selector::SelectorKind::Setter,
        SignatureKind::SubscriptGet(_) => common_selector::SelectorKind::SubscriptGet,
        SignatureKind::SubscriptSet(_) => common_selector::SelectorKind::SubscriptSet,
    }
}

/// Turns a base `name` plus a [`SignatureKind`] into its textual signature.
pub fn make_signature(base: &str, kind: SignatureKind) -> String {
    let arity = match kind {
        SignatureKind::Method(n) => n,
        SignatureKind::Getter => 0,
        SignatureKind::Setter => 0, // Setter has 1 arg but the label list is empty in the AST.
        SignatureKind::SubscriptGet(n) => n,
        SignatureKind::SubscriptSet(n) => n,
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
        assert_eq!(decode_selector("foo(*)").2, SignatureKind::Method(1));
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
