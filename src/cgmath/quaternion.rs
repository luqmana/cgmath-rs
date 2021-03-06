// Copyright 2013 The CGMath Developers. For a full listing of the authors,
// refer to the AUTHORS file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::num::{zero, one, cast};

use angle::{Angle, Rad, acos, sin, sin_cos};
use approx::ApproxEq;
use array::{Array, build};
use matrix::{Matrix3, ToMatrix3, ToMatrix4, Matrix4};
use point::{Point3};
use rotation::{Rotation, Rotation3, Basis3, ToBasis3};
use vector::{Vector3, Vector, EuclideanVector};
use partial_ord::PartOrdFloat;

/// A quaternion in scalar/vector form
#[deriving(Clone, Eq)]
pub struct Quaternion<S> { pub s: S, pub v: Vector3<S> }

array!(impl<S> Quaternion<S> -> [S, ..4] _4)

pub trait ToQuaternion<S: Float> {
    fn to_quaternion(&self) -> Quaternion<S>;
}

impl<S: PartOrdFloat<S>>
Quaternion<S> {
    /// Construct a new quaternion from one scalar component and three
    /// imaginary components
    #[inline]
    pub fn new(w: S, xi: S, yj: S, zk: S) -> Quaternion<S> {
        Quaternion::from_sv(w, Vector3::new(xi, yj, zk))
    }

    /// Construct a new quaternion from a scalar and a vector
    #[inline]
    pub fn from_sv(s: S, v: Vector3<S>) -> Quaternion<S> {
        Quaternion { s: s, v: v }
    }

    /// The additive identity, ie: `q = 0 + 0i + 0j + 0i`
    #[inline]
    pub fn zero() -> Quaternion<S> {
        Quaternion::new(zero(), zero(), zero(), zero())
    }

    /// The multiplicative identity, ie: `q = 1 + 0i + 0j + 0i`
    #[inline]
    pub fn identity() -> Quaternion<S> {
        Quaternion::from_sv(one::<S>(), Vector3::zero())
    }

    /// The result of multiplying the quaternion a scalar
    #[inline]
    pub fn mul_s(&self, value: S) -> Quaternion<S> {
        Quaternion::from_sv(self.s * value, self.v.mul_s(value))
    }

    /// The result of dividing the quaternion a scalar
    #[inline]
    pub fn div_s(&self, value: S) -> Quaternion<S> {
        Quaternion::from_sv(self.s / value, self.v.div_s(value))
    }

    /// The result of multiplying the quaternion by a vector
    #[inline]
    pub fn mul_v(&self, vec: &Vector3<S>) -> Vector3<S>  {
        let tmp = self.v.cross(vec).add_v(&vec.mul_s(self.s.clone()));
        self.v.cross(&tmp).mul_s(cast(2).unwrap()).add_v(vec)
    }

    /// The sum of this quaternion and `other`
    #[inline]
    pub fn add_q(&self, other: &Quaternion<S>) -> Quaternion<S> {
        build(|i| self.i(i).add(other.i(i)))
    }

    /// The difference between this quaternion and `other`
    #[inline]
    pub fn sub_q(&self, other: &Quaternion<S>) -> Quaternion<S> {
        build(|i| self.i(i).add(other.i(i)))
    }

    /// The the result of multipliplying the quaternion by `other`
    pub fn mul_q(&self, other: &Quaternion<S>) -> Quaternion<S> {
        Quaternion::new(self.s * other.s - self.v.x * other.v.x - self.v.y * other.v.y - self.v.z * other.v.z,
                        self.s * other.v.x + self.v.x * other.s + self.v.y * other.v.z - self.v.z * other.v.y,
                        self.s * other.v.y + self.v.y * other.s + self.v.z * other.v.x - self.v.x * other.v.z,
                        self.s * other.v.z + self.v.z * other.s + self.v.x * other.v.y - self.v.y * other.v.x)
    }

    #[inline]
    pub fn mul_self_s(&mut self, s: S) {
        self.each_mut(|_, x| *x = x.mul(&s))
    }

    #[inline]
    pub fn div_self_s(&mut self, s: S) {
        self.each_mut(|_, x| *x = x.div(&s))
    }

    #[inline]
    pub fn add_self_q(&mut self, other: &Quaternion<S>) {
        self.each_mut(|i, x| *x = x.add(other.i(i)));
    }

    #[inline]
    pub fn sub_self_q(&mut self, other: &Quaternion<S>) {
        self.each_mut(|i, x| *x = x.sub(other.i(i)));
    }

    #[inline]
    pub fn mul_self_q(&mut self, other: &Quaternion<S>) {
        *self = self.mul_q(other);
    }

    /// The dot product of the quaternion and `other`
    #[inline]
    pub fn dot(&self, other: &Quaternion<S>) -> S {
        self.s * other.s + self.v.dot(&other.v)
    }

    /// The conjugate of the quaternion
    #[inline]
    pub fn conjugate(&self) -> Quaternion<S> {
        Quaternion::from_sv(self.s.clone(), -self.v.clone())
    }

    /// The squared magnitude of the quaternion. This is useful for
    /// magnitude comparisons where the exact magnitude does not need to be
    /// calculated.
    #[inline]
    pub fn magnitude2(&self) -> S {
        self.s * self.s + self.v.length2()
    }

    /// The magnitude of the quaternion
    ///
    /// # Performance notes
    ///
    /// For instances where the exact magnitude of the quaternion does not need
    /// to be known, for example for quaternion-quaternion magnitude comparisons,
    /// it is advisable to use the `magnitude2` method instead.
    #[inline]
    pub fn magnitude(&self) -> S {
        self.magnitude2().sqrt()
    }

    /// The normalized quaternion
    #[inline]
    pub fn normalize(&self) -> Quaternion<S> {
        self.mul_s(one::<S>() / self.magnitude())
    }

    /// Normalised linear interpolation
    ///
    /// # Return value
    ///
    /// The intoperlated quaternion
    pub fn nlerp(&self, other: &Quaternion<S>, amount: S) -> Quaternion<S> {
        self.mul_s(one::<S>() - amount).add_q(&other.mul_s(amount)).normalize()
    }
}

