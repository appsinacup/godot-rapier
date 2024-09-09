#[cfg(feature = "dim2")]
use godot::classes::physics_server_2d::*;
#[cfg(feature = "dim3")]
use godot::classes::physics_server_3d::*;
use godot::prelude::*;

use super::rapier_shape::RapierShape;
use crate::rapier_wrapper::prelude::*;
use crate::servers::rapier_physics_singleton::PhysicsRids;
use crate::servers::rapier_physics_singleton::PhysicsShapes;
use crate::shapes::rapier_shape::IRapierShape;
use crate::shapes::rapier_shape_base::RapierShapeBase;
#[cfg(feature = "dim2")]
use crate::types::PackedFloatArray;
use crate::types::PackedVectorArray;
pub struct RapierConvexPolygonShape {
    points: PackedVectorArray,
    base: RapierShapeBase,
}
impl RapierConvexPolygonShape {
    pub fn create(rid: Rid, physics_shapes: &mut PhysicsShapes) {
        let shape = Self {
            points: PackedVectorArray::new(),
            base: RapierShapeBase::new(rid),
        };
        physics_shapes.insert(rid, RapierShape::RapierConvexPolygonShape(shape));
    }
}
impl IRapierShape for RapierConvexPolygonShape {
    fn get_base(&self) -> &RapierShapeBase {
        &self.base
    }

    fn get_mut_base(&mut self) -> &mut RapierShapeBase {
        &mut self.base
    }

    fn get_type(&self) -> ShapeType {
        ShapeType::CONVEX_POLYGON
    }

    fn allows_one_way_collision(&self) -> bool {
        true
    }

    fn create_rapier_shape(&mut self, physics_engine: &mut PhysicsEngine) -> ShapeHandle {
        if self.points.len() >= 3 {
            let mut rapier_points = Vec::with_capacity(self.points.len());
            for point in self.points.as_slice() {
                rapier_points.push(vector_to_rapier(*point));
            }
            physics_engine.shape_create_convex_polyline(&rapier_points)
        } else {
            godot_error!("ConvexPolygon must have at least three point");
            ShapeHandle::default()
        }
    }

    fn set_data(
        &mut self,
        data: Variant,
        physics_engine: &mut PhysicsEngine,
        physics_rids: &mut PhysicsRids,
    ) {
        match data.get_type() {
            VariantType::PACKED_VECTOR2_ARRAY | VariantType::PACKED_VECTOR3_ARRAY => {
                if let Ok(arr) = data.try_to::<PackedVectorArray>() {
                    let size = arr.len();
                    if size == 0 {
                        return;
                    }
                    self.points = arr;
                }
            }
            #[cfg(feature = "dim2")]
            VariantType::PACKED_FLOAT64_ARRAY | VariantType::PACKED_FLOAT32_ARRAY => {
                if let Ok(arr) = data.try_to::<PackedFloatArray>() {
                    let size = arr.len() / 4;
                    if size == 0 {
                        return;
                    }
                    self.points = PackedVectorArray::new();
                    for i in 0..size {
                        let idx = i << 2;
                        // skip normals
                        self.points
                            .push(Vector2::new(arr[idx] as real, arr[idx + 1] as real));
                    }
                }
            }
            _ => {
                godot_error!("Invalid shape data");
                return;
            }
        }
        if self.points.len() < 3 {
            godot_error!("ConvexPolygon must have at least three point");
            return;
        }
        let handle = self.create_rapier_shape(physics_engine);
        self.base
            .set_handle_and_reset_aabb(handle, physics_engine, physics_rids);
    }

    fn get_data(&self) -> Variant {
        self.points.to_variant()
    }

    fn get_handle(&self) -> ShapeHandle {
        self.base.get_handle()
    }
}
