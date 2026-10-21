//! Non-authoritative executable section staging.
//!
//! A staged section has passed the structural image encoder, but it has not
//! passed the executable validator and cannot grant runtime authority.

use super::EXECUTABLE_SCHEMA_V1;
use super::image::ExecutableImageV1;
use crate::ArtifactError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedExecutableSectionV1 {
    payload: Vec<u8>,
}

impl StagedExecutableSectionV1 {
    /// Encodes a structurally valid image for a later validation/publication
    /// boundary. This value is not an executable authorization token.
    pub fn from_image(image: &ExecutableImageV1) -> Result<Self, ArtifactError> {
        Ok(Self { payload: image.encode()? })
    }

    pub const fn schema_version(&self) -> u32 {
        EXECUTABLE_SCHEMA_V1
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub(crate) fn into_payload(self) -> Vec<u8> {
        self.payload
    }
}
