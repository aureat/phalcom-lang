use crate::method::{MemberVisibility, MethodKind, MethodObject, RestLayout, RestMode, SignatureKind};
use crate::primitive::attribute::{attribute_attach, attribute_attributes, attribute_freeze};
use crate::primitive::block::{block_arity, block_call_shape, block_call_with_shape, block_ensure, block_name, block_on, block_while_true};
use crate::primitive::boolean::{bool_and, bool_class_new, bool_hash, bool_if_false, bool_if_true, bool_if_true_if_false, bool_not, bool_or};
use crate::primitive::bytes::{
    bytes_class_from_string, bytes_class_new, bytes_raw_at, bytes_raw_copy_into, bytes_raw_equals_constant_time, bytes_raw_fill, bytes_raw_set, bytes_raw_size,
    bytes_raw_slice, bytes_raw_utf8, bytes_raw_utf8_lossy,
};
use crate::primitive::class::{behavior_extract_shape, behavior_methods, behavior_name, class_add, class_new_, class_set_superclass, class_superclass};
use crate::primitive::error::{error_message, error_raise};
use crate::primitive::family::{family_get, family_is_exact, family_pattern, family_receiver, family_selector, family_set};
use crate::primitive::fiber::{fiber_abort, fiber_call, fiber_current, fiber_error, fiber_is_done, fiber_is_root, fiber_new, fiber_try, fiber_yield};
use crate::primitive::float::{
    float_abs, float_ceil, float_class_new, float_floor, float_is_finite, float_is_infinite, float_is_integer, float_is_nan, float_rounded, float_sign,
    float_to_int_exact, float_truncated,
};
use crate::primitive::int::{
    int_and, int_bit_at, int_bit_count, int_bit_length, int_class_new, int_not, int_or, int_shl, int_shr, int_trailing_zeros, int_xor,
};
use crate::primitive::list::{list_class_new, list_raw_at, list_raw_length, list_raw_push, list_raw_set, list_replace_slice, list_to_string};
use crate::primitive::map::{map_class_new, map_raw_get, map_raw_has, map_raw_key_at, map_raw_put, map_raw_remove, map_raw_size, map_raw_value_at};
use crate::primitive::method::{method_bind, method_class_new, method_holder, method_invoke_on_shape, method_selector};
use crate::primitive::method_family::{method_family_bind, method_family_method_for_shape, method_family_selectors, method_family_size};
use crate::primitive::module::{module_class_new, module_does_not_understand};
use crate::primitive::nil::{option_match, some_call, some_new};
use crate::primitive::number::{
    number_add, number_class_new, number_compare, number_div, number_floor_div, number_ge, number_gt, number_hash, number_le, number_lt, number_mod, number_mul, number_negate,
    number_pow, number_sub, number_to_string, number_unary_plus,
};
use crate::primitive::object::{
    message_args, message_labels, message_name, message_selector, object_class, object_does_not_understand, object_eq, object_hash, object_invariant_enter,
    object_invariant_exit, object_matches, object_method_for, object_name, object_neq, object_perform_shape, object_responds_to, object_same, object_set_class,
    object_to_string, object_understands,
};
use crate::primitive::primitive;
use crate::primitive::primitive_internal;
use crate::primitive::primitive_rest;
use crate::primitive::primitive_shape;
use crate::primitive::primitive_static;
use crate::primitive::primitive_static_internal;
use crate::primitive::range::{range_raw_lower, range_raw_upper, range_raw_upper_inclusive};
use crate::primitive::record::{record_raw_label_at, record_raw_size, record_raw_value_at};
use crate::primitive::reflection::*;
use crate::primitive::set::{set_class_new, set_raw_add, set_raw_at, set_raw_has, set_raw_remove, set_raw_size};
use crate::primitive::selector_pattern::selector_pattern_matches;
use crate::primitive::string::{string_add, string_class_new, string_hash, string_raw_byte_at, string_raw_byte_count, string_raw_slice};
use crate::primitive::symbol::{symbol_class_new, symbol_hash, symbol_tostring};
use crate::primitive::system::{system_class_new, system_class_print, system_gc, system_next_scheduled, system_raw_write, system_schedule};
use crate::primitive::tuple::{
    tuple_from_list_internal, tuple_raw_at, tuple_raw_label_at, tuple_raw_labeled, tuple_raw_positional_size, tuple_raw_positionals, tuple_raw_size,
    tuple_raw_slice,
};
use crate::vm::VM;
use phalcom_native_surface::{NATIVE_MEMBERS, NativeDispatch, NativeMemberKind, NativeVisibility};
use std::collections::BTreeSet;

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
        // U5 (control-flow.md §1): `==`/`!=` are ordinary sends, not opcodes.
        primitive!(vm, object_cls, "==", SignatureKind::Method(1), object_eq);
        primitive!(vm, object_cls, "!=", SignatureKind::Method(1), object_neq);
        primitive!(vm, object_cls, "===", SignatureKind::Method(1), object_same);
        primitive!(vm, object_cls, "matches", SignatureKind::Method(1), object_matches);
        primitive!(vm, object_cls, "understands", SignatureKind::Method(1), object_understands);
        // Reflective-send + miss-handler surface (U8, messages-and-selectors.md
        // §5, method-lookup.md §2, ADR-0012). `doesNotUnderstand(_:)` is the
        // terminal miss fallback the `Bytecode::Invoke` handler forwards to;
        // it is an ordinary overridable method so a proxy subclass can
        // intercept. `respondsTo(_:)` is a pure probe that never triggers dNU.
        primitive_rest!(
            vm,
            object_cls,
            "perform",
            "perform(_,***)",
            SignatureKind::Method(1),
            1,
            RestLayout::new(1, Box::new([]), RestMode::Complete { param_index: 0 }),
            object_perform_shape
        );
        primitive!(vm, object_cls, "respondsTo", SignatureKind::Method(1), object_responds_to);
        primitive!(vm, object_cls, "doesNotUnderstand", SignatureKind::Method(1), object_does_not_understand);
        // `Method` reflection surface (U-CORE-3, ADR-0028): reifies the
        // resolved `MethodObject` for a selector, a pure probe like
        // `respondsTo` (never fires dNU on a miss).
        primitive!(vm, object_cls, "methodFor", SignatureKind::Method(1), object_method_for);
        // `@invariant` re-entrancy guard (U-ANNOT-CONTRACTS, ADR-0052 Fix 1):
        // a receiver-scoped native pair the woven prologue/epilogue call —
        // never `.ph`-authored, never part of any public protocol.
        primitive_internal!(vm, object_cls, "_$invariantEnter", SignatureKind::Method(0), object_invariant_enter);
        primitive_internal!(vm, object_cls, "_$invariantExit", SignatureKind::Method(0), object_invariant_exit);
        // Attribute-retention store (M-ATTR-ROOT, `attribute-classes.md`):
        // registered once here since `Object` sits under every class/method/
        // module row in the metaclass tower (ADR-0002 parallel rule) — one
        // site covers all three receiver kinds.
        primitive_internal!(vm, object_cls, "_$attributes", SignatureKind::Getter, attribute_attributes);
        primitive_internal!(vm, object_cls, "_$attach", SignatureKind::Method(1), attribute_attach);
        primitive_internal!(vm, object_cls, "_$freezeAttributes", SignatureKind::Method(0), attribute_freeze);

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
        primitive_shape!(vm, behavior_cls, ">>", SignatureKind::Method(1), behavior_extract_shape);

        let class_cls = vm.universe.classes.class_class;
        primitive!(vm, class_cls, "+", SignatureKind::Method(1), class_add);
        primitive_internal!(vm, class_cls, "_$new", SignatureKind::Method(0), class_new_);

        let number_cls = vm.universe.classes.number_class;
        // U5 (control-flow.md §1): every arithmetic/comparison operator is an
        // ordinary, non-sacred send — never inlined (U5-plan.md §4.1).
        primitive!(vm, number_cls, "+", SignatureKind::Method(1), number_add);
        primitive!(vm, number_cls, "-", SignatureKind::Method(1), number_sub);
        primitive!(vm, number_cls, "*", SignatureKind::Method(1), number_mul);
        primitive!(vm, number_cls, "/", SignatureKind::Method(1), number_div);
        primitive!(vm, number_cls, "%", SignatureKind::Method(1), number_mod);
        primitive!(vm, number_cls, "~/", SignatureKind::Method(1), number_floor_div);
        primitive!(vm, number_cls, "**", SignatureKind::Method(1), number_pow);
        primitive!(vm, number_cls, "<", SignatureKind::Method(1), number_lt);
        primitive!(vm, number_cls, "<=", SignatureKind::Method(1), number_le);
        primitive!(vm, number_cls, ">", SignatureKind::Method(1), number_gt);
        primitive!(vm, number_cls, ">=", SignatureKind::Method(1), number_ge);
        primitive!(vm, number_cls, "compare", SignatureKind::Method(1), number_compare);
        // Unary arithmetic getters: `+x` (identity) and `-x` (negation).
        // Registered as Getters so `-` and `+` bare-selectors dispatch without
        // parentheses (e.g. `x.-` == `-x`, `x.+` == `+x`).
        primitive!(vm, number_cls, "+", SignatureKind::Getter, number_unary_plus);
        primitive!(vm, number_cls, "-", SignatureKind::Getter, number_negate);
        // Value digest (ADR-0023): overrides `Object#hash` with a hash of the
        // mathematical value, class-agnostically (forward-compat §4).
        primitive!(vm, number_cls, "hash", SignatureKind::Getter, number_hash);
        // Decimal-string render (U-CORE-4, ADR-0019 amendment): unreachable
        // from `.ph` (no number->string path), so it earns the one new floor
        // binding this unit adds. Delegates to `Value::to_string`.
        primitive!(vm, number_cls, "toString", SignatureKind::Getter, number_to_string);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(0), number_class_new);
        primitive_static!(vm, number_cls, "new", SignatureKind::Method(1), number_class_new);

        let selector_pattern_cls = vm.universe.classes.selector_pattern_class;
        primitive!(vm, selector_pattern_cls, "matches", SignatureKind::Method(1), selector_pattern_matches);

        let int_cls = vm.universe.classes.int_class;
        primitive!(vm, int_cls, "&", SignatureKind::Method(1), int_and);
        primitive!(vm, int_cls, "|", SignatureKind::Method(1), int_or);
        primitive!(vm, int_cls, "^", SignatureKind::Method(1), int_xor);
        // Bitwise NOT getter: `~x` lowers to bare selector `~` (Getter).
        primitive!(vm, int_cls, "~", SignatureKind::Getter, int_not);
        primitive!(vm, int_cls, "<<", SignatureKind::Method(1), int_shl);
        primitive!(vm, int_cls, ">>", SignatureKind::Method(1), int_shr);
        primitive!(vm, int_cls, "bitAt", SignatureKind::Method(1), int_bit_at);
        primitive!(vm, int_cls, "bitCount", SignatureKind::Getter, int_bit_count);
        primitive!(vm, int_cls, "bitLength", SignatureKind::Getter, int_bit_length);
        primitive!(vm, int_cls, "trailingZeros", SignatureKind::Getter, int_trailing_zeros);
        primitive_static!(vm, int_cls, "new", SignatureKind::Method(0), int_class_new);
        primitive_static!(vm, int_cls, "new", SignatureKind::Method(1), int_class_new);

        let float_cls = vm.universe.classes.float_class;
        primitive_static!(vm, float_cls, "new", SignatureKind::Method(0), float_class_new);
        primitive_static!(vm, float_cls, "new", SignatureKind::Method(1), float_class_new);
        // Float protocol primitives (float-semantics spec):
        // `abs`/`sign` work on any finite-or-not receiver;
        // `floor`/`ceil`/`truncated`/`rounded`/`toIntExact` require finite.
        primitive!(vm, float_cls, "abs", SignatureKind::Getter, float_abs);
        primitive!(vm, float_cls, "sign", SignatureKind::Getter, float_sign);
        primitive!(vm, float_cls, "floor", SignatureKind::Getter, float_floor);
        primitive!(vm, float_cls, "ceil", SignatureKind::Getter, float_ceil);
        primitive!(vm, float_cls, "truncated", SignatureKind::Getter, float_truncated);
        primitive!(vm, float_cls, "rounded", SignatureKind::Getter, float_rounded);
        primitive!(vm, float_cls, "toIntExact", SignatureKind::Getter, float_to_int_exact);
        primitive!(vm, float_cls, "isInteger", SignatureKind::Getter, float_is_integer);
        primitive!(vm, float_cls, "isNaN", SignatureKind::Getter, float_is_nan);
        primitive!(vm, float_cls, "isFinite", SignatureKind::Getter, float_is_finite);
        primitive!(vm, float_cls, "isInfinite", SignatureKind::Getter, float_is_infinite);

        let string_cls = vm.universe.classes.string_class;
        primitive!(vm, string_cls, "+", SignatureKind::Method(1), string_add);
        // Value digest (ADR-0023): the cached djb2 CONTENT hash, so two
        // distinct-handle equal-content strings hash equal (R-INV-1.3).
        primitive!(vm, string_cls, "hash", SignatureKind::Getter, string_hash);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(0), string_class_new);
        primitive_static!(vm, string_cls, "new", SignatureKind::Method(1), string_class_new);
        // U-STRING raw byte-level accessors (ADR-0019 amendment, ADR-0049): minimal
        // native floor for string slicing/indexing — byte length, raw byte read,
        // and UTF-8-aware substring extraction. Not derivable in `.ph` (buffer
        // not observable, allocation not expressible).
        primitive_internal!(vm, string_cls, "_$byteCount", SignatureKind::Getter, string_raw_byte_count);
        primitive_internal!(vm, string_cls, "_$byteAt", SignatureKind::Method(1), string_raw_byte_at);
        primitive_internal!(vm, string_cls, "_$slice", SignatureKind::Method(2), string_raw_slice);

        let bool_cls = vm.universe.classes.bool_class;
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(0), bool_class_new);
        primitive_static!(vm, bool_cls, "new", SignatureKind::Method(1), bool_class_new);
        // Sacred selectors (control-flow.md §2–3): registered here as the
        // real send targets; ADR-0018's inliner special-cases literal-block
        // call sites but always deopts to exactly these on override/mismatch.
        primitive!(vm, bool_cls, "and", SignatureKind::Method(1), bool_and);
        primitive!(vm, bool_cls, "or", SignatureKind::Method(1), bool_or);
        primitive!(vm, bool_cls, "not", SignatureKind::Getter, bool_not);
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
            vm.world_version += 1;
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
        primitive_static!(vm, some_cls, "call", SignatureKind::Method(1), some_call);
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
            vm.world_version += 1;
        }

        let method_cls = vm.universe.classes.method_class;
        primitive_static!(vm, method_cls, "new", SignatureKind::Method(1), method_class_new);
        primitive!(vm, method_cls, "arity", SignatureKind::Getter, block_arity);
        primitive!(vm, method_cls, "name", SignatureKind::Getter, block_name);
        // `Method` reflection surface (U-CORE-3, ADR-0028): applying a
        // reified method to an explicit receiver, closing it over one, and
        // reading its selector/holder. See `primitive::method` module doc.
        primitive_rest!(
            vm,
            method_cls,
            "invokeOn",
            "invokeOn(_,***)",
            SignatureKind::Method(1),
            1,
            RestLayout::new(1, Box::new([]), RestMode::Complete { param_index: 1 }),
            method_invoke_on_shape
        );
        primitive!(vm, method_cls, "bind", SignatureKind::Method(1), method_bind);
        primitive!(vm, method_cls, "selector", SignatureKind::Getter, method_selector);
        primitive!(vm, method_cls, "holder", SignatureKind::Getter, method_holder);

        let family_cls = vm.universe.classes.family_class;
        primitive!(vm, family_cls, "receiver", SignatureKind::Getter, family_receiver);
        primitive!(vm, family_cls, "selector", SignatureKind::Getter, family_selector);
        primitive!(vm, family_cls, "pattern", SignatureKind::Getter, family_pattern);
        primitive!(vm, family_cls, "isExact", SignatureKind::Getter, family_is_exact);
        primitive_shape!(vm, family_cls, "get", SignatureKind::Method(0), family_get);
        primitive_shape!(vm, family_cls, "set", SignatureKind::Method(1), family_set);

        let method_family_cls = vm.universe.classes.method_family_class;
        primitive!(vm, method_family_cls, "selectors", SignatureKind::Getter, method_family_selectors);
        primitive!(vm, method_family_cls, "size", SignatureKind::Getter, method_family_size);
        primitive_shape!(vm, method_family_cls, "methodFor", SignatureKind::Method(1), method_family_method_for_shape);
        primitive!(vm, method_family_cls, "bind", SignatureKind::Method(1), method_family_bind);

        let function_cls = vm.universe.classes.function_class;
        primitive!(vm, function_cls, "arity", SignatureKind::Getter, block_arity);
        primitive!(vm, function_cls, "name", SignatureKind::Getter, block_name);
        primitive_shape!(vm, function_cls, "callWith", SignatureKind::Method(1), block_call_with_shape);
        primitive_rest!(
            vm,
            function_cls,
            "call",
            "call(***)",
            SignatureKind::Method(0),
            0,
            RestLayout::new(0, Box::new([]), RestMode::Complete { param_index: 0 }),
            block_call_shape
        );

        let block_cls = vm.universe.classes.closure_class;
        primitive!(vm, block_cls, "arity", SignatureKind::Getter, block_arity);
        primitive!(vm, block_cls, "name", SignatureKind::Getter, block_name);
        // Sacred loop fallback (control-flow.md §1/§3); `repeat(_)` is
        // deferred — its receiver/semantics aren't pinned by the spec
        // (U5-plan.md BD-U5-2) — see `docs/forge/DEFERRED.md`.
        primitive!(vm, block_cls, "whileTrue", SignatureKind::Method(1), block_while_true);
        // The error-handling catch protocol (U-ERR, ADR-0008,
        // [ADR-0038](../../docs/adr/accepted/0038-amend-floor-admit-block-on-ensure.md)):
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
        primitive_static!(vm, system_cls, "gc", SignatureKind::Getter, system_gc);
        // U-STRING raw I/O seam (ADR-0019 amendment, ADR-0049): raw stdout write of
        // an already-formed `String`, no newline, no formatting — the irreducible
        // literal I/O act that `System.write`/`writeObject_` funnel over.
        primitive_static_internal!(vm, system_cls, "_$write", SignatureKind::Method(1), system_raw_write);

        let module_cls = vm.universe.classes.module_class;
        primitive_static!(vm, module_cls, "new", SignatureKind::Method(0), module_class_new);
        primitive!(vm, module_cls, "doesNotUnderstand", SignatureKind::Method(1), module_does_not_understand);
        primitive!(vm, module_cls, "name", SignatureKind::Getter, module_name);
        primitive!(vm, module_cls, "namespace", SignatureKind::Getter, module_namespace);
        primitive!(vm, module_cls, "package", SignatureKind::Getter, module_package);
        primitive!(vm, module_cls, "rootPackage", SignatureKind::Getter, module_root_package);
        primitive!(vm, module_cls, "packageInfo", SignatureKind::Getter, module_package_info);
        primitive!(vm, module_cls, "exports", SignatureKind::Getter, module_exports);
        primitive!(vm, module_cls, "metadata", SignatureKind::Getter, module_metadata);
        primitive!(vm, module_cls, "dependencies", SignatureKind::Getter, module_dependencies);
        primitive!(vm, module_cls, "uri", SignatureKind::Getter, module_uri);
        primitive!(vm, module_cls, "identity", SignatureKind::Getter, module_identity);
        primitive!(vm, module_cls, "__exports__", SignatureKind::Getter, module_exports);
        primitive!(vm, module_cls, "__export__", SignatureKind::Method(1), module_export_by_name);
        primitive!(vm, module_cls, "__understands__", SignatureKind::Method(1), module_understands);
        primitive!(vm, module_cls, "__metadata__", SignatureKind::Getter, module_metadata);
        primitive!(vm, module_cls, "__dependencies__", SignatureKind::Getter, module_dependencies);
        primitive!(vm, module_cls, "__uri__", SignatureKind::Getter, module_uri);
        primitive!(vm, module_cls, "__name__", SignatureKind::Getter, module_name);
        primitive!(vm, module_cls, "__id__", SignatureKind::Getter, module_identity);
        primitive!(vm, module_cls, "__path__", SignatureKind::Getter, module_path);
        primitive!(vm, module_cls, "toString", SignatureKind::Getter, module_to_string);

        let package_cls = vm.universe.classes.package_class;
        primitive!(vm, package_cls, "package", SignatureKind::Getter, package_package);
        primitive!(vm, package_cls, "parentPackage", SignatureKind::Getter, package_parent_package);
        primitive!(vm, package_cls, "rootPackage", SignatureKind::Getter, package_root_package);
        primitive!(vm, package_cls, "packageInfo", SignatureKind::Getter, package_package_info);
        primitive!(vm, package_cls, "children", SignatureKind::Getter, package_children);
        primitive!(vm, package_cls, "isRoot", SignatureKind::Getter, package_is_root);
        primitive!(vm, package_cls, "__parent__", SignatureKind::Getter, package_parent_package);
        primitive!(vm, package_cls, "__children__", SignatureKind::Getter, package_children);
        primitive!(vm, package_cls, "__version__", SignatureKind::Getter, package_version);
        primitive!(vm, package_cls, "__namespace__", SignatureKind::Getter, package_namespace);
        primitive!(vm, package_cls, "toString", SignatureKind::Getter, package_to_string);

        let project_cls = vm.universe.classes.project_class;
        primitive!(vm, project_cls, "name", SignatureKind::Getter, project_name);
        primitive!(vm, project_cls, "namespace", SignatureKind::Getter, project_namespace);
        primitive!(vm, project_cls, "manifest", SignatureKind::Getter, project_manifest);
        primitive!(vm, project_cls, "rootPackage", SignatureKind::Getter, project_root_package);
        primitive!(vm, project_cls, "dependencies", SignatureKind::Getter, project_dependencies);
        primitive!(vm, project_cls, "developmentEntry", SignatureKind::Getter, project_development_entry);
        primitive!(vm, project_cls, "identity", SignatureKind::Getter, project_identity);
        primitive!(vm, project_cls, "toString", SignatureKind::Getter, project_to_string);

        let project_manifest_cls = vm.universe.classes.project_manifest_class;
        primitive!(vm, project_manifest_cls, "name", SignatureKind::Getter, project_manifest_name);
        primitive!(vm, project_manifest_cls, "namespace", SignatureKind::Getter, project_manifest_namespace);
        primitive!(vm, project_manifest_cls, "version", SignatureKind::Getter, project_manifest_version);
        primitive!(vm, project_manifest_cls, "authors", SignatureKind::Getter, project_manifest_authors);
        primitive!(vm, project_manifest_cls, "description", SignatureKind::Getter, project_manifest_description);
        primitive!(vm, project_manifest_cls, "license", SignatureKind::Getter, project_manifest_license);
        primitive!(vm, project_manifest_cls, "homepage", SignatureKind::Getter, project_manifest_homepage);
        primitive!(vm, project_manifest_cls, "repository", SignatureKind::Getter, project_manifest_repository);
        primitive!(vm, project_manifest_cls, "source", SignatureKind::Getter, project_manifest_source);
        primitive!(vm, project_manifest_cls, "entry", SignatureKind::Getter, project_manifest_entry);
        primitive!(vm, project_manifest_cls, "defaultEntry", SignatureKind::Getter, project_manifest_default_entry);
        primitive!(
            vm,
            project_manifest_cls,
            "dependencyDeclarations",
            SignatureKind::Getter,
            project_manifest_dependencies
        );
        primitive!(vm, project_manifest_cls, "dependencies", SignatureKind::Getter, project_manifest_dependencies);
        primitive!(vm, project_manifest_cls, "toString", SignatureKind::Getter, project_manifest_to_string);

        let package_info_cls = vm.universe.classes.package_info_class;
        primitive!(vm, package_info_cls, "name", SignatureKind::Getter, package_info_name);
        primitive!(vm, package_info_cls, "namespace", SignatureKind::Getter, package_info_namespace);
        primitive!(vm, package_info_cls, "version", SignatureKind::Getter, package_info_version);
        primitive!(vm, package_info_cls, "authors", SignatureKind::Getter, package_info_authors);
        primitive!(vm, package_info_cls, "description", SignatureKind::Getter, package_info_description);
        primitive!(vm, package_info_cls, "license", SignatureKind::Getter, package_info_license);
        primitive!(vm, package_info_cls, "homepage", SignatureKind::Getter, package_info_homepage);
        primitive!(vm, package_info_cls, "repository", SignatureKind::Getter, package_info_repository);
        primitive!(vm, package_info_cls, "requirements", SignatureKind::Getter, package_info_requirements);
        primitive!(vm, package_info_cls, "defaultEntry", SignatureKind::Getter, package_info_default_entry);
        primitive!(vm, package_info_cls, "identity", SignatureKind::Getter, package_info_identity);
        primitive!(vm, package_info_cls, "toString", SignatureKind::Getter, package_info_to_string);

        let package_author_cls = vm.universe.classes.package_author_class;
        primitive!(vm, package_author_cls, "name", SignatureKind::Getter, package_author_name);
        primitive!(vm, package_author_cls, "email", SignatureKind::Getter, package_author_email);
        primitive!(vm, package_author_cls, "url", SignatureKind::Getter, package_author_url);

        let package_requirement_cls = vm.universe.classes.package_requirement_class;
        primitive!(vm, package_requirement_cls, "alias", SignatureKind::Getter, package_requirement_alias);
        primitive!(vm, package_requirement_cls, "package", SignatureKind::Getter, package_requirement_package);
        primitive!(
            vm,
            package_requirement_cls,
            "versionRequirement",
            SignatureKind::Getter,
            package_requirement_version_requirement
        );
        primitive!(vm, package_requirement_cls, "optional", SignatureKind::Getter, package_requirement_optional);

        let resolved_project_dependency_cls = vm.universe.classes.resolved_project_dependency_class;
        primitive!(
            vm,
            resolved_project_dependency_cls,
            "alias",
            SignatureKind::Getter,
            resolved_project_dependency_alias
        );
        primitive!(
            vm,
            resolved_project_dependency_cls,
            "requirement",
            SignatureKind::Getter,
            resolved_project_dependency_requirement
        );
        primitive!(
            vm,
            resolved_project_dependency_cls,
            "packageInfo",
            SignatureKind::Getter,
            resolved_project_dependency_package_info
        );
        primitive!(
            vm,
            resolved_project_dependency_cls,
            "rootPackage",
            SignatureKind::Getter,
            resolved_project_dependency_root_package
        );
        primitive!(
            vm,
            resolved_project_dependency_cls,
            "origin",
            SignatureKind::Getter,
            resolved_project_dependency_origin
        );

        let module_dependency_cls = vm.universe.classes.module_dependency_class;
        primitive!(vm, module_dependency_cls, "module", SignatureKind::Getter, module_dependency_module);
        primitive!(vm, module_dependency_cls, "phase", SignatureKind::Getter, module_dependency_phase);
        primitive!(vm, module_dependency_cls, "reason", SignatureKind::Getter, module_dependency_reason);

        let export_table_cls = vm.universe.classes.export_table_class;
        primitive!(vm, export_table_cls, "names", SignatureKind::Getter, export_table_names);
        primitive!(vm, export_table_cls, "keys", SignatureKind::Getter, export_table_names);
        primitive!(vm, export_table_cls, "size", SignatureKind::Getter, export_table_size);
        primitive!(vm, export_table_cls, "contains", SignatureKind::Method(1), export_table_contains);
        primitive!(vm, export_table_cls, "descriptor", SignatureKind::Method(1), export_table_descriptor);
        primitive!(vm, export_table_cls, "get", SignatureKind::Method(1), export_table_get);

        let export_cls = vm.universe.classes.export_class;
        primitive!(vm, export_cls, "name", SignatureKind::Getter, export_name);
        primitive!(vm, export_cls, "kind", SignatureKind::Getter, export_kind);
        primitive!(vm, export_cls, "module", SignatureKind::Getter, export_module);
        primitive!(vm, export_cls, "value", SignatureKind::Getter, export_value);
        primitive!(vm, export_cls, "isModule", SignatureKind::Getter, export_is_module);
        primitive!(vm, export_cls, "isBinding", SignatureKind::Getter, export_is_binding);

        let child_module_table_cls = vm.universe.classes.child_module_table_class;
        primitive!(vm, child_module_table_cls, "names", SignatureKind::Getter, child_module_table_names);
        primitive!(vm, child_module_table_cls, "size", SignatureKind::Getter, child_module_table_size);
        primitive!(vm, child_module_table_cls, "contains", SignatureKind::Method(1), child_module_table_contains);
        primitive!(vm, child_module_table_cls, "get", SignatureKind::Method(1), child_module_table_get);

        let uri_cls = vm.universe.classes.uri_class;
        primitive!(vm, uri_cls, "toString", SignatureKind::Getter, uri_to_string);
        primitive!(vm, uri_cls, "==", SignatureKind::Method(1), uri_eq);

        let module_identity_cls = vm.universe.classes.module_identity_class;
        primitive!(vm, module_identity_cls, "uri", SignatureKind::Getter, module_identity_uri);
        primitive!(vm, module_identity_cls, "toString", SignatureKind::Getter, module_identity_to_string);

        let package_identity_cls = vm.universe.classes.package_identity_class;
        primitive!(vm, package_identity_cls, "toString", SignatureKind::Getter, package_identity_to_string);

        let project_identity_cls = vm.universe.classes.project_identity_class;
        primitive!(vm, project_identity_cls, "toString", SignatureKind::Getter, project_identity_to_string);

        // Kernel `List` (ADR-0019/0020): internal floor primitives use `_$`.
        // `_$length`/`_$at`/`_$set`/`_$push` are wrapped by `.ph`'s
        // `size`/`at(_:)`/`add(_:)` (`_$set` is
        // implemented but not yet surfaced, see DEFERRED.md); amortized growth
        // folds into `_$push` (`Vec::push`'s own doubling), so there is no
        // separate "grow" primitive. `new()` and `toString` are public
        // primitives directly, mirroring `String`'s `+(_)` (see the U-LIST
        // return contract for why `toString` is native this unit).
        let list_cls = vm.universe.classes.list_class;
        primitive_static!(vm, list_cls, "new", SignatureKind::Method(0), list_class_new);
        primitive_internal!(vm, list_cls, "_$length", SignatureKind::Getter, list_raw_length);
        primitive_internal!(vm, list_cls, "_$at", SignatureKind::Method(1), list_raw_at);
        primitive_internal!(vm, list_cls, "_$set", SignatureKind::Method(2), list_raw_set);
        primitive_internal!(vm, list_cls, "_$push", SignatureKind::Method(1), list_raw_push);
        primitive_internal!(vm, list_cls, "_$replaceSlice", SignatureKind::Method(3), list_replace_slice);
        primitive!(vm, list_cls, "toString", SignatureKind::Getter, list_to_string);

        // Kernel `Bytes` (PDR-0011 ruling 3, U-BYTES): implementation
        // selectors use `_$`. Bulk no-user-code operations are native
        // under the container bulk-op posture; every block-taking selector
        // (`each`/`map`/`filter`/`reduce`) stays `.ph`-inherited from
        // `Iterable` so `Fiber#yield` works mid-iteration (bytes.md §3.1) —
        // do NOT add native combinators here.
        let bytes_cls = vm.universe.classes.bytes_class;
        primitive_static!(vm, bytes_cls, "new", SignatureKind::Method(1), bytes_class_new);
        primitive_static_internal!(vm, bytes_cls, "_$fromString", SignatureKind::Method(1), bytes_class_from_string);
        primitive_internal!(vm, bytes_cls, "_$size", SignatureKind::Getter, bytes_raw_size);
        primitive_internal!(vm, bytes_cls, "_$at", SignatureKind::Method(1), bytes_raw_at);
        primitive_internal!(vm, bytes_cls, "_$set", SignatureKind::Method(2), bytes_raw_set);
        primitive_internal!(vm, bytes_cls, "_$fill", SignatureKind::Method(1), bytes_raw_fill);
        primitive_internal!(vm, bytes_cls, "_$slice", SignatureKind::Method(2), bytes_raw_slice);
        primitive_internal!(vm, bytes_cls, "_$copyInto", SignatureKind::Method(2), bytes_raw_copy_into);
        primitive_internal!(vm, bytes_cls, "_$utf8", SignatureKind::Getter, bytes_raw_utf8);
        primitive_internal!(vm, bytes_cls, "_$utf8Lossy", SignatureKind::Getter, bytes_raw_utf8_lossy);
        primitive_internal!(vm, bytes_cls, "_$equalsConstantTime", SignatureKind::Method(1), bytes_raw_equals_constant_time);

        // Kernel `Map`/`Set` (ADR-0039, U-COLLTYPES Phase 1): the native
        // hash-collection floor. `Map` gets 8 bindings (`new` + 7 native
        // instance ops); `Set` gets 6 (`new` + 5 native instance ops) — a keys-only
        // sibling reusing `Map`'s backing struct (DEC-CT-B) but with its own
        // distinct bindings. `_$keyAt`/`_$valueAt`/`_$at` back
        // `keys`/`values`/`each(_)`; `put_`/`add_` re-enter the VM to send
        // `hash`/`==` on keys (see `primitive::map`'s module doc) and reject a
        // mutable-collection key (DEC-CT-C). `.ph`'s public protocol
        // (`at(_)`/`at(_,put:)`/`size`/`includes(_)`/`remove(_)`/`keys`/
        // `values`/`each(_)`/`add(_)`) wraps these in `core.ph`.
        let map_cls = vm.universe.classes.map_class;
        primitive_static!(vm, map_cls, "new", SignatureKind::Method(0), map_class_new);
        primitive_internal!(vm, map_cls, "_$size", SignatureKind::Getter, map_raw_size);
        primitive_internal!(vm, map_cls, "_$get", SignatureKind::Method(1), map_raw_get);
        primitive_internal!(vm, map_cls, "_$put", SignatureKind::Method(2), map_raw_put);
        primitive_internal!(vm, map_cls, "_$has", SignatureKind::Method(1), map_raw_has);
        primitive_internal!(vm, map_cls, "_$remove", SignatureKind::Method(1), map_raw_remove);
        primitive_internal!(vm, map_cls, "_$keyAt", SignatureKind::Method(1), map_raw_key_at);
        primitive_internal!(vm, map_cls, "_$valueAt", SignatureKind::Method(1), map_raw_value_at);

        let set_cls = vm.universe.classes.set_class;
        primitive_static!(vm, set_cls, "new", SignatureKind::Method(0), set_class_new);
        primitive_internal!(vm, set_cls, "_$size", SignatureKind::Getter, set_raw_size);
        primitive_internal!(vm, set_cls, "_$add", SignatureKind::Method(1), set_raw_add);
        primitive_internal!(vm, set_cls, "_$has", SignatureKind::Method(1), set_raw_has);
        primitive_internal!(vm, set_cls, "_$remove", SignatureKind::Method(1), set_raw_remove);
        primitive_internal!(vm, set_cls, "_$at", SignatureKind::Method(1), set_raw_at);

        // Kernel `Tuple`: raw immutable lane observations only — no mutation.
        // immutability is structural (TupleObject's Box<[Value]>). `.ph`'s
        // `at(_)`/`size`/`each(_)`/`==`/`!=`/`hash` wrap these in `core.ph`.
        let tuple_cls = vm.universe.classes.tuple_class;
        primitive_static_internal!(vm, tuple_cls, "_$fromList", SignatureKind::Method(1), tuple_from_list_internal);
        primitive_internal!(vm, tuple_cls, "_$size", SignatureKind::Getter, tuple_raw_size);
        primitive_internal!(vm, tuple_cls, "_$at", SignatureKind::Method(1), tuple_raw_at);
        primitive_internal!(vm, tuple_cls, "_$positionalSize", SignatureKind::Getter, tuple_raw_positional_size);
        primitive_internal!(vm, tuple_cls, "_$labelAt", SignatureKind::Method(1), tuple_raw_label_at);
        primitive_internal!(vm, tuple_cls, "_$positionals", SignatureKind::Getter, tuple_raw_positionals);
        primitive_internal!(vm, tuple_cls, "_$labeled", SignatureKind::Getter, tuple_raw_labeled);
        primitive_internal!(vm, tuple_cls, "_$slice", SignatureKind::Method(2), tuple_raw_slice);

        let record_cls = vm.universe.classes.record_class;
        primitive_internal!(vm, record_cls, "_$size", SignatureKind::Getter, record_raw_size);
        primitive_internal!(vm, record_cls, "_$labelAt", SignatureKind::Method(1), record_raw_label_at);
        primitive_internal!(vm, record_cls, "_$valueAt", SignatureKind::Method(1), record_raw_value_at);

        // Range syntax uses BuildRange directly. The floor only observes its
        // optional bounds and inclusion bit.
        let range_cls = vm.universe.classes.range_class;
        primitive_internal!(vm, range_cls, "_$lower", SignatureKind::Getter, range_raw_lower);
        primitive_internal!(vm, range_cls, "_$upper", SignatureKind::Getter, range_raw_upper);
        primitive_internal!(vm, range_cls, "_$upperInclusive", SignatureKind::Getter, range_raw_upper_inclusive);

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
        // The predicate form of `Fiber::yield`'s root refusal, so `.ph` code can
        // ask "may I yield?" instead of attempting a yield and inspecting the
        // wreckage — see `fiber_is_root`'s doc and E004.
        primitive!(vm, fiber_cls, "isRoot", SignatureKind::Getter, fiber_is_root);
        primitive!(vm, fiber_cls, "error", SignatureKind::Getter, fiber_error);

        // U-RESOURCE primitives
        let resource_cls = vm.universe.classes.resource_class;
        primitive_static_internal!(
            vm,
            resource_cls,
            "_$register",
            SignatureKind::Method(1),
            crate::primitive::resource::resource_register
        );
        primitive_internal!(
            vm,
            resource_cls,
            "_$close",
            SignatureKind::Method(0),
            crate::primitive::resource::resource_raw_close
        );
        primitive_internal!(
            vm,
            resource_cls,
            "_$isClosed",
            SignatureKind::Getter,
            crate::primitive::resource::resource_raw_is_closed
        );

        let system_cls = vm.universe.classes.system_class;
        primitive_static_internal!(
            vm,
            system_cls,
            "_$leakReport",
            SignatureKind::Getter,
            crate::primitive::resource::system_leak_report
        );
        primitive_static_internal!(
            vm,
            system_cls,
            "_$strictResources",
            SignatureKind::Method(1),
            crate::primitive::resource::system_strict_resources
        );

        validate_native_surface(vm);
    }
}

