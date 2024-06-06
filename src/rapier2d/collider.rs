use crate::rapier2d::convert::*;
use crate::rapier2d::handle::*;
use crate::rapier2d::physics_world::*;
use crate::rapier2d::shape::ShapeInfo;
use crate::rapier2d::user_data::*;
use godot::log::godot_error;
use rapier2d::na::Point2;
use rapier2d::na::Vector2;
use rapier2d::prelude::*;
use salva2d::integrations::rapier::ColliderSampling;
use salva2d::object::Boundary;
use salva2d::parry::either::Either::Left;
use salva2d::parry::either::Either::Right;

const SUBDIVISIONS: u32 = 20;

fn skew_polyline(vertices: &Vec<Point2<Real>>, skew: Real) -> SharedShape {
    // Apply skew transformation to the vertices
    let mut skewed_vertices = Vec::new();
    for vertex in vertices {
        let mut skewed_vertex = *vertex;
        skewed_vertex.x -= skewed_vertex.y * skew;
        skewed_vertices.push(skewed_vertex);
    }

    let len = vertices.len();
    let mut indices = vec![];
    for i in 0..len {
        indices.push([i as u32, ((i + 1) % len) as u32]);
    }
    let collider = ColliderBuilder::convex_decomposition(&skewed_vertices, &indices);
    collider.shape
}

// Function to skew a shape
pub fn skew_shape(shape: &SharedShape, skew: Real) -> SharedShape {
    if skew == 0.0 {
        return shape.clone();
    }
    match shape.shape_type() {
        ShapeType::Compound => {
            if let Some(compound) = shape.as_compound() {
                let shapes = compound.shapes();
                let mut transformed_shapes = Vec::new();
                for (position, sub_shape) in shapes.iter() {
                    let skewed_sub_shape = skew_shape(sub_shape, skew);
                    let transformed_position = *position;
                    transformed_shapes.push((transformed_position, skewed_sub_shape));
                }
                return SharedShape::compound(transformed_shapes);
            }
        }
        ShapeType::Ball => {
            if let Some(ball) = shape.as_ball() {
                return skew_polyline(&ball.to_polyline(SUBDIVISIONS), skew);
            }
        }
        ShapeType::Cuboid => {
            if let Some(cuboid) = shape.as_cuboid() {
                return skew_polyline(&cuboid.to_polyline(), skew);
            }
        }
        ShapeType::Polyline => {
            if let Some(polyline) = shape.as_polyline() {
                return skew_polyline(&polyline.vertices().to_vec(), skew);
            }
        }
        ShapeType::ConvexPolygon => {
            if let Some(convex_polygon) = shape.as_convex_polygon() {
                return skew_polyline(&convex_polygon.points().to_vec(), skew);
            }
        }
        ShapeType::Capsule => {
            if let Some(capsule) = shape.as_capsule() {
                return skew_polyline(&capsule.to_polyline(SUBDIVISIONS), skew);
            }
        }
        _ => {
            godot_error!("Shape type not supported for skewing");
        }
    }
    shape.clone()
}

