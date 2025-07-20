use crate::interner::Symbol;
use crate::method::MethodObject;
use crate::string::{phstring_new, PhString, StringObject};
use crate::value::Value;
use crate::vm::VM;
use indexmap::IndexMap;
use phalcom_common::{phref_new, phref_weak, MaybeWeak, PhRef};

type MethodsMap = IndexMap<Symbol, PhRef<MethodObject>>;

#[derive(Debug, Clone)]
pub struct ClassObject {
    pub name: PhString,
    pub class: MaybeWeak<ClassObject>,
    pub superclass: Option<PhRef<ClassObject>>,
    pub methods: MethodsMap,
}

/// The core method lookup logic with superclass traversal.
/// This version is more efficient as it iterates using Gc pointers,
/// avoiding expensive struct clones.
pub fn lookup_method_in_hierarchy(mut class: PhRef<ClassObject>, selector: Symbol) -> Option<PhRef<MethodObject>> {
    println!("{}", class.borrow().name_copy());
    loop {
        let next_class_maybe;
        {
            // This inner scope limits the lifetime of `class_borrow`.
            let class_borrow = class.borrow();

            // First, check for the method on the current class.
            if let Some(method) = class_borrow.methods.get(&selector) {
                return Some(method.clone());
            }

            // If not found, clone the superclass Gc (if it exists)
            // so we can use it after the borrow ends.
            next_class_maybe = class_borrow.superclass.clone();
        } // `class_borrow` is dropped here, releasing the borrow on `class`.

        // Now that the borrow is released, we can safely check if we have
        // a superclass and assign it to `class` for the next iteration.
        if let Some(next_class) = next_class_maybe {
            class = next_class;
        } else {
            // No superclass, so the search ends here.
            return None;
        }
    }
}

impl ClassObject {
    pub fn new(name: &str, class: MaybeWeak<ClassObject>, superclass: Option<PhRef<ClassObject>>) -> Self {
        let name = phref_new(StringObject::from_str(name));
        Self {
            name,
            class,
            superclass,
            methods: MethodsMap::new(),
        }
    }

    pub fn name(&self) -> PhRef<StringObject> {
        self.name.clone()
    }

    pub fn name_copy(&self) -> String {
        self.name.borrow().value()
    }

    pub fn class(&self) -> PhRef<ClassObject> {
        match self.class {
            MaybeWeak::Weak(ref weak) => weak.upgrade().unwrap_or_else(|| {
                panic!("{}.class dropped, cannot upgrade ref", self.name.borrow().as_str());
            }),
            MaybeWeak::Strong(ref owned) => owned.clone(),
        }
    }

    pub fn superclass(&self) -> Option<PhRef<ClassObject>> {
        self.superclass.clone()
    }

    pub fn superclass_val(&self) -> Value {
        match &self.superclass {
            Some(superclass) => Value::Class(superclass.clone()),
            None => Value::Nil,
        }
    }

    pub fn to_string(&self) -> PhRef<StringObject> {
        let name_borrowed = self.name.borrow();
        let string = phstring_new(format!("<class {}>", name_borrowed.as_str()));
        drop(name_borrowed);
        string
    }

    /// Set the superclass of this class
    pub fn set_superclass(&mut self, class: Option<PhRef<ClassObject>>) {
        self.superclass = class
    }

    /// Set the class of this class (as a weak reference).
    pub fn set_class(&mut self, class: &PhRef<Self>) {
        self.class = MaybeWeak::Weak(phref_weak(class));
    }

    /// Set the class of this class (as a strong reference).
    pub fn set_class_owned(&mut self, class: &PhRef<Self>) {
        self.class = MaybeWeak::Strong(class.clone());
    }

    pub fn add_method(&mut self, selector: Symbol, method: PhRef<MethodObject>) {
        self.methods.insert(selector, method);
    }

    pub fn get_method(&self, selector: Symbol) -> Option<PhRef<MethodObject>> {
        let method = self.methods.get(&selector);
        method.cloned()
    }

    pub fn list_methods(&self, vm: &VM) {
        for item in &self.methods {
            let (sym, method) = item;
            println!("{}", method.borrow().name(vm).borrow().as_str());
        }
    }
}
