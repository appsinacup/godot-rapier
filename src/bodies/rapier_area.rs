#[cfg(feature = "dim2")]
use godot::engine::physics_server_2d::*;
#[cfg(feature = "dim3")]
use godot::engine::physics_server_3d::*;
use godot::log::godot_error;
use godot::meta::ToGodot;
use godot::obj::EngineEnum;
use godot::prelude::*;
use hashbrown::HashMap;
use rapier::geometry::ColliderHandle;
use servers::rapier_physics_server_extra::PhysicsCollisionObjects;
use servers::rapier_physics_server_extra::PhysicsShapes;
use servers::rapier_physics_server_extra::PhysicsSpaces;

use super::rapier_body::RapierBody;
use crate::bodies::rapier_collision_object::*;
use crate::rapier_wrapper::prelude::*;
use crate::spaces::rapier_space::*;
use crate::types::*;
use crate::*;
// #[derive(Serialize, Deserialize, Debug)]
struct MonitorInfo {
    pub rid: Rid,
    pub instance_id: u64,
    pub object_shape_index: u32,
    pub area_shape_index: u32,
    pub collision_object_type: CollisionObjectType,
    pub state: i32,
}
pub enum AreaUpdateMode {
    EnableSpaceOverride,
    DisableSpaceOverride,
    ResetSpaceOverride,
    None,
}
//#[derive(Serialize, Deserialize, Debug)]
pub struct RapierArea {
    gravity_override_mode: AreaSpaceOverrideMode,
    linear_damping_override_mode: AreaSpaceOverrideMode,
    angular_damping_override_mode: AreaSpaceOverrideMode,
    gravity: real,
    gravity_vector: Vector,
    gravity_is_point: bool,
    gravity_point_unit_distance: real,
    linear_damp: real,
    angular_damp: real,
    priority: i32,
    monitorable: bool,
    monitor_callback: Callable,
    area_monitor_callback: Callable,
    monitored_objects: HashMap<(ColliderHandle, ColliderHandle), MonitorInfo>,
    detected_bodies: HashMap<Rid, u32>,
    base: RapierCollisionObject,
}
impl RapierArea {
    pub fn new(rid: Rid) -> Self {
        //let default_linear_damping: real = project_settings.get_setting_with_override("physics/2d/default_linear_damp".into()).to();
        //let default_angular_damping: real = project_settings.get_setting_with_override("physics/2d/default_angular_damp".into()).to();
        Self {
            gravity_override_mode: AreaSpaceOverrideMode::DISABLED,
            linear_damping_override_mode: AreaSpaceOverrideMode::DISABLED,
            angular_damping_override_mode: AreaSpaceOverrideMode::DISABLED,
            gravity: 0.0,
            gravity_vector: Vector::default(),
            gravity_is_point: false,
            gravity_point_unit_distance: 0.0,
            linear_damp: 0.0,
            angular_damp: 0.0,
            priority: 0,
            monitorable: false,
            monitor_callback: Callable::invalid(),
            area_monitor_callback: Callable::invalid(),
            monitored_objects: HashMap::default(),
            detected_bodies: HashMap::default(),
            base: RapierCollisionObject::new(rid, CollisionObjectType::Area),
        }
    }

    pub fn enable_space_override(
        area_rid: &Rid,
        physics_spaces: &mut PhysicsSpaces,
        physics_collision_objects: &mut PhysicsCollisionObjects,
    ) {
        let mut detected_bodies = HashMap::default();
        let mut space_rid = Rid::Invalid;
        if let Some(area_rid) = physics_collision_objects.get(area_rid) {
            if let Some(area) = area_rid.get_area() {
                detected_bodies = area.detected_bodies.clone();
                space_rid = area.get_base().get_space();
            }
        }
        if let Some(space) = physics_spaces.get_mut(&space_rid) {
            for (key, _) in detected_bodies.iter() {
                if let Some([body, area]) = physics_collision_objects.get_many_mut([key, area_rid])
                    && let Some(body) = body.get_mut_body()
                    && let Some(area) = area.get_mut_area()
                {
                    body.add_area(area, space);
                }
            }
            // No need to update anymore if it was scheduled before
            space.area_remove_from_area_update_list(*area_rid);
        }
    }

