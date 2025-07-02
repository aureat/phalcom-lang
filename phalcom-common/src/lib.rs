use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub mod error;

pub type PhRef<T> = Rc<RefCell<T>>;
pub type PhWeakRef<T> = Weak<RefCell<T>>;

#[derive(Debug, Clone)]
pub enum MaybeWeak<A> {
    Strong(PhRef<A>),
    Weak(PhWeakRef<A>),
}

pub fn phref_new<T>(value: T) -> PhRef<T> {
    Rc::new(RefCell::new(value))
}

pub fn phref_weak<T>(value: &PhRef<T>) -> PhWeakRef<T> {
    Rc::downgrade(value)
}

#[cfg(test)]
mod tests {}
