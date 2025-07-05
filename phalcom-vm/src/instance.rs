use crate::class::ClassObject;
use crate::value::Value;
use indexmap::IndexMap;
use phalcom_common::PhRef;

#[derive(Debug, Clone)]
pub struct InstanceObject {
    pub class: PhRef<ClassObject>,
    pub fields: IndexMap<String, Value>, // From indexmap crate
}

impl InstanceObject {
    pub fn new(class: PhRef<ClassObject>) -> Self {
        let fields = IndexMap::new();
        Self { class, fields }
    }

    pub fn class(&self) -> PhRef<ClassObject> {
        self.class.clone()
    }
}