    pub fn disable_space_override(
        area_rid: &Rid,
        physics_spaces: &mut PhysicsSpaces,
        physics_collision_objects: &mut PhysicsCollisionObjects,
    ) {
        let mut detected_bodies = HashMap::default();
        let mut space_rid = Rid::Invalid;
        if let Some(area_rid) = physics_collision_objects.get(area_rid) {
            if let Some(area) = area_rid.get_area() {
                detected_bodies = area.detected_bodies.clone();
                space_rid = area.get_base().get_space();
            }
        }
        if let Some(space) = physics_spaces.get_mut(&space_rid) {
            for (key, _) in detected_bodies.iter() {
                if let Some(body) = physics_collision_objects.get_mut(key)
                    && let Some(body) = body.get_mut_body()
                {
                    body.remove_area(*area_rid, space);
                }
            }
            // No need to update anymore if it was scheduled before
            space.area_remove_from_area_update_list(*area_rid);
        }
    }

    pub fn reset_space_override(
        area_rid: &Rid,
        physics_spaces: &mut PhysicsSpaces,
        physics_collision_objects: &mut PhysicsCollisionObjects,
    ) {
        let mut detected_bodies = HashMap::default();
        let mut space_rid: Rid = Rid::Invalid;
        if let Some(area_rid) = physics_collision_objects.get(area_rid) {
            if let Some(area) = area_rid.get_area() {
                detected_bodies = area.detected_bodies.clone();
                space_rid = area.get_base().get_space();
            }
        }
        if let Some(space) = physics_spaces.get_mut(&space_rid) {
            for (key, _) in detected_bodies {
                if let Some([body, area]) = physics_collision_objects.get_many_mut([&key, area_rid])
                    && let Some(body) = body.get_mut_body()
                    && let Some(area) = area.get_mut_area()
                {
                    body.remove_area(*area_rid, space);
                    body.add_area(area, space);
                }
            }
            space.area_remove_from_area_update_list(*area_rid);
        }
    }

    pub fn on_body_enter(
        &mut self,
        collider_handle: ColliderHandle,
        body: &mut Option<&mut Box<dyn IRapierCollisionObject>>,
        body_shape: usize,
        body_rid: Rid,
        body_instance_id: u64,
        area_collider_handle: ColliderHandle,
        area_shape: usize,
        space: &mut RapierSpace,
    ) {
        if let Some(body) = body {
            if let Some(body) = body.get_mut_body() {
                // Add to keep track of currently detected bodies
                if let Some(detected_body) = self.detected_bodies.get_mut(&body_rid) {
                    *detected_body += 1;
                } else {
                    self.detected_bodies.insert(body_rid, 1);
                    body.add_area(self, space);
                }
                if self.monitor_callback.is_null() {
                    return;
                }
                self.base.area_detection_counter += 1;
                let handle_pair_hash = (collider_handle, area_collider_handle);
                if self.monitored_objects.contains_key(&handle_pair_hash) {
                    godot_error!("Body is already being monitored");
                    return;
                }
                self.monitored_objects.insert(
                    handle_pair_hash,
                    MonitorInfo {
                        rid: body_rid,
                        instance_id: body_instance_id,
                        object_shape_index: body_shape as u32,
                        area_shape_index: area_shape as u32,
                        collision_object_type: CollisionObjectType::Body,
                        state: 1,
                    },
                );
                space.area_add_to_monitor_query_list(self.base.get_rid());
            }
        } else {
            godot_error!("other body is null");
        }
    }

