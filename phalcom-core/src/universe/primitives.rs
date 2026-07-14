use crate::method::MethodObject;
use crate::method::SignatureKind;
use crate::primitive::attribute::{attribute_attach, attribute_attributes, attribute_freeze};
use crate::primitive::boolean::{bool_and, bool_class_new, bool_hash, bool_if_false, bool_if_true, bool_if_true_if_false, bool_not, bool_or};
use crate::primitive::block::{block_arity, block_call, block_call_with, block_ensure, block_name, block_on, block_while_true};
use crate::primitive::class::{behavior_methods, behavior_name, class_add, class_new, class_set_superclass, class_superclass};
use crate::primitive::error::{error_message, error_raise};
use crate::primitive::family::family_does_not_understand;
use crate::primitive::fiber::{fiber_abort, fiber_call, fiber_current, fiber_error, fiber_is_done, fiber_new, fiber_try, fiber_yield};
use crate::primitive::list::{list_class_new, list_raw_at, list_raw_length, list_raw_push, list_raw_set, list_to_string};
use crate::primitive::map::{map_class_new, map_raw_get, map_raw_has, map_raw_key_at, map_raw_put, map_raw_remove, map_raw_size, map_raw_value_at};
use crate::primitive::method::{method_bind, method_class_new, method_holder, method_invoke_on, method_selector};
use crate::primitive::module::{module_class_new, module_does_not_understand};
use crate::primitive::nil::{option_match, some_new};
use crate::primitive::number::{
    number_add, number_class_new, number_div, number_ge, number_gt, number_hash, number_le, number_lt, number_mod, number_mul, number_negated, number_sub,
    number_to_string,
};
use crate::primitive::object::{
    message_args, message_labels, message_name, message_selector, object_class, object_class_new, object_does_not_understand, object_eq, object_hash,
    object_invariant_enter, object_invariant_exit, object_method_for, object_name, object_neq, object_perform, object_perform_with, object_responds_to,
    object_set_class, object_to_string,
};
use crate::primitive::primitive;
use crate::primitive::primitive_static;
use crate::primitive::range::{range_class_new, range_raw_end, range_raw_inclusive, range_raw_start};
use crate::primitive::set::{set_class_new, set_raw_add, set_raw_at, set_raw_has, set_raw_remove, set_raw_size};
use crate::primitive::string::{string_add, string_class_new, string_hash};
use crate::primitive::tuple::{tuple_class_from_list, tuple_raw_at, tuple_raw_size};
use crate::primitive::symbol::{symbol_class_new, symbol_hash, symbol_tostring};
use crate::primitive::system::{system_class_new, system_class_print, system_next_scheduled, system_schedule};
use crate::vm::VM;

use super::Universe;

