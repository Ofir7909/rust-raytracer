mod math;

use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

use math::{ray::Ray, vec3::Vec3};
use rand::Rng;

struct Screen {
    width: u32,
    height: u32,
    buffer: Vec<(u8, u8, u8)>,
}

impl Screen {
    fn new(width: u32, height: u32) -> Screen {
        Screen {
            width,
            height,
            buffer: vec![(0, 0, 0); (width * height) as usize],
        }
    }

    fn write_pixel(&mut self, x: u32, y: u32, pixel: (u8, u8, u8)) {
        self.buffer[(y * self.width + x) as usize] = pixel
    }

    fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

fn write_to_file_ppm(screen: &Screen, filepath: &Path) -> Result<(), io::Error> {
    let parent_dir = filepath.parent().unwrap_or(Path::new(""));
    fs::create_dir_all(parent_dir)?;

    let mut file = BufWriter::new(File::create(&filepath)?);

    write!(file, "P3\n{} {}\n255\n", screen.width, screen.height)?;
    for pixel in screen.buffer.iter() {
        write!(file, "{} {} {}\n", pixel.0, pixel.1, pixel.2)?
    }

    Ok(())
}

#[derive(Default)]
struct HitInfo {
    point: Vec3,
    normal: Vec3,
    t: f32,
    front_face: bool,
}
impl HitInfo {
    fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.front_face = Vec3::dot(&outward_normal, &ray.direction) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -*outward_normal
        }
    }
}

trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo>;
}

struct Sphere {
    center: Vec3,
    radius: f32,
}
impl Sphere {
    fn new(center: Vec3, radius: f32) -> Sphere {
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

type HittableList = Vec<Box<dyn Hittable>>;
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

fn sample_unit_square(rng: &mut impl rand::RngCore) -> Vec3 {
    Vec3::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5, 0.0)
}

fn ray_color(ray: &Ray, world: &impl Hittable) -> Vec3 {
    match world.hit(ray, 0.0, 1000.0) {
        Some(hit_info) => {
            return 0.5 * (hit_info.normal + Vec3::one());
        }
        None => (),
    }

    let sky_color: Vec3 = Vec3::new(0.5, 0.7, 1.0);
    let horizon_color: Vec3 = Vec3::new(1.0, 1.0, 1.0);

    let unit_direction = ray.direction.normalized();
    let a = (unit_direction.y + 1.0) * 0.5;

    a * sky_color + (1.0 - a) * horizon_color
}

fn main() {
    let width = 540;
    let height = 400;
    let samples_per_pixel = 10;

    let mut screen = Screen::new(width, height);

    //Scene
    let mut hittables: HittableList = Vec::new();
    hittables.push(Box::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)));
    hittables.push(Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)));

    // Camera
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * screen.aspect_ratio();
    let camera_center = Vec3::new(0.0, 0.0, 0.0);

    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);
    let viewport_upper_left =
        camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;

    let pixel_delta_u = viewport_u / screen.width as f32;
    let pixel_delta_v = viewport_v / screen.height as f32;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    let mut rng = rand::thread_rng();

    for y in 0..height {
        for x in 0..width {
            let mut color: Vec3 = Vec3::zero();
            for _ in 0..samples_per_pixel {
                let offset = sample_unit_square(&mut rng);
                let pixel_sample = pixel00_loc
                    + pixel_delta_u * (x as f32 + offset.x)
                    + pixel_delta_v * (y as f32 + offset.y);
                let ray_dir = pixel_sample - camera_center;
                let ray = Ray::new(camera_center, ray_dir);

                color += ray_color(&ray, &hittables);
            }

            color /= samples_per_pixel as f32;

            screen.write_pixel(
                x,
                y,
                (
                    (color.x * 255.99) as u8,
                    (color.y * 255.99) as u8,
                    (color.z * 255.99) as u8,
                ),
            );
        }
        print!("\r{}/{}", y, height);
    }
    println!();

    print!("Saving to file... ");
    io::stdout().flush().unwrap();
    write_to_file_ppm(&screen, Path::new("./out/test.ppm")).unwrap();
    println!("Done!");
}
