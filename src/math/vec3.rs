use std::{fmt, ops};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Vec3::new(0.0, 0.0, 0.0);
    pub const ONE: Self = Vec3::new(1.0, 1.0, 1.0);

    pub const RIGHT: Self = Vec3::new(1.0, 0.0, 0.0);
    pub const LEFT: Self = Vec3::new(-1.0, 0.0, 0.0);
    pub const UP: Self = Vec3::new(0.0, 1.0, 0.0);
    pub const DOWN: Self = Vec3::new(0.0, -1.0, 0.0);
    pub const FORWARD: Self = Vec3::new(0.0, 0.0, -1.0);
    pub const BACKWARD: Self = Vec3::new(0.0, 0.0, 1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub const fn uniform(v: f32) -> Vec3 {
        Vec3::new(v, v, v)
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(&mut self) {
        *self /= self.length();
    }

    pub fn normalized(self) -> Vec3 {
        self / self.length()
    }

    pub fn near_zero(&self) -> bool {
        let d = 1e-8;
        self.x.abs() < d && self.y.abs() < d && self.z.abs() < d
    }

    pub fn reflected(&self, normal: &Vec3) -> Vec3 {
        *self - 2.0 * Vec3::dot(self, normal) * *normal
    }

    pub fn refracted(&self, normal: &Vec3, ior_ratio: f32) -> Vec3 {
        let cos_tetha = Vec3::dot(&(-*self), normal);
        let dir_out_perp = (*self + cos_tetha * *normal) * ior_ratio;
        let dir_out_parallel = -1.0 * (1.0 - dir_out_perp.length_squared()).sqrt() * *normal;

        dir_out_perp + dir_out_parallel
    }

    pub fn dot(a: &Vec3, b: &Vec3) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    pub fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec3({},{},{})", self.x, self.y, self.z)
    }
}

impl ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Add<Self> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign<Self> for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl ops::Sub<Self> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign<Self> for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl ops::Mul<Self> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::MulAssign<Self> for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs
    }
}

