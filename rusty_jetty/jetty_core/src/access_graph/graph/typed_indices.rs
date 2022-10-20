//! Typed graph indices
//!
use crate::access_graph::NodeIndex;

pub(crate) trait ToNodeIndex {
    fn get_index(&self) -> NodeIndex;
}

impl ToNodeIndex for NodeIndex {
    fn get_index(&self) -> NodeIndex {
        *self
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct AssetIndex {
    idx: NodeIndex,
}
impl ToNodeIndex for AssetIndex {
    fn get_index(&self) -> NodeIndex {
        self.idx
    }
}

struct UserIndex {
    idx: NodeIndex,
}
impl ToNodeIndex for UserIndex {
    fn get_index(&self) -> NodeIndex {
        self.idx
    }
}

struct GroupIndex {
    idx: NodeIndex,
}
impl ToNodeIndex for GroupIndex {
    fn get_index(&self) -> NodeIndex {
        self.idx
    }
}

struct TagIndex {
    idx: NodeIndex,
}
impl ToNodeIndex for TagIndex {
    fn get_index(&self) -> NodeIndex {
        self.idx
    }
}
