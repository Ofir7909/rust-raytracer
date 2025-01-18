use crate::math::{ray::Ray, vec3::Vec3};

#[derive(Default)]
pub struct HitInfo {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
}
impl HitInfo {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.front_face = Vec3::dot(&outward_normal, &ray.direction) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -*outward_normal
        }
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo>;
}

pub struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Sphere {
        Sphere { center, radius }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
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
        if t <= t_min || t_max <= t {
            t = (h + sqrt_discriminant) / a;
            if t <= t_min || t_max <= t {
                return None;
            }
        }

        let mut hit_info = HitInfo::default();
        hit_info.t = t;
        hit_info.point = ray.at(t);

        let outward_normal = (hit_info.point - self.center).normalized();
        hit_info.set_face_normal(ray, &outward_normal);

        Some(hit_info)
    }
}

pub type HittableList = Vec<Box<dyn Hittable>>;

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        let mut hit_info: Option<HitInfo> = None;
        let mut closest_so_far = t_max;

        for obj in self.iter() {
            match obj.hit(ray, t_min, closest_so_far) {
                Some(info) => {
                    closest_so_far = info.t;
                    hit_info = Some(info);
                }
                None => (),
            }
        }

        hit_info
    }
}
