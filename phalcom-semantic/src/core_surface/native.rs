//! Canonical native-surface import for compiler-owned core semantics.

use crate::declarations::DeclarationTypeTable;
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::ModuleId;
use crate::types::annotation::TypeResolver;
use crate::types::native::{NativeSurfaceImportError, NativeSurfaceImportReport, register_native_surfaces};
use crate::types::store::TypeStore;

/// Imports every generated native surface into semantic dispatch.
///
/// The native catalog is VM-free and immutable. This adapter is the only
/// compiler-owned entry point needed by workspace/checking setup.
pub fn register_core_native_surfaces(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    dispatch: &mut SurfaceDispatchResolver,
) -> Result<NativeSurfaceImportReport, NativeSurfaceImportError> {
    register_native_surfaces(store, declarations, resolver, current_module, dispatch)
}