pub fn scale_shape(shape: &SharedShape, scale: Vector2<Real>) -> SharedShape {
    if scale.x == 1.0 && scale.y == 1.0 {
        return shape.clone();
    }
    match shape.shape_type() {
        ShapeType::Ball => {
            if let Some(new_shape) = shape.as_ball() {
                if let Some(new_shape) = new_shape.scaled(&scale, SUBDIVISIONS) {
                    match new_shape {
                        Left(shape) => return SharedShape::new(shape),
                        Right(shape) => return SharedShape::new(shape),
                    }
                }
            }
        }
        ShapeType::Cuboid => {
            if let Some(new_shape) = shape.as_cuboid() {
                return SharedShape::new(new_shape.scaled(&scale));
            }
        }
        ShapeType::HalfSpace => {
            if let Some(new_shape) = shape.as_halfspace() {
                if let Some(new_shape) = new_shape.scaled(&scale) {
                    return SharedShape::new(new_shape);
                }
            }
        }
        ShapeType::Polyline => {
            if let Some(new_shape) = shape.as_polyline() {
                return SharedShape::new(new_shape.clone().scaled(&scale));
            }
        }
        ShapeType::ConvexPolygon => {
            if let Some(new_shape) = shape.as_convex_polygon() {
                if let Some(new_shape) = new_shape.clone().scaled(&scale) {
                    return SharedShape::new(new_shape);
                }
            }
        }
        ShapeType::Capsule => {
            if let Some(new_shape) = shape.as_capsule() {
                if let Some(new_shape) = new_shape.scaled(&scale, SUBDIVISIONS) {
                    match new_shape {
                        Left(shape) => return SharedShape::new(shape),
                        Right(shape) => return SharedShape::new(shape),
                    }
                }
            }
        }
        ShapeType::Compound => {
            if let Some(new_shape) = shape.as_compound() {
                let new_shapes = new_shape.shapes();
                let mut shapes_vec = Vec::<(Isometry<Real>, SharedShape)>::new();
                for shape in new_shapes {
                    let new_shape = scale_shape(&shape.1, scale);
                    shapes_vec.push((shape.0, new_shape));
                }
                return SharedShape::compound(shapes_vec);
            }
        }
        _ => {
            godot_error!("Shape type not supported for scaling");
        }
    }
    shape.clone()
}

pub struct Material {
    pub friction: Real,
    pub restitution: Real,
    pub contact_skin: Real,
}

pub fn default_material() -> Material {
    Material {
        friction: 1.0,
        restitution: 0.0,
        contact_skin: 0.0,
    }
}

fn shape_is_halfspace(shape: &SharedShape) -> bool {
    if shape.shape_type() == ShapeType::Compound {
        if let Some(shape) = shape.as_compound() {
            for shape in shape.shapes() {
                if shape_is_halfspace(&shape.1) {
                    return true;
                }
            }
        }
    }
    shape.shape_type() == ShapeType::HalfSpace
}

pub fn collider_create_solid(
    world_handle: Handle,
    shape_handle: Handle,
    mat: &Material,
    body_handle: Handle,
    user_data: &UserData,
) -> Handle {
    let physics_engine = physics_engine();
    if let Some(shape) = physics_engine.get_shape(shape_handle) {
        let is_shape_halfspace = shape_is_halfspace(shape);
        let mut collider = ColliderBuilder::new(shape.clone())
            .contact_force_event_threshold(-Real::MAX)
            .build();
        // TODO update when https://github.com/dimforge/rapier/issues/622 is fixed
        if mat.friction >= 0.0 {
            collider.set_friction(mat.friction);
        }
        if mat.restitution >= 0.0 {
            collider.set_restitution(mat.restitution);
        }
        collider.set_friction_combine_rule(CoefficientCombineRule::Multiply);
        collider.set_restitution_combine_rule(CoefficientCombineRule::Max);
        collider.set_density(0.0);
        collider.set_contact_skin(pixels_to_meters(mat.contact_skin));
        collider.set_contact_force_event_threshold(-Real::MAX);
        collider.user_data = user_data.get_data();
        collider.set_active_hooks(
            ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::MODIFY_SOLVER_CONTACTS,
        );
        if let Some(physics_world) = physics_engine.get_world(world_handle) {
            let handle = physics_world.insert_collider(collider, body_handle);
            // register fluid coupling. Dynamic coupling doens't work for halfspace
            let collider_handle = handle_to_collider_handle(handle);
            if !is_shape_halfspace {
                let boundary_handle = physics_world
                    .fluids_pipeline
                    .liquid_world
                    .add_boundary(Boundary::new(Vec::new()));
                physics_world.fluids_pipeline.coupling.register_coupling(
                    boundary_handle,
                    collider_handle,
                    ColliderSampling::DynamicContactSampling,
                );
            }
            return handle;
        }
    }
    invalid_handle()
}

