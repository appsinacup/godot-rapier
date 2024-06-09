use godot::{engine::PhysicsServer2DManager, prelude::*};

use crate::servers::rapier_project_settings::RapierProjectSettings;

#[cfg(feature = "dim2")]
pub mod rapier_physics_server_2d;
#[cfg(feature = "dim3")]
pub mod rapier_physics_server_3d;
pub mod rapier_physics_server_extra;
#[cfg(feature = "dim2")]
pub mod rapier_physics_server_factory_2d;
#[cfg(feature = "dim3")]
pub mod rapier_physics_server_factory_3d;
pub mod rapier_physics_singleton;
pub mod rapier_project_settings;
#[cfg(feature = "dim2")]
pub type RapierPhysicsServer = rapier_physics_server_2d::RapierPhysicsServer2D;
#[cfg(feature = "dim3")]
pub type RapierPhysicsServer = rapier_physics_server_3d::RapierPhysicsServer3D;

#[cfg(feature = "dim2")]
pub fn register_server() {
    let mut manager = PhysicsServer2DManager::singleton();
    let factory =
        crate::servers::rapier_physics_server_factory_2d::RapierPhysicsServerFactory2D::new_alloc();
    manager.register_server("Rapier2D".into(), factory.callable("create_server"));
}

#[cfg(feature = "dim3")]
pub fn register_server() {
    let mut manager = PhysicsServer2DManager::singleton();
    let factory =
        crate::servers::rapier_physics_server_factory_3d::RapierPhysicsServerFactory3D::new_alloc();
    manager.register_server("Rapier3D".into(), factory.callable("create_server"));
}

pub fn register_scene() {
    RapierProjectSettings::register_settings();
}

pub fn unregister_server() {
    // there doesn't seem to be a function to unregister a server.
}

pub fn unregister_scene() {}
