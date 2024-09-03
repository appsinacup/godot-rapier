use bodies::rapier_area::RapierArea;
use bodies::rapier_body::RapierBody;
use bodies::rapier_collision_object_base::CollisionObjectType;
#[cfg(feature = "dim2")]
use godot::classes::physics_server_2d::*;
#[cfg(feature = "dim3")]
use godot::classes::physics_server_3d::*;
use godot::classes::ProjectSettings;
use godot::prelude::*;
use hashbrown::HashSet;
use rapier::geometry::ColliderHandle;
use rapier::prelude::RigidBodyHandle;
use servers::rapier_physics_singleton::get_rid;
use servers::rapier_physics_singleton::PhysicsCollisionObjects;
use servers::rapier_physics_singleton::PhysicsData;
use spaces::rapier_space_state::RapierSpaceState;
use spaces::rapier_space_state::RemovedColliderInfo;

use super::PhysicsDirectSpaceState;
use super::RapierDirectSpaceState;
use crate::bodies::rapier_collision_object::*;
use crate::rapier_wrapper::prelude::*;
use crate::servers::rapier_project_settings::*;
use crate::types::*;
use crate::*;
#[cfg(feature = "dim2")]
const DEFAULT_GRAVITY_VECTOR: &str = "physics/2d/default_gravity_vector";
#[cfg(feature = "dim3")]
const DEFAULT_GRAVITY_VECTOR: &str = "physics/3d/default_gravity_vector";
#[cfg(feature = "dim2")]
const DEFAULT_GRAVITY: &str = "physics/2d/default_gravity";
#[cfg(feature = "dim3")]
const DEFAULT_GRAVITY: &str = "physics/3d/default_gravity";
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct SpaceExport<'a> {
    space: &'a RapierSpaceState,
    world: &'a PhysicsObjects,
}
#[cfg_attr(feature = "serde-serialize", derive(serde::Deserialize))]
pub struct SpaceImport {
    space: RapierSpaceState,
    world: PhysicsObjects,
}
pub struct RapierSpace {
    direct_access: Option<Gd<PhysicsDirectSpaceState>>,
    handle: WorldHandle,
    state: RapierSpaceState,
    contact_max_allowed_penetration: real,
    default_gravity_dir: Vector,
    default_gravity_value: real,
    default_linear_damping: real,
    default_angular_damping: real,
    contact_debug: PackedVectorArray,
    contact_debug_count: usize,
    ghost_collision_distance: real,
}
impl RapierSpace {
    pub fn new(rid: Rid, physics_engine: &mut PhysicsEngine) -> Self {
        let mut direct_access = RapierDirectSpaceState::new_alloc();
        direct_access.bind_mut().set_space(rid);
        let world_settings = WorldSettings {
            particle_radius: RapierProjectSettings::get_fluid_particle_radius() as real,
            smoothing_factor: RapierProjectSettings::get_fluid_smoothing_factor() as real,
            counters_enabled: false,
        };
        let handle = physics_engine.world_create(&world_settings);
        let project_settings = ProjectSettings::singleton();
        let default_gravity_dir: Vector = project_settings
            .get_setting_with_override(DEFAULT_GRAVITY_VECTOR.into())
            .try_to()
            .unwrap_or_default();
        let default_gravity_value =
            variant_to_float(&project_settings.get_setting_with_override(DEFAULT_GRAVITY.into()));
        Self {
            direct_access: Some(direct_access.upcast()),
            handle,
            state: RapierSpaceState::default(),
            contact_max_allowed_penetration: 0.0,
            default_gravity_dir,
            default_gravity_value,
            default_linear_damping: 0.0,
            default_angular_damping: 0.0,
            contact_debug: PackedVectorArray::new(),
            contact_debug_count: 0,
            ghost_collision_distance: RapierProjectSettings::get_ghost_collision_distance(),
        }
    }

    pub fn get_handle(&self) -> WorldHandle {
        self.handle
    }

    pub fn is_valid(&self) -> bool {
        self.handle != WorldHandle::default()
    }

    pub fn body_add_to_mass_properties_update_list(&mut self, body: RigidBodyHandle) {
        self.state.mass_properties_update_list.insert(body);
    }

    pub fn body_remove_from_mass_properties_update_list(&mut self, body: RigidBodyHandle) {
        self.state.mass_properties_update_list.remove(&body);
    }

    pub fn body_add_to_gravity_update_list(&mut self, body: RigidBodyHandle) {
        self.state.gravity_update_list.insert(body);
    }

    pub fn body_remove_from_gravity_update_list(&mut self, body: RigidBodyHandle) {
        self.state.gravity_update_list.remove(&body);
    }