    pub fn on_body_exit(
        &mut self,
        collider_handle: ColliderHandle,
        body: &mut Option<&mut Box<dyn IRapierCollisionObject>>,
        body_shape: usize,
        body_rid: Rid,
        body_instance_id: u64,
        area_collider_handle: ColliderHandle,
        area_shape: usize,
        space: &mut RapierSpace,
    ) {
        if body.is_some() {
            // Remove from currently detected bodies
            if let Some(detected_body) = self.detected_bodies.get_mut(&body_rid) {
                *detected_body -= 1;
                if *detected_body == 0 {
                    self.detected_bodies.remove(&body_rid);
                    if let Some(body) = body {
                        if let Some(body) = body.get_mut_body() {
                            body.remove_area(self.base.get_rid(), space);
                        }
                    }
                }
            }
        }
        if self.monitor_callback.is_null() {
            return;
        }
        if body.is_some() {
            self.base.area_detection_counter -= 1;
        }
        let handle_pair_hash = (collider_handle, area_collider_handle);
        if let hashbrown::hash_map::Entry::Vacant(e) =
            self.monitored_objects.entry(handle_pair_hash)
        {
            e.insert(MonitorInfo {
                rid: body_rid,
                instance_id: body_instance_id,
                object_shape_index: body_shape as u32,
                area_shape_index: area_shape as u32,
                collision_object_type: CollisionObjectType::Body,
                state: -1,
            });
            space.area_add_to_monitor_query_list(self.base.get_rid());
        } else {
            if self.monitored_objects[&handle_pair_hash].state != 1 {
                godot_error!("Body is not being monitored");
                return;
            }
            self.monitored_objects.remove(&handle_pair_hash);
        }
    }

    pub fn on_area_enter(
        &mut self,
        collider_handle: ColliderHandle,
        other_area: &mut Option<&mut Box<dyn IRapierCollisionObject>>,
        other_area_shape: usize,
        other_area_rid: Rid,
        other_area_instance_id: u64,
        area_collider_handle: ColliderHandle,
        area_shape: usize,
        space: &mut RapierSpace,
    ) {
        if self.area_monitor_callback.is_null() {
            return;
        }
        if let Some(other_area) = other_area {
            if let Some(other_area) = other_area.get_mut_area() {
                if !other_area.is_monitorable() {
                    return;
                }
                other_area.base.area_detection_counter += 1;
                let handle_pair_hash = (collider_handle, area_collider_handle);
                if self.monitored_objects.contains_key(&handle_pair_hash) {
                    godot_error!("Area is already being monitored");
                    return;
                }
                self.monitored_objects.insert(
                    handle_pair_hash,
                    MonitorInfo {
                        rid: other_area_rid,
                        instance_id: other_area_instance_id,
                        object_shape_index: other_area_shape as u32,
                        area_shape_index: area_shape as u32,
                        collision_object_type: CollisionObjectType::Area,
                        state: 1,
                    },
                );
                space.area_add_to_monitor_query_list(self.base.get_rid());
            }
        } else {
            godot_error!("other area is null");
        }
    }

    pub fn on_area_exit(
        &mut self,
        collider_handle: ColliderHandle,
        other_area: &mut Option<&mut Box<dyn IRapierCollisionObject>>,
        other_area_shape: usize,
        other_area_rid: Rid,
        other_area_instance_id: u64,
        area_collider_handle: ColliderHandle,
        area_shape: usize,
        space: &mut RapierSpace,
    ) {
        if self.area_monitor_callback.is_null() {
            return;
        }
        if let Some(other_area) = other_area {
            if let Some(other_area) = other_area.get_mut_area() {
                if !other_area.is_monitorable() {
                    return;
                }
                if other_area.base.area_detection_counter == 0 {
                    godot_error!("Area is not being monitored");
                    return;
                }
                other_area.base.area_detection_counter -= 1;
            }
        }
        let handle_pair_hash = (collider_handle, area_collider_handle);
        if let hashbrown::hash_map::Entry::Occupied(mut e) =
            self.monitored_objects.entry(handle_pair_hash)
        {
            e.insert(MonitorInfo {
                rid: other_area_rid,
                instance_id: other_area_instance_id,
                object_shape_index: other_area_shape as u32,
                area_shape_index: area_shape as u32,
                collision_object_type: CollisionObjectType::Area,
                state: -1,
            });
            space.area_add_to_monitor_query_list(self.base.get_rid());
        } else {
            if self.monitored_objects[&handle_pair_hash].state != 1 {
                godot_error!("Area is not being monitored");
                return;
            }
            self.monitored_objects.remove(&handle_pair_hash);
        }
    }

