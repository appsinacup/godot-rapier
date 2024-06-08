use crate::servers2d::rapier_physics_server_2d::*;
use godot::{obj::NewAlloc, prelude::*};

#[derive(GodotClass, Default)]
#[class(base=Object,init,tool)]
pub struct RapierPhysicsServerFactory2D;

#[godot_api]
impl RapierPhysicsServerFactory2D {
    #[func]
    fn create_server() -> Gd<RapierPhysicsServer2D> {
        RapierPhysicsServer2D::new_alloc()
    }
}