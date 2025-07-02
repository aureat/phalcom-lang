use crate::method::MethodObject;
use crate::object::ClassObject;
use crate::primitive::native_number_add;
use crate::universe::Universe;
use crate::value::Value::{Class, Method};
use phalcom_common::MaybeWeak::{Strong, Weak};
use phalcom_common::{MaybeWeak, PhRef, phref_new, phref_weak};
use std::cell::RefCell;

pub fn bootstrap() -> Universe {
    let metaclass_class_ptr: PhRef<ClassObject> = ClassObject::new_instance_of_self("Metaclass");

    let class_class_ptr = ClassObject::new_instance_of_self("Class");
    class_class_ptr
        .borrow_mut()
        .set_class_owned(&metaclass_class_ptr);

    let object_class_ptr = ClassObject::new_instance_of_self("Object");
    object_class_ptr
        .borrow_mut()
        .set_class_owned(&class_class_ptr);

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
    let string_class_ptr = phref_new(ClassObject::new(
        "String",
        Strong(object_class_ptr.clone()),
        Some(class_class_ptr.clone()),
    ));
    let nil_class_ptr = phref_new(ClassObject::new(
        "Nil",
        Strong(object_class_ptr.clone()),
        Some(class_class_ptr.clone()),
    ));
    let boolean_class_ptr = phref_new(ClassObject::new(
        "Boolean",
        Strong(object_class_ptr.clone()),
        Some(class_class_ptr.clone()),
    ));

    // Attach native methods to primitive classes.
    let add_method = MethodObject::new_native(1, native_number_add);
    number_class_ptr
        .borrow_mut()
        .add_method("__add__:", PhRef::new(RefCell::new(add_method)));

    // Return the fully populated Universe.
    Universe {
        object_class: object_class_ptr,
        class_class: class_class_ptr,
        metaclass_class: metaclass_class_ptr,
        number_class: number_class_ptr,
        string_class: string_class_ptr,
        nil_class: nil_class_ptr,
        boolean_class: boolean_class_ptr,
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