    pub fn update_area_override(
        physics_collision_objects: &mut PhysicsCollisionObjects,
        physics_spaces: &mut PhysicsSpaces,
        physics_engine: &mut PhysicsEngine,
        area_rid: &Rid,
    ) {
        let mut detected_bodies = HashMap::default();
        let mut space_rid = Rid::Invalid;
        if let Some(area_rid) = physics_collision_objects.get(area_rid) {
            if let Some(area) = area_rid.get_area() {
                detected_bodies = area.detected_bodies.clone();
                space_rid = area.get_base().get_space();
            }
        }
        if let Some(space) = physics_spaces.get_mut(&space_rid) {
            space.area_remove_from_area_update_list(*area_rid);
        }
        for (detected_body, _) in detected_bodies {
            let mut area_override_settings = None;
            if let Some(body) = physics_collision_objects.get(&detected_body) {
                if let Some(body) = body.get_body() {
                    area_override_settings = Some(
                        body.get_area_override_settings(physics_spaces, physics_collision_objects),
                    );
                }
            }
            if let Some(area_override_settings) = area_override_settings {
                if let Some(body) = physics_collision_objects.get_mut(&detected_body) {
                    if let Some(body) = body.get_mut_body() {
                        body.apply_area_override(
                            area_override_settings,
                            physics_engine,
                            physics_spaces,
                        );
                    }
                }
            }
        }
    }

    pub fn has_any_space_override(&self) -> bool {
        self.gravity_override_mode != AreaSpaceOverrideMode::DISABLED
            || self.linear_damping_override_mode != AreaSpaceOverrideMode::DISABLED
            || self.angular_damping_override_mode != AreaSpaceOverrideMode::DISABLED
    }

    pub fn set_monitor_callback(&mut self, callback: Callable) {
        self.monitor_callback = callback;
    }

    pub fn set_area_monitor_callback(&mut self, callback: Callable) {
        self.area_monitor_callback = callback;
    }

    pub fn set_param(
        &mut self,
        p_param: AreaParameter,
        p_value: Variant,
        physics_spaces: &mut PhysicsSpaces,
    ) -> AreaUpdateMode {
        match p_param {
            AreaParameter::GRAVITY_OVERRIDE_MODE => {
                let had_override = self.has_any_space_override();
                self.gravity_override_mode = AreaSpaceOverrideMode::from_ord(p_value.to());
                let has_override = self.has_any_space_override();
                if has_override != had_override {
                    if has_override {
                        return AreaUpdateMode::EnableSpaceOverride;
                    } else {
                        return AreaUpdateMode::DisableSpaceOverride;
                    }
                }
            }

            AreaParameter::GRAVITY => {
                let new_gravity = p_value.to();
                if new_gravity != self.gravity {
                    self.gravity = new_gravity;
                    if self.gravity_override_mode != AreaSpaceOverrideMode::DISABLED {
                        // Update currently detected bodies
                        if let Some(space) = physics_spaces.get_mut(&self.base.get_space()) {
                            space.area_add_to_area_update_list(self.base.get_rid());
                        }
                    }
                }
            }
            AreaParameter::GRAVITY_VECTOR => {
                let new_gravity_vector = p_value.to();
                if self.gravity_vector != new_gravity_vector {
                    self.gravity_vector = new_gravity_vector;
                    if self.gravity_override_mode != AreaSpaceOverrideMode::DISABLED {
                        // Update currently detected bodies
                        if let Some(space) = physics_spaces.get_mut(&self.base.get_space()) {
                            space.area_add_to_area_update_list(self.base.get_rid());
                        }
                    }
                }
            }
            AreaParameter::GRAVITY_IS_POINT => {
                let new_gravity_is_point = p_value.to();
                if self.gravity_is_point != new_gravity_is_point {
                    self.gravity_is_point = new_gravity_is_point;
                    if self.gravity_override_mode != AreaSpaceOverrideMode::DISABLED {
                        // Update currently detected bodies
                        if let Some(space) = physics_spaces.get_mut(&self.base.get_space()) {
                            space.area_add_to_area_update_list(self.base.get_rid());
                        }
                    }
                }
            }
            AreaParameter::GRAVITY_POINT_UNIT_DISTANCE => {
                let new_gravity_point_unit_distance = p_value.to();
                if self.gravity_point_unit_distance != new_gravity_point_unit_distance {
                    self.gravity_point_unit_distance = new_gravity_point_unit_distance;
                    if self.gravity_override_mode != AreaSpaceOverrideMode::DISABLED {
                        // Update currently detected bodies
                        if let Some(space) = physics_spaces.get_mut(&self.base.get_space()) {
                            space.area_add_to_area_update_list(self.base.get_rid());
                        }
                    }
                }
            }
            AreaParameter::LINEAR_DAMP_OVERRIDE_MODE => {
                let had_override = self.has_any_space_override();
                self.linear_damping_override_mode = AreaSpaceOverrideMode::from_ord(p_value.to());
                let has_override = self.has_any_space_override();
                if has_override != had_override {
                    if has_override {
                        return AreaUpdateMode::EnableSpaceOverride;
                    } else {
                        return AreaUpdateMode::DisableSpaceOverride;
                    }
                }
            }
            AreaParameter::LINEAR_DAMP => {
                let new_linear_damp = p_value.to();
                if self.linear_damp != new_linear_damp {
                    self.linear_damp = new_linear_damp;
                    if self.linear_damping_override_mode != AreaSpaceOverrideMode::DISABLED {
                        // Update currently detected bodies
                        if let Some(space) = physics_spaces.get_mut(&self.base.get_space()) {
                            space.area_add_to_area_update_list(self.base.get_rid());
                        }
                    }
                }
            }
            AreaParameter::ANGULAR_DAMP_OVERRIDE_MODE => {
                let had_override = self.has_any_space_override();
                self.angular_damping_override_mode = AreaSpaceOverrideMode::from_ord(p_value.to());
                let has_override = self.has_any_space_override();
                if has_override != had_override {
                    if has_override {
                        return AreaUpdateMode::EnableSpaceOverride;
                    } else {
                        return AreaUpdateMode::DisableSpaceOverride;
                    }
                }
            }
            AreaParameter::ANGULAR_DAMP => {
                let new_angular_damp = p_value.to();
                if self.angular_damp != new_angular_damp {
                    self.angular_damp = new_angular_damp;
                    if self.angular_damping_override_mode != AreaSpaceOverrideMode::DISABLED {
                        // Update currently detected bodies
                        if let Some(space) = physics_spaces.get_mut(&self.base.get_space()) {
                            space.area_add_to_area_update_list(self.base.get_rid());
                        }
                    }
                }
            }
            AreaParameter::PRIORITY => {
                let new_priority = p_value.to();
                if self.priority != new_priority {
                    self.priority = new_priority;
                    if self.has_any_space_override() {
                        return AreaUpdateMode::ResetSpaceOverride;
                    }
                }
            }
            _ => {}
        }
        AreaUpdateMode::None
    }

