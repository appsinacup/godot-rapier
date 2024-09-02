use godot::prelude::*;
use hashbrown::HashMap;
use rapier::prelude::RigidBodyHandle;

use crate::bodies::rapier_collision_object::IRapierCollisionObject;
use crate::rapier_wrapper::prelude::*;
use crate::servers::rapier_physics_singleton::{get_rid, PhysicsData};
use crate::types::*;
pub struct RapierShapeBase {
    rid: Rid,
    aabb: Rect,
    owners: HashMap<RigidBodyHandle, i32>,
    handle: ShapeHandle,
}
impl Default for RapierShapeBase {
    fn default() -> Self {
        Self {
            rid: Rid::Invalid,
            aabb: Rect::default(),
            owners: HashMap::default(),
            handle: ShapeHandle::default(),
        }
    }
}
impl RapierShapeBase {
    pub(super) fn new(rid: Rid) -> Self {
        Self {
            rid,
            aabb: Rect::default(),
            owners: HashMap::default(),
            handle: ShapeHandle::default(),
        }
    }

    pub(super) fn set_handle(&mut self, handle: ShapeHandle, physics_engine: &mut PhysicsEngine) {
        if self.handle != ShapeHandle::default() {
            self.destroy_shape(physics_engine);
        }
        let rapier_aabb = physics_engine.shape_get_aabb(handle);
        let vertices = rapier_aabb.vertices();
        self.aabb = Rect::new(
            vector_to_godot(vertices[0].coords),
            vector_to_godot(rapier_aabb.extents()),
        );
        self.handle = handle;
    }

    pub fn get_handle(&self) -> ShapeHandle {
        self.handle
    }

    pub fn is_valid(&self) -> bool {
        self.handle != ShapeHandle::default()
    }

    pub fn call_shape_changed(
        owners: HashMap<RigidBodyHandle, i32>,
        shape_rid: Rid,
        physics_data: &mut PhysicsData,
    ) {
        for (owner, _) in owners {
            if let Some(owner) = physics_data.collision_objects.get_mut(get_rid(owner.0)) {
                owner.shape_changed(
                    shape_rid,
                    &mut physics_data.physics_engine,
                    &mut physics_data.shapes,
                    &mut physics_data.spaces,
                );
            }
        }
    }

    pub fn get_aabb(&self, origin: Vector) -> Rect {
        let mut aabb_clone = self.aabb;
        aabb_clone.position += origin;
        aabb_clone
    }

    pub fn add_owner(&mut self, owner: RigidBodyHandle) {
        *self.owners.entry(owner).or_insert(0) += 1;
    }

    pub fn remove_owner(&mut self, owner: RigidBodyHandle) {
        if let Some(count) = self.owners.get_mut(&owner) {
            *count -= 1;
            if *count == 0 {
                self.owners.remove(&owner);
            }
        }
    }

    pub fn get_owners(&self) -> &HashMap<RigidBodyHandle, i32> {
        &self.owners
    }

    pub fn get_rid(&self) -> Rid {
        self.rid
    }

    pub fn destroy_shape(&mut self, physics_engine: &mut PhysicsEngine) {
        if self.handle != ShapeHandle::default() {
            physics_engine.shape_destroy(self.handle);
            self.handle = ShapeHandle::default();
        }
    }
}
impl Drop for RapierShapeBase {
    fn drop(&mut self) {
        if !self.owners.is_empty() {
            godot_error!("RapierShapeBase leaked {} owners", self.owners.len());
        }
        if self.is_valid() {
            godot_error!("RapierShapeBase leaked");
        }
    }
}
