use crate::class::ClassObject;
use crate::primitive::CLASS_NAME;
use crate::primitive::{
    BOOL_NAME, METACLASS_NAME, METHOD_NAME, NIL_NAME, NUMBER_NAME, OBJECT_NAME, STRING_NAME,
    SYMBOL_NAME,
};
use crate::primitive_method;
use crate::string::StringObject;
use phalcom_common::{phref_new, MaybeWeak, PhRef};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Universe {
    pub classes: CoreClasses,
    pub primitive_names: PrimitiveNames,
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
        let metaclass_class_ptr: PhRef<ClassObject> =
            ClassObject::new_instance_of_self("Metaclass");

        let class_class_ptr = ClassObject::new_instance_of_self("Class");
        class_class_ptr
            .borrow_mut()
            .set_class_owned(&metaclass_class_ptr);

        let object_class_ptr = ClassObject::new_instance_of_self("Object");

        object_class_ptr
            .borrow_mut()
            .set_class_owned(&class_class_ptr);
        primitive_method!(object_class_ptr, SIG_NAME, 0, object_name);

        class_class_ptr
            .borrow_mut()
            .set_superclass(Some(object_class_ptr.clone()));

        metaclass_class_ptr
            .borrow_mut()
            .set_superclass(Some(class_class_ptr.clone()));
        metaclass_class_ptr
            .borrow_mut()
            .set_class_owned(&metaclass_class_ptr);

        let number_class_ptr = phref_new(ClassObject::new(
            "Number",
            Strong(object_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));

        primitive_method!(number_class_ptr, SIG_ADD, 1, number_add);
        primitive_method!(number_class_ptr, SIG_DIV, 1, number_div);

        let string_class_ptr = phref_new(ClassObject::new(
            "String",
            Strong(object_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));
        primitive_method!(string_class_ptr, SIG_ADD, 1, string_add);

        let nil_class_ptr = phref_new(ClassObject::new(
            "Nil",
            Strong(object_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));

        let bool_class_ptr = phref_new(ClassObject::new(
            "Bool",
            Strong(object_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));

        let method_class_ptr = phref_new(ClassObject::new(
            "Method",
            Strong(object_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));

        let symbol_class_ptr = phref_new(ClassObject::new(
            "Symbol",
            Strong(object_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));

        let module_class_ptr = phref_new(ClassObject::new(
            "Module",
            Strong(object_class_ptr.clone()),
            Some(class_class_ptr.clone()),
        ));

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
        }
    }

    pub fn create_primitive_names() -> PrimitiveNames {
        PrimitiveNames {
            nil: phref_new(StringObject::from_str(NIL_NAME)),
            bool_: phref_new(StringObject::from_str(BOOL_NAME)),
            number: phref_new(StringObject::from_str(NUMBER_NAME)),
            string: phref_new(StringObject::from_str(STRING_NAME)),
            symbol: phref_new(StringObject::from_str(SYMBOL_NAME)),
            object: phref_new(StringObject::from_str(OBJECT_NAME)),
            method: phref_new(StringObject::from_str(METHOD_NAME)),
            class: phref_new(StringObject::from_str(CLASS_NAME)),
            metaclass: phref_new(StringObject::from_str(METACLASS_NAME)),
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
                name: name.to_string(),
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
}

#[derive(Clone)]
pub struct PrimitiveNames {
    pub nil: PhRef<StringObject>,
    pub bool_: PhRef<StringObject>,
    pub number: PhRef<StringObject>,
    pub string: PhRef<StringObject>,
    pub symbol: PhRef<StringObject>,
    pub object: PhRef<StringObject>,
    pub method: PhRef<StringObject>,
    pub class: PhRef<StringObject>,
    pub metaclass: PhRef<StringObject>,
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