    pub fn get_param(&self, p_param: AreaParameter) -> Variant {
        match p_param {
            AreaParameter::GRAVITY_OVERRIDE_MODE => self.gravity_override_mode.to_variant(),
            AreaParameter::GRAVITY => self.gravity.to_variant(),
            AreaParameter::GRAVITY_VECTOR => self.gravity_vector.to_variant(),
            AreaParameter::GRAVITY_IS_POINT => self.gravity_is_point.to_variant(),
            AreaParameter::GRAVITY_POINT_UNIT_DISTANCE => {
                self.gravity_point_unit_distance.to_variant()
            }
            AreaParameter::LINEAR_DAMP_OVERRIDE_MODE => {
                self.linear_damping_override_mode.to_variant()
            }
            AreaParameter::LINEAR_DAMP => self.linear_damp.to_variant(),
            AreaParameter::ANGULAR_DAMP_OVERRIDE_MODE => {
                self.angular_damping_override_mode.to_variant()
            }
            AreaParameter::ANGULAR_DAMP => self.angular_damp.to_variant(),
            AreaParameter::PRIORITY => self.priority.to_variant(),
            _ => Variant::nil(),
        }
    }

    pub fn get_gravity_point_unit_distance(&self) -> real {
        self.gravity_point_unit_distance
    }

    pub fn get_linear_damp(&self) -> real {
        self.linear_damp
    }

    pub fn get_angular_damp(&self) -> real {
        self.angular_damp
    }

    pub fn set_monitorable(&mut self, monitorable: bool) {
        self.monitorable = monitorable;
    }

    pub fn is_monitorable(&self) -> bool {
        self.monitorable
    }

