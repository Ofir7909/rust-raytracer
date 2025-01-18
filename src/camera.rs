use crate::{
    math::{ray::Ray, vec3::Vec3},
    utils,
};

pub struct Camera {
    position: Vec3,
    pixel00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    pub fn new(width: u32, height: u32, position: Vec3) -> Camera {
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let aspect_ratio = width as f32 / height as f32;
        let viewport_width = viewport_height * aspect_ratio;

        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);
        let viewport_upper_left =
            position - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;

        let pixel_delta_u = viewport_u / width as f32;
        let pixel_delta_v = viewport_v / height as f32;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Camera {
            position,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    pub fn get_ray(&self, x: u32, y: u32) -> Ray {
        let offset = utils::sample_unit_square();
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (x as f32 + offset.x)
            + self.pixel_delta_v * (y as f32 + offset.y);
        let ray_dir = pixel_sample - self.position;

        Ray::new(self.position, ray_dir)
    }
}
