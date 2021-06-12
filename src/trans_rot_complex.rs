/////////////////////////////////////////////
// Translation-Rotation Complex
// This module provides an abstraction for operations involving
//	both translational and rotaional quantities
// Contains a translation and a rotation
// Uses the Glam crate's Vec3 and Quat structs

use bevy::math::prelude::*;
use std::ops::Mul;

#[derive(Clone, Copy)]
pub struct TRC {
    translation: Vec3,
    rotation: Quat
}

impl TRC {
    const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY
    };

    ////////////////////////////////
    // Builder functions
    // Return new instances of TRC

    pub fn translated_by(&self, translation: Vec3) -> Self {
        Self {
            translation: self.translation + translation,
            rotation: self.rotation
        }
    }

    pub fn rotated_by(&self, rotation: Quat) -> Self {
        Self {
            translation: self.translation,
            rotation: rotation * self.rotation
        }
    }

    pub fn inverse(&self) -> Self {
        Self {
            translation: -self.translation,
            rotation: self.rotation.inverse()
        }
    }

    // basically scale the transformaton by a scalar
    // ratio ranges from 0.0 to 1.0
    pub fn interpolate(&self, ratio: f32) -> Self {
        assert!(ratio >= 0.0 && ratio <= 1.0, "Invalid ratio value");
        Self {
            translation: self.translation * ratio,
            rotation: Quat::IDENTITY.slerp(self.rotation, ratio)
        }
    }

    //////////////////////////////////////
    // Mutating function
    // Mutates the existing intance of TRC

    pub fn translate(&mut self, translation: Vec3) {
        self.translation += translation;
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

}

impl Mul for TRC {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            translation: self.translation + other.translation,
            rotation: self.rotation * other.rotation
        }
    }
}