    pub fn get_queries(&mut self) -> Vec<Callable> {
        let mut queries = Vec::default();
        if self.monitored_objects.is_empty() {
            return queries;
        }
        for (_, monitor_info) in &self.monitored_objects {
            let mut arg_array = VariantArray::new();
            arg_array.resize(5, &Variant::nil());
            if monitor_info.state == 0 {
                godot_error!("Is this an error?");
                continue;
            }
            if monitor_info.state > 0 {
                arg_array.set(0, AreaBodyStatus::ADDED.to_variant());
            } else {
                arg_array.set(0, AreaBodyStatus::REMOVED.to_variant());
            }
            arg_array.set(1, monitor_info.rid.to_variant());
            arg_array.set(2, monitor_info.instance_id.to_variant());
            arg_array.set(3, monitor_info.object_shape_index.to_variant());
            arg_array.set(4, monitor_info.area_shape_index.to_variant());
            if monitor_info.collision_object_type == CollisionObjectType::Body {
                if self.monitor_callback.is_valid() {
                    queries.push(self.monitor_callback.bindv(arg_array));
                }
            } else if self.area_monitor_callback.is_valid() {
                queries.push(self.area_monitor_callback.bindv(arg_array));
            }
        }
        self.monitored_objects.clear();
        queries
    }

    pub fn compute_gravity(&self, position: Vector) -> Vector {
        if self.gravity_is_point {
            let gr_unit_dist = self.get_gravity_point_unit_distance();
            let v = self.get_base().get_transform() * self.gravity_vector - position;
            if gr_unit_dist > 0.0 {
                let v_length_sq = v.length_squared();
                if v_length_sq > 0.0 {
                    let gravity_strength = self.gravity * gr_unit_dist * gr_unit_dist / v_length_sq;
                    vector_normalized(v) * gravity_strength
                } else {
                    Vector::default()
                }
            } else {
                vector_normalized(v) * self.gravity
            }
        } else {
            self.gravity_vector * self.gravity
        }
    }

    pub fn clear_detected_bodies(
        area_rid: &Rid,
        physics_spaces: &mut PhysicsSpaces,
        physics_collision_objects: &mut PhysicsCollisionObjects,
    ) {
        let mut previous_space_rid = Rid::Invalid;
        let mut detected_bodies = HashMap::default();
        if let Some(area) = physics_collision_objects.get_mut(area_rid)
            && let Some(area) = area.get_mut_area()
        {
            previous_space_rid = area.get_base().get_space();
            detected_bodies = area.detected_bodies.clone();
            area.detected_bodies.clear();
            area.monitored_objects.clear();
        }
        if let Some(space) = physics_spaces.get_mut(&previous_space_rid) {
            if let Some(area) = physics_collision_objects.get_mut(area_rid)
                && let Some(area) = area.get_mut_area()
            {
                if !area.detected_bodies.is_empty() {
                    space.area_add_to_monitor_query_list(*area_rid);
                }
                space.area_remove_from_area_update_list(*area_rid);
            }
            for (detected_body, _) in detected_bodies {
                if let Some(body) = physics_collision_objects.get_mut(&detected_body) {
                    if let Some(body) = body.get_mut_body() {
                        body.remove_area(*area_rid, space);
                    }
                }
            }
        }
    }
}
// We won't use the pointers between threads, so it should be safe.
unsafe impl Sync for RapierArea {}
impl IRapierCollisionObject for RapierArea {
    fn get_base(&self) -> &RapierCollisionObject {
        &self.base
    }

    fn get_mut_base(&mut self) -> &mut RapierCollisionObject {
        &mut self.base
    }

    fn get_body(&self) -> Option<&RapierBody> {
        None
    }

    fn get_area(&self) -> Option<&RapierArea> {
        Some(self)
    }

    fn get_mut_body(&mut self) -> Option<&mut RapierBody> {
        None
    }

    fn get_mut_area(&mut self) -> Option<&mut RapierArea> {
        Some(self)
    }

    fn set_space(
        &mut self,
        p_space: Rid,
        physics_engine: &mut PhysicsEngine,
        physics_spaces: &mut PhysicsSpaces,
        physics_shapes: &mut PhysicsShapes,
    ) {
        if p_space == self.base.get_space() {
            return;
        }
        self.base.set_space(p_space, physics_engine, physics_spaces);
        self.recreate_shapes(physics_engine, physics_shapes);
    }

