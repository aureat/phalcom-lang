#![allow(unused_variables)]
extern crate core;

pub mod adt;
pub mod bytecode;
pub mod callable;
pub mod chunk;
pub mod data;

pub mod compiler;
pub mod diagnostics;
pub mod error;
pub mod frame;
pub mod heap;
pub(crate) mod iana_links;
#[cfg(any(test, feature = "test-timings"))]
pub mod instrumentation;
pub mod interner;
pub mod interpret;
pub mod method;
pub mod modules;
pub mod native;
#[cfg(feature = "opcode-histogram")]
pub mod opcode_stats;
pub mod parameters;
pub mod primitive;
pub mod product;
pub mod reactor;
pub mod resource;
pub(crate) mod timezone;
pub mod typing;
pub mod universe;
pub mod value;
pub mod vm;

#[cfg(any(test, feature = "test-timings"))]
pub use instrumentation::*;

#[cfg(test)]
mod tests {}

/// Records a synchronous successful phase when observability is enabled.
#[macro_export]
macro_rules! perf_span {
    ($phase:expr, $operation:expr $(,)?) => {{
        #[cfg(feature = "perf-observability")]
        {
            ::telos_observability::PerfContext::current_thread().with_span($phase, |_| ($operation)())
        }
        #[cfg(not(feature = "perf-observability"))]
        {
            ($operation)()
        }
    }};
}

/// Records a fallible phase and the caller-selected outcome for errors.
#[macro_export]
macro_rules! perf_span_result {
    ($phase:expr, $operation:expr, $classify_error:expr $(,)?) => {{
        #[cfg(feature = "perf-observability")]
        {
            ::telos_observability::PerfContext::current_thread().with_span_result($phase, |_| ($operation)(), $classify_error)
        }
        #[cfg(not(feature = "perf-observability"))]
        {
            ($operation)()
        }
    }};
}

/// Adds to the run-scoped counter when the feature is enabled.
#[macro_export]
macro_rules! perf_counter {
    ($counter:expr, $amount:expr $(,)?) => {{
        #[cfg(feature = "perf-observability")]
        {
            ::telos_observability::PerfContext::current_thread().record_counter($counter, $amount as u64);
        }
    }};
}
