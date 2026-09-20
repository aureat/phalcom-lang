//! Native observations owned by source-declared time capabilities.

use crate::error::PhResult;
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Signature: `SystemClock.class::monotonicNanoseconds` — exact monotonic
/// nanoseconds elapsed since this VM was created.
#[phalcom_native_macros::primitive(
    SystemClock,
    "monotonicNanoseconds",
    params = [],
    returns = Int,
    types = "() -> Int",
    side = class,
    visibility = internal,
    effects = [nondeterminism]
)]
pub fn system_clock_monotonic_nanoseconds(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let nanos = BigInt::from(vm.start_time.elapsed().as_nanos());
    Ok(normalize_bigint(nanos, &mut vm.heap))
}

/// Signature: `SystemWallClock.class::unixNanoseconds` — exact signed Unix/POSIX
/// nanoseconds observed from the host wall clock.
#[phalcom_native_macros::primitive(
    SystemWallClock,
    "unixNanoseconds",
    params = [],
    returns = Int,
    types = "() -> Int",
    side = class,
    visibility = internal,
    effects = [nondeterminism]
)]
pub fn system_wall_clock_unix_nanoseconds(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => BigInt::from(duration.as_secs()) * 1_000_000_000u32 + duration.subsec_nanos(),
        Err(error) => -(BigInt::from(error.duration().as_secs()) * 1_000_000_000u32 + error.duration().subsec_nanos()),
    };
    Ok(normalize_bigint(nanos, &mut vm.heap))
}
