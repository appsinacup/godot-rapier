use godot::classes::ProjectSettings;
#[cfg(feature = "dim2")]
use godot::engine::physics_server_2d::*;
#[cfg(feature = "dim3")]
use godot::engine::physics_server_3d::*;
use godot::prelude::*;
use hashbrown::HashMap;
use hashbrown::HashSet;
use rapier::geometry::ColliderHandle;

use super::PhysicsDirectSpaceState;
use super::RapierDirectSpaceState;
use crate::bodies::rapier_collision_object::*;
use crate::rapier_wrapper::prelude::*;
use crate::servers::rapier_physics_singleton::*;
use crate::servers::rapier_project_settings::*;
use crate::*;
pub struct RemovedColliderInfo {
    pub rid: Rid,
    pub instance_id: u64,
    pub shape_index: usize,
    pub collision_object_type: CollisionObjectType,
}
impl RemovedColliderInfo {
    pub fn new(
        rid: Rid,
        instance_id: u64,
        shape_index: usize,
        collision_object_type: CollisionObjectType,
    ) -> Self {
        Self {
            rid,
            instance_id,
            shape_index,
            collision_object_type,
        }
    }
}
#[cfg(feature = "dim2")]
const DEFAULT_GRAVITY_VECTOR: &str = "physics/2d/default_gravity_vector";
#[cfg(feature = "dim3")]
const DEFAULT_GRAVITY_VECTOR: &str = "physics/3d/default_gravity_vector";
#[cfg(feature = "dim2")]
const DEFAULT_GRAVITY: &str = "physics/2d/default_gravity";
#[cfg(feature = "dim3")]
const DEFAULT_GRAVITY: &str = "physics/3d/default_gravity";
pub struct RapierSpace {
    direct_access: Option<Gd<PhysicsDirectSpaceState>>,
    handle: Handle,
    removed_colliders: HashMap<ColliderHandle, RemovedColliderInfo>,
    active_list: HashSet<Rid>,
    mass_properties_update_list: HashSet<Rid>,
    gravity_update_list: HashSet<Rid>,
    state_query_list: HashSet<Rid>,
    monitor_query_list: HashSet<Rid>,
    area_update_list: HashSet<Rid>,
    body_area_update_list: HashSet<Rid>,
    contact_max_allowed_penetration: real,
    default_gravity_dir: Vector,
    default_gravity_value: real,
    default_linear_damping: real,
    default_angular_damping: real,
    island_count: i32,
    active_objects: i32,
    collision_pairs: i32,
    contact_debug: PackedVectorArray,
    contact_debug_count: usize,
}
impl RapierSpace {
    pub fn new(rid: Rid) -> Self {
        let mut direct_access = RapierDirectSpaceState::new_alloc();
        direct_access.bind_mut().set_space(rid);
        let world_settings = WorldSettings {
            particle_radius: RapierProjectSettings::get_fluid_particle_radius() as real,
            smoothing_factor: RapierProjectSettings::get_fluid_smoothing_factor() as real,
            counters_enabled: RapierProjectSettings::counters_enabled(),
        };
        let handle = world_create(&world_settings);
        let project_settings = ProjectSettings::singleton();
        let default_gravity_dir: Vector = project_settings
            .get_setting_with_override(DEFAULT_GRAVITY_VECTOR.into())
            .to();
        let default_gravity_value: real = project_settings
            .get_setting_with_override(DEFAULT_GRAVITY.into())
            .to();
        Self {
            direct_access: Some(direct_access.upcast()),
            handle,
            removed_colliders: HashMap::default(),
            active_list: HashSet::default(),
            mass_properties_update_list: HashSet::default(),
            gravity_update_list: HashSet::default(),
            state_query_list: HashSet::default(),
            monitor_query_list: HashSet::default(),
            area_update_list: HashSet::default(),
            body_area_update_list: HashSet::default(),
            contact_max_allowed_penetration: 0.0,
            default_gravity_dir,
            default_gravity_value,
            default_linear_damping: 0.0,
            default_angular_damping: 0.0,
            island_count: 0,
            active_objects: 0,
            collision_pairs: 0,
            contact_debug: PackedVectorArray::new(),
            contact_debug_count: 0,
        }
    }

    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    pub fn is_valid(&self) -> bool {
        self.handle.is_valid()
    }

