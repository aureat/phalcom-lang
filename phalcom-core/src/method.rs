//! Methods, signatures, and selector encoding.
//!
//! A [`MethodObject`] is a heap [`Object`](crate::heap::Object): either a
//! compiled bytecode closure or a native primitive, plus its [`Signature`] and a
//! handle to its holder class. All object links are `Copy` handles
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)).

use crate::error::PhResult;
use crate::heap::{ClassId, ObjRef};
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;

/// A native Rust method implementation for a core-library method.
///
/// Receives the VM (hence the [`Heap`](crate::heap::Heap)), the receiver, and the
/// arguments, and returns a result [`Value`].
pub type PrimitiveFn = fn(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value>;

/// The shape of a selector: what kind of message it names and its arity.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignatureKind {
    /// An initializer, `init new(_,_)`, of the given arity.
    Initializer(u8),
    /// An ordinary method, `foo(_,_,_)`, of the given arity.
    Method(u8),
    /// A no-argument getter, `foo`.
    Getter,
    /// A one-argument setter, `foo=(_)`.
    Setter,
    /// A subscript getter, `[_]`, of the given arity.
    SubscriptGet(u8),
    /// A subscript setter, `[_]=(_)`, of the given arity.
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
            SignatureKind::Initializer(n) => n,
            SignatureKind::Method(n) => n,
            SignatureKind::Getter => 0,
            SignatureKind::Setter => 1,
            SignatureKind::SubscriptGet(n) => n,
            SignatureKind::SubscriptSet(n) => n + 1,
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

/// Builds the canonical label-encoded selector string for `name`/`labels`/`kind`.
pub fn encode_selector(name: &str, labels: &[Option<String>], kind: SignatureKind) -> String {
    match kind {
        SignatureKind::Initializer(0) => format!("init {name}()"),
        SignatureKind::Initializer(_) => {
            let mut s = format!("init {name}(");
            for label in labels {
                if let Some(lbl) = label {
                    s.push_str(lbl);
                    s.push(':');
                } else {
                    s.push_str("_:");
                }
            }
            s.push(')');
            s
        }
        SignatureKind::Method(0) => format!("{name}()"),
        SignatureKind::Method(_) => {
            let mut s = format!("{name}(");
            for label in labels {
                if let Some(lbl) = label {
                    s.push_str(lbl);
                    s.push(':');
                } else {
                    s.push_str("_:");
                }
            }
            s.push(')');
            s
        }
        SignatureKind::Getter => name.to_string(),
        SignatureKind::Setter => format!("{name}=(_:)"),
        SignatureKind::SubscriptGet(n) => {
            let mut s = "[".to_string();
            for _ in 0..n {
                s.push_str("_:");
            }
            s.push(']');
            s
        }
        SignatureKind::SubscriptSet(n) => {
            let mut s = "[".to_string();
            for _ in 0..n {
                s.push_str("_:");
            }
            s.push_str("]=(_:)");
            s
        }
        // The fixed/minimum arity payload is never spelled into the
        // selector — `sum(*numbers)` and `format(fmt, *args)` both intern as
        // `sum(*)` / `format(*)` (U9 corrections §0 point 3). Only
        // `Signature::positional_arity` (set from the payload in
        // `Signature::new`) distinguishes them at runtime.
        SignatureKind::Variadic(_) => format!("{name}(*)"),
    }
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
/// The subscript kinds ([`SignatureKind::SubscriptGet`] /
/// [`SignatureKind::SubscriptSet`]) carry no name in their encoding, so this
/// returns the conventional placeholder name `"[]"` / `"[]="` for them; feeding
/// that back through [`encode_selector`] reproduces the original string (the
/// encoder ignores the name for subscripts).
///
/// [`SignatureKind::Variadic`] round-trips its *name* but not its fixed
/// arity: the selector text never carries `F` (U9 corrections §0 point 3), so
/// this always decodes to `Variadic(0)` regardless of the original method's
/// real fixed-prefix count.
pub fn decode_selector(selector: &str) -> (String, Vec<Option<String>>, SignatureKind) {
    // Subscript forms start with `[`.
    if let Some(rest) = selector.strip_prefix('[') {
        if let Some(index_part) = rest.strip_suffix("]=(_:)") {
            let n = count_args(index_part);
            return ("[]=".to_string(), vec![None; n as usize], SignatureKind::SubscriptSet(n));
        }
        if let Some(index_part) = rest.strip_suffix(']') {
            let n = count_args(index_part);
            return ("[]".to_string(), vec![None; n as usize], SignatureKind::SubscriptGet(n));
        }
        // Malformed subscript-like string: fall through to the getter default.
    }

    // Argumentful forms contain a `(`; everything else is a bare getter.
    let Some(open) = selector.find('(') else {
        return (selector.to_string(), Vec::new(), SignatureKind::Getter);
    };

    let head = &selector[..open];
    let inner = &selector[open + 1..selector.len().saturating_sub(1)];

    // Setter: `name=(_:)` — an *identifier* head ending in `=`, one arg. The
    // identifier check disambiguates a real setter (`class=(_:)`) from an
    // operator selector that merely ends in `=` (`==(_:)`, `>=(_:)`), which is
    // an ordinary one-argument method.
    if inner == "_:" {
        if let Some(name) = head.strip_suffix('=') {
            if is_identifier(name) {
                return (name.to_string(), vec![None], SignatureKind::Setter);
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

    // Initializer (`init name(...)`) vs ordinary method.
    let (name, is_init) = match head.strip_prefix("init ") {
        Some(rest) => (rest.to_string(), true),
        None => (head.to_string(), false),
    };

    let labels = parse_labels(inner);
    let arity = labels.len() as u8;
    let kind = if is_init { SignatureKind::Initializer(arity) } else { SignatureKind::Method(arity) };
    (name, labels, kind)
}

/// Counts the `_:` argument slots in a subscript index list (`"_:_:"` -> 2).
fn count_args(inner: &str) -> u8 {
    if inner.is_empty() {
        0
    } else {
        inner.matches("_:").count() as u8
    }
}

/// Parses a method/initializer paren body (`"to:duration:"`, `"_:_:"`, `""`)
/// into its per-argument labels: `Some(label)` for a keyword, `None` for the
/// positional placeholder `_`.
fn parse_labels(inner: &str) -> Vec<Option<String>> {
    if inner.is_empty() {
        return Vec::new();
    }
    // Each argument is a `token:` unit; a trailing `:` yields one empty final
    // segment after the split, which is skipped.
    inner
        .split(':')
        .filter(|segment| !segment.is_empty())
        .map(|token| if token == "_" { None } else { Some(token.to_string()) })
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
        SignatureKind::Initializer(n) => n,
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

/// The implementation strategy behind a [`MethodObject`].
#[derive(Debug, Clone, Copy)]
pub enum MethodKind {
    /// Phalcom code compiled to bytecode, by [`ClosureObject`](crate::closure::ClosureObject) handle.
    Closure(ObjRef),
    /// A native Rust function for a core-library method.
    Primitive(PrimitiveFn),
}

/// A callable method: its implementation, signature, and holder class.
#[derive(Debug)]
pub struct MethodObject {
    /// Whether this method is a bytecode closure or a native primitive.
    pub kind: MethodKind,
    /// The method's calling signature.
    pub signature: Signature,
    /// Handle to the class that owns this method, once bound (`None` until set).
    pub holder: Option<ClassId>,
}

impl MethodObject {
    /// Builds a method with the given `kind`, deriving its signature.
    pub fn new(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind, holder: Option<ClassId>) -> Self {
        let signature = Signature::new(selector, sig_kind);
        MethodObject { kind, signature, holder }
    }

    /// Builds an unbound method (holder `None`), typically a compiler-produced
    /// closure that is attached to its class later by `Bytecode::Method`.
    pub fn new_single(selector: Symbol, sig_kind: SignatureKind, kind: MethodKind) -> Self {
        Self::new(selector, sig_kind, kind, None)
    }

    /// Builds a native primitive method held by `holder`.
    pub fn new_primitive(selector: Symbol, sig_kind: SignatureKind, primitive: PrimitiveFn, holder: ClassId) -> Self {
        Self::new(selector, sig_kind, MethodKind::Primitive(primitive), Some(holder))
    }

    /// Returns this method's selector [`Symbol`].
    pub fn selector(&self) -> Symbol {
        self.signature.selector
    }

    /// Returns whether this method is a native primitive.
    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, MethodKind::Primitive(_))
    }

    /// Returns whether this method is a bytecode closure.
    pub fn is_closure(&self) -> bool {
        matches!(self.kind, MethodKind::Closure(_))
    }

    /// Binds this method to its `holder` class handle.
    pub fn set_holder(&mut self, holder: ClassId) {
        self.holder = Some(holder);
    }

    /// Renders this method's debug form, `"<method Holder::selector>"`.
    ///
    /// # Panics
    ///
    /// Panics if the method has no holder set (an internal invariant violation).
    pub fn to_debug(&self, vm: &VM) -> String {
        let name = vm.resolve_symbol(self.signature.selector);
        let holder_name = self
            .holder
            .map_or_else(|| panic!("this shouldn't happen"), |holder| vm.heap.class(holder).name.clone());
        format!("<method {holder_name}::{name}>")
    }
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
        let encoded = encode_selector(
            "move",
            &[Some("to".to_string()), Some("duration".to_string())],
            SignatureKind::Method(2),
        );
        let (name, labels, kind) = decode_selector(&encoded);
        assert_eq!(name, "move");
        assert_eq!(labels, vec![Some("to".to_string()), Some("duration".to_string())]);
        assert_eq!(kind, SignatureKind::Method(2));
    }

    /// A setter is distinguished from an operator that merely ends in `=`.
    #[test]
    fn decode_setter_vs_operator() {
        let setter = encode_selector("class", &[], SignatureKind::Setter);
        let (name, labels, kind) = decode_selector(&setter);
        assert_eq!(name, "class");
        assert_eq!(labels, vec![None]);
        assert_eq!(kind, SignatureKind::Setter);

        // `==(_:)` must decode to a one-arg method, not a setter named `=`.
        let (op_name, _, op_kind) = decode_selector("==(_:)");
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
    }

    /// Subscript kinds decode with their placeholder names and round-trip.
    #[test]
    fn decode_subscripts() {
        let get = encode_selector("", &[], SignatureKind::SubscriptGet(1));
        let (name, _, kind) = decode_selector(&get);
        assert_eq!(name, "[]");
        assert_eq!(kind, SignatureKind::SubscriptGet(1));
        assert_eq!(encode_selector(&name, &[None], kind), get);

        let set = encode_selector("", &[], SignatureKind::SubscriptSet(1));
        let (sname, _, skind) = decode_selector(&set);
        assert_eq!(sname, "[]=");
        assert_eq!(skind, SignatureKind::SubscriptSet(1));
    }
}
