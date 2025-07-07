pub mod number;
pub mod object;
pub mod string;
mod symbol;

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
