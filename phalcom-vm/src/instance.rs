use crate::class::ClassObject;
use crate::string::{phstring_new, PhString, StringObject};
use crate::value::Value;
use indexmap::IndexMap;
use phalcom_common::PhRef;

#[derive(Debug, Clone)]
pub struct InstanceObject {
    pub class: PhRef<ClassObject>,
    pub fields: IndexMap<String, Value>,
}

impl InstanceObject {
    pub fn new(class: PhRef<ClassObject>) -> Self {
        let fields = IndexMap::new();
        Self { class, fields }
    }

    pub fn name(&self) -> PhRef<StringObject> {
        self.class.borrow().name()
    }

    pub fn class(&self) -> PhRef<ClassObject> {
        self.class.clone()
    }

    pub fn to_string(&self) -> PhString {
        let name = self.name();
        let name_borrowed = name.borrow();
        let string = format!("Instance of {}", name_borrowed.as_str());
        drop(name_borrowed);
        phstring_new(string)
    }
    
    pub fn to_debug_string(&self) -> PhString {
        let name = self.name();
        let name_borrowed = name.borrow();
        let string = format!("<instance {}>", name_borrowed.as_str());
        drop(name_borrowed);
        phstring_new(string)
    }
}
