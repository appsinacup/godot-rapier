use godot::classes::*;
#[cfg(feature = "dim2")]
use physics_server_2d::JointType;
#[cfg(feature = "dim3")]
use physics_server_3d::JointType;
use rapier::dynamics::ImpulseJointHandle;
use serde::Deserialize;
use serde::Serialize;

#[cfg(feature = "dim2")]
use super::rapier_damped_spring_joint_2d::RapierDampedSpringJoint2D;
#[cfg(feature = "dim2")]
use super::rapier_pin_joint_2d::RapierPinJoint2D;
use crate::rapier_wrapper::prelude::*;
use crate::*;
#[typetag::serde(tag = "type")]
pub trait IRapierJoint {
    fn get_base(&self) -> &RapierJointBase;
    fn get_mut_base(&mut self) -> &mut RapierJointBase;
    fn get_type(&self) -> JointType;
    #[cfg(feature = "dim2")]
    fn get_damped_spring(&self) -> Option<&RapierDampedSpringJoint2D>;
    #[cfg(feature = "dim2")]
    fn get_pin(&self) -> Option<&RapierPinJoint2D>;
    #[cfg(feature = "dim2")]
    fn get_mut_damped_spring(&mut self) -> Option<&mut RapierDampedSpringJoint2D>;
    #[cfg(feature = "dim2")]
    fn get_mut_pin(&mut self) -> Option<&mut RapierPinJoint2D>;
}
#[derive(Serialize, Deserialize)]
pub struct RapierJointBase {
    max_force: f32,
    handle: ImpulseJointHandle,
    space_handle: WorldHandle,
    disabled_collisions_between_bodies: bool,
}
impl RapierJointBase {
    pub fn new(space_handle: WorldHandle, handle: ImpulseJointHandle) -> Self {
        Self {
            max_force: f32::MAX,
            handle,
            space_handle,
            disabled_collisions_between_bodies: true,
        }
    }

    pub fn get_handle(&self) -> ImpulseJointHandle {
        self.handle
    }

    pub fn get_space_handle(&self) -> WorldHandle {
        self.space_handle
    }

    pub fn set_max_force(&mut self, force: f32) {
        self.max_force = force;
    }

    pub fn get_max_force(&self) -> f32 {
        self.max_force
    }

    pub fn is_valid(&self) -> bool {
        self.space_handle != WorldHandle::default() && self.handle != ImpulseJointHandle::invalid()
    }

    pub fn disable_collisions_between_bodies(
        &mut self,
        disabled: bool,
        physics_engine: &mut PhysicsEngine,
    ) {
        self.disabled_collisions_between_bodies = disabled;
        if self.is_valid() {
            physics_engine.joint_change_disable_collision(
                self.space_handle,
                self.handle,
                self.disabled_collisions_between_bodies,
            );
        }
    }

    pub fn is_disabled_collisions_between_bodies(&self) -> bool {
        self.disabled_collisions_between_bodies
    }

    pub fn copy_settings_from(
        &mut self,
        joint: &RapierJointBase,
        physics_engine: &mut PhysicsEngine,
    ) {
        self.set_max_force(joint.get_max_force());
        self.disable_collisions_between_bodies(
            joint.is_disabled_collisions_between_bodies(),
            physics_engine,
        );
    }

    pub fn destroy_joint(&mut self, physics_engine: &mut PhysicsEngine) {
        physics_engine.destroy_joint(self.space_handle, self.handle);
        self.handle = ImpulseJointHandle::invalid();
    }
}
#[derive(Serialize, Deserialize)]
pub struct RapierEmptyJoint {
    base: RapierJointBase,
}
impl RapierEmptyJoint {
    pub fn new() -> Self {
        Self {
            base: RapierJointBase::new(WorldHandle::default(), ImpulseJointHandle::invalid()),
        }
    }
}
#[typetag::serde]
impl IRapierJoint for RapierEmptyJoint {
    fn get_type(&self) -> JointType {
        JointType::MAX
    }

    fn get_base(&self) -> &RapierJointBase {
        &self.base
    }

    fn get_mut_base(&mut self) -> &mut RapierJointBase {
        &mut self.base
    }

    #[cfg(feature = "dim2")]
    fn get_damped_spring(&self) -> Option<&RapierDampedSpringJoint2D> {
        None
    }

    #[cfg(feature = "dim2")]
    fn get_pin(&self) -> Option<&RapierPinJoint2D> {
        None
    }

    #[cfg(feature = "dim2")]
    fn get_mut_damped_spring(&mut self) -> Option<&mut RapierDampedSpringJoint2D> {
        None
    }

    #[cfg(feature = "dim2")]
    fn get_mut_pin(&mut self) -> Option<&mut RapierPinJoint2D> {
        None
    }
}
impl Drop for RapierJointBase {
    fn drop(&mut self) {
        if self.is_valid() {
            godot_error!("RapierJointBase leaked");
        }
    }
}
