use crate::class::ClassObject;
use crate::method::MethodObject;
use crate::method::SignatureKind;
use crate::primitive::boolean::bool_class_new;
use crate::primitive::class::{class_add, class_new, class_set_superclass, class_superclass};
use crate::primitive::method::method_class_new;
use crate::primitive::module::module_class_new;
use crate::primitive::nil::nil_class_new;
use crate::primitive::number::{number_add, number_class_new, number_div};
use crate::primitive::object::{object_class, object_class_new, object_name, object_set_class};
use crate::primitive::string::{string_add, string_class_new};
use crate::primitive::symbol::{symbol_class_new, symbol_tostring};
use crate::primitive::system::{system_class_new, system_class_print};
use crate::primitive::{primitive, primitive_static, CLASS_NAME, FALSE_NAME, TRUE_NAME};
use crate::primitive::{BOOL_NAME, METACLASS_NAME, METHOD_NAME, NIL_NAME, NUMBER_NAME, OBJECT_NAME, STRING_NAME, SYMBOL_NAME, SYSTEM_NAME};
use crate::string::{phstring_new, StringObject};
use crate::vm::VM;
use phalcom_common::{phref_new, MaybeWeak, PhRef};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Universe {
    pub classes: CoreClasses,
    pub primitive_names: PrimitiveNames,
}

impl Default for Universe {
    fn default() -> Self {
        todo!()
    }
}

impl Universe {
    pub fn new() -> Self {
        let core_classes = Self::bootstrap_core_classes();
        let primitive_names = Self::create_primitive_names();
        Universe {
            classes: core_classes,
            primitive_names,
        }
    }