impl<S: PartOrdFloat<S>>
Quaternion<S> {
    /// Spherical Linear Intoperlation
    ///
    /// Perform a spherical linear interpolation between the quaternion and
    /// `other`. Both quaternions should be normalized first.
    ///
    /// # Return value
    ///
    /// The intoperlated quaternion
    ///
    /// # Performance notes
    ///
    /// The `acos` operation used in `slerp` is an expensive operation, so unless
    /// your quarternions a far away from each other it's generally more advisable
    /// to use `nlerp` when you know your rotations are going to be small.
    ///
    /// - [Understanding Slerp, Then Not Using It]
    ///   (http://number-none.com/product/Understanding%20Slerp,%20Then%20Not%20Using%20It/)
    /// - [Arcsynthesis OpenGL tutorial]
    ///   (http://www.arcsynthesis.org/gltut/Positioning/Tut08%20Interpolation.html)
    pub fn slerp(&self, other: &Quaternion<S>, amount: S) -> Quaternion<S> {
        use std::num::cast;

        let dot = self.dot(other);
        let dot_threshold = cast(0.9995).unwrap();

        // if quaternions are close together use `nlerp`
        if dot > dot_threshold {
            self.nlerp(other, amount)
        } else {
            // stay within the domain of acos()
            // TODO REMOVE WHEN https://github.com/mozilla/rust/issues/12068 IS RESOLVED
            let robust_dot = if dot > one::<S>() {
                one::<S>()
            } else if dot < -one::<S>() {
                -one::<S>()
            } else {
                dot
            };

            let theta: Rad<S> = acos(robust_dot.clone());

            let scale1 = sin(theta.mul_s(one::<S>() - amount));
            let scale2 = sin(theta.mul_s(amount));

            self.mul_s(scale1)
                .add_q(&other.mul_s(scale2))
                .mul_s(sin(theta).recip())
        }
    }
}

impl<S: Float + ApproxEq<S>>
ToMatrix3<S> for Quaternion<S> {
    /// Convert the quaternion to a 3 x 3 rotation matrix
    fn to_matrix3(&self) -> Matrix3<S> {
        let x2 = self.v.x + self.v.x;
        let y2 = self.v.y + self.v.y;
        let z2 = self.v.z + self.v.z;

        let xx2 = x2 * self.v.x;
        let xy2 = x2 * self.v.y;
        let xz2 = x2 * self.v.z;

        let yy2 = y2 * self.v.y;
        let yz2 = y2 * self.v.z;
        let zz2 = z2 * self.v.z;

        let sy2 = y2 * self.s;
        let sz2 = z2 * self.s;
        let sx2 = x2 * self.s;

        Matrix3::new(one::<S>() - yy2 - zz2, xy2 + sz2, xz2 - sy2,
                     xy2 - sz2, one::<S>() - xx2 - zz2, yz2 + sx2,
                     xz2 + sy2, yz2 - sx2, one::<S>() - xx2 - yy2)
    }
}

