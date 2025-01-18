pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<(u8, u8, u8)>,
}

impl Screen {
    pub fn new(width: u32, height: u32) -> Screen {
        Screen {
            width,
            height,
            buffer: vec![(0, 0, 0); (width * height) as usize],
        }
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, pixel: (u8, u8, u8)) {
        self.buffer[(y * self.width + x) as usize] = pixel
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}
