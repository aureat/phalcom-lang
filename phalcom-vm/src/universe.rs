use crate::object::ClassObject;
use phalcom_common::PhRef;

#[derive(Debug, Clone)]
pub struct Universe {
    pub object_class: PhRef<ClassObject>,
    pub class_class: PhRef<ClassObject>,
    pub metaclass_class: PhRef<ClassObject>,
    pub number_class: PhRef<ClassObject>,
    pub string_class: PhRef<ClassObject>,
    pub nil_class: PhRef<ClassObject>,
    pub boolean_class: PhRef<ClassObject>,
}
