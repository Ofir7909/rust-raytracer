mod math;

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use math::{ray::Ray, vec3::Vec3};

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

    let mut file = File::create(&filepath)?;

    write!(file, "P3\n{} {}\n255\n", screen.width, screen.height)?;
    for pixel in screen.buffer.iter() {
        write!(file, "{} {} {}\n", pixel.0, pixel.1, pixel.2)?
    }

    Ok(())
}

fn ray_color(ray: &Ray) -> Vec3 {
    let sky_color: Vec3 = Vec3::new(0.5, 0.7, 1.0);
    let horizon_color: Vec3 = Vec3::new(1.0, 1.0, 1.0);

    let unit_direction = ray.direction.normalized();
    let a = (unit_direction.y + 1.0) * 0.5;

    a * sky_color + (1.0 - a) * horizon_color
}

fn main() {
    let width = 540;
    let height = 400;

    let mut screen = Screen::new(width, height);

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

    for y in 0..height {
        for x in 0..width {
            let pixel_center = pixel00_loc + x as f32 * pixel_delta_u + y as f32 * pixel_delta_v;
            let ray_dir = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_dir);

            let color = ray_color(&ray);

            screen.write_pixel(
                x,
                y,
                (
                    (color.x * 255.0) as u8,
                    (color.y * 255.0) as u8,
                    (color.z * 255.0) as u8,
                ),
            );
        }
        println!("\r\r{}/{}\r\r", y, height)
    }

    print!("Saving to file... ");
    io::stdout().flush().unwrap();
    write_to_file_ppm(&screen, Path::new("./out/test.ppm")).unwrap();
    println!("Done!");
}