impl ops::Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("index out of bounds"),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;

    use super::*;

    #[test]
    fn new_works() {
        let v = Vec3::new(10.0, -5.5, 7.0);
        assert_eq!(v.x, 10.0);
        assert_eq!(v.y, -5.5);
        assert_eq!(v.z, 7.0);
    }

    #[test]
    fn default_is_zero() {
        let v = Vec3::default();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }

    #[test]
    fn uniform() {
        let v = Vec3::uniform(-5.5);
        assert_eq!(v.x, -5.5);
        assert_eq!(v.y, -5.5);
        assert_eq!(v.z, -5.5);
    }

    #[test]
    fn length_squared_works() {
        let v = Vec3::new(10.0, -5.5, 7.0);
        assert_eq!(v.length_squared(), 179.25);
    }

    #[test]
    fn length_works() {
        let v = Vec3::new(10.0, -5.5, 7.0);
        assert_approx_eq!(v.length(), 13.388, 0.01);
    }

    #[test]
    fn normalized_unit_vector_no_change() {
        assert_eq!(Vec3::RIGHT, Vec3::RIGHT.normalized());
    }

    #[test]
    fn normalized_length_eq_one() {
        let v = Vec3::new(10.0, -5.5, 7.0);
        assert_approx_eq!(v.normalized().length(), 1.0);
    }

    #[test]
    fn near_zero_works() {
        assert_eq!(Vec3::ZERO.near_zero(), true);
        assert_eq!(Vec3::ONE.near_zero(), false);
        assert_eq!(Vec3::new(10.0, -5.5, 7.0).near_zero(), false);
        assert_eq!(Vec3::new(0.0, 9e-9, 5e-9).near_zero(), true);
    }

    #[test]
    fn reflected_unit_vector_stays_unit() {
        let v = Vec3::new(10.0, -5.5, 7.0).normalized();
        let n = Vec3::new(3.0, 4.0, -8.0).normalized();
        let refracted = v.reflected(&n);
        assert_eq!(refracted.length(), 1.0);
    }

    #[test]
    fn reflected_correct_x() {
        let v = Vec3::new(10.0, -5.5, 7.0);
        let n = Vec3::RIGHT;
        let reflected = v.reflected(&n);
        assert_eq!(reflected, Vec3::new(-10.0, -5.5, 7.0));
    }

    #[test]
    fn refracted_unit_vector_stays_unit() {
        let v = Vec3::new(10.0, -5.5, 7.0).normalized();
        let n = Vec3::new(3.0, 4.0, -8.0).normalized();
        let refracted = v.refracted(&n, 1.0 / 1.5);
        assert_eq!(refracted.length(), 1.0);
    }

    #[test]
    fn dot_produt_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        let res = Vec3::dot(&a, &b);
        assert_eq!(res, -48.0);

        let res = Vec3::dot(&b, &a);
        assert_eq!(res, -48.0);
    }

    #[test]
    fn cross_produt_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        let res = Vec3::cross(&a, &b);
        assert_eq!(res.x, 16.0);
        assert_eq!(res.y, 101.0);
        assert_eq!(res.z, 56.5);

        let res = Vec3::cross(&b, &a);
        assert_eq!(res.x, -16.0);
        assert_eq!(res.y, -101.0);
        assert_eq!(res.z, -56.5);
    }

    #[test]
    fn negate_oper_works() {
        let v = Vec3::new(10.0, -5.5, 7.0);
        assert_eq!(-v, Vec3::new(-10.0, 5.5, -7.0));
    }

    #[test]
    fn add_oper_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        assert_eq!(a + b, Vec3::new(13.0, -1.5, -1.0));
        assert_eq!(b + a, Vec3::new(13.0, -1.5, -1.0));
    }

    #[test]
    fn add_assign_oper_works() {
        let mut a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        a += b;
        assert_eq!(a, Vec3::new(13.0, -1.5, -1.0));
    }

    #[test]
    fn sub_oper_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        assert_eq!(a - b, Vec3::new(7.0, -9.5, 15.0));
        assert_eq!(b - a, Vec3::new(-7.0, 9.5, -15.0));
    }

    #[test]
    fn sub_assign_oper_works() {
        let mut a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        a -= b;
        assert_eq!(a, Vec3::new(7.0, -9.5, 15.0))
    }

    #[test]
    fn mul_oper_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        assert_eq!(a * b, Vec3::new(30.0, -22.0, -56.0));
        assert_eq!(b * a, Vec3::new(30.0, -22.0, -56.0));
    }

    #[test]
    fn mul_assign_oper_works() {
        let mut a = Vec3::new(10.0, -5.5, 7.0);
        let b = Vec3::new(3.0, 4.0, -8.0);
        a *= b;
        assert_eq!(a, Vec3::new(30.0, -22.0, -56.0));
    }

    #[test]
    fn mul_f32_oper_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        assert_eq!(a * 2.0, Vec3::new(20.0, -11.0, 14.0));
        assert_eq!(2.0 * a, Vec3::new(20.0, -11.0, 14.0));
    }

    #[test]
    fn mul_assign_f32_oper_works() {
        let mut a = Vec3::new(10.0, -5.5, 7.0);
        a *= 2.0;
        assert_eq!(a, Vec3::new(20.0, -11.0, 14.0));
    }

    #[test]
    fn div_f32_oper_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        assert_eq!(a / 2.0, Vec3::new(5.0, -2.75, 3.5));
    }

    #[test]
    fn div_assign_f32_oper_works() {
        let mut a = Vec3::new(10.0, -5.5, 7.0);
        a /= 2.0;
        assert_eq!(a, Vec3::new(5.0, -2.75, 3.5));
    }

    #[test]
    fn index_oper_works() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        assert_eq!(a[0], 10.0);
        assert_eq!(a[1], -5.5);
        assert_eq!(a[2], 7.0);
    }

    #[test]
    #[should_panic]
    fn index_oper_out_of_bounds() {
        let a = Vec3::new(10.0, -5.5, 7.0);
        a[3];
    }
}
