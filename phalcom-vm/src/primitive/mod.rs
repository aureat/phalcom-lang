pub mod class;
pub mod number;
pub mod object;
pub mod string;
pub mod symbol;
pub mod system;

pub const SIG_ADD: &str = "+(_)";
pub const SIG_SUB: &str = "-(_)";
pub const SIG_MUL: &str = "*(_)";
pub const SIG_DIV: &str = "/(_)";
pub const SIG_EQ: &str = "==(_)";
pub const SIG_LT: &str = "<(_)";
pub const SIG_LE: &str = "<=(_)";
pub const SIG_GT: &str = ">(_)";
pub const SIG_GE: &str = ">=(_)";
pub const SIG_AND: &str = "and";
pub const SIG_OR: &str = "or";
pub const SIG_NEG: &str = "-";
pub const SIG_NOT: &str = "not";
pub const SIG_TO_NUMBER: &str = "toNumber";
pub const SIG_TO_STRING: &str = "toString";
pub const SIG_TO_BOOL: &str = "toBool";

pub const SIG_NAME: &str = "name";
pub const SIG_CLASS: &str = "class";
pub const SIG_SUPERCLASS: &str = "superclass";
pub const SIG_TOSTRING: &str = "toString";

pub const NIL_NAME: &str = "Nil";
pub const BOOL_NAME: &str = "Bool";
pub const NUMBER_NAME: &str = "Number";
pub const STRING_NAME: &str = "String";
pub const SYMBOL_NAME: &str = "Symbol";
pub const OBJECT_NAME: &str = "Object";
pub const METHOD_NAME: &str = "Method";
pub const CLASS_NAME: &str = "Class";
pub const METACLASS_NAME: &str = "Metaclass";
pub const SYSTEM_NAME: &str = "System";

pub const TRUE_NAME: &str = "true";
pub const FALSE_NAME: &str = "false";

macro_rules! primitive {
    ($vm:expr, $class:expr, $sig:expr, $sig_kind: expr, $func:expr) => {
        let symbol = $vm.get_or_intern($sig);
        $class
            .borrow_mut()
            .add_method(symbol, phref_new(MethodObject::primitive(symbol, $sig_kind, $func)));
    };
}

macro_rules! primitive_static {
    ($vm:expr, $class:expr, $sig:expr, $sig_kind: expr, $func:expr) => {
        let symbol = $vm.get_or_intern($sig);
        $class
            .borrow()
            .class()
            .borrow_mut()
            .add_method(symbol, phref_new(MethodObject::primitive(symbol, $sig_kind, $func)));
    };
}

pub(crate) use primitive;
pub(crate) use primitive_static;
