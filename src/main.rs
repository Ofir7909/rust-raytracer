mod camera;
mod hittables;
mod materials;
mod math;
mod screen;
mod utils;

use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
    rc::Rc,
};

use camera::Camera;
use hittables::{Hittable, HittableList, Sphere};
use math::{ray::Ray, vec3::Vec3};
use screen::Screen;

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

fn ray_color(ray: &Ray, world: &impl Hittable, depth: u32) -> Vec3 {
    if depth <= 0 {
        return Vec3::zero();
    }
    match world.hit(ray, 0.001, 1000.0) {
        Some(hit_info) => {
            return match hit_info.material.scatter(ray, &hit_info) {
                Some((attenution, scattered_ray)) => {
                    attenution * ray_color(&scattered_ray, world, depth - 1)
                }
                None => Vec3::zero(),
            };
        }
        None => (),
    }

    let sky_color: Vec3 = Vec3::new(0.5, 0.7, 1.0);
    let horizon_color: Vec3 = Vec3::new(1.0, 1.0, 1.0);

    let unit_direction = ray.direction.normalized();
    let a = (unit_direction.y + 1.0) * 0.5;

    a * sky_color + (1.0 - a) * horizon_color
}

fn linear_to_gamma(color: &Vec3) -> Vec3 {
    Vec3::new(
        if color.x > 0.0 { color.x.sqrt() } else { 0.0 },
        if color.y > 0.0 { color.y.sqrt() } else { 0.0 },
        if color.z > 0.0 { color.z.sqrt() } else { 0.0 },
    )
}

fn render(
    screen: &mut Screen,
    scene: &HittableList,
    camera: &Camera,
    samples: u32,
    max_depth: u32,
) {
    for y in 0..screen.height {
        for x in 0..screen.width {
            let mut color: Vec3 = Vec3::zero();
            for _ in 0..samples {
                let ray = camera.get_ray(x, y);
                color += ray_color(&ray, scene, max_depth);
            }

            color /= samples as f32;
            color = linear_to_gamma(&color);

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
        print!("\r{}/{}", y, screen.height);
    }
    println!();
}

fn main() {
    let width = 540;
    let height = 400;
    let samples_per_pixel = 20;
    let max_depth = 10;

    //Materials
    let ground_mat = Rc::new(materials::Lambertian {
        albedo: Vec3::new(0.4, 0.59, 0.56),
    });
    let blue_diffuse = Rc::new(materials::Lambertian {
        albedo: Vec3::new(0.1, 0.2, 0.8),
    });
    let gold_mat = Rc::new(materials::Metal {
        albedo: Vec3::new(0.944, 0.776, 0.373),
        roughness: 0.4,
    });
    let silver_mat = Rc::new(materials::Metal {
        albedo: Vec3::new(0.962, 0.949, 0.922),
        roughness: 1.0,
    });
    let glass_mat = Rc::new(materials::Dielectric { ior: 1.5 });
    let glass_inner_mat = Rc::new(materials::Dielectric { ior: 1.0 / 1.5 });

    //Scene
    let mut hittables: HittableList = Vec::new();

    hittables.push(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, -1.2),
        0.5,
        blue_diffuse.clone(),
    )));
    hittables.push(Box::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        gold_mat.clone(),
    )));
    hittables.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        glass_mat.clone(),
    )));
    hittables.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.4,
        glass_inner_mat.clone(),
    )));
    hittables.push(Box::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        ground_mat.clone(),
    )));

    let mut screen = Screen::new(width, height);
    let camera = Camera::new(width, height, Vec3::zero());

    render(
        &mut screen,
        &hittables,
        &camera,
        samples_per_pixel,
        max_depth,
    );

    print!("Saving to file... ");
    io::stdout().flush().unwrap();
    write_to_file_ppm(&screen, Path::new("./out/test.ppm")).unwrap();
    println!("Done!");
}
