pub mod boolean;
pub mod class;
pub mod method;
pub mod module;
pub mod nil;
pub mod number;
pub mod object;
pub mod string;
pub mod symbol;
pub mod system;

#[non_exhaustive]
pub struct Sig;

#[allow(non_upper_case_globals)]
impl Sig {
    pub const ADD: &'static str = "+(_)";
    pub const SUB: &'static str = "-(_)";
    pub const MUL: &'static str = "*(_)";
    pub const DIV: &'static str = "/(_)";
    pub const EQ: &'static str = "==(_)";
    pub const LT: &'static str = "<(_)";
    pub const LE: &'static str = "<=(_)";
    pub const GT: &'static str = ">(_)";
    pub const GE: &'static str = ">=(_)";
    pub const AND: &'static str = "and(_)";
    pub const OR: &'static str = "or(_)";

    pub const NEG: &'static str = "-";
    pub const NOT: &'static str = "not";

    pub const name: &'static str = "name";
    pub const name_set: &'static str = "name=(_)";
    pub const class: &'static str = "class";
    pub const class_set: &'static str = "class=(_)";
    pub const superclass: &'static str = "superclass";
    pub const superclass_set: &'static str = "superclass=(_)";

    pub const toString: &'static str = "toString";
    pub const toNumber: &'static str = "toNumber";
    pub const toBool: &'static str = "toBool";
    pub const toDebug: &'static str = "toDebug";

    pub const new: &'static str = "new()";
    pub const new_1: &'static str = "new(_)";
    pub const new_2: &'static str = "new(_,_)";
}

#[non_exhaustive]
pub struct ClassName;

#[allow(non_upper_case_globals)]
impl ClassName {
    pub const Nil: &'static str = "Nil";
    pub const Bool: &'static str = "Bool";
    pub const Number: &'static str = "Number";
    pub const String: &'static str = "String";
    pub const Symbol: &'static str = "Symbol";
    pub const System: &'static str = "System";
    pub const Module: &'static str = "Module";
    pub const Object: &'static str = "Object";
    pub const Class: &'static str = "Class";
    pub const Metaclass: &'static str = "Metaclass";
    pub const Method: &'static str = "Method";
    pub const List: &'static str = "List";
    pub const Range: &'static str = "Range";
    pub const Map: &'static str = "Map";
    pub const Fiber: &'static str = "Fiber";
    pub const Future: &'static str = "Future";
}

#[non_exhaustive]
pub struct ObjectName;

#[allow(non_upper_case_globals)]
impl ObjectName {
    pub const Nil: &'static str = "nil";
    pub const True: &'static str = "true";
    pub const False: &'static str = "false";
}

macro_rules! primitive {
    ($vm:expr, $class:expr, $sig:expr, $sig_kind: expr, $func:expr) => {
        let symbol = $vm.get_or_intern($sig);
        let method = MethodObject::new_primitive(symbol, $sig_kind, $func, PhRef::downgrade(&$class));
        $class.borrow_mut().add_method(symbol, phref_new(method));
    };
}

macro_rules! primitive_static {
    ($vm:expr, $class:expr, $sig:expr, $sig_kind: expr, $func:expr) => {
        let symbol = $vm.get_or_intern($sig);
        let method = MethodObject::new_primitive(symbol, $sig_kind, $func, PhRef::downgrade(&$class));
        $class.borrow().class().borrow_mut().add_method(symbol, phref_new(method));
    };
}

pub(crate) use primitive;
pub(crate) use primitive_static;
