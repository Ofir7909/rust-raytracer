use std::sync::Arc;

use crate::{
    materials::Material,
    math::{aabb::AABB, interval::Interval, ray::Ray, vec3::Vec3},
};

pub struct HitInfo {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub front_face: bool,
}

impl HitInfo {
    pub fn new(material: Arc<dyn Material>) -> HitInfo {
        HitInfo {
            material,
            point: Default::default(),
            normal: Default::default(),
            t: Default::default(),
            u: Default::default(),
            v: Default::default(),
            front_face: Default::default(),
        }
    }
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.front_face = Vec3::dot(&outward_normal, &ray.direction) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -*outward_normal
        }
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_range: &Interval) -> Option<HitInfo>;
    fn bounding_box(&self) -> &AABB;
}

pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Arc<dyn Material>,
    bounding_box: AABB,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Arc<dyn Material>) -> Sphere {
        let rvec = Vec3::uniform(radius);
        let bounding_box = AABB::from_points(&(center - rvec), &(center + rvec));

        Sphere {
            center,
            radius,
            material,
            bounding_box,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: &Interval) -> Option<HitInfo> {
        let origin_to_center = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = Vec3::dot(&ray.direction, &origin_to_center);
        let c = origin_to_center.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_discriminant = discriminant.sqrt();

        let mut t = (h - sqrt_discriminant) / a;
        if !t_range.surrounds(t) {
            t = (h + sqrt_discriminant) / a;
            if !t_range.surrounds(t) {
                return None;
            }
        }

        let mut hit_info = HitInfo::new(self.material.clone());
        hit_info.t = t;
        hit_info.point = ray.at(t);

        let outward_normal = (hit_info.point - self.center).normalized();
        hit_info.set_face_normal(ray, &outward_normal);

        Some(hit_info)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}

pub struct Quad {
    origin: Vec3,
    u: Vec3,
    v: Vec3,
    normal: Vec3,
    d: f32,
    w: Vec3,
    material: Arc<dyn Material>,
    bounding_box: AABB,
}

impl Quad {
    pub fn new(origin: Vec3, u: Vec3, v: Vec3, material: Arc<dyn Material>) -> Quad {
        let perp = Vec3::cross(&u, &v);
        let normal = perp.normalized();
        let d = Vec3::dot(&normal, &origin);
        let w = perp / perp.length_squared();

        let bbox_diagonal1 = AABB::from_points(&origin, &(origin + u + v));
        let bbox_diagonal2 = AABB::from_points(&(origin + u), &(origin + v));
        let bounding_box = AABB::combine(&bbox_diagonal1, &bbox_diagonal2);

        Quad {
            origin,
            u,
            v,
            normal,
            d,
            w,
            material,
            bounding_box,
        }
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, t_range: &Interval) -> Option<HitInfo> {
        let denom = Vec3::dot(&self.normal, &ray.direction);
        if denom.abs() < 1e-8 {
            return None;
        }

        let nom = self.d - Vec3::dot(&self.normal, &ray.origin);
        let t = nom / denom;
        if !t_range.surrounds(t) {
            return None;
        }

        let hit_point = ray.at(t);
        let q_p_vector = hit_point - self.origin;
        let alpha = Vec3::dot(&self.w, &Vec3::cross(&q_p_vector, &self.v));
        let beta = Vec3::dot(&self.w, &Vec3::cross(&self.u, &q_p_vector));

        const UNIT_INTERVAL: Interval = Interval::new(0.0, 1.0);
        if !UNIT_INTERVAL.contains(alpha) || !UNIT_INTERVAL.contains(beta) {
            return None;
        }

        let mut hit_info = HitInfo::new(self.material.clone());
        hit_info.t = t;
        hit_info.point = hit_point;
        hit_info.set_face_normal(ray, &self.normal);

        Some(hit_info)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Arc<dyn Hittable>>,
    bounding_box: AABB,
}

impl HittableList {
    pub fn new() -> HittableList {
        return Default::default();
    }

    pub fn add(&mut self, obj: Arc<dyn Hittable>) {
        self.bounding_box = AABB::combine(&self.bounding_box, obj.bounding_box());
        self.objects.push(obj);
    }

    pub fn reserve(&mut self, count: usize) {
        self.objects.reserve(count);
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_range: &Interval) -> Option<HitInfo> {
        let mut hit_info: Option<HitInfo> = None;
        let mut closest_so_far = t_range.end;

        for obj in self.objects.iter() {
            match obj.hit(ray, &Interval::new(t_range.start, closest_so_far)) {
                Some(info) => {
                    closest_so_far = info.t;
                    hit_info = Some(info);
                }
                None => (),
            }
        }

        hit_info
    }

    fn bounding_box(&self) -> &AABB {
        return &self.bounding_box;
    }
}

pub struct BVHNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bounding_box: AABB,
}

impl BVHNode {
    pub fn new(objects: &mut [Arc<dyn Hittable>]) -> BVHNode {
        let mut bounding_box = AABB::EMPTY;
        for obj in objects.iter() {
            bounding_box = AABB::combine(&bounding_box, obj.bounding_box());
        }

        let axis: u32 = bounding_box.longest_axis();

        let left;
        let right;
        match objects.len() {
            1 => {
                left = objects[0].clone();
                right = objects[0].clone();
            }
            2 => {
                left = objects[0].clone();
                right = objects[1].clone();
            }
            _ => {
                objects.sort_by(|a, b| {
                    let a_start = a.bounding_box()[axis as usize].start;
                    let b_start = b.bounding_box()[axis as usize].start;
                    a_start.total_cmp(&b_start)
                });

                let mid = objects.len() / 2;
                left = Arc::new(BVHNode::new(&mut objects[..mid]));
                right = Arc::new(BVHNode::new(&mut objects[mid..]));
            }
        }

        BVHNode {
            left,
            right,
            bounding_box,
        }
    }

    pub fn from_hittable_list(hittable_list: &mut HittableList) -> BVHNode {
        BVHNode::new(&mut hittable_list.objects)
    }
}

impl Hittable for BVHNode {
    fn hit(&self, ray: &Ray, t_range: &Interval) -> Option<HitInfo> {
        if !self.bounding_box.hit(ray, *t_range) {
            return None;
        }

        let left_hit = self.left.hit(ray, t_range);
        match left_hit {
            Some(left_hit) => {
                let new_t_range = Interval::new(t_range.start, left_hit.t);
                let hit_right = self.right.hit(ray, &new_t_range);
                match hit_right {
                    Some(hit_right) => Some(hit_right),
                    None => Some(left_hit),
                }
            }
            None => self.right.hit(ray, t_range),
        }
    }

    fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}