pub fn collider_create_sensor(
    world_handle: Handle,
    shape_handle: Handle,
    body_handle: Handle,
    user_data: &UserData,
) -> Handle {
    let physics_engine = physics_engine();
    if let Some(shape) = physics_engine.get_shape(shape_handle) {
        let mut collider = ColliderBuilder::new(shape.clone()).build();
        collider.set_sensor(true);
        collider.set_active_events(ActiveEvents::COLLISION_EVENTS);
        let mut collision_types = collider.active_collision_types();
        // Area vs Area
        collision_types |= ActiveCollisionTypes::FIXED_FIXED;
        // Area vs CharacterBody
        collision_types |= ActiveCollisionTypes::KINEMATIC_FIXED;
        collider.set_active_collision_types(collision_types);
        collider.user_data = user_data.get_data();
        collider.set_active_hooks(ActiveHooks::FILTER_INTERSECTION_PAIR);
        if let Some(physics_world) = physics_engine.get_world(world_handle) {
            return physics_world.insert_collider(collider, body_handle);
        }
    }
    invalid_handle()
}

pub fn collider_destroy(world_handle: Handle, handle: Handle) {
    let physics_engine = physics_engine();
    if let Some(physics_world) = physics_engine.get_world(world_handle) {
        let collider_handle = handle_to_collider_handle(handle);
        physics_world
            .fluids_pipeline
            .coupling
            .unregister_coupling(collider_handle);
        physics_world.remove_collider(handle);
    }
}

pub fn collider_get_position(world_handle: Handle, handle: Handle) -> Vector2<Real> {
    let physics_engine = physics_engine();
    if let Some(physics_world) = physics_engine.get_world(world_handle) {
        let collider_handle = handle_to_collider_handle(handle);
        if let Some(collider) = physics_world
            .physics_objects
            .collider_set
            .get(collider_handle)
        {
            let collider_vector = collider.translation();
            return vector_meters_to_pixels(*collider_vector);
        }
    }
    Vector::default()
}

pub fn collider_get_angle(world_handle: Handle, handle: Handle) -> Real {
    let physics_engine = physics_engine();
    if let Some(physics_world) = physics_engine.get_world(world_handle) {
        let collider_handle = handle_to_collider_handle(handle);
        if let Some(collider) = physics_world
            .physics_objects
            .collider_set
            .get(collider_handle)
        {
            return collider.rotation().angle();
        }
    }
    0.0
}

pub fn collider_set_transform(world_handle: Handle, handle: Handle, shape_info: ShapeInfo) {
    let position = vector_pixels_to_meters(shape_info.pixel_position);
    let physics_engine = physics_engine();

    if let Some(shape) = physics_engine.get_shape(shape_info.handle) {
        let scaled_shape = scale_shape(shape, shape_info.scale);
        let new_shape = skew_shape(&scaled_shape, shape_info.skew);
        if let Some(physics_world) = physics_engine.get_world(world_handle) {
            let collider_handle = handle_to_collider_handle(handle);
            if let Some(collider) = physics_world
                .physics_objects
                .collider_set
                .get_mut(collider_handle)
            {
                collider.set_position_wrt_parent(Isometry::new(position, shape_info.rotation));
                collider.set_shape(new_shape);
            }
        }
    }
}

pub fn collider_set_collision_events_enabled(world_handle: Handle, handle: Handle, enable: bool) {
    let physics_engine = physics_engine();
    if let Some(physics_world) = physics_engine.get_world(world_handle) {
        let collider_handle = handle_to_collider_handle(handle);
        if let Some(collider) = physics_world
            .physics_objects
            .collider_set
            .get_mut(collider_handle)
        {
            let mut active_events = collider.active_events();
            if enable {
                active_events |= ActiveEvents::COLLISION_EVENTS;
            } else {
                active_events &= !ActiveEvents::COLLISION_EVENTS;
            }
            collider.set_active_events(active_events);
        }
    }
}

pub fn collider_set_contact_force_events_enabled(
    world_handle: Handle,
    handle: Handle,
    enable: bool,
) {
    let physics_engine = physics_engine();
    if let Some(physics_world) = physics_engine.get_world(world_handle) {
        let collider_handle = handle_to_collider_handle(handle);
        if let Some(collider) = physics_world
            .physics_objects
            .collider_set
            .get_mut(collider_handle)
        {
            let mut active_events = collider.active_events();
            if enable {
                active_events |= ActiveEvents::CONTACT_FORCE_EVENTS;
            } else {
                active_events &= !ActiveEvents::CONTACT_FORCE_EVENTS;
            }
            collider.set_active_events(active_events);
        }
    }
}