    pub fn body_add_to_active_list(&mut self, body: RigidBodyHandle) {
        self.state.active_list.insert(body);
    }

    pub fn body_remove_from_active_list(&mut self, body: RigidBodyHandle) {
        self.state.active_list.remove(&body);
    }

    pub fn body_add_to_state_query_list(&mut self, body: RigidBodyHandle) {
        self.state.state_query_list.insert(body);
    }

    pub fn body_remove_from_state_query_list(&mut self, body: RigidBodyHandle) {
        self.state.state_query_list.remove(&body);
    }

    pub fn body_add_to_force_integrate_list(&mut self, body: RigidBodyHandle) {
        self.state.force_integrate_query_list.insert(body);
    }

    pub fn body_remove_from_force_integrate_list(&mut self, body: RigidBodyHandle) {
        self.state.force_integrate_query_list.remove(&body);
    }

    pub fn area_add_to_monitor_query_list(&mut self, area: RigidBodyHandle) {
        self.state.monitor_query_list.insert(area);
    }

    pub fn area_add_to_area_update_list(&mut self, area: RigidBodyHandle) {
        self.state.area_update_list.insert(area);
    }

    pub fn area_remove_from_area_update_list(&mut self, area: RigidBodyHandle) {
        self.state.area_update_list.remove(&area);
    }

    pub fn body_add_to_area_update_list(&mut self, body: RigidBodyHandle) {
        self.state.body_area_update_list.insert(body);
    }

    pub fn body_remove_from_area_update_list(&mut self, body: RigidBodyHandle) {
        self.state.body_area_update_list.remove(&body);
    }

    pub fn add_removed_collider(
        &mut self,
        handle: ColliderHandle,
        rid: Rid,
        rb_handle: RigidBodyHandle,
        instance_id: u64,
        shape_index: usize,
        collision_object_type: CollisionObjectType,
    ) {
        self.state.removed_colliders.insert(
            handle,
            RemovedColliderInfo::new(
                rid,
                rb_handle,
                instance_id,
                shape_index,
                collision_object_type,
            ),
        );
    }

    pub fn get_removed_collider_info(
        &mut self,
        handle: &ColliderHandle,
    ) -> Option<&RemovedColliderInfo> {
        self.state.removed_colliders.get(handle)
    }