    fn add_shape(
        &mut self,
        p_shape: godot::prelude::Rid,
        p_transform: Transform,
        p_disabled: bool,
        physics_engine: &mut PhysicsEngine,
        physics_spaces: &mut PhysicsSpaces,
        physics_shapes: &mut PhysicsShapes,
    ) {
        RapierCollisionObject::add_shape(
            self,
            p_shape,
            p_transform,
            p_disabled,
            physics_engine,
            physics_spaces,
            physics_shapes,
        );
    }

    fn set_shape(
        &mut self,
        p_index: usize,
        p_shape: Rid,
        physics_engine: &mut PhysicsEngine,
        physics_spaces: &mut PhysicsSpaces,
        physics_shapes: &mut PhysicsShapes,
    ) {
        RapierCollisionObject::set_shape(
            self,
            p_index,
            p_shape,
            physics_engine,
            physics_spaces,
            physics_shapes,
        );
    }

    fn set_shape_transform(
        &mut self,
        p_index: usize,
        p_transform: Transform,
        physics_engine: &mut PhysicsEngine,
        physics_spaces: &mut PhysicsSpaces,
        physics_shapes: &mut PhysicsShapes,
    ) {
        RapierCollisionObject::set_shape_transform(
            self,
            p_index,
            p_transform,
            physics_engine,
            physics_spaces,
            physics_shapes,
        );
    }

    fn set_shape_disabled(
        &mut self,
        p_index: usize,
        p_disabled: bool,
        physics_engine: &mut PhysicsEngine,
        physics_spaces: &mut PhysicsSpaces,
        physics_shapes: &mut PhysicsShapes,
    ) {
        RapierCollisionObject::set_shape_disabled(
            self,
            p_index,
            p_disabled,
            physics_engine,
            physics_spaces,
            physics_shapes,
        );
    }

    fn remove_shape_rid(
        &mut self,
        shape: Rid,
        physics_engine: &mut PhysicsEngine,
        physics_spaces: &mut PhysicsSpaces,
        physics_shapes: &mut PhysicsShapes,
    ) {
        // remove a shape, all the times it appears
        let mut i = 0;
        while i < self.base.shapes.len() {
            if self.base.shapes[i].shape == shape {
                self.remove_shape_idx(i, physics_engine, physics_spaces, physics_shapes);
            } else {
                i += 1;
            }
        }
    }

    fn remove_shape_idx(
        &mut self,
        p_index: usize,
        physics_engine: &mut PhysicsEngine,
        physics_spaces: &mut PhysicsSpaces,
        physics_shapes: &mut PhysicsShapes,
    ) {
        RapierCollisionObject::remove_shape_idx(
            self,
            p_index,
            physics_engine,
            physics_spaces,
            physics_shapes,
        );
    }

    fn create_shape(
        &mut self,
        shape: CollisionObjectShape,
        p_shape_index: usize,
        physics_engine: &mut PhysicsEngine,
        physics_shapes: &mut PhysicsShapes,
    ) -> ColliderHandle {
        if !self.base.is_space_valid() {
            return ColliderHandle::invalid();
        }
        let mat = self.init_material();
        self.base
            .create_shape(shape, p_shape_index, mat, physics_engine, physics_shapes)
    }

    fn init_material(&self) -> Material {
        default_material()
    }

    fn recreate_shapes(
        &mut self,
        physics_engine: &mut PhysicsEngine,
        physics_shapes: &mut PhysicsShapes,
    ) {
        RapierCollisionObject::recreate_shapes(self, physics_engine, physics_shapes);
    }

    fn shape_changed(
        &mut self,
        p_shape: Rid,
        physics_engine: &mut PhysicsEngine,
        physics_shapes: &mut PhysicsShapes,
        physics_spaces: &mut PhysicsSpaces,
    ) {
        RapierCollisionObject::shape_changed(
            self,
            p_shape,
            physics_engine,
            physics_shapes,
            physics_spaces,
        );
    }

    fn shapes_changed(
        &mut self,
        _physics_engine: &mut PhysicsEngine,
        _physics_spaces: &mut PhysicsSpaces,
    ) {
    }
}
