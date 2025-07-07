use crate::class::{lookup_method_in_hierarchy, ClassObject};
use crate::instance::InstanceObject;
use crate::interner::Symbol;
use crate::method::MethodObject;
use crate::primitive::{
    BOOL_NAME, CLASS_NAME, METHOD_NAME, NIL_NAME, NUMBER_NAME, STRING_NAME, SYMBOL_NAME,
};
use crate::string::StringObject;
use crate::vm::VM;
use phalcom_common::{phref_new, PhRef};
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(PhRef<StringObject>),
    Symbol(Symbol),
    Instance(PhRef<InstanceObject>),
    Class(PhRef<ClassObject>),
    Method(PhRef<MethodObject>),
}

impl Value {
    pub fn string_from_str(s: &str) -> Self {
        Value::String(phref_new(StringObject::from_str(s)))
    }

    pub fn string_from(s: String) -> Self {
        Value::String(phref_new(StringObject::from_string(s)))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err("Type Error: Expected a Number.".to_string()),
        }
    }

    pub fn as_string(&self) -> Result<Rc<String>, String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            _ => Err("Type Error: Expected a String.".to_string()),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err("Type Error: Expected a Bool.".to_string()),
        }
    }

    pub fn as_symbol(&self) -> Result<Symbol, String> {
        match self {
            Value::Symbol(s) => Ok(*s),
            _ => Err("Type Error: Expected a Symbol.".to_string()),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "Nil",
            Value::Bool(_) => "Bool",
            Value::Number(_) => "Number",
            Value::String(_) => "String",
            Value::Symbol(_) => "Symbol",
            Value::Class(_) => "Class",
            Value::Instance(_) => "Instance",
            Value::Method(_) => "Method",
        }
    }

    pub fn name(&self, vm: &VM) -> Value {
        match self {
            Value::Nil => NIL_NAME.to_string(),
            Value::Bool(_) => BOOL_NAME.to_string(),
            Value::Number(_) => NUMBER_NAME.to_string(),
            Value::String(_) => STRING_NAME.to_string(),
            Value::Symbol(symbol) => SYMBOL_NAME.to_string(),
            Value::Instance(instance) => instance.borrow().name().to_string(),
            Value::Class(class) => class.borrow().name().to_string(),
            Value::Method(method) => method.borrow().name().to_string(),
        }
    }

    pub fn class(&self, vm: &VM) -> PhRef<ClassObject> {
        match self {
            Value::Nil => vm.classes.nil_class.clone(),
            Value::Bool(_) => vm.classes.bool_class.clone(),
            Value::Number(_) => vm.classes.number_class.clone(),
            Value::String(_) => vm.classes.string_class.clone(),
            Value::Symbol(_) => vm.classes.symbol_class.clone(),
            Value::Method(method) => vm.classes.method_class.clone(),
            Value::Instance(instance) => instance.borrow().class(),
            Value::Class(class) => class.borrow().class(),
        }
    }

    pub fn name_str(&self, _vm: &VM) -> &str {
        match self {
            Value::Nil => NIL_NAME,
            Value::Bool(_) => BOOL_NAME,
            Value::Number(_) => NUMBER_NAME,
            Value::String(_) => STRING_NAME,
            Value::Symbol(_) => SYMBOL_NAME,
            Value::Instance(instance) => "",
            Value::Class(class) => CLASS_NAME,
            Value::Method(method) => METHOD_NAME,
        }
    }

    pub fn lookup_method(&self, vm: &VM, selector: &str) -> Option<PhRef<MethodObject>> {
        let value_class = self.class(vm);
        lookup_method_in_hierarchy(value_class, selector)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Instance(a), Value::Instance(b)) => Rc::ptr_eq(a, b),
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(a, b),
            (Value::Method(a), Value::Method(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

fn hash_f64<H: Hasher>(num: f64, state: &mut H) {
    let bits: u64 = num.to_bits();
    bits.hash(state);
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => 0.hash(state),
            Value::Number(f64_ref) => hash_f64(*f64_ref, state),
            Value::Bool(v) => v.hash(state),
            Value::String(v) => v.hash(state),
            Value::Class(v) => v.as_ptr().hash(state),
            Value::Method(v) => v.as_ptr().hash(state),
            Value::Symbol(v) => v.0.hash(state),
            Value::Instance(v) => v.as_ptr().hash(state),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Symbol(s) => write!(f, "Symbol({})", s.0),
            Self::Instance(_) => write!(f, "<instance>"),
            Self::Class(c) => write!(f, "<class {}>", c.borrow().name()),
            Self::Method(_) => write!(f, "<method>"),
        }
    }
}
