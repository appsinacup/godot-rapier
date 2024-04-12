use rapier2d::data::Index;
use rapier2d::prelude::*;
use salva2d::object::FluidHandle;
use salva2d::object::ContiguousArenaIndex;

#[repr(C)]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Handle {
    id : u32,
    generation : u32,
}
impl Default for Handle {
    fn default() -> Handle {
        Handle {
            id : u32::MAX,
            generation : u32::MAX,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct HandleDouble {
    id : usize,
    generation : u64,
}
impl Default for HandleDouble {
    fn default() -> HandleDouble {
        HandleDouble {
            id : usize::MAX,
            generation : u64::MAX,
        }
    }
}


impl Handle {
    pub fn is_valid(&self) -> bool {
        return (self.id != u32::MAX) && (self.generation != u32::MAX);
    }
}

impl HandleDouble {
    pub fn is_valid(&self) -> bool {
        return (self.id != usize::MAX) && (self.generation != u64::MAX);
    }
}

pub fn world_handle_to_handle(world_handle : Index) -> Handle {
    let raw_parts = world_handle.into_raw_parts();
    return Handle {
        id : raw_parts.0,
        generation : raw_parts.1,
    }
}


pub fn handle_to_world_handle(handle : Handle) -> Index {
    return Index::from_raw_parts(handle.id, handle.generation);
}

pub fn shape_handle_to_handle(shape_handle : Index) -> Handle {
    let raw_parts = shape_handle.into_raw_parts();
    return Handle {
        id : raw_parts.0,
        generation : raw_parts.1,
    }
}


pub fn handle_to_shape_handle(handle : Handle) -> Index {
    return Index::from_raw_parts(handle.id, handle.generation);
}

pub fn collider_handle_to_handle(collider_handle : ColliderHandle) -> Handle {
    let raw_parts = collider_handle.into_raw_parts();
    return Handle {
        id : raw_parts.0,
        generation : raw_parts.1,
    }
}

pub fn handle_to_collider_handle(handle : Handle) -> ColliderHandle {
    return ColliderHandle::from_raw_parts(handle.id, handle.generation);
}

pub fn rigid_body_handle_to_handle(rigid_body_handle : RigidBodyHandle) -> Handle {
    let raw_parts = rigid_body_handle.into_raw_parts();
    return Handle {
        id : raw_parts.0,
        generation : raw_parts.1,
    }
}

pub fn handle_to_rigid_body_handle(handle : Handle) -> RigidBodyHandle {
    return RigidBodyHandle::from_raw_parts(handle.id, handle.generation);
}

pub fn joint_handle_to_handle(joint_handle : ImpulseJointHandle) -> Handle {
    let raw_parts = joint_handle.into_raw_parts();
    return Handle {
        id : raw_parts.0,
        generation : raw_parts.1,
    }
}

pub fn handle_to_joint_handle(handle : Handle) -> ImpulseJointHandle {
    return ImpulseJointHandle::from_raw_parts(handle.id, handle.generation);
}

pub fn fluid_handle_to_handle(fluid_handle : FluidHandle) -> HandleDouble {
    let contiguous_index: ContiguousArenaIndex = fluid_handle.into();
    let raw_parts = contiguous_index.into_raw_parts();
    return HandleDouble {
        id : raw_parts.0,
        generation : raw_parts.1,
    }
}

pub fn handle_to_fluid_handle(handle : HandleDouble) -> FluidHandle {
    return FluidHandle::from(ContiguousArenaIndex::from_raw_parts(handle.id, handle.generation));
}


#[no_mangle]
pub extern "C" fn invalid_handle() -> Handle {
    Handle {
        id : u32::MAX,
        generation : u32::MAX,
    }
}

#[no_mangle]
pub extern "C" fn invalid_handle_double() -> HandleDouble {
    HandleDouble {
        id : usize::MAX,
        generation : u64::MAX,
    }
}

#[no_mangle]
pub extern "C" fn is_handle_valid(handle : Handle) -> bool {
    return handle.is_valid();
}

#[no_mangle]
pub extern "C" fn is_handle_valid_double(handle : HandleDouble) -> bool {
    return handle.is_valid();
}

#[no_mangle]
pub extern "C" fn are_handles_equal(handle1 : Handle, handle2 : Handle) -> bool {
    return (handle1.id == handle2.id) && (handle1.generation == handle2.generation);
}

#[no_mangle]
pub extern "C" fn are_handles_equal_double(handle1 : HandleDouble, handle2 : HandleDouble) -> bool {
    return (handle1.id == handle2.id) && (handle1.generation == handle2.generation);
}
