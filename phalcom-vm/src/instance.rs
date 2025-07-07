use crate::class::ClassObject;
use crate::value::Value;
use indexmap::IndexMap;
use phalcom_common::PhRef;
use std::rc::Rc;

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

    pub fn name(&self) -> Rc<String> {
        self.class.borrow().name()
    }

    pub fn class(&self) -> PhRef<ClassObject> {
        self.class.clone()
    }
}
