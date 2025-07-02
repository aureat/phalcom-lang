use crate::instance::InstanceObject;
use crate::method::MethodObject;
use crate::object::{ClassObject, lookup_method_in_hierarchy};
use crate::universe::Universe;
use crate::value::Value::Class;
use phalcom_common::PhRef;
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<String>),
    Instance(PhRef<InstanceObject>),
    Class(PhRef<ClassObject>),
    Method(PhRef<MethodObject>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(a, b),
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(a, b),
            (Value::Method(a), Value::Method(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Value {
    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err("Type Error: Expected a Number.".to_string()),
        }
    }

    pub fn class(&self, universe: &Universe) -> PhRef<ClassObject> {
        match self {
            // For unboxed, primitive values, we ask the Universe for their global class.
            Value::Nil => universe.nil_class.clone(),
            Value::Boolean(_) => universe.boolean_class.clone(),
            Value::Number(_) => universe.number_class.clone(),
            Value::String(_) => universe.string_class.clone(),

            // For heap-allocated objects, they know their own class.
            Value::Instance(instance) => instance.borrow().class(),
            Value::Class(class) => class.borrow().class(),
            Value::Method(method) => unimplemented!("Method::class() not yet implemented"),
        }
    }

    pub fn lookup_method(
        &self,
        universe: &Universe,
        selector: &str,
    ) -> Option<PhRef<MethodObject>> {
        let value_class = self.class(universe);
        lookup_method_in_hierarchy(value_class, selector)
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Instance(_) => write!(f, "<instance>"),
            Self::Class(c) => write!(f, "<class {}>", c.borrow().name), // Assuming Class has a name
            Self::Method(_) => write!(f, "<method>"),
        }
    }
}
