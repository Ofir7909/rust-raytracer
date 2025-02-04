use std::sync::Arc;

use crate::math::vec3::Vec3;

pub trait Texture: Send + Sync {
    fn sample(&self, u: f32, v: f32, p: &Vec3) -> Vec3;
}

pub struct SolidColorTexture {
    pub color: Vec3,
}

impl Texture for SolidColorTexture {
    fn sample(&self, _: f32, _: f32, _: &Vec3) -> Vec3 {
        self.color
    }
}

pub struct CheckerTexture {
    pub even_texture: Arc<dyn Texture>,
    pub odd_texture: Arc<dyn Texture>,
}

impl Texture for CheckerTexture {
    fn sample(&self, u: f32, v: f32, p: &Vec3) -> Vec3 {
        let x_int = p.x.floor() as i32;
        let y_int = p.y.floor() as i32;
        let z_int = p.z.floor() as i32;

        let is_even = (x_int + y_int + z_int) % 2 == 0;

        if is_even {
            self.even_texture.sample(u, v, p)
        } else {
            self.odd_texture.sample(u, v, p)
        }
    }
}