impl<S: Float + ApproxEq<S>>
ToMatrix4<S> for Quaternion<S> {
    /// Convert the quaternion to a 4 x 4 rotation matrix
    fn to_matrix4(&self) -> Matrix4<S> {
        let x2 = self.v.x + self.v.x;
        let y2 = self.v.y + self.v.y;
        let z2 = self.v.z + self.v.z;

        let xx2 = x2 * self.v.x;
        let xy2 = x2 * self.v.y;
        let xz2 = x2 * self.v.z;

        let yy2 = y2 * self.v.y;
        let yz2 = y2 * self.v.z;
        let zz2 = z2 * self.v.z;

        let sy2 = y2 * self.s;
        let sz2 = z2 * self.s;
        let sx2 = x2 * self.s;

        Matrix4::new(one::<S>() - yy2 - zz2, xy2 + sz2, xz2 - sy2, zero::<S>(),
                     xy2 - sz2, one::<S>() - xx2 - zz2, yz2 + sx2, zero::<S>(),
                     xz2 + sy2, yz2 - sx2, one::<S>() - xx2 - yy2, zero::<S>(),
                     zero::<S>(), zero::<S>(), zero::<S>(), one::<S>())
    }
}

impl<S: PartOrdFloat<S>>
Neg<Quaternion<S>> for Quaternion<S> {
    #[inline]
    fn neg(&self) -> Quaternion<S> {
        Quaternion::from_sv(-self.s, -self.v)
    }
}

impl<S: fmt::Show> fmt::Show for Quaternion<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "{} + {}i + {}j + {}k",
                self.s,
                self.v.x,
                self.v.y,
                self.v.z)
    }
}

// Quaternion Rotation impls

impl<S: PartOrdFloat<S>>
ToBasis3<S> for Quaternion<S> {
    #[inline]
    fn to_rot3(&self) -> Basis3<S> { Basis3::from_quaternion(self) }
}

impl<S: Float> ToQuaternion<S> for Quaternion<S> {
    #[inline]
    fn to_quaternion(&self) -> Quaternion<S> { self.clone() }
}

impl<S: PartOrdFloat<S>>
Rotation<S, [S, ..3], Vector3<S>, Point3<S>> for Quaternion<S> {
    #[inline]
    fn identity() -> Quaternion<S> { Quaternion::identity() }

    #[inline]
    fn look_at(dir: &Vector3<S>, up: &Vector3<S>) -> Quaternion<S> {
        Matrix3::look_at(dir, up).to_quaternion()
    }

    #[inline]
    fn between_vectors(a: &Vector3<S>, b: &Vector3<S>) -> Quaternion<S> {
        //http://stackoverflow.com/questions/1171849/
        //finding-quaternion-representing-the-rotation-from-one-vector-to-another
        Quaternion::from_sv(one::<S>() + a.dot(b), a.cross(b)).normalize()
    }

    #[inline]
    fn rotate_vector(&self, vec: &Vector3<S>) -> Vector3<S> { self.mul_v(vec) }

    #[inline]
    fn concat(&self, other: &Quaternion<S>) -> Quaternion<S> { self.mul_q(other) }

    #[inline]
    fn concat_self(&mut self, other: &Quaternion<S>) { self.mul_self_q(other); }

    #[inline]
    fn invert(&self) -> Quaternion<S> { self.conjugate().div_s(self.magnitude2()) }

    #[inline]
    fn invert_self(&mut self) { *self = self.invert() }
}

impl<S: PartOrdFloat<S>>
Rotation3<S> for Quaternion<S>
{
    #[inline]
    fn from_axis_angle(axis: &Vector3<S>, angle: Rad<S>) -> Quaternion<S> {
        let (s, c) = sin_cos(angle.mul_s(cast(0.5).unwrap()));
        Quaternion::from_sv(c, axis.mul_s(s))
    }

    fn from_euler(x: Rad<S>, y: Rad<S>, z: Rad<S>) -> Quaternion<S> {
        // http://en.wikipedia.org/wiki/Conversion_between_quaternions_and_Euler_angles#Conversion
        let (sx2, cx2) = sin_cos(x.mul_s(cast(0.5).unwrap()));
        let (sy2, cy2) = sin_cos(y.mul_s(cast(0.5).unwrap()));
        let (sz2, cz2) = sin_cos(z.mul_s(cast(0.5).unwrap()));

        Quaternion::new(cz2 * cx2 * cy2 + sz2 * sx2 * sy2,
                        sz2 * cx2 * cy2 - cz2 * sx2 * sy2,
                        cz2 * sx2 * cy2 + sz2 * cx2 * sy2,
                        cz2 * cx2 * sy2 - sz2 * sx2 * cy2)
    }
}