    pub fn body_add_to_mass_properties_update_list(&mut self, body: Rid) {
        self.mass_properties_update_list.insert(body);
    }

    pub fn body_remove_from_mass_properties_update_list(&mut self, body: Rid) {
        self.mass_properties_update_list.remove(&body);
    }

    pub fn body_add_to_gravity_update_list(&mut self, body: Rid) {
        self.gravity_update_list.insert(body);
    }

    pub fn body_remove_from_gravity_update_list(&mut self, body: Rid) {
        self.gravity_update_list.remove(&body);
    }

    pub fn body_add_to_active_list(&mut self, body: Rid) {
        self.active_list.insert(body);
    }

    pub fn body_remove_from_active_list(&mut self, body: Rid) {
        self.active_list.remove(&body);
    }

    pub fn body_add_to_state_query_list(&mut self, body: Rid) {
        self.state_query_list.insert(body);
    }

    pub fn body_remove_from_state_query_list(&mut self, body: Rid) {
        self.state_query_list.remove(&body);
    }

    pub fn area_add_to_monitor_query_list(&mut self, area: Rid) {
        self.monitor_query_list.insert(area);
    }

    pub fn area_add_to_area_update_list(&mut self, area: Rid) {
        self.area_update_list.insert(area);
    }

    pub fn area_remove_from_area_update_list(&mut self, area: Rid) {
        self.area_update_list.remove(&area);
    }

    pub fn body_add_to_area_update_list(&mut self, body: Rid) {
        self.body_area_update_list.insert(body);
    }

    pub fn body_remove_from_area_update_list(&mut self, body: Rid) {
        self.body_area_update_list.remove(&body);
    }

    pub fn add_removed_collider(
        &mut self,
        handle: ColliderHandle,
        rid: Rid,
        instance_id: u64,
        shape_index: usize,
        collision_object_type: CollisionObjectType,
    ) {
        self.removed_colliders.insert(
            handle,
            RemovedColliderInfo::new(rid, instance_id, shape_index, collision_object_type),
        );
    }

    pub fn get_removed_collider_info(
        &mut self,
        handle: &ColliderHandle,
    ) -> Option<&RemovedColliderInfo> {
        self.removed_colliders.get(handle)
    }

    pub fn call_queries(&mut self) {
        for body_rid in self.state_query_list.clone() {
            if let Some(body) = bodies_singleton().collision_objects.get_mut(&body_rid) {
                if let Some(body) = body.get_mut_body() {
                    if !body.is_active() {
                        self.body_remove_from_state_query_list(body.get_base().get_rid());
                    }
                    if let Some(direct_state) = body.get_direct_state().cloned() {
                        let fi_callback_data = body.get_force_integration_callback();
                        if let Some(fi_callback_data) = fi_callback_data {
                            if fi_callback_data.callable.is_valid() {
                                let mut arg_array = Array::new();
                                arg_array.push(direct_state.to_variant());
                                arg_array.push(fi_callback_data.udata.to_variant());
                                fi_callback_data.callable.callv(arg_array);
                            }
                        }
                        let state_sync_callback = body.get_state_sync_callback();
                        //  Sync body server with Godot by sending body direct state
                        if state_sync_callback.is_valid() {
                            let mut arg_array = Array::new();
                            arg_array.push(direct_state.to_variant());
                            state_sync_callback.callv(arg_array);
                        }
                    }
                }
            }
        }
        for area_rid in self.monitor_query_list.clone() {
            if let Some(area) = bodies_singleton().collision_objects.get_mut(&area_rid) {
                if let Some(area) = area.get_mut_area() {
                    area.call_queries();
                }
            }
        }
    }

    pub fn get_last_step() -> real {
        let project_settings = ProjectSettings::singleton();
        let physics_fps = project_settings
            .get_setting_with_override("physics/common/physics_ticks_per_second".into());
        let mut last_step = 1e-3;
        if !physics_fps.is_nil() {
            last_step = 1.0 / (physics_fps.to::<i32>() as f32);
        }
        last_step
    }

