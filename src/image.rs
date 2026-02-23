use std::f32;

use crate::{color::Color, math::Vec3};

pub struct HdrImage {
    /// The width of the image in pixels.
    width: u32,
    /// The height of the image in pixels.
    height: u32,
    /// The pixel colors of the image.
    buf: Vec<Color>,
}

impl HdrImage {
    pub fn new(width: u32, height: u32, buf: Vec<Color>) -> Self {
        assert!(width * height == buf.len() as u32);
        Self { width, height, buf }
    }

    /// Sample the background of this image in camera's field of view.
    pub fn sample(&self, dir: Vec3) -> Color {
        let polar = dir.y.acos();
        let azimuth = dir.z.atan2(dir.x) + f32::consts::PI;
        let x = azimuth / f32::consts::TAU * (self.width - 1) as f32;
        let y = polar / f32::consts::PI * (self.height - 1) as f32;
        self.bilinear_sample(x, y)
    }

    /// Sample the pixel color using bilinear interpolation.
    pub fn bilinear_sample(&self, x: f32, y: f32) -> Color {
        let x0 = (x as u32).min(self.width - 1);
        let y0 = (y as u32).min(self.height - 1);
        let dx = x - x0 as f32;
        let dy = y - y0 as f32;
        let color00 = self.buf[(y0 * self.width + x0) as usize];
        let color01 = self.buf[(y0 * self.width + (x0 + 1)) as usize];
        let color10 = self.buf[((y0 + 1) * self.width + x0) as usize];
        let color11 = self.buf[((y0 + 1) * self.width + (x0 + 1)) as usize];
        let color0 = color00.lerp(color01, dx);
        let color1 = color10.lerp(color11, dx);
        let color = color0.lerp(color1, dy);
        color
    }
}
