use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub static FULL_VM_BOOTSTRAPS: AtomicUsize = AtomicUsize::new(0);
pub static CANONICAL_UNIVERSE_PRODUCT_BUILDS: AtomicUsize = AtomicUsize::new(0);
pub static BOOTSTRAP_IMAGE_INSTANTIATIONS: AtomicUsize = AtomicUsize::new(0);
pub static UNIVERSE_MODULE_COMPILATIONS: AtomicUsize = AtomicUsize::new(0);
pub static UNIVERSE_INITIALIZER_EXECUTIONS: AtomicUsize = AtomicUsize::new(0);

pub fn full_vm_bootstrap_count() -> usize {
    FULL_VM_BOOTSTRAPS.load(Ordering::Relaxed)
}

pub fn canonical_universe_product_build_count() -> usize {
    CANONICAL_UNIVERSE_PRODUCT_BUILDS.load(Ordering::Relaxed)
}

pub fn bootstrap_image_instantiation_count() -> usize {
    BOOTSTRAP_IMAGE_INSTANTIATIONS.load(Ordering::Relaxed)
}

pub fn universe_module_compilation_count() -> usize {
    UNIVERSE_MODULE_COMPILATIONS.load(Ordering::Relaxed)
}

pub fn universe_initializer_execution_count() -> usize {
    UNIVERSE_INITIALIZER_EXECUTIONS.load(Ordering::Relaxed)
}

pub fn reset_core_counters() {
    FULL_VM_BOOTSTRAPS.store(0, Ordering::Relaxed);
    CANONICAL_UNIVERSE_PRODUCT_BUILDS.store(0, Ordering::Relaxed);
    BOOTSTRAP_IMAGE_INSTANTIATIONS.store(0, Ordering::Relaxed);
    UNIVERSE_MODULE_COMPILATIONS.store(0, Ordering::Relaxed);
    UNIVERSE_INITIALIZER_EXECUTIONS.store(0, Ordering::Relaxed);
}

/// Returns the number of modules in the canonical eager bootstrap order.
pub fn canonical_bootstrap_module_count() -> Result<usize, String> {
    crate::modules::canonical_universe_program()
        .map(|program| program.bootstrap_order().len())
        .map_err(ToString::to_string)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct VmBootstrapTiming {
    pub canonical_universe_product: Duration,
    pub kernel_native: Duration,
    pub universe_module_compilation: Duration,
    pub universe_initializer_execution: Duration,
}

#[inline]
pub fn record_full_vm_bootstrap() {
    FULL_VM_BOOTSTRAPS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_canonical_universe_product_build() {
    CANONICAL_UNIVERSE_PRODUCT_BUILDS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_bootstrap_image_instantiation() {
    BOOTSTRAP_IMAGE_INSTANTIATIONS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_universe_module_compilation() {
    UNIVERSE_MODULE_COMPILATIONS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_universe_initializer_execution() {
    UNIVERSE_INITIALIZER_EXECUTIONS.fetch_add(1, Ordering::Relaxed);
}

#[derive(Debug, Default, Clone)]
pub struct FixtureTiming {
    pub compile_io: Duration,
    pub compile_parse: Duration,
    pub compile_link: Duration,
    pub semantic_session_init: Duration,
    pub semantic_update: Duration,
    pub compile_projection: Duration,

    pub canonical_universe_product: Duration,
    pub vm_kernel_native: Duration,
    pub vm_universe_compile: Duration,
    pub vm_universe_execute: Duration,

    pub user_execution: Duration,
}

impl FixtureTiming {
    pub fn is_timings_enabled() -> bool {
        std::env::var("PHALCOM_TEST_TIMINGS")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    pub fn print_if_enabled(&self, label: &str) {
        if Self::is_timings_enabled() {
            eprintln!(
                "[TIMING] {label} - io: {:?}, parse: {:?}, discovery+link: {:?}, semantic_session_init: {:?}, semantic_update: {:?}, user_projection: {:?}, canonical_universe_product: {:?}, kernel+native: {:?}, universe_vm_compile: {:?}, universe_initializers: {:?}, user_execution: {:?}",
                self.compile_io,
                self.compile_parse,
                self.compile_link,
                self.semantic_session_init,
                self.semantic_update,
                self.compile_projection,
                self.canonical_universe_product,
                self.vm_kernel_native,
                self.vm_universe_compile,
                self.vm_universe_execute,
                self.user_execution
            );
        }
    }
}