    pub fn set_default_area_param(&mut self, param: AreaParameter, value: Variant) {
        match param {
            AreaParameter::GRAVITY => self.default_gravity_value = value.to(),
            AreaParameter::GRAVITY_VECTOR => self.default_gravity_dir = value.to(),
            AreaParameter::LINEAR_DAMP => self.default_linear_damping = value.to(),
            AreaParameter::ANGULAR_DAMP => self.default_angular_damping = value.to(),
            _ => {}
        }
    }

    pub fn get_default_area_param(&self, param: AreaParameter) -> Variant {
        match param {
            AreaParameter::GRAVITY => self.default_gravity_value.to_variant(),
            AreaParameter::GRAVITY_VECTOR => self.default_gravity_dir.to_variant(),
            AreaParameter::LINEAR_DAMP => self.default_linear_damping.to_variant(),
            AreaParameter::ANGULAR_DAMP => self.default_angular_damping.to_variant(),
            _ => (0.0).to_variant(),
        }
    }

    pub fn get_island_count(&self) -> i32 {
        self.island_count
    }

    pub fn get_active_objects(&self) -> i32 {
        self.active_objects
    }

    pub fn get_collision_pairs(&self) -> i32 {
        self.collision_pairs
    }

    pub fn set_debug_contacts(&mut self, max_contacts: i32) {
        self.contact_debug.resize(max_contacts as usize);
    }

    pub fn is_debugging_contacts(&self) -> bool {
        !self.contact_debug.is_empty()
    }

    pub fn add_debug_contact(&mut self, contact: Vector) {
        if self.contact_debug_count < self.contact_debug.len() {
            self.contact_debug[self.contact_debug_count] = contact;
            self.contact_debug_count += 1;
        }
    }

    pub fn get_debug_contacts(&self) -> &PackedVectorArray {
        &self.contact_debug
    }

    pub fn get_debug_contact_count(&self) -> i32 {
        self.contact_debug_count as i32
    }

    pub fn before_step(&mut self) {
        self.contact_debug_count = 0
    }

    pub fn after_step(&mut self) {
        // Needed only for one physics step to retrieve lost info
        self.removed_colliders.clear();
        self.active_objects = world_get_active_objects_count(self.handle) as i32;
        for body in self.active_list.clone() {
            if let Some(body) = bodies_singleton().collision_objects.get_mut(&body) {
                if let Some(body) = body.get_mut_body() {
                    body.on_update_active(self);
                }
            }
        }
    }

    pub fn get_direct_state(&self) -> &Option<Gd<PhysicsDirectSpaceState>> {
        &self.direct_access
    }

    pub fn get_active_list(&self) -> &HashSet<Rid> {
        &self.active_list
    }

    pub fn get_mass_properties_update_list(&self) -> &HashSet<Rid> {
        &self.mass_properties_update_list
    }

    pub fn reset_mass_properties_update_list(&mut self) {
        self.mass_properties_update_list.clear();
    }

    pub fn get_area_update_list(&self) -> &HashSet<Rid> {
        &self.area_update_list
    }

    pub fn get_body_area_update_list(&self) -> &HashSet<Rid> {
        &self.body_area_update_list
    }

    pub fn get_gravity_update_list(&self) -> &HashSet<Rid> {
        &self.gravity_update_list
    }

    pub fn get_contact_max_allowed_penetration(&self) -> real {
        self.contact_max_allowed_penetration
    }

    pub fn export_json(&self) -> String {
        world_export_json(self.handle)
    }

    pub fn export_binary(&self) -> PackedByteArray {
        let mut buf = PackedByteArray::new();
        let binary_data = world_export_binary(self.handle);
        buf.resize(binary_data.len());
        for i in 0..binary_data.len() {
            buf[i] = binary_data[i];
        }
        buf
    }
}
impl Drop for RapierSpace {
    fn drop(&mut self) {
        if self.handle.is_valid() {
            world_destroy(self.handle);
        }
    }
}
