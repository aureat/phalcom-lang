use crate::method::MethodObject;
use crate::value::Value;
use indexmap::IndexMap;
use phalcom_common::{MaybeWeak, PhRef, phref_new, phref_weak};

type MethodsMap = IndexMap<String, PhRef<MethodObject>>; // Selector -> MethodObject

#[derive(Debug, Clone)]
pub struct ClassObject {
    pub name: String,
    pub class: MaybeWeak<ClassObject>,
    pub superclass: Option<PhRef<ClassObject>>,
    pub methods: MethodsMap,
}

/// The core method lookup logic with superclass traversal.
/// This version is more efficient as it iterates using Gc pointers,
/// avoiding expensive struct clones.
pub fn lookup_method_in_hierarchy(
    mut class: PhRef<ClassObject>,
    selector: &str,
) -> Option<PhRef<MethodObject>> {
    loop {
        let next_class_maybe;
        {
            // This inner scope limits the lifetime of `class_borrow`.
            let class_borrow = class.borrow();

            // First, check for the method on the current class.
            if let Some(method) = class_borrow.methods.get(selector) {
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
    pub fn new(
        name: &str,
        class: MaybeWeak<ClassObject>,
        superclass: Option<PhRef<ClassObject>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            class,
            superclass,
            methods: MethodsMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn class(&self) -> PhRef<ClassObject> {
        match self.class {
            MaybeWeak::Weak(ref weak) => weak.upgrade().unwrap_or_else(|| {
                panic!("superclass dropped, cannot upgrade ref ({})", self.name())
            }),
            MaybeWeak::Strong(ref owned) => owned.clone(),
        }
    }

    pub fn superclass(&self) -> Option<PhRef<ClassObject>> {
        self.superclass.clone()
    }

    /// Set the superclass of this class (as a weak reference).
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

    pub fn add_method(&mut self, selector: &str, method: PhRef<MethodObject>) {
        self.methods.insert(selector.to_string(), method);
    }

    pub fn get_method(&self, selector: &str) -> Option<PhRef<MethodObject>> {
        let method = self.methods.get(selector);
        method.cloned()
    }
}
