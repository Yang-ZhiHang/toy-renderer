use image::ImageReader;

use crate::color::{self, Color};
use crate::image::HdrImage;
use crate::light::Light;
use crate::math::Vec3;
use crate::{bvh::BvhNode, object::Object};

#[derive(Default)]
pub struct Scene {
    /// The list of objects in the scene.
    pub objects: Vec<Object>,

    /// The list of lights in the scene.
    pub lights: Vec<Light>,

    /// The BVH for the scene.
    pub bvh: Option<BvhNode>,

    /// The background color of the scene
    pub background: Background,
}

impl Scene {
    /// Create a empty scene.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the background of the scene.
    pub fn background(mut self, background: Background) -> Self {
        self.background = background;
        self
    }

    /// Builder-style add that consumes and returns the Scene.
    pub fn with_obj(mut self, obj: Object) -> Self {
        self.objects.push(obj);
        self
    }

    /// Builder-style batch add that consumes and returns the Scene.
    pub fn with_obj_list<I>(mut self, obj_list: I) -> Self
    where
        I: IntoIterator<Item = Object>,
    {
        self.objects.extend(obj_list);
        self
    }

    /// Add a Light to Scene.
    pub fn with_light(mut self, light: Light) -> Self {
        self.lights.push(light);
        self
    }

    /// Add a list of Light to Scene.
    pub fn with_lights<I>(mut self, lights: I) -> Self
    where
        I: IntoIterator<Item = Light>,
    {
        self.lights.extend(lights);
        self
    }

    /// Build BVH from current objects which should call after scene setup.
    /// After built the BVH, you can't add more objects or lights to scene. Or else you should call this function again.
    pub fn build_bvh(mut self) -> Self {
        if self.objects.is_empty() {
            self.bvh = None;
        } else {
            self.bvh = Some(BvhNode::build(self.objects.clone()));
        }
        self
    }
}

pub enum Background {
    /// Solid color
    Color(Color),
    /// Panorama image
    Image(HdrImage),
}

impl Default for Background {
    fn default() -> Self {
        Self::Color(color::BLACK)
    }
}

impl Background {
    /// Create `Background` from solid color.
    pub fn from_color(color: Color) -> Self {
        Self::Color(color)
    }

    /// Create `Background` from a panorama image path.
    /// High dynamic range image usually stored in disk using RGBE compression algorithm.
    /// RGBE uses 4 bytes(u8) to analog float.
    /// Firstly, We need read and decode the image.
    /// Then, get the pixel into array and create `Background` struct.
    pub fn from_hdr(path: &str) -> Self {
        // Read and decode.
        let img = ImageReader::open(path)
            .expect("Failed to open file")
            .decode()
            .expect("Failed to decode image");
        // Get pixels into array, BTW width and height.
        let (width, height, pixels) = match img {
            image::DynamicImage::ImageRgb32F(inner) => {
                let (w, h) = inner.dimensions();
                (w, h, inner.into_raw())
            }
            _ => {
                let inner = img.to_rgb32f();
                let (w, h) = inner.dimensions();
                (w, h, inner.into_raw())
            }
        };
        // Create `HdrImage` struct.
        Self::Image(HdrImage::new(
            width,
            height,
            pixels
                .chunks_exact(3)
                .map(|p| Color::new(p[0], p[1], p[2]))
                .collect(),
        ))
    }

    /// Get the color of background in specified ray direction.
    pub fn sample(&self, dir: Vec3) -> Color {
        match self {
            Self::Color(c) => *c,
            Self::Image(image) => image.sample(dir),
        }
    }
}
