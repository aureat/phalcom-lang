//! Executable compatibility is independent from compiler-image compatibility.

use crate::executable::constant::{MethodSignatureV1, StableClassRefV1};
use crate::executable::{EMITTER_ABI_V1, EXECUTABLE_SCHEMA_V1, NATIVE_RUNTIME_ABI_V1, PORTABLE_BYTECODE_ABI_V1, VM_OPERAND_FRAME_ABI_V1};
use crate::module_world::{Decode, Wire, record, tagged};
use crate::{ArtifactError, BinaryWriter, ValidationLimits};

tagged!(CompilationProfileV1 {
    0 => Debug,
    1 => Release,
    2 => Unchecked,
});

record!(PortableNativeAnchorV1 {
    binding_key: String,
    owner: StableClassRefV1,
    signature: MethodSignatureV1,
    contract_identity: Option<String>,
});

record!(PrimordialRoleV1 {
    role: String,
    class: StableClassRefV1,
});

record!(ExecutableCompatibilityV1 {
    executable_schema: u32,
    portable_bytecode_abi: u32,
    emitter_abi: u32,
    native_runtime_abi: u32,
    native_executable_abi_digest: [u8; 32],
    compilation_profile: CompilationProfileV1,
    vm_operand_frame_abi: u32,
});

impl ExecutableCompatibilityV1 {
    /// Builds the current compatibility header from canonically ordered ABI
    /// anchors. Canonical Universe builds use [`CompilationProfileV1::Debug`].
    pub fn current(
        compilation_profile: CompilationProfileV1,
        native_anchors: &[PortableNativeAnchorV1],
        primordial_roles: &[PrimordialRoleV1],
    ) -> Result<Self, ArtifactError> {
        Ok(Self {
            executable_schema: EXECUTABLE_SCHEMA_V1,
            portable_bytecode_abi: PORTABLE_BYTECODE_ABI_V1,
            emitter_abi: EMITTER_ABI_V1,
            native_runtime_abi: NATIVE_RUNTIME_ABI_V1,
            native_executable_abi_digest: native_executable_abi_digest(NATIVE_RUNTIME_ABI_V1, native_anchors, primordial_roles)?,
            compilation_profile,
            vm_operand_frame_abi: VM_OPERAND_FRAME_ABI_V1,
        })
    }
}

/// Hashes canonical native executable identity without observing function
/// addresses or registration order. Callers sort anchors by `binding_key` and
/// primordial roles by `role` before calling; the order is then validated and
/// streamed into the domain-separated digest.
pub fn native_executable_abi_digest(
    native_runtime_abi: u32,
    anchors: &[PortableNativeAnchorV1],
    primordial_roles: &[PrimordialRoleV1],
) -> Result<[u8; 32], ArtifactError> {
    fn encoded<T: Wire>(value: &T) -> Result<Vec<u8>, ArtifactError> {
        let mut writer = BinaryWriter::new();
        value.write(&mut writer)?;
        Ok(writer.into_vec())
    }

    if anchors.windows(2).any(|pair| pair[0].binding_key >= pair[1].binding_key) {
        return Err(ArtifactError::LimitExceeded("native ABI anchors must be strictly sorted by binding key".into()));
    }
    if primordial_roles.windows(2).any(|pair| pair[0].role >= pair[1].role) {
        return Err(ArtifactError::LimitExceeded("primordial roles must be strictly sorted by role".into()));
    }

    let mut hasher = blake3::Hasher::new_derive_key("phalcom.telosc.executable.native-abi.v1");
    hasher.update(&native_runtime_abi.to_le_bytes());
    hasher.update(
        &u64::try_from(anchors.len())
            .map_err(|_| ArtifactError::LimitExceeded("native anchor count exceeds digest width".into()))?
            .to_le_bytes(),
    );
    for anchor in anchors {
        let row = encoded(anchor)?;
        hasher.update(
            &u64::try_from(row.len())
                .map_err(|_| ArtifactError::LimitExceeded("native anchor row exceeds digest width".into()))?
                .to_le_bytes(),
        );
        hasher.update(&row);
    }
    hasher.update(
        &u64::try_from(primordial_roles.len())
            .map_err(|_| ArtifactError::LimitExceeded("primordial role count exceeds digest width".into()))?
            .to_le_bytes(),
    );
    for role in primordial_roles {
        let row = encoded(role)?;
        hasher.update(
            &u64::try_from(row.len())
                .map_err(|_| ArtifactError::LimitExceeded("primordial role row exceeds digest width".into()))?
                .to_le_bytes(),
        );
        hasher.update(&row);
    }
    Ok(*hasher.finalize().as_bytes())
}