    pub fn get_queries(
        &mut self,
        physics_data_collision_objects: &mut PhysicsCollisionObjects,
    ) -> Vec<(Callable, Vec<Variant>)> {
        let mut queries = Vec::default();
        for body_handle in self
            .state
            .state_query_list
            .union(&self.state.force_integrate_query_list)
        {
            if let Some(body) = physics_data_collision_objects.get_mut(get_rid(body_handle.0)) {
                if let Some(body) = body.get_mut_body() {
                    body.create_direct_state();
                }
            }
            if let Some(body) = physics_data_collision_objects.get(get_rid(body_handle.0)) {
                if let Some(body) = body.get_body() {
                    if let Some(direct_state) = body.get_direct_state() {
                        if let Some(state_sync_callback) = body.get_state_sync_callback() {
                            queries.push((
                                state_sync_callback.clone(),
                                vec![direct_state.to_variant()],
                            ));
                        }
                        if let Some(direct_state) = body.get_direct_state() {
                            if let Some(fi_callback_data) = body.get_force_integration_callback() {
                                if fi_callback_data.udata.is_nil() {
                                    queries.push((
                                        fi_callback_data.callable.clone(),
                                        vec![direct_state.to_variant()],
                                    ));
                                } else {
                                    queries.push((
                                        fi_callback_data.callable.clone(),
                                        vec![
                                            direct_state.to_variant(),
                                            fi_callback_data.udata.clone(),
                                        ],
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        for area_handle in self.state.monitor_query_list.clone() {
            if let Some(area) = physics_data_collision_objects.get(get_rid(area_handle.0)) {
                if let Some(area) = area.get_area() {
                    let area_queries = &mut area.get_queries();
                    queries.append(area_queries);
                }
            }
        }
        queries
    }

    pub fn update_after_queries(
        &mut self,
        physics_data_collision_objects: &mut PhysicsCollisionObjects,
    ) {
        for area_handle in self.state.monitor_query_list.clone() {
            if let Some(area) = physics_data_collision_objects.get_mut(get_rid(area_handle.0)) {
                if let Some(area) = area.get_mut_area() {
                    area.clear_monitored_objects();
                }
            }
        }
    }

    pub fn step(
        step: real,
        space_rid: &Rid,
        physics_data: &mut PhysicsData,
        settings: SimulationSettings,
    ) {
        let mut area_update_list = HashSet::default();
        if let Some(space) = physics_data.spaces.get_mut(space_rid) {
            space.state.time_stepped += step;
            area_update_list = space.get_area_update_list().clone();
        }
        for area in area_update_list {
            RapierArea::update_area_override(
                &mut physics_data.collision_objects,
                &mut physics_data.spaces,
                &mut physics_data.physics_engine,
                &area,
            );
        }
        let Some(space) = physics_data.spaces.get_mut(space_rid) else {
            return;
        };
        let body_area_update_list = space.get_body_area_update_list().clone();
        let gravity_update_list = space.get_gravity_update_list().clone();
        let default_gravity_value: real =
            variant_to_float(&space.get_default_area_param(AreaParameter::GRAVITY));
        let default_gravity_dir = space
            .get_default_area_param(AreaParameter::GRAVITY_VECTOR)
            .try_to()
            .unwrap_or_default();
        let space_handle = space.get_handle();
        space.before_step();
        for body in space.get_active_list() {
            if let Some(body) = physics_data.collision_objects.get_mut(get_rid(body.0))
                && let Some(body) = body.get_mut_body()
            {
                body.reset_contact_count();
            }
        }
        for body in space.get_mass_properties_update_list() {
            if let Some(body) = physics_data.collision_objects.get_mut(get_rid(body.0))
                && let Some(body) = body.get_mut_body()
            {
                body.update_mass_properties(false, &mut physics_data.physics_engine);
            }
        }
        space.reset_mass_properties_update_list();
        for body in &body_area_update_list {
            RapierBody::apply_area_override_to_body(
                body,
                &mut physics_data.physics_engine,
                &mut physics_data.spaces,
                &mut physics_data.collision_objects,
            );
        }
        for body in gravity_update_list {
            // first update the area override
            RapierBody::apply_area_override_to_body(
                &body,
                &mut physics_data.physics_engine,
                &mut physics_data.spaces,
                &mut physics_data.collision_objects,
            );
            if let Some(body) = physics_data.collision_objects.get_mut(get_rid(body.0))
                && let Some(body) = body.get_mut_body()
            {
                body.update_gravity(step, &mut physics_data.physics_engine);
            }
        }
        let mut settings = settings;
        settings.pixel_liquid_gravity =
            vector_to_rapier(default_gravity_dir) * default_gravity_value;
        settings.pixel_gravity = vector_to_rapier(default_gravity_dir) * default_gravity_value;
        if let Some(space) = physics_data.spaces.get_mut(space_rid) {
            // this calls into rapier
            physics_data.physics_engine.world_step(
                space_handle,
                &settings,
                RapierSpace::collision_filter_body_callback,
                RapierSpace::collision_modify_contacts_callback,
                space,
                &mut physics_data.collision_objects,
            );
            space.after_step(
                &mut physics_data.physics_engine,
                &mut physics_data.collision_objects,
            );
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
            AreaParameter::GRAVITY => self.default_gravity_value = variant_to_float(&value),
            AreaParameter::GRAVITY_VECTOR => {
                self.default_gravity_dir = value.try_to().unwrap_or_default()
            }
            AreaParameter::LINEAR_DAMP => self.default_linear_damping = variant_to_float(&value),
            AreaParameter::ANGULAR_DAMP => self.default_angular_damping = variant_to_float(&value),
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
        self.state.island_count
    }

    pub fn get_active_objects(&self) -> i32 {
        self.state.active_objects
    }

    pub fn get_collision_pairs(&self) -> i32 {
        self.state.collision_pairs
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

    pub fn after_step(
        &mut self,
        physics_engine: &mut PhysicsEngine,
        physics_collision_objects: &mut PhysicsCollisionObjects,
    ) {
        // Needed only for one physics step to retrieve lost info
        self.state.removed_colliders.clear();
        self.state.active_objects =
            physics_engine.world_get_active_objects_count(self.handle) as i32;
        for body in self.state.active_list.clone() {
            if let Some(body) = physics_collision_objects.get_mut(get_rid(body.0)) {
                if let Some(body) = body.get_mut_body() {
                    body.on_update_active(self, physics_engine);
                }
            }
        }
    }

    pub fn get_direct_state(&self) -> &Option<Gd<PhysicsDirectSpaceState>> {
        &self.direct_access
    }

    pub fn get_active_list(&self) -> &HashSet<RigidBodyHandle> {
        &self.state.active_list
    }

    pub fn get_mass_properties_update_list(&self) -> &HashSet<RigidBodyHandle> {
        &self.state.mass_properties_update_list
    }

    pub fn reset_mass_properties_update_list(&mut self) {
        self.state.mass_properties_update_list.clear();
    }

    pub fn get_area_update_list(&self) -> &HashSet<RigidBodyHandle> {
        &self.state.area_update_list
    }

    pub fn get_body_area_update_list(&self) -> &HashSet<RigidBodyHandle> {
        &self.state.body_area_update_list
    }

    pub fn get_gravity_update_list(&self) -> &HashSet<RigidBodyHandle> {
        &self.state.gravity_update_list
    }

    pub fn get_contact_max_allowed_penetration(&self) -> real {
        self.contact_max_allowed_penetration
    }

    pub fn get_active_bodies(&self) -> Vec<RigidBodyHandle> {
        self.state.active_list.clone().into_iter().collect()
    }

    #[cfg(feature = "serde-serialize")]
    pub fn export_space_json(&self, physics_engine: &mut PhysicsEngine) -> String {
        if let Some(inner) = physics_engine.world_export(self.handle) {
            let export = SpaceExport {
                space: &self.state,
                world: inner,
            };
            match serde_json::to_string_pretty(&export) {
                Ok(s) => return s,
                Err(e) => {
                    godot_error!("Failed to serialize space to json: {}", e);
                }
            }
        }
        "{}".to_string()
    }

    #[cfg(feature = "serde-serialize")]
    pub fn export_space_binary(&self, physics_engine: &mut PhysicsEngine) -> PackedByteArray {
        let mut buf = PackedByteArray::new();
        if let Some(inner) = physics_engine.world_export(self.handle) {
            let export = SpaceExport {
                space: &self.state,
                world: inner,
            };
            match bincode::serialize(&export) {
                Ok(binary_data) => {
                    buf.resize(binary_data.len());
                    for i in 0..binary_data.len() {
                        buf[i] = binary_data[i];
                    }
                }
                Err(e) => {
                    godot_error!("Failed to serialize space to binary: {}", e);
                }
            }
        }
        buf
    }

    #[cfg(feature = "serde-serialize")]
    pub fn import_space_json(&mut self, physics_engine: &mut PhysicsEngine, data: String) {
        match serde_json::from_str::<SpaceImport>(&data) {
            Ok(import) => {
                self.state = import.space;
                let physics_objects = import.world;
                let world_settings = WorldSettings {
                    particle_radius: RapierProjectSettings::get_fluid_particle_radius() as real,
                    smoothing_factor: RapierProjectSettings::get_fluid_smoothing_factor() as real,
                    counters_enabled: false,
                };
                physics_engine.world_import(self.get_handle(), &world_settings, physics_objects);
            }
            Err(e) => {
                godot_error!("Failed to serialize space to json: {}", e);
            }
        }
    }

    #[cfg(feature = "serde-serialize")]
    pub fn import_space_binary(
        &mut self,
        physics_engine: &mut PhysicsEngine,
        data: PackedByteArray,
    ) {
        match bincode::deserialize::<SpaceImport>(data.as_slice()) {
            Ok(import) => {
                self.state = import.space;
                let physics_objects = import.world;
                let world_settings = WorldSettings {
                    particle_radius: RapierProjectSettings::get_fluid_particle_radius() as real,
                    smoothing_factor: RapierProjectSettings::get_fluid_smoothing_factor() as real,
                    counters_enabled: false,
                };
                physics_engine.world_import(self.get_handle(), &world_settings, physics_objects);
            }
            Err(e) => {
                godot_error!("Failed to serialize space to json: {}", e);
            }
        }
    }

    pub fn get_ghost_collision_distance(&self) -> real {
        self.ghost_collision_distance
    }

    pub fn reset_space_if_empty(&mut self, physics_engine: &mut PhysicsEngine) {
        if self.is_valid() {
            let world_settings = WorldSettings {
                particle_radius: RapierProjectSettings::get_fluid_particle_radius() as real,
                smoothing_factor: RapierProjectSettings::get_fluid_smoothing_factor() as real,
                counters_enabled: false,
            };
            physics_engine.world_reset_if_empty(self.handle, &world_settings);
        }
    }

    pub fn destroy_space(&mut self, physics_engine: &mut PhysicsEngine) {
        if self.is_valid() {
            physics_engine.world_destroy(self.handle);
            self.handle = WorldHandle::default();
        }
    }
}
impl Drop for RapierSpace {
    fn drop(&mut self) {
        if self.is_valid() {
            godot_error!("RapierSpace leaked");
        }
    }
}
