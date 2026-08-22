//! Indexed kind graph.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct KindNodeId(pub u32);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum KindNode {
    Type,
    RecordRow,
    Arrow { parameters: Box<[KindNodeId]>, result: KindNodeId },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct KindNodeEntry {
    pub node: KindNode,
    pub structural_fingerprint: Fingerprint128,
}
