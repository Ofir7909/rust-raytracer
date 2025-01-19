use crate::{
    hittables::HitInfo,
    math::{ray::Ray, vec3::Vec3},
    utils,
};

pub trait Material {
    fn scatter(&self, ray_in: &Ray, hit_info: &HitInfo) -> Option<(Vec3, Ray)>;
}

pub struct Lambertian {
    pub albedo: Vec3,
}

impl Material for Lambertian {
    fn scatter(&self, _: &Ray, hit_info: &HitInfo) -> Option<(Vec3, Ray)> {
        let mut dir: Vec3 = hit_info.normal + utils::random_unit_vector();
        if dir.near_zero() {
            dir = hit_info.normal;
        }

        Some((self.albedo, Ray::new(hit_info.point, dir)))
    }
}

pub struct Metal {
    pub albedo: Vec3,
    pub roughness: f32,
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit_info: &HitInfo) -> Option<(Vec3, Ray)> {
        let reflected_dir = ray_in.direction.reflected(&hit_info.normal);
        let reflected_dir =
            reflected_dir.normalized() + self.roughness * utils::random_unit_vector();
        if Vec3::dot(&reflected_dir, &hit_info.normal) > 0.0 {
            Some((self.albedo, Ray::new(hit_info.point, reflected_dir)))
        } else {
            None
        }
    }
}
