use image::{ImageBuffer, RgbImage};

use crate::color::{Color, color_bytes};

/// A buffer to store the result of path tracing.
pub struct Buffer {
    /// The width of image.
    width: u32,
    /// The height of image.
    height: u32,
    /// The sample colors of image.
    /// The first index: the location of pixel in (y * width + x)
    /// The second index: the color of different iteration rounds.
    samples: Vec<Vec<Color>>,
}

impl Buffer {
    /// Create a empty buffer with width and height.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            samples: vec![vec![]; (width * height) as usize],
        }
    }

    /// Push a color of new iteration round into the buffer.
    pub fn add_sample(&mut self, x: u32, y: u32, color: Color) {
        assert!(x < self.width && y < self.height, "Invalid pixel location!");
        let index = (y * self.width + x) as usize;
        self.samples[index].push(color);
    }

    /// Extend a list of colors into the buffer.
    pub fn add_samples(&mut self, colors: Vec<Color>) {
        for (index, color) in colors.iter().enumerate() {
            self.samples[index].push(*color);
        }
    }

    /// Transite the buffer into rgb image.
    pub fn image(&self) -> RgbImage {
        let mut buf = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.get_color(x, y);
                let [r, g, b] = color_bytes(color);
                buf.push(r);
                buf.push(g);
                buf.push(b);
            }
        }
        ImageBuffer::from_raw(self.width, self.height, buf).expect("Incorrect image size.")
    }

    /// Get the average color in iteration rounds color.
    pub fn get_color(&self, x: u32, y: u32) -> Color {
        let index = (y * self.width + x) as usize;
        let color: Color = self.samples[index].iter().sum();
        let count = self.samples[index].len();
        color / count as f64
    }
}
