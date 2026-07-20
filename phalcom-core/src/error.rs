use crate::compiler::lib::CompilerError;
use crate::value::Value;
use phalcom_ast::error::SyntaxError;
use std::io;
// use std::io::Error as IoError;
use thiserror::Error;

pub type PhResult<T> = Result<T, PhError>;

#[derive(Error, Debug, Clone)]
pub enum PhError {
    #[error(transparent)]
    Runtime(#[from] RuntimeError),

    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Parse(#[from] SyntaxError),

    #[error(transparent)]
    Compile(#[from] CompilerError),

    #[error("{0}")]
    StringError(String),

    #[error("{0}")]
    StrError(&'static str),
}

fn format_num_arguments(args: usize) -> String {
    if args == 1 {
        String::from("1 argument")
    } else {
        format!("{} arguments", args)
    }
}

impl From<&'static str> for PhError {
    fn from(err: &'static str) -> Self {
        PhError::StrError(err)
    }
}

impl<T> From<IoError> for PhResult<T> {
    fn from(err: IoError) -> Self {
        Err(PhError::Io(err))
    }
}

impl From<io::Error> for PhError {
    fn from(err: io::Error) -> Self {
        PhError::Io(IoError::Message(err.to_string()))
    }
}

#[derive(Error, Debug, Clone)]
pub enum IoError {
    #[error("{0}")]
    Message(String),
}

#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("Method {signature} expected {}, got {found}", format_num_arguments(*expected))]
    Arity { signature: &'static str, expected: usize, found: usize },

    #[error("Expected {expected}, got {found}")]
    Type { expected: &'static str, found: &'static str },

    /// A recursion ceiling was exceeded
    /// ([PDR-0007](../../docs/decisions/0007-bounded-call-depth-and-native-reentrancy.md)).
    ///
    /// One variant covers both of the VM's counters — `.ph` call-frame depth and
    /// native re-entrancy depth — because they are the same failure to the user and
    /// differ only in which resource ran out. `what` names the counter and `limit`
    /// its ceiling, so the message identifies a limit the reader can actually act on.
    ///
    /// Before this existed, unbounded `.ph` recursion did not fail at all: frames
    /// live in a heap `Vec`, so it grew until the OS killed the process — measured at
    /// over five minutes with no diagnostic. This is an ordinary raise, so
    /// [ADR-0008](../../docs/adr/accepted/0008-layered-exceptions-and-result.md)'s
    /// terminating unwind applies and `ensure` blocks still run.
    #[error("{what} limit exceeded ({limit}); the computation recurses too deeply")]
    DepthExceeded {
        /// Human-readable name of the counter that tripped.
        what: &'static str,
        /// The ceiling that was hit.
        limit: usize,
    },

    /// The surface-`Error` unwind payload — the `Raise(error)` half of
    /// [ADR-0008](../../docs/adr/accepted/0008-layered-exceptions-and-result.md)'s
    /// single unwind primitive (the sibling of U10's `Return`/
    /// [`Bytecode::ReturnNonLocal`](crate::bytecode::Bytecode::ReturnNonLocal)).
    ///
    /// `error` is a surface `Error` subclass instance (catchable, `isA(Error)`,
    /// U-CORE-6); `rendered` is a snapshot of its `message` at raise time, used
    /// only for the uncaught-render path (`{rendered}` below) — never itself
    /// read by a future `on`/`ensure`/fiber consumer, which reads `error`
    /// instead. Replaces the retired `MessageNotUnderstood` variant: a genuine
    /// `doesNotUnderstand(_:)` miss now raises a surface
    /// [`MessageNotUnderstood`](crate::universe::CoreClasses::message_not_understood_class)
    /// through this payload
    /// ([`object_does_not_understand`](crate::primitive::object::object_does_not_understand)),
    /// rather than a bespoke native variant.
    #[error("{rendered}")]
    Raise {
        /// The raised surface `Error` (or subclass) instance — the value a
        /// future `on(_)`/`ensure`/fiber-result-slot consumer intercepts.
        error: Value,
        /// A display snapshot of `error`'s `message` at raise time, used only
        /// to render the uncaught-error trace; never itself catchable.
        rendered: String,
    },

    #[error("Unsupported operation '{op}' for {value}")]
    UnsupportedOperation { op: &'static str, value: String },

    #[error("Binary operation '{op}' not supported for {left} and {right}")]
    BinaryNotSupported {
        op: &'static str,
        left: String,
        right: String,
    },

    #[error("Unary operation '{op}' not supported for {value}")]
    UnaryNotSupported { op: &'static str, value: String },

    #[error("Can't set superclass of a class")]
    InvalidSetSuper,

    #[error("Can't set class of an object")]
    InvalidSetClass,

    #[error("Undefined variable `{0}`")]
    UndefinedVar(String),

    #[error("Division by zero")]
    ZeroDivision,

    #[error("Can't convert {found} to {expected}")]
    TypeConversion { expected: &'static str, found: &'static str },

    #[error("Superclass `{0}` is not a class")]
    InvalidSuperClass(String),

    #[error("{0}")]
    NotAllowed(String),

    #[error("Invalid argument: {0}")]
    ArgumentError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    /// A non-local `return` inside a block tried to unwind to its home method
    /// activation, but that activation is no longer live — the block escaped its
    /// defining method and was invoked after that method had already returned.
    ///
    /// This is Phalcom's `BlockCannotReturn` (Smalltalk lineage): the
    /// [`Bytecode::ReturnNonLocal`](crate::bytecode::Bytecode::ReturnNonLocal)
    /// handler compares the executing block frame's
    /// [`home_frame_token`](crate::frame::CallFrame::home_frame_token) against
    /// the live frame stack, and raises this variant when no frame matches the
    /// token's `(frame_index, generation)` — turning a would-be use-after-free
    /// into a clean runtime error (blocks.md §5, object-model.md §4,
    /// [ADR-0013](../../docs/adr/accepted/0013-block-closure-upvalues.md)). Detail beyond
    /// the fixed message is intentionally omitted, matching the plain-`thiserror`
    /// shape of every neighboring variant (no span, no miette).
    #[error("non-local return from a block whose home method frame is no longer alive (DeadFrameError)")]
    DeadFrameError,

    /// A `Map`/`Set` key's `hash`/`==` tried to structurally mutate (insert or
    /// remove an entry of) the very collection [`crate::primitive::map::locate`]/
    /// [`crate::primitive::set::locate`] is currently disambiguating a candidate
    /// for.
    ///
    /// Raised **at the mutation call site** inside the user's `hash`/`==`
    /// method — not back in `locate`'s caller — so the traceback blames the
    /// culprit line, not the collection operation that innocently triggered
    /// it (`docs/deferred/error-handling-followups.md` §1, RULED
    /// 2026-07-20). Overwriting an *existing* key's value in place is not
    /// structural and remains legal from within `hash`/`==`; only slot
    /// creation/removal is guarded.
    ///
    /// Maps to `kind: #concurrentMutation` once PDR-0010's `kind` field lands
    /// (`docs/spec/traceback/implementation-spec.md` §8.1).
    #[error("cannot mutate this {collection} inside 'hash'/'==' of its own key (concurrent mutation)")]
    ConcurrentMutation {
        /// The collection's display name — `"Map"` or `"Set"`.
        collection: &'static str,
    },

    #[error("{0}")]
    Message(String),
}

/// Failure returned by [`crate::heap::MapObject`]'s structural-mutation
/// methods (`insert_new`/`remove_at`).
///
/// Realizes the G0 reentrancy lock
/// (`docs/deferred/error-handling-followups.md` §1, RULED 2026-07-20): a
/// key's `hash`/`==` may not structurally mutate the collection currently
/// disambiguating it via [`crate::primitive::map::locate`]/
/// [`crate::primitive::set::locate`]. Lives in `error` rather than `heap::map`
/// so both the heap layer and the `primitive::map`/`primitive::set` layer can
/// name it without a cross-module visibility widening.
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMutationError {
    /// The collection is locked — a reentrant `hash`/`==` send is in
    /// progress (see [`crate::heap::MapObject::enter_reentrant_send`]).
    /// Callers convert this into a catchable [`RuntimeError::ConcurrentMutation`].
    #[error("collection is locked by a reentrant hash/== send")]
    Locked,
    /// `slot` does not refer to a live entry. Defense-in-depth only: every
    /// caller derives `slot` from a fresh `MapObject::bucket` scan performed
    /// under the same lock discipline, so this should be unreachable in
    /// practice — it converts a hypothetical future logic bug into a
    /// diagnosable error instead of a panic.
    #[error("slot does not refer to a live entry")]
    OutOfRange,
}

#[macro_export]
macro_rules! ensure_arity {
    ($signature: expr, $args:expr, $expected:expr) => {
        if $args.len() != $expected {
            return Err(RuntimeError::Arity {
                signature: $signature,
                expected: $expected,
                found: $args.len(),
            }
            .into());
        }
    };
}

#[macro_export]
macro_rules! expect_value {
    ($value:expr, String) => {{
        match $value {
            Value::String(s) => s.clone(),
            found => {
                return Err(RuntimeError::Type {
                    expected: "String",
                    found: found.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Number) => {{
        match $value {
            Value::Number(n) => *n,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Number",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Bool) => {{
        match $value {
            Value::Bool(b) => b,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Bool",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Symbol) => {{
        match $value {
            Value::Symbol(s) => s,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Symbol",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Nil) => {{
        match $value {
            Value::Nil => (),
            other => {
                return Err(RuntimeError::Type {
                    expected: "Nil",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Instance) => {{
        match $value {
            Value::Instance(inst) => inst,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Instance",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Class) => {{
        match $value {
            Value::Class(class) => class,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Class",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Method) => {{
        match $value {
            Value::Method(method) => method,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Method",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }}; // ($value:expr, $type:ident) => {{
        //     return Err(RuntimeError::Type {
        //         expected: stringify!($type),
        //         found: $value.type_name(),
        //     }
        //     .into());
        // }};
}

// #[macro_export]
// macro_rules! ensure_instance_of {
//     ($vm:expr, $val:expr, $class_id:expr) => {{
//         let inst = expect!($val, Instance);
//         if inst.borrow().class().borrow().symbol != $class_id {
//             return Err(RuntimeError::Type {
//                 expected: $vm.symbol_to_string($class_id).borrow().as_str(),
//                 found: inst.borrow().class().borrow().name.as_str(),
//             }
//             .into());
//         }
//         inst
//     }};
// }
