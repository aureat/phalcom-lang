use crate::class::ClassObject;
use crate::method::MethodObject;
use crate::primitive::number::{number_add, number_div};
use crate::primitive::object::object_name;
use crate::primitive::string::string_add;
use crate::primitive::{SIG_ADD, SIG_DIV, SIG_NAME};
use crate::primitive_method;
use crate::universe::Classes;
use phalcom_common::MaybeWeak::Strong;
use phalcom_common::{phref_new, MaybeWeak, PhRef};
use std::cell::RefCell;

pub fn bootstrap() -> Classes {
    let metaclass_class_ptr: PhRef<ClassObject> = ClassObject::new_instance_of_self("Metaclass");

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

    // Return the fully populated Universe.
    Classes {
        object_class: object_class_ptr,
        class_class: class_class_ptr,
        metaclass_class: metaclass_class_ptr,
        number_class: number_class_ptr,
        string_class: string_class_ptr,
        nil_class: nil_class_ptr,
        bool_class: bool_class_ptr,
        method_class: method_class_ptr,
        symbol_class: symbol_class_ptr,
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
