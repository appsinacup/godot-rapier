use rapier::data::Index;
use rapier::prelude::*;
use salva::object::ContiguousArenaIndex;
use salva::object::FluidHandle;
use serde::Deserialize;
use serde::Serialize;
#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct Handle {
    id: u32,
    generation: u32,
}
impl Default for Handle {
    fn default() -> Handle {
        Handle {
            id: u32::MAX,
            generation: u32::MAX,
        }
    }
}
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct HandleDouble {
    id: usize,
    generation: u64,
}
impl Default for HandleDouble {
    fn default() -> HandleDouble {
        HandleDouble {
            id: usize::MAX,
            generation: u64::MAX,
        }
    }
}
impl Handle {
    pub fn is_valid(&self) -> bool {
        (self.id != u32::MAX) && (self.generation != u32::MAX)
    }
}
impl HandleDouble {
    pub fn is_valid(&self) -> bool {
        (self.id != usize::MAX) && (self.generation != u64::MAX)
    }
}
pub fn world_handle_to_handle(world_handle: Index) -> Handle {
    let raw_parts = world_handle.into_raw_parts();
    Handle {
        id: raw_parts.0,
        generation: raw_parts.1,
    }
}
pub fn handle_to_world_handle(handle: Handle) -> Index {
    Index::from_raw_parts(handle.id, handle.generation)
}
pub fn shape_handle_to_handle(shape_handle: Index) -> Handle {
    let raw_parts = shape_handle.into_raw_parts();
    Handle {
        id: raw_parts.0,
        generation: raw_parts.1,
    }
}
pub fn handle_to_shape_handle(handle: Handle) -> Index {
    Index::from_raw_parts(handle.id, handle.generation)
}
pub fn fluid_handle_to_handle(fluid_handle: FluidHandle) -> HandleDouble {
    let contiguous_index: ContiguousArenaIndex = fluid_handle.into();
    let raw_parts = contiguous_index.into_raw_parts();
    HandleDouble {
        id: raw_parts.0,
        generation: raw_parts.1,
    }
}
pub fn handle_to_fluid_handle(handle: HandleDouble) -> FluidHandle {
    FluidHandle::from(ContiguousArenaIndex::from_raw_parts(
        handle.id,
        handle.generation,
    ))
}
pub fn invalid_handle() -> Handle {
    Handle {
        id: u32::MAX,
        generation: u32::MAX,
    }
}
pub fn invalid_handle_double() -> HandleDouble {
    HandleDouble {
        id: usize::MAX,
        generation: u64::MAX,
    }
}
pub fn handle_pair_hash(
    handle1: ColliderHandle,
    handle2: ColliderHandle,
) -> (ColliderHandle, ColliderHandle) {
    (handle1, handle2)
}
