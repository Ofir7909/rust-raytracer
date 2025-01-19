use crate::math::vec3::Vec3;
use rand::Rng;

pub fn sample_unit_square() -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5, 0.0)
}

pub fn random_unit_vector() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let v = Vec3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );
        let len_squared = v.length_squared();
        if 1e-10 <= len_squared && len_squared <= 1.0 {
            return v.normalized();
        }
    }
}

pub fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
    let unit_v = random_unit_vector();
    if Vec3::dot(&unit_v, normal) > 0.0 {
        unit_v
    } else {
        -unit_v
    }
}
