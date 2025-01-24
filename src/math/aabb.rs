use std::ops;

use super::{interval::Interval, ray::Ray, vec3::Vec3};

#[derive(Debug, Default)]
pub struct AABB {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl AABB {
    pub const EMPTY: Self = AABB::new(Interval::EMPTY, Interval::EMPTY, Interval::EMPTY);
    pub const UNIVERSE: Self =
        AABB::new(Interval::UNIVERSE, Interval::UNIVERSE, Interval::UNIVERSE);

    pub const fn new(x: Interval, y: Interval, z: Interval) -> AABB {
        AABB { x, y, z }
    }

    pub const fn combine(a: &AABB, b: &AABB) -> AABB {
        AABB {
            x: Interval::combine(&a.x, &b.x),
            y: Interval::combine(&a.y, &b.y),
            z: Interval::combine(&a.z, &b.z),
        }
    }

    pub fn from_points(a: &Vec3, b: &Vec3) -> AABB {
        let x = if a.x <= b.x {
            Interval::new(a.x, b.x)
        } else {
            Interval::new(b.x, a.x)
        };

        let y = if a.y <= b.y {
            Interval::new(a.y, b.y)
        } else {
            Interval::new(b.y, a.y)
        };

        let z = if a.z <= b.z {
            Interval::new(a.z, b.z)
        } else {
            Interval::new(b.z, a.z)
        };

        AABB::new(x, y, z)
    }
    pub fn hit(&self, ray: &Ray, mut t_range: Interval) -> bool {
        for i in 0..3 {
            let axis_interval = self[i];

            let t0 = (axis_interval.start - ray.origin[i]) / ray.direction[i];
            let t1 = (axis_interval.end - ray.origin[i]) / ray.direction[i];

            let (t0, t1) = if t0 <= t1 { (t0, t1) } else { (t1, t0) };
            t_range.start = t_range.start.max(t0);
            t_range.end = t_range.end.min(t1);

            if t_range.end <= t_range.start {
                return false;
            }
        }

        true
    }

    pub fn longest_axis(&self) -> u32 {
        if self.x.size() >= self.y.size() {
            if self.x.size() >= self.z.size() {
                0
            } else {
                2
            }
        } else {
            if self.y.size() >= self.z.size() {
                1
            } else {
                2
            }
        }
    }
}

impl ops::Index<usize> for AABB {
    type Output = Interval;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("index out of bounds"),
        }
    }
}
