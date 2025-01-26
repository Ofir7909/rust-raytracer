use rand::Rng;

use crate::{
    hittables::HitInfo,
    math::{ray::Ray, vec3::Vec3},
    utils,
};

pub trait Material: Send + Sync {
    fn scatter(&self, _ray_in: &Ray, _hit_info: &HitInfo) -> Option<(Vec3, Ray)> {
        None
    }

    fn emitted(&self, _hit_info: &HitInfo) -> Vec3 {
        Vec3::ZERO
    }
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

pub struct Dielectric {
    pub ior: f32,
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, hit_info: &HitInfo) -> Option<(Vec3, Ray)> {
        let ri = if hit_info.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };

        let unit_dir = ray_in.direction.normalized();

        let cos_theta = Vec3::dot(&(-unit_dir), &hit_info.normal);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction =
            if cannot_refract || reflectance(cos_theta, ri) > rand::thread_rng().gen::<f32>() {
                // Must reflect
                unit_dir.reflected(&hit_info.normal)
            } else {
                unit_dir.refracted(&hit_info.normal, ri)
            };

        Some((Vec3::ONE, Ray::new(hit_info.point, direction)))
    }
}

fn reflectance(cos_theta: f32, ior: f32) -> f32 {
    // Schlick's approximation
    let r0 = (1.0 - ior) / (1.0 + ior);
    let r0 = r0 * r0;

    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
}

pub struct DiffuseLight {
    pub color: Vec3,
}

impl Material for DiffuseLight {
    fn emitted(&self, _hit_info: &HitInfo) -> Vec3 {
        self.color
    }
}
