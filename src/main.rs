use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

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

fn main() {
    let width = 3;
    let height = 2;

    let mut screen = Screen::new(width, height);
    screen.buffer[0] = (255, 0, 0);
    screen.buffer[1] = (0, 255, 0);
    screen.buffer[2] = (0, 0, 255);
    screen.buffer[3] = (255, 255, 0);
    screen.buffer[4] = (255, 255, 255);
    screen.buffer[5] = (0, 0, 0);

    write_to_file_ppm(&screen, Path::new("test.ppm")).unwrap();

    println!("Hello, world!");
}
