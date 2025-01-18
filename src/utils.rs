use crate::math::vec3::Vec3;
use rand::Rng;

pub fn sample_unit_square() -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5, 0.0)
}
