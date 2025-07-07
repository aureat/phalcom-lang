mod ast;
mod boolean;
pub mod bootstrap;
pub mod bytecode;
pub mod chunk;
pub mod class;
pub mod compiler;
mod error;
pub mod frame;
pub mod instance;
mod interner;
pub mod method;
mod nil;
pub mod primitive;
mod string;
pub mod universe;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests {}

macro_rules! primitive_method {
    ($class:expr, $sig:expr, $arity:expr, $func:expr) => {
        $class
            .borrow_mut()
            .add_method($sig, phref_new(MethodObject::new_primitive($arity, $func)));
    };
}

pub(crate) use primitive_method;
