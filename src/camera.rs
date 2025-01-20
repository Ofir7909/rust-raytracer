use crate::{
    math::{ray::Ray, vec3::Vec3},
    utils,
};

pub struct Camera {
    position: Vec3,
    pixel00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,

    defocus_angle: f32,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new(
        width: u32,
        height: u32,
        position: Vec3,
        vfov_deg: f32,
        lookat: Vec3,
        up: Vec3,
        defocus_angle: f32,
        focus_dist: f32,
    ) -> Camera {
        let aspect_ratio = width as f32 / height as f32;

        let h = (vfov_deg.to_radians() / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (position - lookat).normalized();
        let u = Vec3::cross(&up, &w).normalized();
        let v = Vec3::cross(&w, &u);

        let viewport_u = viewport_width * u;
        let viewport_v = -viewport_height * v;
        let viewport_upper_left = position - focus_dist * w - viewport_u / 2.0 - viewport_v / 2.0;

        let pixel_delta_u = viewport_u / width as f32;
        let pixel_delta_v = viewport_v / height as f32;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius = focus_dist * (defocus_angle.to_radians() / 2.0).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Camera {
            position,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn get_ray(&self, x: u32, y: u32) -> Ray {
        let offset = utils::sample_unit_square();
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (x as f32 + offset.x)
            + self.pixel_delta_v * (y as f32 + offset.y);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.position
        } else {
            let p = utils::random_in_unit_disk();
            self.position + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
        };

        let ray_dir = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_dir)
    }
}