impl Universe {
    /// Installs every native primitive method onto the core classes.
    pub fn install_primitives(vm: &mut VM) {
        let object_cls = vm.universe.classes.object_class;
        primitive!(vm, object_cls, "name", SignatureKind::Getter, object_name);
        primitive!(vm, object_cls, "class", SignatureKind::Getter, object_class);
        primitive!(vm, object_cls, "class", SignatureKind::Setter, object_set_class);
        primitive!(vm, object_cls, "toString", SignatureKind::Getter, object_to_string);
        // Identity `hash` (object-model.md §8, ADR-0023): immediates override it
        // per-type below; every heap object inherits this handle digest.
        primitive!(vm, object_cls, "hash", SignatureKind::Getter, object_hash);
        primitive_static!(vm, object_cls, "new", SignatureKind::Method(0), object_class_new);
        // U5 (control-flow.md §1): `==`/`!=` are ordinary sends, not opcodes.
        primitive!(vm, object_cls, "==", SignatureKind::Method(1), object_eq);
        primitive!(vm, object_cls, "!=", SignatureKind::Method(1), object_neq);
        // Reflective-send + miss-handler surface (U8, messages-and-selectors.md
        // §5, method-lookup.md §2, ADR-0012). `doesNotUnderstand(_:)` is the
        // terminal miss fallback the `Bytecode::Invoke` handler forwards to;
        // it is an ordinary overridable method so a proxy subclass can
        // intercept. `respondsTo(_:)` is a pure probe that never triggers dNU.
        primitive!(vm, object_cls, "perform", SignatureKind::Method(1), object_perform);
        primitive!(vm, object_cls, "perform", SignatureKind::Method(2), object_perform_with);
        primitive!(vm, object_cls, "respondsTo", SignatureKind::Method(1), object_responds_to);
        primitive!(vm, object_cls, "doesNotUnderstand", SignatureKind::Method(1), object_does_not_understand);
        // `Method` reflection surface (U-CORE-3, ADR-0028): reifies the
        // resolved `MethodObject` for a selector, a pure probe like
        // `respondsTo` (never fires dNU on a miss).
        primitive!(vm, object_cls, "methodFor", SignatureKind::Method(1), object_method_for);
        // `@invariant` re-entrancy guard (U-ANNOT-CONTRACTS, ADR-0052 Fix 1):
        // a receiver-scoped native pair the woven prologue/epilogue call —
        // never `.ph`-authored, never part of any public protocol.
        primitive!(vm, object_cls, "__invariantEnter", SignatureKind::Method(0), object_invariant_enter);
        primitive!(vm, object_cls, "__invariantExit", SignatureKind::Method(0), object_invariant_exit);
        // Attribute-retention store (M-ATTR-ROOT, `attribute-classes.md`):
        // registered once here since `Object` sits under every class/method/
        // module row in the metaclass tower (ADR-0002 parallel rule) — one
        // site covers all three receiver kinds.
        primitive!(vm, object_cls, "__attributes", SignatureKind::Getter, attribute_attributes);
        primitive!(vm, object_cls, "__attach", SignatureKind::Method(1), attribute_attach);
        primitive!(vm, object_cls, "__freezeAttributes", SignatureKind::Method(0), attribute_freeze);

        // `Message` accessors (U8): native getters reading the reified-send
        // slots directly (`VM::new_message`); `Message` has no `.ph` surface.
        // `name` deliberately shadows `Object::name` on a `Message` receiver —
        // it returns the sent *method* name, not the class name.
        let message_cls = vm.universe.classes.message_class;
        primitive!(vm, message_cls, "selector", SignatureKind::Getter, message_selector);
        primitive!(vm, message_cls, "name", SignatureKind::Getter, message_name);
        primitive!(vm, message_cls, "labels", SignatureKind::Getter, message_labels);
        primitive!(vm, message_cls, "args", SignatureKind::Getter, message_args);

        let behavior_cls = vm.universe.classes.behavior_class;
        primitive!(vm, behavior_cls, "superclass", SignatureKind::Getter, class_superclass);
        primitive!(vm, behavior_cls, "superclass", SignatureKind::Setter, class_set_superclass);
        // Class-side reflection (ADR-0023): `name` returns the receiver class's
        // OWN name (shadowing `Object#name`, which yields the *metaclass* name
        // "C class" for a class receiver); `methods` enumerates the receiver's
        // own method dictionary as selector Symbols. Installed on `Behavior` so
        // `Class` and `Metaclass` both inherit them (mirrors `superclass`).
        primitive!(vm, behavior_cls, "name", SignatureKind::Getter, behavior_name);
        primitive!(vm, behavior_cls, "methods", SignatureKind::Getter, behavior_methods);

        let class_cls = vm.universe.classes.class_class;
        primitive!(vm, class_cls, "+", SignatureKind::Method(1), class_add);
        primitive!(vm, class_cls, "new", SignatureKind::Method(0), class_new);

        let number_cls = vm.universe.classes.number_class;
        // U5 (control-flow.md §1): every arithmetic/comparison operator is an
        // ordinary, non-sacred send — never inlined (U5-plan.md §4.1).
        primitive!(vm, number_cls, "+", SignatureKind::Method(1), number_add);
        primitive!(vm, number_cls, "-", SignatureKind::Method(1), number_sub);
        primitive!(vm, number_cls, "*", SignatureKind::Method(1), number_mul);
        primitive!(vm, number_cls, "/", SignatureKind::Method(1), number_div);
        primitive!(vm, number_cls, "%", SignatureKind::Method(1), number_mod);
        primitive!(vm, number_cls, "<", SignatureKind::Method(1), number_lt);
        primitive!(vm, number_cls, "<=", SignatureKind::Method(1), number_le);
        primitive!(vm, number_cls, ">", SignatureKind::Method(1), number_gt);
        primitive!(vm, number_cls, ">=", SignatureKind::Method(1), number_ge);
        primitive!(vm, number_cls, "negated", SignatureKind::Method(0), number_negated);
        // Value digest (ADR-0023): overrides `Object#hash` with a hash of the
        // mathematical value, class-agnostically (forward-compat §4).
        primitive!(vm, number_cls, "hash", SignatureKind::Getter, number_hash);
        // Decimal-string render (U-CORE-4, ADR-0019 amendment): unreachable
        // from `.ph` (no number->string path), so it earns the one new floor
        // binding this unit adds. Delegates to `Value::to_string`.
        primitive!(vm, number_cls, "toString", SignatureKind::Getter, number_to_string);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(0), number_class_new);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(1), number_class_new);

        let string_cls = vm.universe.classes.string_class;
        primitive!(vm, string_cls, "+", SignatureKind::Method(1), string_add);
        // Value digest (ADR-0023): the cached djb2 CONTENT hash, so two
        // distinct-handle equal-content strings hash equal (R-INV-1.3).
        primitive!(vm, string_cls, "hash", SignatureKind::Getter, string_hash);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(0), string_class_new);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(1), string_class_new);

        let bool_cls = vm.universe.classes.bool_class;
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(0), bool_class_new);
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(1), bool_class_new);
        // Sacred selectors (control-flow.md §2–3): registered here as the
        // real send targets; ADR-0018's inliner special-cases literal-block
        // call sites but always deopts to exactly these on override/mismatch.
        primitive!(vm, bool_cls, "and", SignatureKind::Method(1), bool_and);
        primitive!(vm, bool_cls, "or", SignatureKind::Method(1), bool_or);
        primitive!(vm, bool_cls, "not", SignatureKind::Method(0), bool_not);
        primitive!(vm, bool_cls, "ifTrue", SignatureKind::Method(1), bool_if_true);
        primitive!(vm, bool_cls, "ifFalse", SignatureKind::Method(1), bool_if_false);
        // control-flow.md §3's `ifTrue(_)ifFalse(_)` is Smalltalk's
        // independently-worded `ifTrue:ifFalse:` keyword pair; Phalcom's
        // selector model (ADR-0012) has no such shape — one base name plus
        // per-argument *labels* on that name. U5 realizes the same paired
        // conditional as `ifTrue(_:ifFalse:)`: one positional arg plus one
        // `ifFalse:`-labeled arg, fully expressible via the existing
        // `encode_selector`, and is what `if/else` desugars to (ADR-0018).
        {
            let sig_str = crate::method::encode_selector("ifTrue", &[None, Some("ifFalse".to_string())], SignatureKind::Method(2));
            let symbol = vm.get_or_intern(&sig_str);
            let method = MethodObject::new_primitive(symbol, SignatureKind::Method(2), bool_if_true_if_false, bool_cls);
            let method_id = vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
            vm.heap.class_mut(bool_cls).add_method(symbol, method_id);
        }
        // Value digest (ADR-0023): 1 for `true`, 0 for `false` — distinct and
        // stable. NOT a sacred selector (no deopt budget; see `bool_hash`).
        primitive!(vm, bool_cls, "hash", SignatureKind::Getter, bool_hash);

        let symbol_cls = vm.universe.classes.symbol_class;
        primitive!(vm, symbol_cls, "toString", SignatureKind::Getter, symbol_tostring);
        // Value digest (ADR-0023): the interned id, so equal symbols agree.
        primitive!(vm, symbol_cls, "hash", SignatureKind::Getter, symbol_hash);
        primitive_static!(vm, symbol_cls, "new", SignatureKind::Method(1), symbol_class_new);

        // Absence substrate (ADR-0007): `Some(_)` construction and the
        // `match(some:none:)` eliminator. Bootstrapped as Rust primitives so U6
        // does not depend on U7's user-facing `construct`. The rich combinator
        // suite (`map`/`flatMap`/`orElse`/…) is U-STD's job, defined over
        // `match` in `core.ph`. There is intentionally no surface `Nil` class or
        // `nil` constructor — the private `Value::Nil` sentinel (ADR-0010) is
        // surfaced to `None` at read boundaries only.
        let some_cls = vm.universe.classes.some_class;
        primitive_static!(vm, some_cls, "new", SignatureKind::Method(1), some_new);

        // `match(some:none:)` is installed on the abstract `Option` class so
        // both `Some` and `None` inherit it (values-and-absence.md §3.2). Both
        // arguments are labeled, so `make_signature` (label-free) can't build
        // the selector; encode it explicitly like `ifTrue(_:ifFalse:)`.
        let option_cls = vm.universe.classes.option_class;
        {
            let sig_str = crate::method::encode_selector("match", &[Some("some".to_string()), Some("none".to_string())], SignatureKind::Method(2));
            let symbol = vm.get_or_intern(&sig_str);
            let method = MethodObject::new_primitive(symbol, SignatureKind::Method(2), option_match, option_cls);
            let method_id = vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
            vm.heap.class_mut(option_cls).add_method(symbol, method_id);
        }

        let method_cls = vm.universe.classes.method_class;
        primitive_static!(vm, method_cls, "new", SignatureKind::Method(1), method_class_new);
        // `Method` reflection surface (U-CORE-3, ADR-0028): applying a
        // reified method to an explicit receiver, closing it over one, and
        // reading its selector/holder. See `primitive::method` module doc.
        primitive!(vm, method_cls, "invokeOn", SignatureKind::Method(2), method_invoke_on);
        primitive!(vm, method_cls, "bind", SignatureKind::Method(1), method_bind);
        primitive!(vm, method_cls, "selector", SignatureKind::Getter, method_selector);
        primitive!(vm, method_cls, "holder", SignatureKind::Getter, method_holder);

        // `call` is registered per arity (functions.md §1: `call`, `call(_:)`,
        // `call(_:_:)`, …) since Phalcom dispatch keys on the arity-encoded
        // selector, not a single variadic entry point. `callWith(_:)` takes one
        // packed argument (deferred to a plain forward until `List` lands, see
        // `docs/forge/DEFERRED.md`).
        const MAX_CALL_ARITY: u8 = 4;

        let function_cls = vm.universe.classes.function_class;
        primitive!(vm, function_cls, "arity", SignatureKind::Getter, block_arity);
        primitive!(vm, function_cls, "name", SignatureKind::Getter, block_name);
        primitive!(vm, function_cls, "callWith", SignatureKind::Method(1), block_call_with);
        for n in 0..=MAX_CALL_ARITY {
            primitive!(vm, function_cls, "call", SignatureKind::Method(n), block_call);
        }

        let block_cls = vm.universe.classes.block_class;
        primitive!(vm, block_cls, "arity", SignatureKind::Getter, block_arity);
        primitive!(vm, block_cls, "name", SignatureKind::Getter, block_name);
        primitive!(vm, block_cls, "callWith", SignatureKind::Method(1), block_call_with);
        for n in 0..=MAX_CALL_ARITY {
            primitive!(vm, block_cls, "call", SignatureKind::Method(n), block_call);
        }
        // Sacred loop fallback (control-flow.md §1/§3); `repeat(_)` is
        // deferred — its receiver/semantics aren't pinned by the spec
        // (U5-plan.md BD-U5-2) — see `docs/forge/DEFERRED.md`.
        primitive!(vm, block_cls, "whileTrue", SignatureKind::Method(1), block_while_true);
        // The error-handling catch protocol (U-ERR, ADR-0008,
        // [ADR-0038](../../docs/adr/0038-amend-floor-admit-block-on-ensure.md)):
        // the +2 floor bindings this unit amends the frozen floor to admit.
        // `try`/`on`/`catch`/`ensure` (ADR-0031) are parser sugar over these
        // two sends — see `primitive::block::block_on`/`block_ensure` for the
        // exact catch/cleanup semantics. Installed on `Block` only (mirroring
        // `whileTrue`, not `call`/`arity`/`name`/`callWith`): every `on`/
        // `ensure` receiver at a `try` desugar site or inside `Function#attempt`
        // is always a literal `{ }` block, never a bare `Function`.
        primitive!(vm, block_cls, "on", SignatureKind::Method(2), block_on);
        primitive!(vm, block_cls, "ensure", SignatureKind::Method(1), block_ensure);

        let system_cls = vm.universe.classes.system_class;
        primitive_static!(vm, system_cls, "print", SignatureKind::Method(1), system_class_print);
        primitive_static!(vm, system_cls, "new", SignatureKind::Method(0), system_class_new);
        // Native ready-queue scheduler seam (U-SCHED, floor-census.md
        // amendment): `schedule(_)` wraps a `Function` as a fresh `Fiber` and
        // enqueues it on `VM::ready_queue`; `nextScheduled` pops the FIFO's
        // front as `Option<Fiber>`. Neither runs the fiber — see
        // `primitive::system::system_schedule`/`system_next_scheduled` for
        // the drain contract and `VM::run`'s root-drive pump.
        primitive_static!(vm, system_cls, "schedule", SignatureKind::Method(1), system_schedule);
        primitive_static!(vm, system_cls, "nextScheduled", SignatureKind::Getter, system_next_scheduled);

        let module_cls = vm.universe.classes.module_class;
        primitive_static!(vm, module_cls, "new", SignatureKind::Method(0), module_class_new);
        // Member access as an ordinary send (object-model.md §4, U15
        // DEC-U15): overrides `Object`'s default miss handler so `math.pi`/
        // `math.distance(1, 2)` reach the module's own global table before
        // falling through to `MessageNotUnderstood` — see
        // `primitive::module::module_does_not_understand`.
        primitive!(vm, module_cls, "doesNotUnderstand", SignatureKind::Method(1), module_does_not_understand);

        // `Family` call router (selectors.md §3, U16-Open, ADR-0047): every
        // bare-call selector shape (`call()`, `call(_:)`, `call(to:duration:)`,
        // …) misses `Family`'s own (empty) method table and lands here, which
        // rebuilds the real selector from the family's base name + the missed
        // call's decoded labels and re-dispatches — see
        // `primitive::family::family_does_not_understand`.
        let family_cls = vm.universe.classes.family_class;
        primitive!(vm, family_cls, "doesNotUnderstand", SignatureKind::Method(1), family_does_not_understand);

        // Kernel `List` (ADR-0019/0020): five native floor primitives.
        // `length_`/`at_`/`set_`/`push_` are internal (U-NATIVE-MARKER: the
        // trailing `_` marks a native/private primitive selector) — `.ph`'s
        // `size`/`at(_:)`/`add(_:)` wrap the first three (`set_` is
        // implemented but not yet surfaced, see DEFERRED.md); amortized growth
        // folds into `push_` (`Vec::push`'s own doubling), so there is no
        // separate "grow" primitive. `new()` and `toString` are public
        // primitives directly, mirroring `String`'s `+(_)` (see the U-LIST
        // return contract for why `toString` is native this unit).
        let list_cls = vm.universe.classes.list_class;
        primitive_static!(vm, list_cls, "new", SignatureKind::Method(0), list_class_new);
        primitive!(vm, list_cls, "length_", SignatureKind::Getter, list_raw_length);
        primitive!(vm, list_cls, "at_", SignatureKind::Method(1), list_raw_at);
        primitive!(vm, list_cls, "set_", SignatureKind::Method(2), list_raw_set);
        primitive!(vm, list_cls, "push_", SignatureKind::Method(1), list_raw_push);
        primitive!(vm, list_cls, "toString", SignatureKind::Getter, list_to_string);

        // Kernel `Map`/`Set` (ADR-0039, U-COLLTYPES Phase 1): the native
        // hash-collection floor. `Map` gets 8 bindings (`new` + 7 native
        // instance ops); `Set` gets 6 (`new` + 5 native instance ops) — a keys-only
        // sibling reusing `Map`'s backing struct (DEC-CT-B) but with its own
        // distinct bindings. `keyAt_`/`valueAt_`/`at_` back
        // `keys`/`values`/`each(_)`; `put_`/`add_` re-enter the VM to send
        // `hash`/`==` on keys (see `primitive::map`'s module doc) and reject a
        // mutable-collection key (DEC-CT-C). `.ph`'s public protocol
        // (`at(_)`/`at(_,put:)`/`size`/`includes(_)`/`remove(_)`/`keys`/
        // `values`/`each(_)`/`add(_)`) wraps these in `core.ph`.
        let map_cls = vm.universe.classes.map_class;
        primitive_static!(vm, map_cls, "new", SignatureKind::Method(0), map_class_new);
        primitive!(vm, map_cls, "size_", SignatureKind::Getter, map_raw_size);
        primitive!(vm, map_cls, "get_", SignatureKind::Method(1), map_raw_get);
        primitive!(vm, map_cls, "put_", SignatureKind::Method(2), map_raw_put);
        primitive!(vm, map_cls, "has_", SignatureKind::Method(1), map_raw_has);
        primitive!(vm, map_cls, "remove_", SignatureKind::Method(1), map_raw_remove);
        primitive!(vm, map_cls, "keyAt_", SignatureKind::Method(1), map_raw_key_at);
        primitive!(vm, map_cls, "valueAt_", SignatureKind::Method(1), map_raw_value_at);

        let set_cls = vm.universe.classes.set_class;
        primitive_static!(vm, set_cls, "new", SignatureKind::Method(0), set_class_new);
        primitive!(vm, set_cls, "size_", SignatureKind::Getter, set_raw_size);
        primitive!(vm, set_cls, "add_", SignatureKind::Method(1), set_raw_add);
        primitive!(vm, set_cls, "has_", SignatureKind::Method(1), set_raw_has);
        primitive!(vm, set_cls, "remove_", SignatureKind::Method(1), set_raw_remove);
        primitive!(vm, set_cls, "at_", SignatureKind::Method(1), set_raw_at);

        // Kernel `Tuple` (ADR-0039, U-COLLTYPES Phase 2): 3 native floor
        // primitives (fromList/size_/at_) — no mutation primitive, since
        // immutability is structural (TupleObject's Box<[Value]>). `.ph`'s
        // `at(_)`/`size`/`each(_)`/`==`/`!=`/`hash` wrap these in `core.ph`.
        let tuple_cls = vm.universe.classes.tuple_class;
        primitive_static!(vm, tuple_cls, "fromList", SignatureKind::Method(1), tuple_class_from_list);
        primitive!(vm, tuple_cls, "size_", SignatureKind::Getter, tuple_raw_size);
        primitive!(vm, tuple_cls, "at_", SignatureKind::Method(1), tuple_raw_at);

        // Kernel `Range` (ADR-0039, U-COLLTYPES Phase 3): 4 native floor
        // primitives — the WHOLE floor (three bound-field reads + the
        // allocator). `.ph`'s `size`/`at(_)`/`includes(_)`/`first`/`last`/
        // `each(_)`/`toList`/`==`/`hash` are all derived over these +
        // `Number` arithmetic in `core.ph`.
        let range_cls = vm.universe.classes.range_class;
        primitive_static!(vm, range_cls, "new", SignatureKind::Method(3), range_class_new);
        primitive!(vm, range_cls, "start_", SignatureKind::Getter, range_raw_start);
        primitive!(vm, range_cls, "end_", SignatureKind::Getter, range_raw_end);
        primitive!(vm, range_cls, "inclusive_", SignatureKind::Getter, range_raw_inclusive);

        // `Error` root (U-CORE-6, ADR-0008): `message` is a native slot-0
        // accessor (mirrors `Message`'s accessors — a `.ph` getter over this
        // field would trip the read-before-write check); `raise` initiates
        // the unified unwind's `Raise` payload (`throw expr === expr.raise()`,
        // ADR-0031 §1). Installed only on `Error`, so a non-`Error` receiver
        // has no `raise` (R-INV-6.3). +2 floor bindings, ADR-0023-cleared.
        let error_cls = vm.universe.classes.error_class;
        primitive!(vm, error_cls, "message", SignatureKind::Getter, error_message);
        primitive!(vm, error_cls, "raise", SignatureKind::Method(0), error_raise);

        // `Fiber` (U-FIBER, ADR-0030): `new(_)` builds; `call`/`try` (arity
        // 0 or 1) resume; `yield`/`current`/`abort` are class-side (sent to
        // `Fiber` itself, not an instance).
        let fiber_cls = vm.universe.classes.fiber_class;
        primitive_static!(vm, fiber_cls, "new", SignatureKind::Method(1), fiber_new);
        primitive!(vm, fiber_cls, "call", SignatureKind::Method(0), fiber_call);
        primitive!(vm, fiber_cls, "call", SignatureKind::Method(1), fiber_call);
        primitive!(vm, fiber_cls, "try", SignatureKind::Method(0), fiber_try);
        primitive!(vm, fiber_cls, "try", SignatureKind::Method(1), fiber_try);
        primitive_static!(vm, fiber_cls, "yield", SignatureKind::Method(0), fiber_yield);
        primitive_static!(vm, fiber_cls, "yield", SignatureKind::Method(1), fiber_yield);
        primitive_static!(vm, fiber_cls, "current", SignatureKind::Getter, fiber_current);
        primitive_static!(vm, fiber_cls, "abort", SignatureKind::Method(1), fiber_abort);
        // U-FIBER-REFLECT: `isDone`/`error` are pure instance-side reads over
        // `FiberObject::status`/`result` — no scheduler dependency.
        primitive!(vm, fiber_cls, "isDone", SignatureKind::Getter, fiber_is_done);
        primitive!(vm, fiber_cls, "error", SignatureKind::Getter, fiber_error);
    }
}
