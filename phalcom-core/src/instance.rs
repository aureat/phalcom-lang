use crate::class::ClassObject;
use crate::interner::Symbol;
use crate::string::{phstring_new, PhString};
use crate::value::Value;
use indexmap::IndexMap;
use phalcom_common::PhRef;

#[derive(Debug, Clone)]
pub struct InstanceObject {
    pub class: PhRef<ClassObject>,
    pub fields: IndexMap<Symbol, Value>,
}

impl InstanceObject {
    pub fn new(class: PhRef<ClassObject>) -> Self {
        let fields = IndexMap::new();
        Self { class, fields }
    }

    pub fn name(&self) -> PhString {
        self.class.borrow().name()
    }

    pub fn class(&self) -> PhRef<ClassObject> {
        self.class.clone()
    }

    pub fn to_debug_ph(&self) -> PhString {
        phstring_new(self.to_debug())
    }

    pub fn to_debug(&self) -> String {
        format!("<{} instance>", self.name().borrow().as_str())
    }
}
