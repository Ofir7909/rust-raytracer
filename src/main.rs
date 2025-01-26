#![allow(dead_code)]

mod camera;
mod hittables;
mod materials;
mod math;
mod screen;
mod utils;

use std::{
    f32::INFINITY,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
    sync::Arc,
    thread,
};

use camera::Camera;
use hittables::{BVHNode, Hittable, HittableList, Quad, Sphere};
use math::{interval::Interval, ray::Ray, vec3::Vec3};
use rand::Rng;
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

fn ray_color(ray: &Ray, world: &impl Hittable, depth: u32, background_color: &Vec3) -> Vec3 {
    if depth <= 0 {
        return Vec3::ZERO;
    }
    match world.hit(ray, &Interval::new(0.001, INFINITY)) {
        Some(hit_info) => {
            let color_from_emission = hit_info.material.emitted(&hit_info);
            match hit_info.material.scatter(ray, &hit_info) {
                Some((attenution, scattered_ray)) => {
                    color_from_emission
                        + attenution * ray_color(&scattered_ray, world, depth - 1, background_color)
                }
                None => color_from_emission,
            }
        }
        None => *background_color,
    }
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
    scene: &impl Hittable,
    camera: &Camera,
    background_color: &Vec3,
    samples: u32,
    max_depth: u32,
    thread_count: u32,
) {
    let mut colors = vec![Vec3::ZERO; (screen.width * screen.height) as usize];
    thread::scope(|scope| {
        let width = screen.width;
        let height = screen.height;

        let thread_with_extra_sample = samples % thread_count;
        let base_samples_per_thread = samples / thread_count;

        let mut handles = vec![];
        handles.reserve(thread_count as usize);

        for i in 0..thread_count {
            let samples_in_thread = base_samples_per_thread + (i < thread_with_extra_sample) as u32;
            let handle = scope.spawn(move || {
                let mut colors_local = vec![Vec3::ZERO; (width * height) as usize];

                for y in 0..height {
                    for x in 0..width {
                        let i = (y * width + x) as usize;
                        for _ in 0..samples_in_thread {
                            let ray = camera.get_ray(x, y);
                            colors_local[i] += ray_color(&ray, scene, max_depth, background_color);
                        }
                    }
                }
                colors_local
            });
            handles.push(handle);
        }

        for h in handles {
            let colors_local = h.join().unwrap();
            colors = colors
                .iter()
                .zip(colors_local.iter())
                .map(|(a, b)| *a + *b)
                .collect();
        }
    });

    for y in 0..screen.height {
        for x in 0..screen.width {
            let mut color = colors[(y * screen.width + x) as usize];
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
    }
}

fn create_scene(width: u32, height: u32) -> (HittableList, Camera, Vec3) {
    let ground_mat = Arc::new(materials::Lambertian {
        albedo: Vec3::new(0.4, 0.59, 0.56),
    });
    let blue_diffuse = Arc::new(materials::Lambertian {
        albedo: Vec3::new(0.1, 0.2, 0.8),
    });
    let gold_mat = Arc::new(materials::Metal {
        albedo: Vec3::new(0.944, 0.776, 0.373),
        roughness: 0.4,
    });
    let glass_mat = Arc::new(materials::Dielectric { ior: 1.5 });
    let glass_inner_mat = Arc::new(materials::Dielectric { ior: 1.0 / 1.5 });

    let mut hittables = HittableList::new();

    hittables.add(Arc::new(Sphere::new(
        Vec3::new(0.0, 0.0, -1.2),
        0.5,
        blue_diffuse.clone(),
    )));
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        gold_mat.clone(),
    )));
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        glass_mat.clone(),
    )));
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.4,
        glass_inner_mat.clone(),
    )));
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        ground_mat.clone(),
    )));

    let camera = Camera::new(
        width,
        height,
        Vec3::new(-2.0, 2.0, 1.0),
        30.0,
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::UP,
        10.0,
        3.4,
    );

    let background_color = Vec3::new(0.5, 0.7, 1.0);

    (hittables, camera, background_color)
}

fn create_final_scene(width: u32, height: u32) -> (HittableList, Camera, Vec3) {
    let mut rng = rand::thread_rng();

    let mut hittables = HittableList::new();
    hittables.reserve(22 * 22 + 10);

    // Ground
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::new(materials::Lambertian {
            albedo: Vec3::new(0.4, 0.59, 0.56),
        }),
    )));

    // Big spheres
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        Arc::new(materials::Dielectric { ior: 1.5 }),
    )));
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        Arc::new(materials::Lambertian {
            albedo: Vec3::new(0.4, 0.2, 0.1),
        }),
    )));
    hittables.add(Arc::new(Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        Arc::new(materials::Metal {
            albedo: Vec3::new(0.7, 0.6, 0.5),
            roughness: 0.1,
        }),
    )));

    // Small spheres
    for x in -11..11 {
        for z in -11..11 {
            let radius = 0.2;
            let center = Vec3::new(
                x as f32 + rng.gen_range::<f32, _>(0.1..0.9),
                radius,
                z as f32 + 0.9 * rng.gen_range::<f32, _>(0.1..0.9),
            );

            let material: Arc<dyn materials::Material> = match rng.gen::<f32>() {
                x if x < 0.7 => Arc::new(materials::Lambertian {
                    albedo: Vec3::new(rng.gen(), rng.gen(), rng.gen()),
                }),
                x if x < 0.9 => Arc::new(materials::Metal {
                    albedo: Vec3::new(rng.gen(), rng.gen(), rng.gen()),
                    roughness: rng.gen(),
                }),
                _ => Arc::new(materials::Dielectric { ior: 1.5 }),
            };

            hittables.add(Arc::new(Sphere::new(center, radius, material)));
        }
    }

    let camera = Camera::new(
        width,
        height,
        Vec3::new(13.0, 2.0, 3.0),
        20.0,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::UP,
        0.6,
        10.0,
    );

    let background_color = Vec3::new(0.5, 0.7, 1.0);

    (hittables, camera, background_color)
}

