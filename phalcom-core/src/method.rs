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
        };
        Signature {
            selector,
            kind,
            positional_arity,
            variadic: false,
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
    }
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