/// Verifies that runtime primitive registration agrees with the canonical
/// VM-free descriptor surface consumed by editor semantics.
fn validate_native_surface(vm: &VM) {
    type NativeKey = (String, String, NativeDispatch, NativeMemberKind, NativeVisibility);

    let expected = NATIVE_MEMBERS
        .iter()
        .map(|member| {
            (
                member.class.to_string(),
                member.selector.to_string(),
                member.side,
                member.kind,
                member.visibility,
            )
        })
        .collect::<BTreeSet<NativeKey>>();
    let classes = vm.universe.classes;
    let rows = [
        classes.object_class,
        classes.behavior_class,
        classes.class_class,
        classes.metaclass_class,
        classes.number_class,
        classes.int_class,
        classes.float_class,
        classes.string_class,
        classes.nil_class,
        classes.bool_class,
        classes.true_class,
        classes.false_class,
        classes.method_class,
        classes.function_class,
        classes.closure_class,
        classes.bound_method_class,
        classes.method_family_class,
        classes.bound_method_family_class,
        classes.symbol_class,
        classes.selector_pattern_class,
        classes.module_class,
        classes.system_class,
        classes.option_class,
        classes.some_class,
        classes.none_class,
        classes.unit_class,
        classes.iterable_class,
        classes.list_class,
        classes.map_class,
        classes.set_class,
        classes.tuple_class,
        classes.record_class,
        classes.range_class,
        classes.bytes_class,
        classes.message_class,
        classes.error_class,
        classes.message_not_understood_class,
        classes.fiber_class,
        classes.cannot_yield_across_native_frame_class,
        classes.family_class,
        classes.resource_class,
        classes.use_after_close_error_class,
        classes.package_class,
        classes.project_class,
        classes.project_manifest_class,
        classes.package_info_class,
        classes.package_author_class,
        classes.package_requirement_class,
        classes.resolved_project_dependency_class,
        classes.module_dependency_class,
        classes.export_table_class,
        classes.export_class,
        classes.child_module_table_class,
        classes.uri_class,
        classes.module_identity_class,
        classes.package_identity_class,
        classes.project_identity_class,
    ];
    let mut actual = BTreeSet::new();
    for class in rows {
        let class_name = vm.heap.class(class).name.clone();
        for (symbol, method_id) in &vm.heap.class(class).methods {
            let method = vm.heap.method(*method_id);
            let MethodKind::Primitive(_) = method.kind else { continue };
            actual.insert((
                class_name.clone(),
                vm.resolve_symbol(*symbol).to_string(),
                NativeDispatch::Instance,
                native_kind(method.signature.kind),
                native_visibility(method.visibility),
            ));
        }

        let meta = vm.heap.class(class).class;
        for (symbol, method_id) in &vm.heap.class(meta).methods {
            let method = vm.heap.method(*method_id);
            let MethodKind::Primitive(_) = method.kind else { continue };
            actual.insert((
                class_name.clone(),
                vm.resolve_symbol(*symbol).to_string(),
                NativeDispatch::Class,
                native_kind(method.signature.kind),
                native_visibility(method.visibility),
            ));
        }
    }
    if actual != expected {
        let missing = expected.difference(&actual).collect::<Vec<_>>();
        let unexpected = actual.difference(&expected).collect::<Vec<_>>();
        panic!("runtime primitive registration drifted from phalcom-native-surface; missing={missing:?}; unexpected={unexpected:?}");
    }
}

fn native_kind(kind: SignatureKind) -> NativeMemberKind {
    match kind {
        SignatureKind::Getter => NativeMemberKind::Getter,
        SignatureKind::Setter => NativeMemberKind::Setter,
        SignatureKind::Method(_) | SignatureKind::SubscriptGet(_) | SignatureKind::SubscriptSet(_) => NativeMemberKind::Method,
    }
}

fn native_visibility(visibility: MemberVisibility) -> NativeVisibility {
    match visibility {
        MemberVisibility::Public => NativeVisibility::Public,
        MemberVisibility::Private | MemberVisibility::Protected | MemberVisibility::Internal => NativeVisibility::Internal,
    }
}