fn create_quads_scene(width: u32, height: u32) -> (HittableList, Camera, Vec3) {
    let left_red = Arc::new(materials::Lambertian {
        albedo: Vec3::new(1.0, 0.2, 0.2),
    });
    let back_green = Arc::new(materials::Lambertian {
        albedo: Vec3::new(0.2, 1.0, 0.2),
    });
    let right_blue = Arc::new(materials::Lambertian {
        albedo: Vec3::new(0.2, 0.2, 1.0),
    });
    let upper_orange = Arc::new(materials::Lambertian {
        albedo: Vec3::new(1.0, 0.5, 0.0),
    });
    let lower_teal = Arc::new(materials::Lambertian {
        albedo: Vec3::new(0.2, 0.8, 0.8),
    });

    let mut hittables = HittableList::new();

    hittables.add(Arc::new(Quad::new(
        Vec3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        back_green.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        upper_orange.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal.clone(),
    )));

    let camera = Camera::new(
        width,
        height,
        Vec3::BACKWARD * 9.0,
        80.0,
        Vec3::ZERO,
        Vec3::UP,
        0.0,
        1.0,
    );

    let background_color = Vec3::new(0.5, 0.7, 1.0);

    (hittables, camera, background_color)
}

fn create_lights_scene(width: u32, height: u32) -> (HittableList, Camera, Vec3) {
    let mut hittables = HittableList::new();

    hittables.add(Arc::new(Sphere::new(
        Vec3::new(0.0, 0.5, 0.0),
        0.5,
        Arc::new(materials::Lambertian {
            albedo: Vec3::new(0.2, 0.2, 0.9),
        }),
    )));

    //Floor
    hittables.add(Arc::new(Quad::new(
        Vec3::new(-500.0, 0.0, -500.0),
        Vec3::RIGHT * 1000.0,
        Vec3::BACKWARD * 1000.0,
        Arc::new(materials::Lambertian {
            albedo: Vec3::uniform(0.5),
        }),
    )));

    // Light
    hittables.add(Arc::new(Quad::new(
        Vec3::new(1.0, 0.0, -0.8),
        Vec3::UP * 1.0,
        Vec3::BACKWARD * 1.6,
        Arc::new(materials::DiffuseLight {
            color: Vec3::new(1.0, 1.0, 1.0) * 4.0,
        }),
    )));

    let camera = Camera::new(
        width,
        height,
        Vec3::new(-0.6, 0.7, 2.0),
        50.0,
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::UP,
        0.0,
        1.0,
    );

    let background_color = Vec3::uniform(0.002);

    (hittables, camera, background_color)
}

fn create_cornell_scene(width: u32, height: u32) -> (HittableList, Camera, Vec3) {
    let red_wall = Arc::new(materials::Lambertian {
        albedo: Vec3::new(0.65, 0.05, 0.05),
    });
    let white_wall = Arc::new(materials::Lambertian {
        albedo: Vec3::uniform(0.73),
    });
    let green_wall = Arc::new(materials::Lambertian {
        albedo: Vec3::new(0.12, 0.45, 0.15),
    });

    let mut hittables = HittableList::new();

    hittables.add(Arc::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::UP * 555.0,
        Vec3::BACKWARD * 555.0,
        green_wall.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::UP * 555.0,
        Vec3::BACKWARD * 555.0,
        red_wall.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::RIGHT * 555.0,
        Vec3::BACKWARD * 555.0,
        white_wall.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::LEFT * 555.0,
        Vec3::FORWARD * 555.0,
        white_wall.clone(),
    )));
    hittables.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::RIGHT * 555.0,
        Vec3::UP * 555.0,
        white_wall.clone(),
    )));

    // Light
    hittables.add(Arc::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::LEFT * 130.0,
        Vec3::FORWARD * 105.0,
        Arc::new(materials::DiffuseLight {
            color: Vec3::new(1.0, 1.0, 1.0) * 15.0,
        }),
    )));

    let camera = Camera::new(
        width,
        height,
        Vec3::new(278.0, 278.0, -800.0),
        40.0,
        Vec3::new(278.0, 278.0, 0.0),
        Vec3::UP,
        0.0,
        1.0,
    );

    let background_color = Vec3::ZERO;

    (hittables, camera, background_color)
}

fn main() {
    let scene_index = 4;
    let width = 1080 / 2;
    let height = 1080 / 2;
    let samples_per_pixel = 200;
    let max_depth = 20;
    let thread_count = 8;

    let mut screen = Screen::new(width, height);

    let (mut hittables, camera, background_color) = match scene_index {
        0 => create_scene(width, height),
        1 => create_final_scene(width, height),
        2 => create_quads_scene(width, height),
        3 => create_lights_scene(width, height),
        4 => create_cornell_scene(width, height),
        _ => panic!("Unknown scene"),
    };
    create_cornell_scene(width, height);

    let world: BVHNode = BVHNode::from_hittable_list(&mut hittables);

    println!("Starting render.");
    let start_time = std::time::Instant::now();

    render(
        &mut screen,
        &world,
        &camera,
        &background_color,
        samples_per_pixel,
        max_depth,
        thread_count,
    );

    let duration = start_time.elapsed();
    println!(
        "Render took {}:{} ",
        duration.as_secs() / 60,
        duration.as_secs() % 60
    );

    print!("Saving to file... ");
    io::stdout().flush().unwrap();
    write_to_file_ppm(&screen, Path::new("./out/test.ppm")).unwrap();
    println!("Done!");
}
