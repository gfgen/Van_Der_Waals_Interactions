/////////////////////////////////////////////
// Translation-Rotation Complex
// This module provides an abstraction for operations involving
//	both translational and rotational quantities
// Contains a translation and a rotation
// Uses the Glam crate's Vec3 and Quat structs

use bevy::math::prelude::*;
use std::ops::{Add, AddAssign, Sub, SubAssign, Neg, Mul, MulAssign};

#[derive(Clone, Copy)]
pub struct TRC {
    pub translation: Vec3,
    pub rotation: Quat
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

// These operators are chain application
// WARNING: application order is left to right, 
//      opposite the multiplication convention
// WARNING: These are not commutative
impl Add for TRC {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            translation: other.translation + self.translation,
            rotation: other.rotation * self.rotation
        }
    }
}

impl AddAssign for TRC {
    fn add_assign(&mut self, rhs: Self) {
        self.translation = rhs.translation + self.translation;
        self.rotation = rhs.rotation * self.rotation;
    }
}

impl Sub for TRC {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + other.inverse()
    }
}

impl SubAssign for TRC {
    fn sub_assign(&mut self, rhs: Self) {
        *self += rhs.inverse();
    }
}

impl Neg for TRC {
    type Output = Self;

    fn neg(self) -> Self {
        self.inverse()       
    }
}


// Since infintesimal rotations are commutative,
//      the rotation component is represented by a vector
#[derive(Clone, Copy)]
pub struct TRCInfintesimal {
    pub translation: Vec3,
    pub rotation: Vec3
}

impl TRCInfintesimal {
    const ZERO: Self = Self {
        translation: Vec3::ZERO,
        rotation: Vec3::ZERO
    };

    pub fn new(translation: Vec3, rotation: Vec3) -> Self {
        Self {
            translation,
            rotation
        }
    }

    // integrate over dx
    pub fn integrate(&self, dx: f32) -> TRC {
        let rotation_length = self.rotation.length();
        let rotation_axis = self.rotation / rotation_length;
        TRC {
            translation: self.translation * dx,
            rotation: Quat::from_axis_angle(rotation_axis, rotation_length * dx)
        }
    }
}

impl Add for TRCInfintesimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            translation: other.translation + self.translation,
            rotation: other.rotation + self.rotation
        }
    }
}

impl AddAssign for TRCInfintesimal {
    fn add_assign(&mut self, rhs: Self) {
        self.translation = rhs.translation + self.translation;
        self.rotation = rhs.rotation + self.rotation;
    }
}

impl Sub for TRCInfintesimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

impl SubAssign for TRCInfintesimal {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

impl Neg for TRCInfintesimal {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            translation: -self.translation,
            rotation: -self.rotation
        }  
    }
}

impl Mul<f32> for TRCInfintesimal {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
       Self {
           translation: self.translation * rhs,
           rotation: self.rotation * rhs
       } 
    }
}

