use std::sync::Arc;

use rand::Rng;

use crate::{
    materials::Material,
    math::{aabb::AABB, interval::Interval, ray::Ray, vec3::Vec3},
};

pub struct HitInfo {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f32,
    pub front_face: bool,
}

impl HitInfo {
    pub fn new(material: Arc<dyn Material>) -> HitInfo {
        HitInfo {
            material,
            point: Default::default(),
            normal: Default::default(),
            t: Default::default(),
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
    pub fn new(objects: &mut Vec<Arc<dyn Hittable>>, start: usize, end: usize) -> BVHNode {
        let axis: u32 = rand::thread_rng().gen_range(0..=2);

        let left;
        let right;
        let span = end - start;
        match span {
            1 => {
                left = objects[start].clone();
                right = objects[start].clone();
            }
            2 => {
                left = objects[start].clone();
                right = objects[start + 1].clone();
            }
            _ => {
                let objects_slice = &mut objects[start..end];
                objects_slice.sort_by(|a, b| {
                    let a_start = a.bounding_box()[axis as usize].start;
                    let b_start = b.bounding_box()[axis as usize].start;
                    a_start.total_cmp(&b_start)
                });

                let mid = start + span / 2;
                left = Arc::new(BVHNode::new(objects, start, mid));
                right = Arc::new(BVHNode::new(objects, mid, end));
            }
        }

        let bounding_box = AABB::combine(left.bounding_box(), right.bounding_box());
        BVHNode {
            left,
            right,
            bounding_box,
        }
    }

    pub fn from_hittable_list(hittable_list: &mut HittableList) -> BVHNode {
        let end = hittable_list.objects.len();
        BVHNode::new(&mut hittable_list.objects, 0, end)
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
