use crate::class::ClassObject;
use phalcom_common::PhRef;

#[derive(Debug, Clone)]
pub struct Classes {
    pub object_class: PhRef<ClassObject>,
    pub class_class: PhRef<ClassObject>,
    pub metaclass_class: PhRef<ClassObject>,
    pub number_class: PhRef<ClassObject>,
    pub string_class: PhRef<ClassObject>,
    pub nil_class: PhRef<ClassObject>,
    pub bool_class: PhRef<ClassObject>,
    pub method_class: PhRef<ClassObject>,
    pub symbol_class: PhRef<ClassObject>,
}

pub struct CommonSymbols {
    pub nil_name:
}
