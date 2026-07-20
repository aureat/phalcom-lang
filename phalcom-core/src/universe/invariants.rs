use crate::heap::{ClassId, Heap};

use super::Universe;

impl Universe {
    /// Asserts the kernel tower's shape (`object-model.md` §5–6 step 7).
    ///
    /// Checks every apex `.class`/`.superclass` relationship in the §5 table
    /// plus the four sanity checks (§5): the closed metaclass loop, the
    /// parallel rule holding for an ordinary core class, and that every
    /// metaclass's superclass chain terminates. Called once from [`crate::vm::VM::new`]
    /// right after [`Universe::install_primitives`]; the caller
    /// `.expect()`s the result, since a malformed kernel cannot run any
    /// program correctly.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a description of the first violated invariant.
    pub fn verify_invariants(&self, heap: &Heap) -> Result<(), String> {
        let c = &self.classes;

        let object_metaclass = heap.class(c.object_class).class;
        let behavior_metaclass = heap.class(c.behavior_class).class;
        let class_metaclass = heap.class(c.class_class).class;
        let metaclass_metaclass = heap.class(c.metaclass_class).class;

        if object_metaclass == c.object_class {
            return Err("Object.class must not equal Object itself".to_string());
        }
        if heap.class(c.behavior_class).superclass != Some(c.object_class) {
            return Err("Behavior.superclass should be Object".to_string());
        }
        if heap.class(c.class_class).superclass != Some(c.behavior_class) {
            return Err("Class.superclass should be Behavior".to_string());
        }
        if heap.class(c.metaclass_class).superclass != Some(c.behavior_class) {
            return Err("Metaclass.superclass should be Behavior".to_string());
        }
        if heap.class(c.object_class).superclass.is_some() {
            return Err("Object.superclass should be None".to_string());
        }

        if heap.class(object_metaclass).class != c.metaclass_class {
            return Err("Object.class.class should be Metaclass".to_string());
        }
        if heap.class(behavior_metaclass).class != c.metaclass_class {
            return Err("Behavior.class.class should be Metaclass".to_string());
        }
        if heap.class(class_metaclass).class != c.metaclass_class {
            return Err("Class.class.class should be Metaclass".to_string());
        }
        if heap.class(metaclass_metaclass).class != c.metaclass_class {
            return Err("Metaclass.class.class should be Metaclass".to_string());
        }
        // The closed loop: Metaclass.class == Metaclass class, and
        // (Metaclass class).class == Metaclass.
        if heap.class(c.metaclass_class).class != metaclass_metaclass {
            return Err("Metaclass.class should be Metaclass class".to_string());
        }

        if heap.class(object_metaclass).superclass != Some(c.class_class) {
            return Err("Object.class.superclass should be Class".to_string());
        }
        if heap.class(behavior_metaclass).superclass != Some(object_metaclass) {
            return Err("Behavior.class.superclass should be Object.class".to_string());
        }
        if heap.class(class_metaclass).superclass != Some(behavior_metaclass) {
            return Err("Class.class.superclass should be Behavior.class".to_string());
        }
        if heap.class(metaclass_metaclass).superclass != Some(behavior_metaclass) {
            return Err("Metaclass.class.superclass should be Behavior.class".to_string());
        }

        // R-INV-0.2 — the parallel rule ([ADR-0002](../../../docs/adr/accepted/0002-metaclass-tower-parallel-rule.md))
        // holds for *every* ordinary (non-apex) core row, not just `Number`:
        // `X.class.superclass == X.superclass.class`. Includes the U11 `True`/
        // `False` rows (both resolve to `Bool class`) and the absence /
        // collection / message rows. Any newly-added row that breaks the rule
        // fails boot rather than silently mis-dispatching statics.
        let ordinary_rows: [(&str, ClassId); 25] = [
            ("Number", c.number_class),
            ("String", c.string_class),
            ("Nil", c.nil_class),
            ("Bool", c.bool_class),
            ("True", c.true_class),
            ("False", c.false_class),
            ("Method", c.method_class),
            ("Function", c.function_class),
            ("Block", c.block_class),
            ("Symbol", c.symbol_class),
            ("Module", c.module_class),
            ("System", c.system_class),
            ("Option", c.option_class),
            ("Some", c.some_class),
            ("None", c.none_class),
            ("Iterable", c.iterable_class),
            ("List", c.list_class),
            ("Map", c.map_class),
            ("Set", c.set_class),
            ("Tuple", c.tuple_class),
            ("Range", c.range_class),
            ("Bytes", c.bytes_class),
            ("Message", c.message_class),
            ("Error", c.error_class),
            ("MessageNotUnderstood", c.message_not_understood_class),
        ];
        for (name, class_id) in ordinary_rows {
            let meta = heap.class(class_id).class;
            let superclass = heap
                .class(class_id)
                .superclass
                .ok_or_else(|| format!("{name}.superclass should be set (parallel rule)"))?;
            let expected_meta_super = heap.class(superclass).class;
            if heap.class(meta).superclass != Some(expected_meta_super) {
                return Err(format!("{name}.class.superclass should be {name}.superclass.class (parallel rule)"));
            }
        }

        // R-INV-1.5 (boot half) — `Method` re-parents under `Function`
        // ([ADR-0006](../../../docs/adr/accepted/0006-function-as-abstract-callable-root.md),
        // decisions.md §4.1), so it inherits the call protocol instead of
        // redefining it. Guards the load-order fix in `create_core_classes`.
        if heap.class(c.method_class).superclass != Some(c.function_class) {
            return Err("Method.superclass should be Function (ADR-0006 re-parent)".to_string());
        }

        // R-INV-3.1 (boot half) — the callable tower: `Block` is `Function`'s
        // other sibling row, so both `Method` and `Block` share the call
        // protocol root (U-CORE-3, [ADR-0028](../../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)).
        // The `ordinary_rows` parallel-rule loop above already covers the
        // metaclass-level half for both rows; this asserts the plain
        // superclass link explicitly, mirroring the `Method` check above.
        if heap.class(c.block_class).superclass != Some(c.function_class) {
            return Err("Block.superclass should be Function (ADR-0006)".to_string());
        }

        // R-INV-0.3 (structural half) — absence never surfaces at boot: the
        // shared `None` singleton is an `Instance` of `None` (not a class
        // object), and `None` is a distinct class from the unreachable `Nil`
        // (ADR-0007/0010). The global-resolves-to-the-singleton-*value* half is
        // asserted inline in `VM::new` (it needs the core module).
        if c.none_class == c.nil_class {
            return Err("None and Nil must be distinct classes".to_string());
        }
        match heap.get(c.none_singleton) {
            crate::heap::Object::Instance(instance) if instance.class == c.none_class => {}
            crate::heap::Object::Instance(_) => return Err("None singleton must be an instance of None".to_string()),
            _ => return Err("None singleton must be an instance object, not a class object".to_string()),
        }

        // R-INV-0.4 — fixed-slot layout for the two directly-stamped classes
        // ([ADR-0011](../../../docs/adr/accepted/0011-static-instance-slot-layout.md)):
        // `Some` has one field (`_value`) and `Message` has four. Fences the
        // E→F bootstrap edge (bootstrap-phases §4).
        if heap.class(c.some_class).field_count != 1 {
            return Err("Some.field_count should be 1 (ADR-0011)".to_string());
        }
        if heap.class(c.message_class).field_count != 4 {
            return Err("Message.field_count should be 4 (ADR-0011)".to_string());
        }
        if heap.class(c.error_class).field_count != 1 {
            return Err("Error.field_count should be 1 (ADR-0011, U-CORE-6)".to_string());
        }
        if heap.class(c.message_not_understood_class).field_count != 2 {
            return Err("MessageNotUnderstood.field_count should be 2 (ADR-0011, U-CORE-6)".to_string());
        }

        // R-INV-6.1 — `MessageNotUnderstood < Error < Object`, explicit beyond
        // the generic parallel-rule loop above (U-CORE-6,
        // invariant-requirements.md §U-CORE-6).
        if heap.class(c.error_class).superclass != Some(c.object_class) {
            return Err("Error.superclass should be Object (U-CORE-6)".to_string());
        }
        if heap.class(c.message_not_understood_class).superclass != Some(c.error_class) {
            return Err("MessageNotUnderstood.superclass should be Error (U-CORE-6)".to_string());
        }

        // Every metaclass's superclass chain terminates (bounded walk guards
        // against a cycle turning into a hang instead of a failure).
        let mut current = heap.class(c.number_class).class;
        let mut steps = 0;
        loop {
            steps += 1;
            if steps > 64 {
                return Err("metaclass superclass chain did not terminate within 64 steps".to_string());
            }
            match heap.class(current).superclass {
                Some(next) => current = next,
                None => break,
            }
        }

        Ok(())
    }
}