    pub fn bootstrap_core_classes() -> CoreClasses {
        let metaclass_class_ptr: PhRef<ClassObject> = ClassObject::new_instance_of_self("Metaclass");

        let class_class_ptr = ClassObject::new_instance_of_self("Class");
        class_class_ptr.borrow_mut().set_class_owned(&metaclass_class_ptr);

        let object_class_class_ptr = phref_new(ClassObject::new(
            "Object.class",
            MaybeWeak::Strong(metaclass_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));
        let object_class_ptr = ClassObject::new_instance_of_self("Object");
        object_class_ptr.borrow_mut().set_class_owned(&object_class_class_ptr);

        class_class_ptr.borrow_mut().set_superclass(Some(object_class_ptr.clone()));

        metaclass_class_ptr.borrow_mut().set_superclass(Some(class_class_ptr.clone()));
        metaclass_class_ptr.borrow_mut().set_class_owned(&metaclass_class_ptr);

        // Helper function to create a class with its metaclass
        let create_core_class = |name: &str, superclass: Option<PhRef<ClassObject>>| -> PhRef<ClassObject> {
            let metaclass_name = format!("{name}.class");
            let metaclass = phref_new(ClassObject::new(
                &metaclass_name,
                MaybeWeak::Strong(metaclass_class_ptr.clone()),
                Some(class_class_ptr.clone()),
            ));

            let class = phref_new(ClassObject::new(name, MaybeWeak::Strong(metaclass.clone()), superclass));

            class
        };

        let number_class_ptr = create_core_class("Number", Some(object_class_ptr.clone()));
        let string_class_ptr = create_core_class("String", Some(object_class_ptr.clone()));
        let nil_class_ptr = create_core_class("Nil", Some(object_class_ptr.clone()));
        let bool_class_ptr = create_core_class("Bool", Some(object_class_ptr.clone()));
        let method_class_ptr = create_core_class("Method", Some(object_class_ptr.clone()));
        let symbol_class_ptr = create_core_class("Symbol", Some(object_class_ptr.clone()));
        let module_class_ptr = create_core_class("Module", Some(object_class_ptr.clone()));
        let system_class_ptr = create_core_class("System", Some(object_class_ptr.clone()));

        // Return the fully populated Universe.
        CoreClasses {
            object_class: object_class_ptr,
            class_class: class_class_ptr,
            metaclass_class: metaclass_class_ptr,
            number_class: number_class_ptr,
            string_class: string_class_ptr,
            nil_class: nil_class_ptr,
            bool_class: bool_class_ptr,
            method_class: method_class_ptr,
            symbol_class: symbol_class_ptr,
            module_class: module_class_ptr,
            system_class: system_class_ptr,
        }
    }

    pub fn install_primitives(vm: &mut VM) {
        let object_cls = vm.universe.classes.object_class.clone();
        primitive!(vm, object_cls, "name", SignatureKind::Getter, object_name);
        primitive!(vm, object_cls, "class", SignatureKind::Getter, object_class);
        primitive!(vm, object_cls, "class=(_)", SignatureKind::Setter, object_set_class);
        primitive!(vm, object_cls, "toString", SignatureKind::Getter, object_name);
        primitive_static!(vm, object_cls, "new()", SignatureKind::Method(1), object_class_new);

        let class_cls = vm.universe.classes.class_class.clone();
        primitive!(vm, class_cls, "superclass", SignatureKind::Getter, class_superclass);
        primitive!(vm, class_cls, "superclass=(_)", SignatureKind::Setter, class_set_superclass);
        primitive!(vm, class_cls, "+(_)", SignatureKind::Method(1), class_add);
        primitive!(vm, class_cls, "new()", SignatureKind::Method(0), class_new);

        let number_cls = vm.universe.classes.number_class.clone();
        primitive!(vm, number_cls, "+(_)", SignatureKind::Method(1), number_add);
        primitive!(vm, number_cls, "/(_)", SignatureKind::Method(1), number_div);
        primitive_static!(vm, number_cls, "new()", SignatureKind::Method(0), number_class_new);
        primitive_static!(vm, number_cls, "new(_)", SignatureKind::Method(1), number_class_new);

        let string_cls = vm.universe.classes.string_class.clone();
        primitive!(vm, string_cls, "+(_)", SignatureKind::Method(1), string_add);
        primitive_static!(vm, string_cls, "new()", SignatureKind::Method(0), string_class_new);
        primitive_static!(vm, string_cls, "new(_)", SignatureKind::Method(1), string_class_new);

        let bool_cls = vm.universe.classes.bool_class.clone();
        primitive_static!(vm, bool_cls, "new()", SignatureKind::Method(0), bool_class_new);
        primitive_static!(vm, bool_cls, "new(_)", SignatureKind::Method(1), bool_class_new);

        let symbol_cls = vm.universe.classes.symbol_class.clone();
        primitive!(vm, symbol_cls, "toString", SignatureKind::Getter, symbol_tostring);
        primitive_static!(vm, symbol_cls, "new(_)", SignatureKind::Method(1), symbol_class_new);

        let nil_cls = vm.universe.classes.nil_class.clone();
        primitive_static!(vm, nil_cls, "new()", SignatureKind::Method(0), nil_class_new);

        let bool_cls = vm.universe.classes.bool_class.clone();
        primitive_static!(vm, bool_cls, "new(_)", SignatureKind::Method(1), nil_class_new);

        let method_cls = vm.universe.classes.method_class.clone();
        primitive_static!(vm, method_cls, "new(_)", SignatureKind::Method(1), method_class_new);

        let system_cls = vm.universe.classes.system_class.clone();
        primitive_static!(vm, system_cls, "print(_)", SignatureKind::Method(1), system_class_print);
        primitive_static!(vm, system_cls, "new()", SignatureKind::Method(0), system_class_new);

        let module_cls = vm.universe.classes.module_class.clone();
        primitive_static!(vm, module_cls, "new()", SignatureKind::Method(0), module_class_new);
    }

    pub fn create_primitive_names() -> PrimitiveNames {
        PrimitiveNames {
            nil: phref_new(StringObject::from_str(NIL_NAME)),
            bool_: phref_new(StringObject::from_str(BOOL_NAME)),
            true_: phref_new(StringObject::from_str(TRUE_NAME)),
            false_: phref_new(StringObject::from_str(FALSE_NAME)),
            number: phref_new(StringObject::from_str(NUMBER_NAME)),
            string: phref_new(StringObject::from_str(STRING_NAME)),
            symbol: phref_new(StringObject::from_str(SYMBOL_NAME)),
            object: phref_new(StringObject::from_str(OBJECT_NAME)),
            method: phref_new(StringObject::from_str(METHOD_NAME)),
            class: phref_new(StringObject::from_str(CLASS_NAME)),
            metaclass: phref_new(StringObject::from_str(METACLASS_NAME)),
            system: phref_new(StringObject::from_str(SYSTEM_NAME)),
        }
    }
}

impl ClassObject {
    fn new_instance_of_self(name: &str) -> PhRef<ClassObject> {
        PhRef::new_cyclic(|weak_self| {
            RefCell::new(Self {
                // The key: the class of this object is a weak pointer to itself,
                // which will be upgraded to the final Rc.
                class: MaybeWeak::Weak(weak_self.clone()),
                name: phstring_new(name.to_string()),
                superclass: None,
                methods: Default::default(),
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct CoreClasses {
    pub object_class: PhRef<ClassObject>,
    pub class_class: PhRef<ClassObject>,
    pub metaclass_class: PhRef<ClassObject>,
    pub number_class: PhRef<ClassObject>,
    pub string_class: PhRef<ClassObject>,
    pub nil_class: PhRef<ClassObject>,
    pub bool_class: PhRef<ClassObject>,
    pub method_class: PhRef<ClassObject>,
    pub symbol_class: PhRef<ClassObject>,
    pub module_class: PhRef<ClassObject>,
    pub system_class: PhRef<ClassObject>,
}

#[derive(Clone)]
pub struct PrimitiveNames {
    pub nil: PhRef<StringObject>,
    pub bool_: PhRef<StringObject>,
    pub true_: PhRef<StringObject>,
    pub false_: PhRef<StringObject>,
    pub number: PhRef<StringObject>,
    pub string: PhRef<StringObject>,
    pub symbol: PhRef<StringObject>,
    pub object: PhRef<StringObject>,
    pub method: PhRef<StringObject>,
    pub class: PhRef<StringObject>,
    pub metaclass: PhRef<StringObject>,
    pub system: PhRef<StringObject>,
}

impl PrimitiveNames {
    pub fn bool_name(&self, b: bool) -> PhRef<StringObject> {
        if b { self.true_.clone() } else { self.false_.clone() }
    }
}

impl std::fmt::Debug for PrimitiveNames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrimitiveNames")
            .field("nil", &"PhRef<StringObject>")
            .field("bool_", &"PhRef<StringObject>")
            .field("number", &"PhRef<StringObject>")
            .field("string", &"PhRef<StringObject>")
            .field("symbol", &"PhRef<StringObject>")
            .field("object", &"PhRef<StringObject>")
            .field("method", &"PhRef<StringObject>")
            .field("class", &"PhRef<StringObject>")
            .field("metaclass", &"PhRef<StringObject>")
            .finish()
    }
}
