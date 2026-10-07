//! The random package's native floor: fill an existing byte buffer from the OS.

use crate::error::PhResult;
use crate::primitive::expect_bytes;
use crate::value::Value;
use crate::vm::VM;

#[phalcom_native_macros::primitive(
    SecureRandom,
    "rawFill(_)",
    params = [Bytes],
    returns = "Option<String>",
    types = "(Bytes) -> Option<String>",
    visibility = internal,
    effects = [mutation, nondeterminism, blocking]
)]
pub fn secure_random_raw_fill(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_bytes(vm, &args[0])?;
    let outcome = fill_system_random(vm.heap.bytes_mut(id).as_mut_slice());
    // The mutable buffer borrow is released before allocating a source String.
    match outcome {
        Ok(()) => Ok(Value::none()),
        Err(error) => Ok(vm.alloc_string_value(error.to_string()).wrap_some()?),
    }
}

fn fill_system_random(bytes: &mut [u8]) -> Result<(), getrandom::Error> {
    #[cfg(test)]
    if INJECT_FAILURE.with(|flag| flag.get()) {
        return Err(getrandom::Error::UNSUPPORTED);
    }
    getrandom::fill(bytes)
}

#[cfg(test)]
thread_local! {
    static INJECT_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_random_injected_failure_is_lifted_to_source_entropy_error() {
        let mut vm = VM::new();
        let mut session = crate::modules::compile_session::ProgramCompilationSession::standalone().unwrap();
        let program = session
            .compile_fixture(
                std::path::Path::new("secure-failure.ph"),
                r#"
from universe.random import SecureRandom, EntropyError
let mut caught = false
try { SecureRandom.system.bytes(32) } on EntropyError error { caught = true }
"#,
            )
            .unwrap();
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                INJECT_FAILURE.with(|flag| flag.set(false));
            }
        }
        INJECT_FAILURE.with(|flag| flag.set(true));
        let _reset = Reset;
        vm.run_compiled(&program).unwrap();
        let module = vm.module_registry.get(&program.entry).unwrap().object;
        let symbol = vm.interner.find("caught").unwrap();
        assert_eq!(vm.heap.module(module).get(symbol), Some(Value::bool(true)));
    }

    #[test]
    fn secure_random_native_success_fills_empty_and_nonempty_buffers() {
        for size in [0, 1, 32] {
            let mut bytes = vec![0; size];
            fill_system_random(&mut bytes).unwrap();
            assert_eq!(bytes.len(), size);
        }
    }
}
