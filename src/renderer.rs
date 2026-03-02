use std::f64;

use glam::DVec3;
use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rayon::prelude::*;

use crate::buffer::Buffer;
use crate::camera::Camera;
use crate::color::{self, Color};
use crate::interval::Interval;
use crate::light::Light;
use crate::math::Ray;
use crate::math::random;
use crate::scene::Scene;
use crate::shape::{HitRecord, Hittable};

pub struct Renderer {
    /// The camera to use
    pub cam: Camera,

    /// The scene to render
    pub scene: Scene,

    /// The width of output image
    pub width: u32,

    /// The height of output image
    pub height: u32,

    /// The number of samplings for one pixel in an image.
    pub num_samples: u32,

    /// The maximum number of the light bounces in the image.
    pub max_bounces: u32,
}

impl Renderer {
    /// Create a renderer from camera and scene.
    pub const fn new(cam: Camera, scene: Scene) -> Self {
        Self {
            cam,
            scene,
            width: 800,
            height: 600,
            max_bounces: 50,
            num_samples: 100,
        }
    }

    /// Set the width of output image.
    pub const fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the height of output image.
    pub const fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Set number of samplings for one pixel.
    pub const fn num_samples(mut self, n: u32) -> Self {
        self.num_samples = n;
        self
    }

    /// Set maximum number of the light bounces for renderer.
    pub const fn max_bounces(mut self, n: u32) -> Self {
        self.max_bounces = n;
        self
    }

    /// Trace the ray and return the color.
    pub fn trace_ray(&self, ray: &Ray, num_bounces: u32, rng: &mut StdRng) -> Color {
        if num_bounces == 0 {
            return color::BLACK;
        }

        // Start ray interval above zero (1e-3) to avoid shadow acne.
        match self.intersect(ray, Interval::new(1e-3, f64::INFINITY)) {
            None => self.scene.background.sample(ray.dir),
            Some(rec) => {
                let mut color = rec.material().emittance * rec.material().color;
                let v = -ray.dir;
                // Sample two kinds of light: directive light, indirective light.
                // 1. directive light. The light only bounces one time.
                color += self.sample_lights(&rec, ray.t, v, rng);
                // 2. indirective light which means bounced light.
                if let Some((l, pdf)) = rec.material().scatter(rng, rec.normal, v, rec.front_face) {
                    let f = rec.material().bsdf(l, v, rec.normal, rec.front_face);
                    let scatter = Ray::new(rec.p, l, ray.t);
                    let indirect = 1.0 / pdf
                        * f
                        * rec.normal.dot(l).abs()
                        * self.trace_ray(&scatter, num_bounces - 1, rng);
                    if indirect.is_finite() {
                        color += indirect.min(DVec3::splat(100.0));
                    }
                }
                color
            }
        }
    }

    /// Sample the ray towards lights in the scene for the given `world_pos` and return the color.
    fn sample_lights(
        &self,
        rec: &HitRecord,
        shutter_time: f64,
        ray_view: DVec3,
        rng: &mut StdRng,
    ) -> Color {
        let mut color_from_lights = Color::ZERO;
        let material = rec.material();
        let pos = rec.p;
        let n = rec.normal;
        let front_face = rec.front_face;

        for light in &self.scene.lights {
            match light {
                Light::Ambient(color_ambient) => {
                    color_from_lights += color_ambient * material.color;
                }
                _ => {
                    let (intensity, ray_light, t_micro) = light.illuminate(pos, rng, shutter_time);
                    let close_hit = self
                        .intersect(
                            &Ray::new(pos, ray_light, shutter_time),
                            Interval::new(1e-3, t_micro - 1e-3),
                        )
                        .map(|rec| rec.t);

                    // The light can reach the world position `pos`.
                    if close_hit.is_none() {
                        let f = material.bsdf(ray_light, ray_view, n, front_face);
                        // The integrand of monte carlo integral.
                        // intensity equals to (attenuation * pdf)
                        color_from_lights += f * intensity * n.dot(ray_light).abs();
                    }
                }
            }
        }
        color_from_lights
    }

    /// Get the pixel color of a specified location in film plane.
    pub fn get_color(&self, col: u32, row: u32, iterations: u32, rng: &mut StdRng) -> Color {
        let mut pixel_color = Color::default();
        // Sampling stratifications + Monte Carlo approximation.
        let iter_sqrt = (iterations as f64).sqrt() as u32;
        for y in 0..iter_sqrt {
            for x in 0..iter_sqrt {
                let s = (col as f64 + (x as f64 + random()) / iter_sqrt as f64) / self.width as f64;
                let t =
                    (row as f64 + (y as f64 + random()) / iter_sqrt as f64) / self.height as f64;
                let r = self.cam.get_ray(s, t, rng);
                let sample_color = self.trace_ray(&r, self.max_bounces, rng);
                // Avoid NaN and infinity in color which may cause pixel acne.
                if sample_color.is_finite() {
                    pixel_color += sample_color;
                }
            }
        }
        pixel_color / iterations as f64
    }

    /// Get all pixel colors in film plane and store into `buffer`.
    pub fn sample(&self, iterations: u32, buffer: &mut Buffer) {
        // Progress bar
        let pb = ProgressBar::new(self.height as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("=>-"),
        );

        // Pixel colors
        let colors: Vec<_> = (0..self.height)
            .into_par_iter()
            .map(|row| {
                let mut rng = StdRng::from_os_rng();
                let row_pixels: Vec<Color> = (0..self.width)
                    .map(|col| self.get_color(col, row, iterations, &mut rng))
                    .collect();

                // Update progress bar after finish each row
                pb.inc(1);
                row_pixels
            })
            .flatten()
            .collect();
        buffer.add_samples(colors);
        pb.finish_with_message("Done!");
    }

    /// Render the image for given scene and return `RgbImage`.
    pub fn render(&self) -> RgbImage {
        let mut buffer = Buffer::new(self.width, self.height);
        self.sample(self.num_samples, &mut buffer);
        buffer.image()
    }

    /// Render the image for given scene and call customized function for each epoch.
    pub fn iterative_render<F>(&self, interval: u32, callback: F)
    where
        F: Fn(u32, &Buffer),
    {
        let mut buffer = Buffer::new(self.width, self.height);
        // The accumulate value is used in callback to get progress information.
        let mut iterations_acc = 0;
        // The max number to sample is `self.samples`, so we need limit it.
        // For each epoch, the sample result will be stored in corresponding pxiel position in `Buffer` which is
        // flatten pixel color array.
        while iterations_acc < self.num_samples {
            let step = interval.min(self.num_samples - iterations_acc);
            self.sample(step, &mut buffer);
            iterations_acc += step;
            callback(iterations_acc, &buffer);
        }
    }
}

impl Hittable for Renderer {
    /// Get closest intersection of ray with intersectable objects.
    fn intersect(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        if let Some(bvh) = &self.scene.bvh {
            return bvh.intersect(r, ray_t);
        }
        let mut rec = None;
        let mut closest_so_far = ray_t.max;
        for obj in &self.scene.objects {
            let search_interval = Interval::new(ray_t.min, closest_so_far);
            if let Some(obj_rec) = obj.intersect(r, search_interval) {
                closest_so_far = obj_rec.t;
                rec = Some(obj_rec);
            }
        }
        rec
    }
}
